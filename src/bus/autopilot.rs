use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, PublicKey};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn list(&self) -> Result<Vec<Autopilot>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/autopilots", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Autopilot {
    pub id: String,
    pub config: AutopilotConfig,
    pub current_period: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct AutopilotConfig {
    #[serde(rename = "contracts")]
    pub contract_config: ContractConfig,
    #[serde(rename = "hosts")]
    pub host_config: HostConfig,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContractConfig {
    pub set: String,
    pub amount: u64,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub allowance: BigDecimal,
    pub period: u64,
    pub renew_window: u64,
    pub download: u64,
    pub upload: u64,
    pub storage: u64,
    pub prune: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct HostConfig {
    #[serde(rename = "allowRedundantIPs")]
    pub allow_redundant_ips: bool,
    pub max_downtime_hours: u64,
    pub min_protocol_version: String,
    pub min_recent_scan_failures: u64,
    pub score_overrides: Option<BTreeMap<PublicKey, f64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
[
  {
    "id": "autopilot",
    "config": {
      "contracts": {
        "set": "autopilot",
        "amount": 300,
        "allowance": "150000000000000000000000000000",
        "period": 6048,
        "renewWindow": 2016,
        "download": 1000000000000,
        "upload": 100000000000000,
        "storage": 101000000000000,
        "prune": false
      },
      "hosts": {
        "allowRedundantIPs": false,
        "maxDowntimeHours": 1440,
        "minProtocolVersion": "1.6.0",
		"minRecentScanFailures": 10,
		"scoreOverrides": null

      },
      "wallet": {
        "defragThreshold": 1000
      }
    },
    "currentPeriod": 428982
  }
]
"#;

        let autopilots: Vec<Autopilot> = serde_json::from_str(&json)?;
        assert_eq!(autopilots.len(), 1);

        let autopilot = autopilots.get(0).unwrap();
        assert_eq!(autopilot.id, "autopilot");
        assert_eq!(
            autopilot.config.contract_config.allowance,
            BigDecimal::from_str("150000000000000000000000000000").unwrap()
        );

        Ok(())
    }
}
