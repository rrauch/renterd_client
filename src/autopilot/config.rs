use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, PublicKey};
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

    pub async fn list(&self) -> Result<AutopilotConfig, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./autopilot/config", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
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
    #[serde(with = "crate::number_as_string")]
    pub allowance: u128,
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

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
        {
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
      }
   }
        "#;
        let config: AutopilotConfig = serde_json::from_str(&json)?;
        assert_eq!(
            config.contract_config.allowance,
            150000000000000000000000000000
        );
        assert_eq!(config.contract_config.set, "autopilot");
        assert_eq!(config.contract_config.download, 1000000000000);

        assert_eq!(config.host_config.allow_redundant_ips, false);
        assert_eq!(config.host_config.max_downtime_hours, 1440);
        assert_eq!(config.host_config.score_overrides, None);

        Ok(())
    }
}
