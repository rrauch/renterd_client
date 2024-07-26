use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub(super) async fn get(&self) -> Result<State, Error> {
        Ok(self
            .inner
            .send_api_request(get_req())
            .await?
            .json()
            .await?)
    }
}

fn get_req() -> ApiRequest {
    ApiRequestBuilder::get("./worker/state").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    pub id: String,
    #[serde(flatten)]
    pub common: crate::State,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use chrono::DateTime;

    #[test]
    fn get() -> anyhow::Result<()> {
        let req = get_req();
        assert_eq!(req.path, "./worker/state");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "id": "worker",
  "startTime": "2023-09-21T08:25:18.542303234Z",
  "network": "Mainnet",
  "version": "v0.5.0-166-gaaf22529",
  "commit": "aaf22529",
  "os": "linux",
  "buildTime": "2023-09-20T14:03:05Z"
}
        "#;
        let state: State = serde_json::from_str(&json)?;
        assert_eq!(state.id, "worker");
        assert_eq!(
            state.common.build_time,
            DateTime::parse_from_rfc3339("2023-09-20T14:03:05Z")?
        );
        Ok(())
    }
}
