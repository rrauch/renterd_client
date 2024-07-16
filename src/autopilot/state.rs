use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

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
            .send_api_request(&get_req())
            .await?
            .json()
            .await?)
    }
}

fn get_req() -> ApiRequest {
    ApiRequestBuilder::get("./autopilot/state").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    pub configured: bool,
    pub migrating: bool,
    pub migrating_last_start: DateTime<FixedOffset>,
    pub pruning: bool,
    pub pruning_last_start: DateTime<FixedOffset>,
    pub scanning: bool,
    pub scanning_last_start: DateTime<FixedOffset>,
    #[serde(with = "crate::duration_ms")]
    #[serde(rename = "uptimeMs")]
    pub uptime: Duration,
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
        assert_eq!(req.path, "./autopilot/state");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "configured": true,
  "migrating": true,
  "migratingLastStart": "2023-09-21T08:31:01Z",
  "pruning": false,
  "pruningLastStart": "2023-09-20T11:09:58Z",
  "scanning": false,
  "scanningLastStart": "2023-09-21T12:09:58Z",
  "uptimeMs": 17297166,
  "startTime": "2023-09-21T08:25:18.542303234Z",
  "network": "Mainnet",
  "version": "v0.5.0-166-gaaf22529",
  "commit": "aaf22529",
  "os": "linux",
  "buildTime": "2023-09-20T14:03:05Z"
}
        "#;
        let state: State = serde_json::from_str(&json)?;
        assert_eq!(state.configured, true);
        assert_eq!(state.migrating, true);
        assert_eq!(
            state.migrating_last_start,
            DateTime::parse_from_rfc3339("2023-09-21T08:31:01Z")?
        );
        assert_eq!(state.pruning, false);
        assert_eq!(
            state.pruning_last_start,
            DateTime::parse_from_rfc3339("2023-09-20T11:09:58Z")?
        );
        assert_eq!(state.scanning, false);
        assert_eq!(state.uptime.as_millis(), 17297166);

        Ok(())
    }
}
