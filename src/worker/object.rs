use crate::Error::InvalidDataError;
use crate::InvalidDataError::{InvalidContentLength, InvalidLastModified};
use crate::{
    encode_object_path, ApiRequest, ApiRequestBuilder, ClientInner, Error, RequestContent,
};
use chrono::{DateTime, FixedOffset};
use futures::{AsyncRead, AsyncSeek, TryStreamExt};
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED};
use reqwest_file::RequestFile;
use std::io::SeekFrom;
use std::sync::Arc;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn download<S: AsRef<str>>(
        &self,
        path: S,
        bucket: Option<String>,
    ) -> Result<Option<DownloadableObject>, Error> {
        let object_path = path.as_ref().to_string();
        let resp = match self
            .inner
            .send_api_request_optional(download_head_req(path, &bucket))
            .await?
        {
            Some(resp) => resp,
            None => return Ok(None),
        };

        let accept_byte_ranges = resp
            .headers()
            .get(ACCEPT_RANGES)
            .and_then(|accept_ranges| accept_ranges.to_str().ok())
            .map(|accept_ranges| accept_ranges.starts_with("bytes"))
            .unwrap_or(false);

        // due to a bug in reqwest, `content_length()` for head requests always returns `Some(0)`
        // the `Content-Length` header value is correct though, so manually parsing it is a workaround
        // see: https://github.com/seanmonstar/reqwest/issues/1814
        let content_length = resp
            .headers()
            .get(CONTENT_LENGTH)
            .map(|cl| {
                cl.to_str()
                    .map_err(|_| InvalidDataError(InvalidContentLength))
                    .and_then(|cl| {
                        cl.parse::<u64>()
                            .map_err(|_| InvalidDataError(InvalidContentLength))
                    })
            })
            .transpose()?;

        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|content_type| content_type.to_str().map(|s| s.to_string()).ok())
            .filter(|s| !s.is_empty());

        let etag = resp
            .headers()
            .get(ETAG)
            .and_then(|content_type| content_type.to_str().map(|s| s.to_string()).ok())
            .filter(|s| !s.is_empty());

        let last_modified = if let Some(date_header) = resp.headers().get(LAST_MODIFIED) {
            Some(
                DateTime::parse_from_rfc2822(
                    date_header
                        .to_str()
                        .map_err(|_| InvalidDataError(InvalidLastModified))?,
                )
                .map_err(|_| InvalidDataError(InvalidLastModified))?,
            )
        } else {
            None
        };

        Ok(Some(DownloadableObject {
            path: object_path,
            bucket,
            etag,
            length: content_length,
            last_modified,
            seekable: accept_byte_ranges && content_length.map_or(false, |len| len > 0),
            content_type,
            inner: self.inner.clone(),
        }))
    }

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

    pub async fn upload<S: AsRef<str>, U: AsyncRead + Send + Sync + Unpin + 'static>(
        &self,
        path: S,
        content_type: Option<String>,
        bucket: Option<String>,
        stream: U,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(upload_req(path, content_type, bucket, stream))
            .await?;
        Ok(())
    }
}

fn upload_req<S: AsRef<str>, U: AsyncRead + Send + Sync + Unpin + 'static>(
    path: S,
    content_type: Option<String>,
    bucket: Option<String>,
    stream: U,
) -> ApiRequest {
    let url = encode_object_path(path, "./worker/objects");
    let params = bucket.map(|b| vec![("bucket", b)]);

    ApiRequestBuilder::put(url)
        .params(params)
        .content(Some(RequestContent::Stream(Box::new(stream), content_type)))
        .build()
}

fn delete_req<S: AsRef<str>>(path: S, bucket: Option<String>, batch: bool) -> ApiRequest {
    let url = encode_object_path(path, "./worker/objects");
    let mut params = Vec::with_capacity(2);
    if let Some(bucket) = bucket {
        params.push(("bucket", bucket))
    };
    params.push(("batch", batch.to_string()));
    ApiRequestBuilder::delete(url).params(Some(params)).build()
}

fn download_head_req<S: AsRef<str>>(path: S, bucket: &Option<String>) -> ApiRequest {
    let (path, params) = dl_req_prep(path, bucket);
    ApiRequestBuilder::head(path).params(params).build()
}

fn download_get_req<S: AsRef<str>>(path: S, bucket: &Option<String>) -> ApiRequest {
    let (path, params) = dl_req_prep(path, bucket);
    ApiRequestBuilder::get(path).params(params).build()
}

fn dl_req_prep<S: AsRef<str>>(
    path: S,
    bucket: &Option<String>,
) -> (String, Option<Vec<(&'static str, String)>>) {
    let params = bucket.clone().map(|b| vec![("bucket", b)]);
    //todo: find out how renterd actually expects the path to be encoded
    let path = encode_object_path(path, "./worker/objects");
    (path, params)
}

pub struct DownloadableObject {
    pub path: String,
    pub bucket: Option<String>,
    pub length: Option<u64>,
    pub content_type: Option<String>,
    pub seekable: bool,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<FixedOffset>>,
    inner: Arc<ClientInner>,
}

impl DownloadableObject {
    pub async fn open_stream(&self) -> Result<impl AsyncRead + Send + Unpin, Error> {
        let resp = self
            .inner
            .send_api_request(download_get_req(&self.path, &self.bucket))
            .await?;

        Ok(resp
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .into_async_read())
    }

    pub async fn open_seekable_stream(
        &self,
        initial_offset: impl Into<Option<u64>>,
    ) -> Result<impl AsyncRead + AsyncSeek + Send + Unpin, Error> {
        if !self.seekable {
            return Err(Error::NotSeekable(self.path.clone()));
        }

        let req_builder = self
            .inner
            .api_request_builder(download_get_req(&self.path, &self.bucket))
            .await?;

        let mut file: RequestFile = RequestFile::with_size(req_builder, self.length);
        file.seek(SeekFrom::Start(initial_offset.into().unwrap_or(0)))
            .await?;

        Ok(file.compat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use futures::io::Cursor;

    #[test]
    fn download_req() -> anyhow::Result<()> {
        let req = download_head_req("/foo/bar", &None);
        assert_eq!(req.path, "./worker/objects/foo/bar");
        assert_eq!(req.request_type, RequestType::Head);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let req = download_get_req("/foo/bar/baz/test.file", &Some("testbucket".to_string()));
        assert_eq!(req.path, "./worker/objects/foo/bar/baz/test.file");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(
            req.params,
            Some(vec![("bucket".into(), "testbucket".into())])
        );
        assert_eq!(req.content, None);

        Ok(())
    }

    #[test]
    fn delete() -> anyhow::Result<()> {
        let req = delete_req("/foo/bar/file.ext", Some("bucket_name".to_string()), false);
        assert_eq!(req.path, "./worker/objects/foo/bar/file.ext");
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
    fn upload() -> anyhow::Result<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3];
        let cursor = Cursor::new(data);

        let req = upload_req(
            "/foo/bar/file.ext",
            Some("application/funny-bytes".to_string()),
            Some("bucket_name".to_string()),
            cursor,
        );

        assert_eq!(req.path, "./worker/objects/foo/bar/file.ext");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(
            req.params,
            Some(vec![("bucket".into(), "bucket_name".into())])
        );
        if let Some(RequestContent::Stream(_stream, content_type)) = req.content {
            assert_eq!(content_type, Some("application/funny-bytes".to_string()));
        } else {
            panic!("expected stream content");
        }
        Ok(())
    }
}
