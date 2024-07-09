use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, RequestContent, RequestType};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
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

    pub async fn get_by_name<S: AsRef<str>>(&self, name: S) -> Result<Option<Bucket>, Error> {
        let url = format!("./bus/bucket/{}", name.as_ref());
        match self
            .inner
            .send_api_request(&url, &RequestType::Get(None), true)
            .await?
        {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    pub async fn create<S: AsRef<str>>(
        &self,
        name: S,
        public_read_access: bool,
    ) -> Result<(), Error> {
        let req = create_req(name.as_ref(), public_read_access)?;
        let _ = self
            .inner
            .send_api_request("./bus/buckets", &req, false)
            .await?;
        Ok(())
    }

    pub async fn update_policy<S: AsRef<str>>(
        &self,
        name: S,
        public_read_access: bool,
    ) -> Result<(), Error> {
        let url = format!("./bus/bucket/{}/policy", name.as_ref());
        let req = update_req(public_read_access)?;
        let _ = self.inner.send_api_request(&url, &req, false).await?;
        Ok(())
    }

    pub async fn delete<S: AsRef<str>>(&self, name: S) -> Result<(), Error> {
        let url = format!("./bus/bucket/{}", name.as_ref());
        let _ = self
            .inner
            .send_api_request(&url, &RequestType::Delete(None), false)
            .await?;
        Ok(())
    }
}

fn update_req(public_read_access: bool) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Put(
        Some(RequestContent::Json(
            serde_json::to_value(UpdatePolicyRequest {
                policy: &Policy { public_read_access },
            })
            .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePolicyRequest<'a> {
    policy: &'a Policy,
}

fn create_req(name: &str, public_read_access: bool) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(CreateRequest {
                name,
                policy: &Policy { public_read_access },
            })
            .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateRequest<'a> {
    name: &'a str,
    policy: &'a Policy,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub created_at: DateTime<FixedOffset>,
    pub name: String,
    pub policy: Policy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub public_read_access: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn list() -> anyhow::Result<()> {
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

    #[test]
    fn create() -> anyhow::Result<()> {
        let json = r#"
        {
    "name": "movies",
    "policy": {
      "publicReadAccess": false
    }
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        match create_req("movies", false)? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }

    #[test]
    fn update_policy() -> anyhow::Result<()> {
        let json = r#"
        {
    "policy": {
      "publicReadAccess": true
    }
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        match update_req(true)? {
            RequestType::Put(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }
}
