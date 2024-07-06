use crate::ClientInner;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    objects: objects::Api,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self {
            objects: objects::Api::new(inner.clone()),
        }
    }

    pub fn objects(&self) -> &objects::Api {
        &self.objects
    }
}

pub mod objects {
    use crate::Error::InvalidDataError;
    use crate::{ClientInner, Error};
    use bigdecimal::BigDecimal;
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

        pub async fn list(&self, bucket: Option<String>) -> Result<Stats, Error> {
            Ok(serde_json::from_value(
                self.inner
                    .get_json("./bus/stats/objects", bucket.map(|b| vec![("bucket", b)]))
                    .await?,
            )
            .map_err(|e| InvalidDataError(e.into()))?)
        }
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(rename_all(deserialize = "camelCase"))]
    pub struct Stats {
        pub num_objects: u64,
        pub num_unfinished_objects: u64,
        #[serde(with = "bigdecimal::serde::json_num")]
        pub min_health: BigDecimal,
        pub total_objects_size: u64,
        pub total_unfinished_objects_size: u64,
        pub total_sectors_size: u64,
        pub total_uploaded_size: u64,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn deserialize_list() -> anyhow::Result<()> {
            let json = r#"
{
	"numObjects": 8,
	"numUnfinishedObjects": 0,
	"minHealth": 1,
	"totalObjectsSize": 5586849,
	"totalUnfinishedObjectsSize": 0,
	"totalSectorsSize": 0,
	"totalUploadedSize": 0
}
        "#;
            let stats: Stats = serde_json::from_str(&json)?;
            assert_eq!(stats.num_objects, 8);
            assert_eq!(stats.num_unfinished_objects, 0);
            assert_eq!(stats.min_health, BigDecimal::from_str("1")?);
            assert_eq!(stats.total_objects_size, 5586849);
            assert_eq!(stats.total_unfinished_objects_size, 0);
            assert_eq!(stats.total_sectors_size, 0);
            assert_eq!(stats.total_uploaded_size, 0);
            Ok(())
        }
    }
}
