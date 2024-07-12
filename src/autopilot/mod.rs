pub mod config;
pub mod state;
pub mod trigger;

use crate::autopilot::config::Api as ConfigApi;
use crate::autopilot::state::Api as StateApi;
use crate::autopilot::trigger::Api as TriggerApi;
use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Autopilot {
    config: ConfigApi,
    state: StateApi,
    trigger: TriggerApi,
}

impl Autopilot {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            config: ConfigApi::new(inner.clone()),
            state: StateApi::new(inner.clone()),
            trigger: TriggerApi::new(inner.clone()),
        }
    }

    pub fn config(&self) -> &ConfigApi {
        &self.config
    }

    pub fn state(&self) -> &StateApi {
        &self.state
    }

    pub fn trigger(&self) -> &TriggerApi {
        &self.trigger
    }
}
