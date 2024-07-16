use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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

    pub(super) async fn get(&self) -> Result<Memory, Error> {
        Ok(self
            .inner
            .send_api_request(&get_req())
            .await?
            .json()
            .await?)
    }
}

fn get_req() -> ApiRequest {
    ApiRequestBuilder::get("./worker/memory").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Memory {
    pub download: MemoryStatus,
    pub upload: MemoryStatus,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MemoryStatus {
    pub available: u64,
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;

    #[test]
    fn get() -> anyhow::Result<()> {
        let req = get_req();
        assert_eq!(req.path, "./worker/memory");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
	"download": {
		"available": 1053741824,
		"total": 1073741824
	},
	"upload": {
		"available": 1063741824,
		"total": 1083741824
	}
}
        "#;

        let mem: Memory = serde_json::from_str(&json)?;
        assert_eq!(mem.download.available, 1053741824);
        assert_eq!(mem.download.total, 1073741824);
        assert_eq!(mem.upload.available, 1063741824);
        assert_eq!(mem.upload.total, 1083741824);

        Ok(())
    }
}
