use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
use chrono::{DateTime, FixedOffset};
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

    pub async fn state(&self) -> Result<State, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/consensus/state", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    pub block_height: u64,
    pub last_block_time: DateTime<FixedOffset>,
    pub synced: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_state() -> anyhow::Result<()> {
        let json = r#"
        {
  "blockHeight": 436326,
  "lastBlockTime": "2023-09-22T14:37:32Z",
  "synced": true
}
        "#;

        let state: State = serde_json::from_str(&json)?;
        assert_eq!(state.block_height, 436326);
        assert_eq!(state.synced, true);
        assert_eq!(
            state.last_block_time,
            DateTime::parse_from_rfc3339("2023-09-22T14:37:32Z")?
        );

        Ok(())
    }
}
