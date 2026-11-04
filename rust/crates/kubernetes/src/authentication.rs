//! TokenReview adapter; no unverified JWT claims establish identity.
use crate::Error;
use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec};
use kube::{Api, Client, Config, api::PostParams};

#[derive(Clone)]
pub struct ServiceAccountReviewer {
    client: Client,
}
impl std::fmt::Debug for ServiceAccountReviewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServiceAccountReviewer([REDACTED])")
    }
}
impl ServiceAccountReviewer {
    pub fn in_cluster() -> Result<Self, Error> {
        let config = Config::incluster().map_err(|_| Error::Configuration)?;
        Ok(Self {
            client: Client::try_from(config)?,
        })
    }
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }
    pub async fn review(
        &self,
        token: &str,
        audience: &str,
    ) -> Result<Option<(String, Vec<String>)>, Error> {
        let api: Api<TokenReview> = Api::all(self.client.clone());
        let review = api
            .create(
                &PostParams::default(),
                &TokenReview {
                    spec: TokenReviewSpec {
                        token: Some(token.to_owned()),
                        audiences: Some(vec![audience.to_owned()]),
                    },
                    ..Default::default()
                },
            )
            .await?;
        let Some(status) = review.status else {
            return Ok(None);
        };
        if status.authenticated != Some(true) || status.error.is_some() {
            return Ok(None);
        }
        let Some(username) = status.user.and_then(|user| user.username) else {
            return Ok(None);
        };
        let audiences = status.audiences.unwrap_or_default();
        if !audiences.iter().any(|a| a == audience) {
            return Ok(None);
        }
        Ok(Some((username, audiences)))
    }
}
