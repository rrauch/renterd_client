use crate::Error::InvalidDataError;
use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, RequestContent};
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

    pub(super) async fn trigger(&self, force_scan: bool) -> Result<bool, Error> {
        let resp: TriggerResponse = self
            .inner
            .send_api_request(trigger_req(force_scan)?)
            .await?
            .json()
            .await?;

        Ok(resp.triggered)
    }
}

fn trigger_req(force_scan: bool) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(TriggerRequest { force_scan })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./autopilot/trigger")
        .content(content)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TriggerRequest {
    force_scan: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TriggerResponse {
    triggered: bool,
}

#[cfg(test)]
mod tests {
    use crate::autopilot::trigger::{trigger_req, TriggerResponse};
    use crate::{RequestContent, RequestType};

    #[test]
    fn trigger() -> anyhow::Result<()> {
        let json = r#"
            {
	"forceScan": true
}
            "#;
        let expected = serde_json::from_str(json)?;

        let req = trigger_req(true)?;
        assert_eq!(req.path, "./autopilot/trigger");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        let json = r#"
        {
        "triggered": false
        }
        "#;
        let resp: TriggerResponse = serde_json::from_str(json)?;
        assert_eq!(resp.triggered, false);

        Ok(())
    }
}
