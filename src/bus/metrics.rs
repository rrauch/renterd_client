use crate::{ClientInner, Error};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Api {
    contract: contract::Api,
    churn: churn::Api,
    contract_set: contract_set::Api,
    contract_prune: contract_prune::Api,
    wallet: wallet::Api,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            contract: contract::Api::new(inner.clone()),
            churn: churn::Api::new(inner.clone()),
            contract_set: contract_set::Api::new(inner.clone()),
            contract_prune: contract_prune::Api::new(inner.clone()),
            wallet: wallet::Api::new(inner.clone()),
        }
    }

    pub fn contract(&self) -> &contract::Api {
        &self.contract
    }

    pub fn churn(&self) -> &churn::Api {
        &self.churn
    }

    pub fn contract_set(&self) -> &contract_set::Api {
        &self.contract_set
    }

    pub fn contract_prune(&self) -> &contract_prune::Api {
        &self.contract_prune
    }

    pub fn wallet(&self) -> &wallet::Api {
        &self.wallet
    }
}

async fn list(
    inner: &ClientInner,
    key: &str,
    mut params: Vec<(&str, String)>,
    start: &DateTime<Utc>,
    interval: &Duration,
    number_intervals: u16,
) -> Result<Value, Error> {
    params.push(("start", start.to_rfc3339()));
    params.push(("interval", format!("{}", interval.as_millis())));
    params.push(("n", format!("{}", number_intervals)));
    let url = format!("./bus/metric/{}", key);
    inner.get_json(&url, Some(params)).await
}

pub mod contract {
    use crate::bus::metrics::list;
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error, FileContractId, PublicKey};
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::Deserialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(
            &self,
            contract_id: Option<FileContractId>,
            host_key: Option<PublicKey>,
            start: &DateTime<Utc>,
            interval: &Duration,
            number_intervals: u16,
        ) -> Result<Vec<Metric>, Error> {
            Ok(serde_json::from_value(
                list(
                    &self.inner,
                    "contract",
                    [
                        contract_id.map(|i| ("contractID", i.to_string())),
                        host_key.map(|k| ("hostKey", k.to_string())),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                    start,
                    interval,
                    number_intervals,
                )
                .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Metric {
        pub timestamp: DateTime<FixedOffset>,
        #[serde(rename = "contractID")]
        pub contract_id: FileContractId,
        pub host_key: PublicKey,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub remaining_collateral: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub remaining_funds: BigDecimal,
        pub revision_number: u64,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub upload_spending: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub download_spending: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub fund_account_spending: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub delete_spending: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub list_spending: BigDecimal,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn deserialize_list() -> anyhow::Result<()> {
            let json = r#"
            [
  {
    "timestamp": "2023-11-15T13:28:55.827Z",
    "contractID": "fcid:1d81af86ea9eb469a8e75dd2ac06634968b2b52b57a59b7f20cbbee027c8de51",
    "hostKey": "ed25519:09af708191b47e049a0b41dc499512d74ffb970dc734d23a4c31d0e2a51c82c7",
    "remainingCollateral": "1884119797797265750707921322",
    "remainingFunds": "736084597384116381740839188",
    "revisionNumber": 1038,
    "uploadSpending": "0",
    "downloadSpending": "0",
    "fundAccountSpending": "52911264215272148089095828",
    "deleteSpending": "0",
    "listSpending": "0"
  },
  {
    "timestamp": "2023-11-15T14:12:53.233Z",
    "contractID": "fcid:20b32f830c92cf3a50a194721d37d7de38e05093ee8a0bb367df9311babded7f",
    "hostKey": "ed25519:9501d2bc7d622f387c23630388e43339f02389aa45e709f9c5ef1a9ac51356b3",
    "remainingCollateral": "175701918250120093047546316",
    "remainingFunds": "75044554735529963303116337",
    "revisionNumber": 6068,
    "uploadSpending": "4952248376059614389469184",
    "downloadSpending": "0",
    "fundAccountSpending": "0",
    "deleteSpending": "0",
    "listSpending": "0"
  }
]
            "#;

            let metrics: Vec<Metric> = serde_json::from_str(&json)?;
            assert_eq!(metrics.len(), 2);

            assert_eq!(
                metrics.get(0).unwrap().timestamp,
                DateTime::parse_from_rfc3339("2023-11-15T13:28:55.827Z")?
            );

            assert_eq!(
                metrics.get(0).unwrap().contract_id,
                "fcid:1d81af86ea9eb469a8e75dd2ac06634968b2b52b57a59b7f20cbbee027c8de51"
                    .try_into()?
            );

            assert_eq!(
                metrics.get(0).unwrap().host_key,
                "ed25519:09af708191b47e049a0b41dc499512d74ffb970dc734d23a4c31d0e2a51c82c7"
                    .try_into()?
            );

            assert_eq!(
                metrics.get(1).unwrap().remaining_collateral,
                BigDecimal::from_str("175701918250120093047546316")?
            );

            assert_eq!(
                metrics.get(1).unwrap().remaining_funds,
                BigDecimal::from_str("75044554735529963303116337")?
            );

            assert_eq!(
                metrics.get(1).unwrap().upload_spending,
                BigDecimal::from_str("4952248376059614389469184")?
            );

            assert_eq!(metrics.get(1).unwrap().revision_number, 6068);

            Ok(())
        }
    }
}

pub mod churn {
    use crate::bus::metrics::list;
    use crate::deserialize_option_string;
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error, FileContractId};
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::Deserialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(
            &self,
            name: Option<String>,
            direction: Option<String>,
            reason: Option<String>,
            start: &DateTime<Utc>,
            interval: &Duration,
            number_intervals: u16,
        ) -> Result<Vec<Metric>, Error> {
            Ok(serde_json::from_value(
                list(
                    &self.inner,
                    "churn",
                    [
                        name.map(|n| ("name", n)),
                        direction.map(|d| ("direction", d)),
                        reason.map(|r| ("reason", r)),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                    start,
                    interval,
                    number_intervals,
                )
                .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Metric {
        pub direction: String,
        #[serde(rename = "contractID")]
        pub contract_id: FileContractId,
        pub name: String,
        #[serde(deserialize_with = "deserialize_option_string")]
        pub reason: Option<String>,
        pub timestamp: DateTime<FixedOffset>,
    }

    //todo: add tests when we have some test data
}

pub mod contract_set {
    use crate::bus::metrics::list;
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::Deserialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(
            &self,
            name: Option<String>,
            start: &DateTime<Utc>,
            interval: &Duration,
            number_intervals: u16,
        ) -> Result<Vec<Metric>, Error> {
            Ok(serde_json::from_value(
                list(
                    &self.inner,
                    "contractset",
                    [name.map(|n| ("name", n))].into_iter().flatten().collect(),
                    start,
                    interval,
                    number_intervals,
                )
                .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Metric {
        pub contracts: i64,
        pub name: String,
        pub timestamp: DateTime<FixedOffset>,
    }

    //todo: add tests when we have some test data
}

pub mod contract_prune {
    use crate::bus::metrics::list;
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error, FileContractId, PublicKey};
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::Deserialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(
            &self,
            contract_id: Option<FileContractId>,
            host_key: Option<PublicKey>,
            host_version: Option<String>,
            start: &DateTime<Utc>,
            interval: &Duration,
            number_intervals: u16,
        ) -> Result<Vec<Metric>, Error> {
            Ok(serde_json::from_value(
                list(
                    &self.inner,
                    "contractprune",
                    [
                        contract_id.map(|c| ("contractID", c.to_string())),
                        host_key.map(|h| ("hostKey", h.to_string())),
                        host_version.map(|h| ("hostVersion", h)),
                    ]
                    .into_iter()
                    .flatten()
                    .collect(),
                    start,
                    interval,
                    number_intervals,
                )
                .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Metric {
        pub timestamp: DateTime<FixedOffset>,
        #[serde(rename = "contractID")]
        pub contract_id: FileContractId,
        pub host_key: PublicKey,
        pub host_version: String,
        pub pruned: u64,
        pub remaining: u64,
        #[serde(with = "crate::duration_ns")]
        pub duration: Duration,
    }

    //todo: add tests when we have some test data
}

pub mod wallet {
    use crate::bus::metrics::list;
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::Deserialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(
            &self,
            start: &DateTime<Utc>,
            interval: &Duration,
            number_intervals: u16,
        ) -> Result<Vec<Metric>, Error> {
            Ok(serde_json::from_value(
                list(
                    &self.inner,
                    "wallet",
                    vec![],
                    start,
                    interval,
                    number_intervals,
                )
                .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Metric {
        pub timestamp: DateTime<FixedOffset>,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub confirmed: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub spendable: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub unconfirmed: BigDecimal,
    }

    //todo: add tests when we have some test data
}
