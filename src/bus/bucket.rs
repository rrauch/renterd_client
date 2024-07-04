use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
use chrono::{DateTime, FixedOffset};
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

    pub async fn list(&self) -> Result<Vec<Bucket>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/buckets", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Bucket {
    pub created_at: DateTime<FixedOffset>,
    pub name: String,
    pub policy: Policy,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Policy {
    pub public_read_access: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
        [
  {
    "createdAt": "2023-09-05T16:01:33.620354105Z",
    "name": "default",
    "policy": {
      "publicReadAccess": false
    }
  },
  {
    "createdAt": "2023-09-19T16:03:02.737150758Z",
    "name": "photos",
    "policy": {
      "publicReadAccess": false
    }
  },
  {
    "createdAt": "2023-09-19T16:03:13.684005651Z",
    "name": "backups",
    "policy": {
      "publicReadAccess": false
    }
  },
  {
    "createdAt": "2023-09-22T19:30:21.728956389Z",
    "name": "test",
    "policy": {
      "publicReadAccess": true
    }
  }
]
        "#;

        let buckets: Vec<Bucket> = serde_json::from_str(&json)?;
        assert_eq!(4, buckets.len());

        assert_eq!(
            buckets.get(0).unwrap().created_at,
            DateTime::parse_from_rfc3339("2023-09-05T16:01:33.620354105Z")?
        );

        assert_eq!(buckets.get(1).unwrap().name, "photos");
        assert_eq!(buckets.get(2).unwrap().policy.public_read_access, false);
        assert_eq!(buckets.get(3).unwrap().policy.public_read_access, true);

        Ok(())
    }
}
