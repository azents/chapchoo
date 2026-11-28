//! Durable Agent-local names. Deleted integrations leave their reservations.

use super::*;
#[derive(QueryableByName)]
pub(super) struct Slot {
    #[diesel(sql_type = BigInt)]
    pub(super) slot: i64,
}

macro_rules! get_or_reserve {
    ($connection:ident, $dialect:ident, $agent:ident, $integration:expr) => {{
    async {
    for _ in 0..32 {
        if let Some(row) = diesel::sql_query($dialect.query(
            "SELECT slot FROM agent_mcp_keys WHERE agent_id=? AND integration_id=?",
        ))
        .bind::<IdentitySql, _>($agent)
        .bind::<IdentitySql, _>($integration)
        .get_result::<keys::Slot>($connection)
        .await
        .optional()?
        {
            return u64::try_from(row.slot).map_err(|_| StoreError::Integrity);
        }

        let mut random = [0; 8];
        getrandom::fill(&mut random).map_err(chapchoo_crypto::CryptoError::Random)?;
        let seed = u64::from_le_bytes(random);
        let mut lower = 1_i64;
        let mut candidate = None;
        for _ in 0..10 {
            let upper = lower * 10;
            let start = lower + i64::try_from(seed % (9 * lower.cast_unsigned()))
                .map_err(|_| StoreError::Integrity)?;
            candidate = diesel::sql_query($dialect.query(
                "WITH used AS (
                    SELECT slot FROM agent_mcp_keys WHERE agent_id=? AND slot>=? AND slot<?
                ), candidates AS (
                    SELECT CAST(? AS BIGINT) AS slot
                    UNION ALL SELECT CAST(? AS BIGINT)
                    UNION ALL SELECT slot+1 FROM used
                )
                SELECT c.slot FROM candidates c WHERE c.slot<? AND NOT EXISTS
                    (SELECT 1 FROM used u WHERE u.slot=c.slot)
                ORDER BY CASE WHEN c.slot>=? THEN 0 ELSE 1 END, c.slot LIMIT 1",
            ))
            .bind::<IdentitySql, _>($agent)
            .bind::<BigInt, _>(lower)
            .bind::<BigInt, _>(upper)
            .bind::<BigInt, _>(lower)
            .bind::<BigInt, _>(start)
            .bind::<BigInt, _>(upper)
            .bind::<BigInt, _>(start)
            .get_result::<keys::Slot>($connection)
            .await
            .optional()?;
            if candidate.is_some() {
                break;
            }
            lower = upper;
        }
        let slot = candidate.ok_or(StoreError::Integrity)?.slot;
        if diesel::sql_query($dialect.query(
            "INSERT INTO agent_mcp_keys (agent_id,integration_id,slot) VALUES (?,?,?) ON CONFLICT DO NOTHING",
        ))
        .bind::<IdentitySql, _>($agent)
        .bind::<IdentitySql, _>($integration)
        .bind::<BigInt, _>(slot)
        .execute($connection)
        .await? == 1
        {
            return u64::try_from(slot).map_err(|_| StoreError::Integrity);
        }
        // Another preparation reserved this integration or candidate.
    }
    Err(StoreError::Conflict)
    }.await
    }};
}
pub(super) use get_or_reserve;
