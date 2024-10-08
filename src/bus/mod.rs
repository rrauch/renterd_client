pub mod account;
pub mod alert;
pub mod autopilot;
pub mod bucket;
pub mod consensus;
pub mod contract;
pub mod host;
pub mod metrics;
pub mod object;
pub mod setting;
pub mod state;
pub mod stats;
pub mod syncer;
pub mod txpool;
pub mod wallet;
pub mod webhook;

use crate::bus::account::Api as AccountApi;
use crate::bus::alert::Api as AlertApi;
use crate::bus::autopilot::Api as AutopilotApi;
use crate::bus::bucket::Api as BucketApi;
use crate::bus::consensus::Api as ConsensusApi;
use crate::bus::contract::Api as ContractApi;
use crate::bus::host::Api as HostApi;
use crate::bus::metrics::Api as MetricsApi;
use crate::bus::object::Api as ObjectApi;
use crate::bus::setting::Api as SettingApi;
use crate::bus::state::Api as StateApi;
use crate::bus::stats::Api as StatsApi;
use crate::bus::syncer::Api as SyncerApi;
use crate::bus::txpool::Api as TxpoolApi;
use crate::bus::wallet::Api as WalletApi;
use crate::bus::webhook::Api as WebhookApi;
use crate::{ClientInner, Error};
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
    object: ObjectApi,
    setting: SettingApi,
    state: StateApi,
    stats: StatsApi,
    syncer: SyncerApi,
    txpool: TxpoolApi,
    wallet: WalletApi,
    webhook: WebhookApi,
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
            object: ObjectApi::new(inner.clone()),
            setting: SettingApi::new(inner.clone()),
            state: StateApi::new(inner.clone()),
            stats: StatsApi::new(inner.clone()),
            syncer: SyncerApi::new(inner.clone()),
            txpool: TxpoolApi::new(inner.clone()),
            wallet: WalletApi::new(inner.clone()),
            webhook: WebhookApi::new(inner.clone()),
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

    pub fn object(&self) -> &ObjectApi {
        &self.object
    }

    pub fn setting(&self) -> &SettingApi {
        &self.setting
    }

    pub async fn state(&self) -> Result<state::State, Error> {
        self.state.get().await
    }

    pub fn stats(&self) -> &StatsApi {
        &self.stats
    }

    pub fn syncer(&self) -> &SyncerApi {
        &self.syncer
    }

    pub fn txpool(&self) -> &TxpoolApi {
        &self.txpool
    }

    pub fn wallet(&self) -> &WalletApi {
        &self.wallet
    }

    pub fn webhook(&self) -> &WebhookApi {
        &self.webhook
    }
}
