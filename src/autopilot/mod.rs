pub mod config;
mod state;

use crate::autopilot::config::Api as ConfigApi;
use crate::autopilot::state::Api as StateApi;
use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Autopilot {
    config: ConfigApi,
    state: StateApi,
}

impl Autopilot {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            config: ConfigApi::new(inner.clone()),
            state: StateApi::new(inner.clone()),
        }
    }

    pub fn config(&self) -> &ConfigApi {
        &self.config
    }
}
