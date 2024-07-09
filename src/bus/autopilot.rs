use crate::autopilot::config::AutopilotConfig;
use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, RequestContent, RequestType};
use serde::{Deserialize, Serialize};
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

    pub async fn get_by_id<S: AsRef<str>>(&self, id: S) -> Result<Option<Autopilot>, Error> {
        let url = format!("./bus/autopilot/{}", id.as_ref());
        if let Some(resp) = self
            .inner
            .send_api_request(&url, &RequestType::Get(None), true)
            .await?
        {
            Ok(Some(resp.json().await?))
        } else {
            Ok(None)
        }
    }

    pub async fn update(&self, autopilot: &Autopilot) -> Result<(), Error> {
        let url = format!("./bus/autopilot/{}", autopilot.id.as_str());
        let req = update_req(autopilot)?;
        let _ = self.inner.send_api_request(&url, &req, false).await?;
        Ok(())
    }
}

fn update_req(autopilot: &Autopilot) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Put(
        Some(RequestContent::Json(
            serde_json::to_value(autopilot).map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Autopilot {
    pub id: String,
    pub config: AutopilotConfig,
    pub current_period: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autopilot::config::{ContractConfig, HostConfig};
    use serde_json::Value;

    #[test]
    fn list() -> anyhow::Result<()> {
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
            150000000000000000000000000000
        );

        Ok(())
    }

    #[test]
    fn update() -> anyhow::Result<()> {
        let json = r#"{
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
            "minProtocolVersion": "1.5",
            "minRecentScanFailures": 0,
            "scoreOverrides": null
        }
    },
    "currentPeriod": 428982
}
"#;
        let expected: Value = serde_json::from_str(&json)?;

        let autopilot = Autopilot {
            id: "autopilot".to_string(),
            config: AutopilotConfig {
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
                    min_protocol_version: "1.5".to_string(),
                    min_recent_scan_failures: 0,
                    score_overrides: None,
                },
            },
            current_period: 428982,
        };

        match update_req(&autopilot)? {
            RequestType::Put(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }
}
