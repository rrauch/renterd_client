pub mod config;
pub mod state;
pub mod trigger;

use crate::autopilot::config::Api as ConfigApi;
use crate::autopilot::state::{Api as StateApi, State};
use crate::autopilot::trigger::Api as TriggerApi;
use crate::{ClientInner, Error};
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

    pub async fn state(&self) -> Result<State, Error> {
        self.state.get().await
    }

    pub async fn trigger(&self, force_scan: bool) -> Result<bool, Error> {
        self.trigger.trigger(force_scan).await
    }
}
