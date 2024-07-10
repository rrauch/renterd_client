use crate::autopilot::config::AutopilotConfig;
use crate::Error::InvalidDataError;
use crate::{
    ApiRequest, ApiRequestBuilder, ClientInner, Error, PublicKey, RequestContent, RequestType,
};
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
        Ok(self
            .inner
            .send_api_request(&list_req())
            .await?
            .json()
            .await?)
    }

    pub async fn get_by_id<S: AsRef<str>>(&self, id: S) -> Result<Option<Autopilot>, Error> {
        match self.inner.send_api_request_optional(&get_req(id)).await? {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    pub async fn update(&self, autopilot: &Autopilot) -> Result<(), Error> {
        let req = update_req(autopilot)?;
        let _ = self.inner.send_api_request(&req).await?;
        Ok(())
    }

    pub async fn check_host<S: AsRef<str>>(
        &self,
        id: S,
        host_key: &PublicKey,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&check_host_req(id, host_key))
            .await?;
        Ok(())
    }
}

fn check_host_req<S: AsRef<str>>(id: S, host_key: &PublicKey) -> ApiRequest {
    ApiRequestBuilder::put(format!(
        "./bus/autopilot/{}/host/{}/check",
        id.as_ref(),
        host_key.to_string()
    ))
    .build()
}

fn list_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/autopilots").build()
}

fn get_req<S: AsRef<str>>(id: S) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/autopilot/{}", id.as_ref())).build()
}

fn update_req(autopilot: &Autopilot) -> Result<ApiRequest, Error> {
    let url = format!("./bus/autopilot/{}", autopilot.id.as_str());
    let content = Some(RequestContent::Json(
        serde_json::to_value(autopilot).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::put(url).content(content).build())
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
        let req = list_req();
        assert_eq!(req.path, "./bus/autopilots");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

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

        let req = update_req(&autopilot)?;
        assert_eq!(req.path, "./bus/autopilot/autopilot");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn check_host() -> anyhow::Result<()> {
        let req = check_host_req(
            "autopilot",
            &"ed25519:70b75b1acff1f80f9ace0c048ce8651586254e23d19ba405dc6f226e81d08ca2"
                .try_into()?,
        );
        assert_eq!(req.path, "./bus/autopilot/autopilot/host/ed25519:70b75b1acff1f80f9ace0c048ce8651586254e23d19ba405dc6f226e81d08ca2/check");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        Ok(())
    }
}
