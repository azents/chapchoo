use super::*;

pub struct SaveHarnessSupply {
    pub agent_id: String,
    pub request_id: String,
    pub expected_request_id: Option<String>,
    pub supply: HarnessSupply,
}
pub struct HarnessSupplySelection {
    pub supply: HarnessSupply,
    /// Absence means the server-derived managed default, not a saved owner change.
    pub saved: Option<HarnessSupplyReceipt>,
}
pub struct HarnessSupplyReceipt {
    pub request_id: String,
    pub accepted_at: i64,
}
macro_rules! safe_debug {
    ($name:ident) => {
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    };
}
safe_debug!(SaveHarnessSupply);
safe_debug!(HarnessSupplySelection);
safe_debug!(HarnessSupplyReceipt);

#[derive(diesel::QueryableByName)]
struct SupplyRow {
    #[diesel(sql_type=Text)]
    supply_kind: String,
    #[diesel(sql_type=Nullable<Text>)]
    executable_path: Option<String>,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=BigInt)]
    accepted_at: i64,
    #[diesel(sql_type=Nullable<Text>)]
    expected_request_id: Option<String>,
}
impl SupplyRow {
    fn supply(&self) -> Result<HarnessSupply, StoreError> {
        match (self.supply_kind.as_str(), &self.executable_path) {
            ("MANAGED", None) => Ok(HarnessSupply::Managed),
            ("EXISTING", Some(path)) => Ok(HarnessSupply::Existing(
                ExistingCodexPath::parse(path.clone()).map_err(|_| StoreError::Integrity)?,
            )),
            _ => Err(StoreError::Integrity),
        }
    }
    fn receipt(&self) -> HarnessSupplyReceipt {
        HarnessSupplyReceipt {
            request_id: self.request_id.clone(),
            accepted_at: self.accepted_at,
        }
    }
}
const SAVED_SUPPLY: &str = "SELECT supply_kind,executable_path,current_request_id AS request_id,selected_at AS accepted_at,CAST(NULL AS TEXT) AS expected_request_id FROM agent_harness_supplies WHERE agent_id=?";

impl Store {
    pub async fn harness_supply(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        now: i64,
    ) -> Result<HarnessSupplySelection, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = diesel::sql_query(dialect.query(SAVED_SUPPLY))
                .bind::<IdentitySql, _>(agent)
                .get_result::<SupplyRow>(connection)
                .await
                .optional()?;
            match row {
                Some(row) => Ok(HarnessSupplySelection {
                    supply: row.supply()?,
                    saved: Some(row.receipt()),
                }),
                None => Ok(HarnessSupplySelection {
                    supply: HarnessSupply::Managed,
                    saved: None,
                }),
            }
        })
    }

    /// Save only; this operation has no runtime/Computer side effects.
    pub async fn save_harness_supply(
        &self,
        actor: &SecretDigest,
        draft: &SaveHarnessSupply,
        now: i64,
    ) -> Result<HarnessSupplyReceipt, StoreError> {
        request_id(&draft.request_id)?;
        if let Some(expected) = &draft.expected_request_id {
            request_id(expected)?;
        }
        let agent = EntityId::from_dto(EntityKind::Agent, &draft.agent_id)?;
        let (kind, path) = match &draft.supply {
            HarnessSupply::Managed => ("MANAGED", None),
            HarnessSupply::Existing(path) => ("EXISTING", Some(path.as_str())),
        };
        operation!(self, |connection, dialect| {
            let current_agent = owned_agent!(connection, dialect, actor, agent, now);
            let previous = diesel::sql_query(dialect.query(
                "SELECT * FROM agent_harness_supply_requests WHERE agent_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(agent)
            .bind::<Text, _>(&draft.request_id)
            .get_result::<SupplyRow>(connection)
            .await
            .optional()?;
            if let Some(previous) = previous {
                if previous.supply()? != draft.supply
                    || previous.expected_request_id != draft.expected_request_id
                {
                    return Err(StoreError::Conflict);
                }
                return Ok(previous.receipt());
            }
            if current_agent.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            let current = diesel::sql_query(dialect.query(SAVED_SUPPLY))
                .bind::<IdentitySql, _>(agent)
                .get_result::<SupplyRow>(connection)
                .await
                .optional()?;
            if current.as_ref().map(|row| row.request_id.as_str())
                != draft.expected_request_id.as_deref()
            {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query("INSERT INTO agent_harness_supplies(agent_id,supply_kind,executable_path,current_request_id,selected_at) VALUES(?,?,?,?,?) ON CONFLICT(agent_id) DO UPDATE SET supply_kind=excluded.supply_kind,executable_path=excluded.executable_path,current_request_id=excluded.current_request_id,selected_at=excluded.selected_at"))
                .bind::<IdentitySql,_>(agent).bind::<Text,_>(kind).bind::<Nullable<Text>,_>(path)
                .bind::<Text,_>(&draft.request_id).bind::<BigInt,_>(now).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO agent_harness_supply_requests(agent_id,request_id,expected_request_id,supply_kind,executable_path,accepted_at) VALUES(?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(agent).bind::<Text,_>(&draft.request_id)
                .bind::<Nullable<Text>,_>(draft.expected_request_id.as_deref()).bind::<Text,_>(kind)
                .bind::<Nullable<Text>,_>(path).bind::<BigInt,_>(now).execute(connection).await?;
            Ok(HarnessSupplyReceipt {
                request_id: draft.request_id.clone(),
                accepted_at: now,
            })
        })
    }
}
