use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
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

    pub async fn list(&self) -> Result<(Vec<Webhook>, Vec<Queue>), Error> {
        let resp: Response =
            serde_json::from_value(self.inner.get_json("./bus/webhooks", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?;

        Ok((resp.webhooks, resp.queues))
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Response {
    webhooks: Vec<Webhook>,
    queues: Vec<Queue>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Webhook {
    pub module: String,
    #[serde(deserialize_with = "crate::deserialize_option_string")]
    pub event: Option<String>,
    pub url: String,
    pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Queue {
    pub url: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
       {
  "webhooks": [
    {
      "module": "alerts",
      "event": "",
      "url": "http://192.168.1.174:8080/hooks"
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
        assert_eq!(webhooks.len(), 1);
        assert_eq!(queues.len(), 1);

        assert_eq!(webhooks.get(0).unwrap().module, "alerts");
        assert_eq!(webhooks.get(0).unwrap().event, None);
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
}
