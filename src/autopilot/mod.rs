pub mod config;

use crate::autopilot::config::Api as ConfigApi;
use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Autopilot {
    config: ConfigApi,
}

impl Autopilot {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            config: ConfigApi::new(inner.clone()),
        }
    }

    pub fn config(&self) -> &ConfigApi {
        &self.config
    }
}
