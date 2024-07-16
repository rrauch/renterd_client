pub mod memory;
pub mod r#object;
pub mod state;
pub mod stats;

use crate::worker::memory::Api as MemoryApi;
use crate::worker::object::Api as ObjectApi;
use crate::worker::state::Api as StateApi;
use crate::worker::stats::Api as StatsApi;
use crate::{ApiRequestBuilder, ClientInner, Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct Worker {
    inner: Arc<ClientInner>,
    memory: MemoryApi,
    state: StateApi,
    stats: StatsApi,
    object: ObjectApi,
}

impl Worker {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            inner: inner.clone(),
            memory: MemoryApi::new(inner.clone()),
            state: StateApi::new(inner.clone()),
            stats: StatsApi::new(inner.clone()),
            object: ObjectApi::new(inner.clone()),
        }
    }

    pub async fn id(&self) -> Result<String, Error> {
        Ok(self
            .inner
            .send_api_request(&ApiRequestBuilder::get("./worker/id").build())
            .await?
            .json()
            .await?)
    }

    pub async fn memory(&self) -> Result<memory::Memory, Error> {
        self.memory.get().await
    }

    pub async fn state(&self) -> Result<state::State, Error> {
        self.state.get().await
    }

    pub fn stats(&self) -> &StatsApi {
        &self.stats
    }

    pub fn object(&self) -> &ObjectApi {
        &self.object
    }
}
