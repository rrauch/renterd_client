pub mod memory;

use crate::worker::memory::Api as MemoryApi;
use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct Worker {
    inner: Arc<ClientInner>,
    memory: MemoryApi,
}

impl Worker {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            inner: inner.clone(),
            memory: MemoryApi::new(inner.clone()),
        }
    }

    pub async fn id(&self) -> Result<String, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./worker/id", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }

    pub fn memory(&self) -> &MemoryApi {
        &self.memory
    }
}
