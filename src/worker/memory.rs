use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
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

    pub async fn list(&self) -> Result<Memory, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./worker/memory", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
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

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
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
