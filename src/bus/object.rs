use crate::Error::InvalidDataError;
use crate::{
    encode_object_path, ApiRequest, ApiRequestBuilder, ClientInner, Error, Percentage,
    RequestContent,
};
use chrono::{DateTime, FixedOffset};
use either::Either;
use futures::{StreamExt, TryStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn get<S: AsRef<str>>(
        &self,
        path: S,
        bucket: Option<String>,
        prefix: Option<String>,
        offset: Option<usize>,
        marker: Option<String>,
        limit: Option<usize>,
    ) -> Result<Option<Either<Object, (Vec<Metadata>, bool)>>, Error> {
        Ok(
            match self
                .inner
                .send_api_request_optional(get_req(path, bucket, prefix, offset, marker, limit))
                .await?
            {
                Some(resp) => {
                    let resp: GetResponse = resp.json().await?;
                    Some(if resp.object.is_some() {
                        Either::Left(resp.object.unwrap())
                    } else {
                        Either::Right((resp.entries.unwrap_or(vec![]), resp.has_more))
                    })
                }
                None => None,
            },
        )
    }

    pub async fn get_stream<S: ToString>(
        &self,
        path: S,
        batch_size: NonZeroUsize,
        prefix: Option<String>,
        bucket: Option<String>,
    ) -> Result<
        Option<Either<Object, impl TryStream<Ok = Vec<Metadata>, Error = Error> + Send + Unpin>>,
        Error,
    > {
        let path = path.to_string();
        let batch_size = batch_size.get();
        let inner = self.inner.clone();

        let (objects, has_more) = match _get(
            &inner,
            path.as_str(),
            bucket.clone(),
            prefix.clone(),
            Some(0),
            None,
            Some(batch_size),
        )
        .await?
        {
            None => {
                return Ok(None);
            }
            Some(Either::Left(object)) => {
                return Ok(Some(Either::Left(object)));
            }
            Some(Either::Right((objects, has_more))) => {
                let objects = if objects.is_empty() {
                    None
                } else {
                    Some(objects)
                };
                (objects, has_more)
            }
        };

        let initial_state = (objects, has_more, 0usize);

        Ok(Some(Either::Right(
            futures::stream::try_unfold(initial_state, move |state| {
                let inner = inner.clone();
                let path = path.clone();
                let prefix = prefix.clone();
                let bucket = bucket.clone();

                async move {
                    let objects = state.0;
                    let has_more = state.1;
                    let mut offset = state.2;

                    if let Some(objects) = objects {
                        // initial objects not yet returned
                        offset += objects.len();
                        return Ok(Some((objects, (None, has_more, offset))));
                    }

                    if !has_more {
                        // end of stream reached
                        return Ok(None);
                    }

                    match _get(
                        &inner,
                        path.as_str(),
                        bucket.clone(),
                        prefix.clone(),
                        Some(offset),
                        None,
                        Some(batch_size),
                    )
                    .await?
                    {
                        Some(Either::Right((objects, has_more))) => {
                            if objects.is_empty() {
                                Ok(None)
                            } else {
                                offset += objects.len();
                                Ok(Some((objects, (None, has_more, offset))))
                            }
                        }
                        _ => Err(Error::UnexpectedResponse(
                            "expected a directory listing".to_string(),
                        )),
                    }
                }
            })
            .boxed(),
        )))
    }

    pub fn list(
        &self,
        batch_size: NonZeroUsize,
        prefix: Option<String>,
        bucket: Option<String>,
    ) -> Result<impl TryStream<Ok = Vec<Metadata>, Error = Error> + Send + Unpin, Error> {
        let inner = self.inner.clone();
        let batch_size = batch_size.get();
        let initial_state = (true, None);

        Ok(futures::stream::try_unfold(initial_state, move |state| {
            let inner = inner.clone();
            let prefix = prefix.clone();
            let bucket = bucket.clone();

            async move {
                let has_more = state.0;
                let next_marker = state.1;
                if !has_more {
                    return Ok(None);
                }

                let (objects, has_more, next_marker) = {
                    let resp: ListResponse = inner
                        .send_api_request(list_req(
                            prefix.clone(),
                            bucket.clone(),
                            next_marker,
                            batch_size,
                        )?)
                        .await?
                        .json()
                        .await?;
                    (resp.objects, resp.has_more, resp.next_marker)
                };

                if objects.is_empty() {
                    return Ok(None);
                }

                Ok(Some((objects, (has_more, next_marker))))
            }
        })
        .boxed())
    }

    //todo: implement PUT /objects/*key function

    pub async fn delete<S: AsRef<str>>(
        &self,
        path: S,
        bucket: Option<String>,
        batch: bool,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(delete_req(path, bucket, batch))
            .await?;
        Ok(())
    }

    pub async fn copy(
        &self,
        source_path: String,
        source_bucket: String,
        destination_path: String,
        destination_bucket: String,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(copy_req(
                source_path,
                source_bucket,
                destination_path,
                destination_bucket,
            )?)
            .await?;
        //todo: check if renterd actually responds with content here contrary to the docs
        Ok(())
    }

    pub async fn rename(
        &self,
        from: String,
        to: String,
        bucket: String,
        force: bool,
        mode: RenameMode,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(rename_req(from, to, bucket, force, mode)?)
            .await?;
        Ok(())
    }

    pub async fn search(
        &self,
        key: Option<String>,
        bucket: Option<String>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Metadata>, Error> {
        Ok(self
            .inner
            .send_api_request(search_req(key, bucket, offset, limit))
            .await?
            .json()
            .await?)
    }
}

async fn _get<S: AsRef<str>>(
    inner: &ClientInner,
    path: S,
    bucket: Option<String>,
    prefix: Option<String>,
    offset: Option<usize>,
    marker: Option<String>,
    limit: Option<usize>,
) -> Result<Option<Either<Object, (Vec<Metadata>, bool)>>, Error> {
    Ok(
        match inner
            .send_api_request_optional(get_req(path, bucket, prefix, offset, marker, limit))
            .await?
        {
            Some(resp) => {
                let resp: GetResponse = resp.json().await?;
                Some(if resp.object.is_some() {
                    Either::Left(resp.object.unwrap())
                } else {
                    Either::Right((resp.entries.unwrap_or(vec![]), resp.has_more))
                })
            }
            None => None,
        },
    )
}

fn list_req(
    prefix: Option<String>,
    bucket: Option<String>,
    marker: Option<String>,
    limit: usize,
) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(ListRequest {
            prefix,
            marker,
            bucket,
            limit,
        })
        .map_err(|e| InvalidDataError(e.into()))?,
    ));

    Ok(ApiRequestBuilder::post("./bus/objects/list")
        .content(content)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket: Option<String>,
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListResponse {
    has_more: bool,
    #[serde(deserialize_with = "crate::empty_string_as_none")]
    next_marker: Option<String>,
    #[serde(deserialize_with = "crate::deserialize_null_default")]
    objects: Vec<Metadata>,
}

fn search_req(
    key: Option<String>,
    bucket: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> ApiRequest {
    let params: Vec<_> = [
        key.map(|k| ("key", k)),
        bucket.map(|b| ("bucket", b)),
        offset.map(|o| ("offset", format!("{}", o))),
        limit.map(|l| ("limit", format!("{}", l))),
    ]
    .into_iter()
    .flatten()
    .collect();
    let params = (!params.is_empty()).then(|| params);
    ApiRequestBuilder::get("./bus/search/objects")
        .params(params)
        .build()
}

fn rename_req(
    from: String,
    to: String,
    bucket: String,
    force: bool,
    mode: RenameMode,
) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(RenameRequest {
            bucket,
            force,
            from,
            to,
            mode,
        })
        .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/objects/rename")
        .content(content)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameRequest {
    bucket: String,
    force: bool,
    from: String,
    to: String,
    mode: RenameMode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenameMode {
    Single,
    Multi,
}

fn copy_req(
    source_path: String,
    source_bucket: String,
    destination_path: String,
    destination_bucket: String,
) -> Result<ApiRequest, Error> {
    let content = Some(RequestContent::Json(
        serde_json::to_value(CopyRequest {
            source_bucket,
            source_path,
            destination_bucket,
            destination_path,
        })
        .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/objects/copy")
        .content(content)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CopyRequest {
    source_bucket: String,
    source_path: String,
    destination_bucket: String,
    destination_path: String,
    //todo: clarify use of `mimeType` and `metadata` fields
}

fn delete_req<S: AsRef<str>>(path: S, bucket: Option<String>, batch: bool) -> ApiRequest {
    let url = encode_object_path(path, "./bus/objects");
    let mut params = Vec::with_capacity(2);
    if let Some(bucket) = bucket {
        params.push(("bucket", bucket))
    };
    params.push(("batch", batch.to_string()));
    ApiRequestBuilder::delete(url).params(Some(params)).build()
}

fn get_req<S: AsRef<str>>(
    path: S,
    bucket: Option<String>,
    prefix: Option<String>,
    offset: Option<usize>,
    marker: Option<String>,
    limit: Option<usize>,
) -> ApiRequest {
    let path = encode_object_path(path, "./bus/objects");
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
struct GetResponse {
    entries: Option<Vec<Metadata>>,
    object: Option<Object>,
    has_more: bool,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Metadata {
    #[serde(rename = "eTag")]
    pub etag: Option<String>,
    #[serde(deserialize_with = "crate::deserialize_percentage_from_decimal")]
    pub health: Percentage,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn get_dir() -> anyhow::Result<()> {
        let req = get_req(
            "/foo/",
            Some("foo_bucket".to_string()),
            None,
            None,
            None,
            None,
        );
        assert_eq!(req.path, "./bus/objects/foo/");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(
            req.params,
            Some(vec![("bucket".into(), "foo_bucket".into())])
        );
        assert_eq!(req.content, None);

        let json = r#"
        {
	"hasMore": false,
	"entries": [
		{
			"eTag": "d41d8cd98f00b204e9800998ecf8427e",
			"health": 1.2,
			"modTime": "2024-07-05T12:37:58.998523074Z",
			"name": "/foo/",
			"size": 5586849,
			"mimeType": "text/plain"
		}
	]
}
        "#;

        let resp: GetResponse = serde_json::from_str(&json)?;
        assert!(resp.entries.is_some());
        let entries = resp.entries.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.get(0).unwrap().name, "/foo/");
        assert_eq!(
            entries.get(0).unwrap().mod_time,
            DateTime::parse_from_rfc3339("2024-07-05T12:37:58.998523074Z")?
        );
        assert_eq!(
            entries.get(0).unwrap().health.as_decimal(),
            &BigDecimal::from_str("1.2")?,
        );
        assert_eq!(entries.get(0).unwrap().size, 5586849);
        Ok(())
    }

    #[test]
    fn get_file() -> anyhow::Result<()> {
        let req = get_req("/foo/This is a file.zip", None, None, None, None, None);
        assert_eq!(req.path, "./bus/objects/foo/This is a file.zip");

        let json = r#"
        {
	"hasMore": false,
	"object": {
		"eTag": "322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9",
		"health": 1.25,
		"modTime": "2024-06-27T11:56:19.05151211Z",
		"name": "/foo/bar/test.zip",
		"size": 3657244,
		"key": "key:aba60a4c1b9ff360214a68f09f890f9afc00d1bf23c8c9435a02311b10ff1d61",
		"slabs": [
			{
				"slab": {
					"health": 1.25,
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

        let resp: GetResponse = serde_json::from_str(&json)?;
        assert!(resp.object.is_some());
        let object = resp.object.unwrap();
        assert_eq!(object.metadata.name, "/foo/bar/test.zip");
        assert_eq!(object.metadata.size, 3657244);
        assert_eq!(
            object.metadata.etag,
            Some("322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9".to_string())
        );
        assert_eq!(
            object.metadata.health.as_decimal(),
            &BigDecimal::from_str("1.25")?
        );

        //todo: test slabs
        Ok(())
    }

    #[test]
    fn list() -> anyhow::Result<()> {
        let json = r#"
        {
    "bucket": "bucket_name",
    "limit": 5,
    "prefix": "/foo/",
    "marker": "marker_name"
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = list_req(
            Some("/foo/".to_string()),
            Some("bucket_name".to_string()),
            Some("marker_name".to_string()),
            5,
        )?;

        assert_eq!(req.path, "./bus/objects/list");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        let json = r#"
        {
  "hasMore": true,
  "nextMarker": "next_marker_value",
  "objects": [
    {
      "eTag": "322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9",
		"health": 1,
		"modTime": "2024-06-27T11:56:19.05151211Z",
		"name": "/foo/bar/test.zip",
		"size": 3657244
    },
    {
      "eTag": "d41d8cd98f00b204e9800998ecf8427e",
			"health": 1.2,
			"modTime": "2024-07-05T12:37:58.998523074Z",
			"name": "/foo/",
			"size": 5586849,
			"mimeType": "text/plain"
    }
    ]
    }
        "#;

        let resp: ListResponse = serde_json::from_str(&json)?;
        assert_eq!(resp.has_more, true);
        assert_eq!(resp.next_marker, Some("next_marker_value".to_string()));
        assert_eq!(resp.objects.len(), 2);

        assert_eq!(
            resp.objects.get(0).unwrap().etag,
            Some("322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9".to_string())
        );
        assert_eq!(resp.objects.get(0).unwrap().name, "/foo/bar/test.zip");
        assert_eq!(resp.objects.get(0).unwrap().size, 3657244);

        assert_eq!(resp.objects.get(1).unwrap().name, "/foo/");
        assert_eq!(
            resp.objects.get(1).unwrap().mime_type,
            Some("text/plain".to_string())
        );

        let json = r#"
        {
	"hasMore": false,
	"nextMarker": "",
	"objects": null
}
        "#;

        let resp: ListResponse = serde_json::from_str(&json)?;
        assert_eq!(resp.has_more, false);
        assert_eq!(resp.next_marker, None);
        assert_eq!(resp.objects.len(), 0);

        Ok(())
    }

    #[test]
    fn delete() -> anyhow::Result<()> {
        let req = delete_req("/foo/bar/file.ext", Some("bucket_name".to_string()), false);
        assert_eq!(req.path, "./bus/objects/foo/bar/file.ext");
        assert_eq!(req.request_type, RequestType::Delete);
        assert_eq!(
            req.params,
            Some(vec![
                ("bucket".into(), "bucket_name".into()),
                ("batch".into(), "false".into())
            ])
        );
        assert_eq!(req.content, None);
        Ok(())
    }

    #[test]
    fn copy() -> anyhow::Result<()> {
        let json = r#"
        {
    "sourceBucket": "default",
    "sourcePath": "/foo/bar/file1",
    "destinationBucket": "default",
    "destinationPath": "/foo/bar/file2"
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = copy_req(
            "/foo/bar/file1".to_string(),
            "default".to_string(),
            "/foo/bar/file2".to_string(),
            "default".to_string(),
        )?;
        assert_eq!(req.path, "./bus/objects/copy");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }

    #[test]
    fn rename() -> anyhow::Result<()> {
        let json = r#"
        {
    "bucket": "mybucket",
    "from": "/foo/old",
    "to": "/foo/new",
    "mode": "single",
    "force": false
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = rename_req(
            "/foo/old".to_string(),
            "/foo/new".to_string(),
            "mybucket".to_string(),
            false,
            RenameMode::Single,
        )?;
        assert_eq!(req.path, "./bus/objects/rename");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }

    #[test]
    fn search() -> anyhow::Result<()> {
        let req = search_req(
            Some("search_key".to_string()),
            Some("bucket_name".to_string()),
            Some(10),
            Some(20),
        );
        assert_eq!(req.path, "./bus/search/objects");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(
            req.params,
            Some(vec![
                ("key".into(), "search_key".into()),
                ("bucket".into(), "bucket_name".into()),
                ("offset".into(), "10".into()),
                ("limit".into(), "20".into())
            ])
        );
        assert_eq!(req.content, None);

        let json = r#"
    [
      {
		"eTag": "322fc5d8660ed6b05e60aa17b08897c149841991ce8070c83c84eb00b39bcdd9",
		"health": 1,
		"modTime": "2024-06-27T11:56:19.05151211Z",
		"name": "/foo/bar/test.zip",
		"size": 3657244
	  },
	  {
			"eTag": "d41d8cd98f00b204e9800998ecf8427e",
			"health": 1.2,
			"modTime": "2024-07-05T12:37:58.998523074Z",
			"name": "/foo/",
			"size": 5586849,
			"mimeType": "text/plain"
	  }
    ]
        "#;

        let resp: Vec<Metadata> = serde_json::from_str(&json)?;
        assert_eq!(resp.len(), 2);
        assert_eq!(resp.get(0).unwrap().name, "/foo/bar/test.zip");
        assert_eq!(resp.get(0).unwrap().size, 3657244);
        assert_eq!(
            resp.get(1).unwrap().etag,
            Some("d41d8cd98f00b204e9800998ecf8427e".to_string())
        );
        assert_eq!(
            resp.get(1).unwrap().mime_type,
            Some("text/plain".to_string())
        );
        Ok(())
    }
}
