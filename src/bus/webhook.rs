use crate::Error::InvalidDataError;
use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, RequestContent};
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

    pub async fn get_all(&self) -> Result<(Vec<Webhook>, Vec<Queue>), Error> {
        let resp: Response = self
            .inner
            .send_api_request(&get_all_req())
            .await?
            .json()
            .await?;

        Ok((resp.webhooks, resp.queues))
    }
    pub async fn register(&self, webhook: &Webhook) -> Result<(), Error> {
        let _ = self.inner.send_api_request(&register_req(webhook)?).await?;
        Ok(())
    }

    pub async fn delete(&self, webhook: &Webhook) -> Result<(), Error> {
        let _ = self.inner.send_api_request(&delete_req(webhook)?).await?;
        Ok(())
    }

    pub async fn broadcast(&self, event: &Event) -> Result<(), Error> {
        let _ = self.inner.send_api_request(&broadcast_req(event)?).await?;
        Ok(())
    }
}

fn get_all_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/webhooks").build()
}

fn register_req(webhook: &Webhook) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(webhook).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/webhooks")
        .content(content)
        .build())
}

fn delete_req(webhook: &Webhook) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(webhook).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/webhook/delete")
        .content(content)
        .build())
}

fn broadcast_req(event: &Event) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(event).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/webhooks/action")
        .content(content)
        .build())
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Response {
    #[serde(deserialize_with = "crate::deserialize_null_default")]
    webhooks: Vec<Webhook>,
    #[serde(deserialize_with = "crate::deserialize_null_default")]
    queues: Vec<Queue>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Module {
    Alerts,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    Register,
    Dismiss,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub module: Module,
    #[serde(deserialize_with = "crate::empty_string_as_none")]
    #[serde(serialize_with = "crate::none_as_empty_string")]
    #[serde(rename = "event")]
    pub event_type: Option<EventType>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub module: Module,
    #[serde(rename = "event")]
    pub event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Vec<String>>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub url: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use serde_json::Value;

    #[test]
    fn get_all() -> anyhow::Result<()> {
        let req = get_all_req();
        assert_eq!(req.path, "./bus/webhooks");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
{
	"webhooks": null,
	"queues": null
}
        "#;
        let resp: Response = serde_json::from_str(&json)?;
        assert_eq!(resp.webhooks.len(), 0);
        assert_eq!(resp.queues.len(), 0);

        let json = r#"
       {
  "webhooks": [
    {
      "module": "alerts",
      "event": "",
      "url": "http://192.168.1.174:8080/hooks"
    },
    {
      "module": "alerts",
      "event": "dismiss",
      "url": "http://192.168.1.174:8080/dismiss"
    }
  ],
  "queues": [
    {
      "url": "http://192.168.1.174:8080/hooks",
      "size": 2563
    }
  ]
}
"#;
        let resp: Response = serde_json::from_str(&json)?;
        let webhooks = resp.webhooks;
        let queues = resp.queues;
        assert_eq!(webhooks.len(), 2);
        assert_eq!(queues.len(), 1);

        assert_eq!(webhooks.get(0).unwrap().module, Module::Alerts);
        assert_eq!(webhooks.get(0).unwrap().event_type, None);
        assert_eq!(
            webhooks.get(1).unwrap().event_type,
            Some(EventType::Dismiss)
        );
        assert_eq!(
            webhooks.get(0).unwrap().url,
            "http://192.168.1.174:8080/hooks"
        );
        assert_eq!(webhooks.get(0).unwrap().headers, None);

        assert_eq!(queues.get(0).unwrap().size, 2563);
        assert_eq!(
            queues.get(0).unwrap().url,
            "http://192.168.1.174:8080/hooks"
        );
        Ok(())
    }

    #[test]
    fn register() -> anyhow::Result<()> {
        let json = r#"
        {
    "module": "alerts",
    "event": "",
    "url": "http://192.168.1.174:8080/hooks"
}
        "#;
        let expected: Value = serde_json::from_str(json)?;

        let req = register_req(&Webhook {
            module: Module::Alerts,
            event_type: None,
            url: "http://192.168.1.174:8080/hooks".to_string(),
            headers: None,
        })?;
        assert_eq!(req.path, "./bus/webhooks");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn delete() -> anyhow::Result<()> {
        let json = r#"
        {
    "module": "alerts",
    "event": "register",
    "url": "http://192.168.1.174:8080/hooks"
}
        "#;
        let expected: Value = serde_json::from_str(json)?;

        let req = delete_req(&Webhook {
            module: Module::Alerts,
            event_type: Some(EventType::Register),
            url: "http://192.168.1.174:8080/hooks".to_string(),
            headers: None,
        })?;
        assert_eq!(req.path, "./bus/webhook/delete");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn broadcast() -> anyhow::Result<()> {
        let json = r#"
        {
    "module": "alerts",
    "event": "dismiss",
    "payload": [
        "foo",
        "bar"
    ]
    }
        "#;
        let expected: Value = serde_json::from_str(json)?;

        let req = broadcast_req(&Event {
            module: Module::Alerts,
            event_type: EventType::Dismiss,
            payload: Some(vec!["foo".to_string(), "bar".to_string()]),
        })?;
        assert_eq!(req.path, "./bus/webhooks/action");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }
}
