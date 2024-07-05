pub mod account;
pub mod alert;
pub mod autopilot;
pub mod bucket;
pub mod consensus;
pub mod contract;
pub mod host;
pub mod metrics;

use crate::bus::account::Api as AccountApi;
use crate::bus::alert::Api as AlertApi;
use crate::bus::autopilot::Api as AutopilotApi;
use crate::bus::bucket::Api as BucketApi;
use crate::bus::consensus::Api as ConsensusApi;
use crate::bus::contract::Api as ContractApi;
use crate::bus::host::Api as HostApi;
use crate::bus::metrics::Api as MetricsApi;
use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Bus {
    account: AccountApi,
    alert: AlertApi,
    autopilot: AutopilotApi,
    bucket: BucketApi,
    consensus: ConsensusApi,
    contract: ContractApi,
    host: HostApi,
    metrics: MetricsApi,
}

impl Bus {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            account: AccountApi::new(inner.clone()),
            alert: AlertApi::new(inner.clone()),
            autopilot: AutopilotApi::new(inner.clone()),
            bucket: BucketApi::new(inner.clone()),
            consensus: ConsensusApi::new(inner.clone()),
            contract: ContractApi::new(inner.clone()),
            host: HostApi::new(inner.clone()),
            metrics: MetricsApi::new(inner.clone()),
        }
    }

    pub fn account(&self) -> &AccountApi {
        &self.account
    }

    pub fn alert(&self) -> &AlertApi {
        &self.alert
    }

    pub fn autopilot(&self) -> &AutopilotApi {
        &self.autopilot
    }

    pub fn bucket(&self) -> &BucketApi {
        &self.bucket
    }

    pub fn consensus(&self) -> &ConsensusApi {
        &self.consensus
    }

    pub fn contract(&self) -> &ContractApi {
        &self.contract
    }

    pub fn host(&self) -> &HostApi {
        &self.host
    }

    pub fn metrics(&self) -> &MetricsApi {
        &self.metrics
    }
}
