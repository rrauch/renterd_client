use crate::Error::InvalidDataError;
use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, PublicKey, RequestContent};
use serde::{Deserialize, Serialize};
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
        Ok(self
            .inner
            .send_api_request(&list_req())
            .await?
            .json()
            .await?)
    }

    pub async fn update(&self, autopilot_config: &AutopilotConfig) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&update_req(autopilot_config)?)
            .await?;
        Ok(())
    }

    //todo: config evaluation


}

fn list_req() -> ApiRequest {
    ApiRequestBuilder::get("./autopilot/config").build()
}

fn update_req(autopilot_config: &AutopilotConfig) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(autopilot_config).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::put("./autopilot/config")
        .content(content)
        .build())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutopilotConfig {
    #[serde(rename = "contracts")]
    pub contract_config: ContractConfig,
    #[serde(rename = "hosts")]
    pub host_config: HostConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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
    use crate::RequestType;
    use serde_json::Value;

    #[test]
    fn list() -> anyhow::Result<()> {
        let req = list_req();
        assert_eq!(req.path, "./autopilot/config");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

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

    #[test]
    fn update() -> anyhow::Result<()> {
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
        let expected: Value = serde_json::from_str(json)?;

        let req = update_req(&AutopilotConfig {
            contract_config: ContractConfig {
                set: "autopilot".to_string(),
                amount: 300,
                allowance: 150000000000000000000000000000,
                period: 6048,
                renew_window: 2016,
                download: 1000000000000,
                upload: 100000000000000,
                storage: 101000000000000,
                prune: false,
            },
            host_config: HostConfig {
                allow_redundant_ips: false,
                max_downtime_hours: 1440,
                min_protocol_version: "1.6.0".to_string(),
                min_recent_scan_failures: 10,
                score_overrides: None,
            },
        })?;
        assert_eq!(req.path, "./autopilot/config");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }
}
