use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn address(&self) -> Result<String, Error> {
        Ok(self
            .inner
            .send_api_request(&address_req())
            .await?
            .json()
            .await?)
    }

    pub async fn peers(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .inner
            .send_api_request(&peers_req())
            .await?
            .json()
            .await?)
    }
}

fn address_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/syncer/address").build()
}

fn peers_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/syncer/peers").build()
}

#[cfg(test)]
mod tests {
    use crate::bus::syncer::{address_req, peers_req};
    use crate::RequestType;

    #[test]
    fn address() -> anyhow::Result<()> {
        let req = address_req();
        assert_eq!(req.path, "./bus/syncer/address");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        "127.102.123.11:9881""#;
        let address: String = serde_json::from_str(&json)?;
        assert_eq!(address, "127.102.123.11:9881");
        Ok(())
    }

    #[test]
    fn peers() -> anyhow::Result<()> {
        let req = peers_req();
        assert_eq!(req.path, "./bus/syncer/peers");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        [
	"127.81.56.1:11081",
	"127.172.172.2:9881",
	"127.85.181.3:30023",
	"127.60.251.4:9881",
	"127.19.232.5:9881",
	"127.53.18.6:9881",
	"127.81.56.7:9881",
	"127.6.48.8:9881"
]
        "#;
        let peers: Vec<String> = serde_json::from_str(&json)?;
        assert_eq!(peers.len(), 8);
        assert_eq!(peers.get(0).unwrap(), "127.81.56.1:11081");
        assert_eq!(peers.get(3).unwrap(), "127.60.251.4:9881");
        assert_eq!(peers.get(7).unwrap(), "127.6.48.8:9881");
        Ok(())
    }
}
