use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
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

    pub async fn list(&self) -> Result<State, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./worker/state", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
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
    use chrono::DateTime;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
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
