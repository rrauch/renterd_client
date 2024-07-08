use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, State as CommonState};
use chrono::DateTime;
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
            serde_json::from_value(self.inner.get_json("./bus/state", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    #[serde(flatten)]
    pub common: CommonState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
        {
  "startTime": "2023-09-22T19:08:16.677593561Z",
  "network": "Mainnet",
  "version": "7fb1758",
  "commit": "7fb1758",
  "os": "linux",
  "buildTime": "2023-09-22T13:50:06Z"
}
        "#;
        let state: State = serde_json::from_str(&json)?;
        assert_eq!(
            state.common.start_time,
            DateTime::parse_from_rfc3339("2023-09-22T19:08:16.677593561Z")?
        );
        assert_eq!(state.common.network, "Mainnet");
        assert_eq!(state.common.version, "7fb1758");
        assert_eq!(state.common.os, "linux");
        assert_eq!(
            state.common.build_time,
            DateTime::parse_from_rfc3339("2023-09-22T13:50:06Z")?
        );

        Ok(())
    }
}
