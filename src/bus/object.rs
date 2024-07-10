use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error};
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_json::Value;
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

    pub async fn list<S: AsRef<str>>(
        &self,
        path: S,
        bucket: Option<String>,
        prefix: Option<String>,
        offset: Option<usize>,
        marker: Option<String>,
        limit: Option<usize>,
    ) -> Result<(Option<Object>, Option<Vec<Metadata>>, bool), Error> {
        match self
            .inner
            .send_api_request_optional(&list_req(path, bucket, prefix, offset, marker, limit))
            .await?
        {
            Some(resp) => {
                let response: ListResponse = resp.json().await?;
                Ok((response.object, response.entries, response.has_more))
            }
            None => Ok((None, None, false)),
        }
    }
}

fn list_req<S: AsRef<str>>(
    path: S,
    bucket: Option<String>,
    prefix: Option<String>,
    offset: Option<usize>,
    marker: Option<String>,
    limit: Option<usize>,
) -> ApiRequest {
    let path = encode_path(path);
    let params: Vec<_> = [
        bucket.map(|b| ("bucket", b)),
        prefix.map(|p| ("prefix", p)),
        offset.map(|o| ("offset", format!("{}", o))),
        marker.map(|m| ("marker", m)),
        limit.map(|l| ("limit", format!("{}", l))),
    ]
    .into_iter()
    .flatten()
    .collect();
    let params = (!params.is_empty()).then(|| params);

    ApiRequestBuilder::get(path).params(params).build()
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct ListResponse {
    entries: Option<Vec<Metadata>>,
    object: Option<Object>,
    has_more: bool,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Metadata {
    #[serde(rename = "eTag")]
    pub etag: Option<String>,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub health: BigDecimal,
    pub mod_time: DateTime<FixedOffset>,
    pub name: String,
    pub size: u64,
    pub mime_type: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Object {
    #[serde(rename = "metadata")]
    pub user_metadata: Option<BTreeMap<String, String>>,
    pub key: Option<String>,       //todo: implement `EncryptionKey` type
    pub slabs: Option<Vec<Value>>, //todo: implement Slab types
    #[serde(flatten)]
    pub metadata: Metadata,
}

fn encode_path<S: AsRef<str>>(path: S) -> String {
    format!(
        "./bus/objects/{}",
        urlencoding::encode(path.as_ref().trim_start_matches('/'))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    //todo: test list_req

    #[test]
    fn deserialize_list_dir() -> anyhow::Result<()> {
        let json = r#"
        {
	"hasMore": false,
	"entries": [
		{
			"eTag": "d41d8cd98f00b204e9800998ecf8427e",
			"health": 1,
			"modTime": "2024-07-05T12:37:58.998523074Z",
			"name": "/foo/",
			"size": 5586849,
			"mimeType": "text/plain"
		}
	]
}
        "#;

        let resp: ListResponse = serde_json::from_str(&json)?;
        assert!(resp.entries.is_some());
        let entries = resp.entries.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.get(0).unwrap().name, "/foo/");
        assert_eq!(
            entries.get(0).unwrap().mod_time,
            DateTime::parse_from_rfc3339("2024-07-05T12:37:58.998523074Z")?
        );
        assert_eq!(entries.get(0).unwrap().size, 5586849);
        Ok(())
    }

    #[test]
    fn deserialize_list_file() -> anyhow::Result<()> {
        let json = r#"
        {
	"hasMore": false,
	"object": {
		"eTag": "322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9",
		"health": 1,
		"modTime": "2024-06-27T11:56:19.05151211Z",
		"name": "/foo/bar/test.zip",
		"size": 3657244,
		"key": "key:aba60a4c1b9ff360214a68f09f890f9afc00d1bf23c8c9435a02311b10ff1d61",
		"slabs": [
			{
				"slab": {
					"health": 1,
					"key": "key:6317e69fb2048ed2137e245b19b91b6f037d929db17c0d9a70cb47be3544b2af",
					"minShards": 2
				},
				"offset": 0,
				"length": 3657244
			}
		]
	}
}
        "#;

        let resp: ListResponse = serde_json::from_str(&json)?;
        assert!(resp.object.is_some());
        let object = resp.object.unwrap();
        assert_eq!(object.metadata.name, "/foo/bar/test.zip");
        assert_eq!(object.metadata.size, 3657244);
        assert_eq!(
            object.metadata.etag,
            Some("322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9".to_string())
        );
        assert_eq!(object.metadata.health, BigDecimal::from_str("1")?);

        //todo: test slabs
        Ok(())
    }
}
