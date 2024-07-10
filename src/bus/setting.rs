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
    use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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
            Ok(self
                .inner
                .send_api_request(&list_req())
                .await?
                .json()
                .await?)
        }
    }

    fn list_req() -> ApiRequest {
        ApiRequestBuilder::get("./bus/setting/contractset").build()
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub default: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn list() -> anyhow::Result<()> {
            let req = list_req();
            assert_eq!(req.path, "./bus/setting/contractset");
            assert_eq!(req.request_type, RequestType::Get);
            assert_eq!(req.params, None);
            assert_eq!(req.content, None);

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
    use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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
            Ok(self
                .inner
                .send_api_request(&list_req())
                .await?
                .json()
                .await?)
        }
    }

    fn list_req() -> ApiRequest {
        ApiRequestBuilder::get("./bus/setting/gouging").build()
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        #[serde(rename = "maxRPCPrice")]
        #[serde(with = "crate::number_as_string")]
        pub max_rpc_price: u128,
        #[serde(with = "crate::number_as_string")]
        pub max_contract_price: u128,
        #[serde(with = "crate::number_as_string")]
        pub max_download_price: u128,
        #[serde(with = "crate::number_as_string")]
        pub max_upload_price: u128,
        #[serde(with = "crate::number_as_string")]
        pub max_storage_price: u128,
        pub host_block_height_leeway: u32,
        #[serde(with = "crate::duration_ns")]
        pub min_price_table_validity: Duration,
        #[serde(with = "crate::duration_ns")]
        pub min_account_expiry: Duration,
        #[serde(with = "crate::number_as_string")]
        pub min_max_ephemeral_account_balance: u128,
        pub migration_surcharge_multiplier: u64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn list() -> anyhow::Result<()> {
            let req = list_req();
            assert_eq!(req.path, "./bus/setting/gouging");
            assert_eq!(req.request_type, RequestType::Get);
            assert_eq!(req.params, None);
            assert_eq!(req.content, None);

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
            assert_eq!(settings.max_download_price, 1000000000000000000000000000);
            assert_eq!(
                settings.min_max_ephemeral_account_balance,
                1000000000000000000000000
            );
            assert_eq!(settings.max_storage_price, 69444444444);
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
    use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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
            Ok(self
                .inner
                .send_api_request(&list_req())
                .await?
                .json()
                .await?)
        }
    }

    fn list_req() -> ApiRequest {
        ApiRequestBuilder::get("./bus/setting/redundancy").build()
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
        use crate::RequestType;

        #[test]
        fn list() -> anyhow::Result<()> {
            let req = list_req();
            assert_eq!(req.path, "./bus/setting/redundancy");
            assert_eq!(req.request_type, RequestType::Get);
            assert_eq!(req.params, None);
            assert_eq!(req.content, None);

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
    use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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
            Ok(self
                .inner
                .send_api_request(&list_req())
                .await?
                .json()
                .await?)
        }
    }

    fn list_req() -> ApiRequest {
        ApiRequestBuilder::get("./bus/setting/s3authentication").build()
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Settings {
        pub v4_keypairs: BTreeMap<String, String>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn list() -> anyhow::Result<()> {
            let req = list_req();
            assert_eq!(req.path, "./bus/setting/s3authentication");
            assert_eq!(req.request_type, RequestType::Get);
            assert_eq!(req.params, None);
            assert_eq!(req.content, None);

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
    use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
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
            Ok(self
                .inner
                .send_api_request(&list_req())
                .await?
                .json()
                .await?)
        }
    }

    fn list_req() -> ApiRequest {
        ApiRequestBuilder::get("./bus/setting/uploadpacking").build()
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
        use crate::RequestType;

        #[test]
        fn list() -> anyhow::Result<()> {
            let req = list_req();
            assert_eq!(req.path, "./bus/setting/uploadpacking");
            assert_eq!(req.request_type, RequestType::Get);
            assert_eq!(req.params, None);
            assert_eq!(req.content, None);

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
