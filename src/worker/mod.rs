pub mod memory;
pub mod state;
pub mod stats;

use crate::worker::memory::Api as MemoryApi;
use crate::worker::state::Api as StateApi;
use crate::worker::stats::Api as StatsApi;
use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct Worker {
    inner: Arc<ClientInner>,
    memory: MemoryApi,
    state: StateApi,
    stats: StatsApi,
}

impl Worker {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            inner: inner.clone(),
            memory: MemoryApi::new(inner.clone()),
            state: StateApi::new(inner.clone()),
            stats: StatsApi::new(inner.clone()),
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

    pub fn state(&self) -> &StateApi {
        &self.state
    }

    pub fn stats(&self) -> &StatsApi {
        &self.stats
    }
}
