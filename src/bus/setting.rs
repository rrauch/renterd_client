use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    contract_set: contract_set::Api,
    gouging: gouging::Api,
    redundancy: redundancy::Api,
    s3_authentication: s3_authentication::Api,
    upload_packing: upload_packing::Api,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            contract_set: contract_set::Api::new(inner.clone()),
            gouging: gouging::Api::new(inner.clone()),
            redundancy: redundancy::Api::new(inner.clone()),
            s3_authentication: s3_authentication::Api::new(inner.clone()),
            upload_packing: upload_packing::Api::new(inner.clone()),
        }
    }

    pub fn contract_set(&self) -> &contract_set::Api {
        &self.contract_set
    }

    pub fn gouging(&self) -> &gouging::Api {
        &self.gouging
    }

    pub fn redundancy(&self) -> &redundancy::Api {
        &self.redundancy
    }

    pub fn s3_authentication(&self) -> &s3_authentication::Api {
        &self.s3_authentication
    }

    pub fn upload_packing(&self) -> &upload_packing::Api {
        &self.upload_packing
    }
}

pub mod contract_set {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(&self) -> Result<Settings, Error> {
            Ok(serde_json::from_value(
                self.inner
                    .get_json("./bus/setting/contractset", None)
                    .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub default: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize() -> anyhow::Result<()> {
            let json = r#"
            {
	"default": "autopilot"
}
            "#;

            let settings: Settings = serde_json::from_str(&json)?;
            assert_eq!(settings.default, "autopilot");
            Ok(())
        }
    }
}

pub mod gouging {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use bigdecimal::BigDecimal;
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

        pub async fn list(&self) -> Result<Settings, Error> {
            Ok(
                serde_json::from_value(self.inner.get_json("./bus/setting/gouging", None).await?)
                    .map_err(|e| InvalidDataError(e.into()))?,
            )
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        #[serde(rename = "maxRPCPrice")]
        #[serde(with = "bigdecimal::serde::json_num")]
        pub max_rpc_price: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub max_contract_price: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub max_download_price: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub max_upload_price: BigDecimal,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub max_storage_price: BigDecimal,
        pub host_block_height_leeway: u32,
        #[serde(with = "crate::duration_ns")]
        pub min_price_table_validity: Duration,
        #[serde(with = "crate::duration_ns")]
        pub min_account_expiry: Duration,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub min_max_ephemeral_account_balance: BigDecimal,
        pub migration_surcharge_multiplier: u64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn deserialize() -> anyhow::Result<()> {
            let json = r#"
            {
	"hostBlockHeightLeeway": 6,
	"maxContractPrice": "15000000000000000000000000",
	"maxDownloadPrice": "1000000000000000000000000000",
	"maxRPCPrice": "1000000000000000000000",
	"maxStoragePrice": "69444444444",
	"maxUploadPrice": "100000000000000000000000000",
	"migrationSurchargeMultiplier": 10,
	"minAccountExpiry": 86400000000000,
	"minMaxEphemeralAccountBalance": "1000000000000000000000000",
	"minPriceTableValidity": 300000000000
}
            "#;

            let settings: Settings = serde_json::from_str(&json)?;
            assert_eq!(settings.host_block_height_leeway, 6);
            assert_eq!(settings.migration_surcharge_multiplier, 10);
            assert_eq!(
                settings.max_download_price,
                BigDecimal::from_str("1000000000000000000000000000")?
            );
            assert_eq!(
                settings.min_max_ephemeral_account_balance,
                BigDecimal::from_str("1000000000000000000000000")?
            );
            assert_eq!(
                settings.max_storage_price,
                BigDecimal::from_str("69444444444")?
            );
            assert_eq!(
                settings.min_account_expiry,
                Duration::from_nanos(86400000000000),
            );
            assert_eq!(
                settings.min_price_table_validity,
                Duration::from_nanos(300000000000),
            );

            Ok(())
        }
    }
}

pub mod redundancy {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(&self) -> Result<Settings, Error> {
            Ok(serde_json::from_value(
                self.inner
                    .get_json("./bus/setting/redundancy", None)
                    .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub min_shards: u64,
        pub total_shards: u64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize() -> anyhow::Result<()> {
            let json = r#"
            {
	"minShards": 2,
	"totalShards": 6
}
            "#;

            let settings: Settings = serde_json::from_str(&json)?;
            assert_eq!(settings.min_shards, 2);
            assert_eq!(settings.total_shards, 6);
            Ok(())
        }
    }
}

pub mod s3_authentication {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(&self) -> Result<Settings, Error> {
            Ok(serde_json::from_value(
                self.inner
                    .get_json("./bus/setting/s3authentication", None)
                    .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub v4_keypairs: BTreeMap<String, String>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize() -> anyhow::Result<()> {
            let json = r#"
            {
	"v4Keypairs": {
        "foo_key" : "foo_value",
        "bar_key" : "bar_value"
    }
}
            "#;

            let settings: Settings = serde_json::from_str(&json)?;
            assert_eq!(settings.v4_keypairs.len(), 2);
            assert_eq!(settings.v4_keypairs.get("foo_key").unwrap(), "foo_value");
            assert_eq!(settings.v4_keypairs.get("bar_key").unwrap(), "bar_value");
            Ok(())
        }
    }
}

pub mod upload_packing {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct Api {
        inner: Arc<ClientInner>,
    }

    impl Api {
        pub(super) fn new(inner: Arc<ClientInner>) -> Self {
            Self { inner }
        }

        pub async fn list(&self) -> Result<Settings, Error> {
            Ok(serde_json::from_value(
                self.inner
                    .get_json("./bus/setting/uploadpacking", None)
                    .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub enabled: bool,
        pub slab_buffer_max_size_soft: i64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize() -> anyhow::Result<()> {
            let json = r#"
            {
	"enabled": true,
	"slabBufferMaxSizeSoft": 4294967296
}
            "#;

            let settings: Settings = serde_json::from_str(&json)?;
            assert_eq!(settings.enabled, true);
            assert_eq!(settings.slab_buffer_max_size_soft, 4294967296);
            Ok(())
        }
    }
}
