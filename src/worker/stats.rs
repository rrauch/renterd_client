use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, Percentage, PublicKey};
use bandwidth::Bandwidth;
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

    pub async fn download(&self) -> Result<Download, Error> {
        Ok(self
            .inner
            .send_api_request(&download_req())
            .await?
            .json()
            .await?)
    }

    pub async fn upload(&self) -> Result<Upload, Error> {
        Ok(self
            .inner
            .send_api_request(&upload_req())
            .await?
            .json()
            .await?)
    }
}

fn download_req() -> ApiRequest {
    ApiRequestBuilder::get("./worker/stats/downloads").build()
}

fn upload_req() -> ApiRequest {
    ApiRequestBuilder::get("./worker/stats/uploads").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Download {
    #[serde(rename = "avgDownloadSpeedMbps")]
    #[serde(deserialize_with = "crate::deserialize_mbps_float")]
    pub avg_download_speed: Bandwidth,
    #[serde(rename = "avgOverdrivePct")]
    pub avg_overdrive: Percentage,
    pub healthy_downloaders: u64,
    pub num_downloaders: u64,
    #[serde(rename = "downloadersStats")]
    pub downloaders: Vec<Downloader>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Downloader {
    #[serde(rename = "avgSectorDownloadSpeedMbps")]
    #[serde(deserialize_with = "crate::deserialize_mbps_float")]
    pub avg_sector_download_speed: Bandwidth,
    pub host_key: PublicKey,
    pub num_downloads: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Upload {
    #[serde(rename = "avgSlabUploadSpeedMbps")]
    #[serde(deserialize_with = "crate::deserialize_mbps_float")]
    pub avg_upload_speed: Bandwidth,
    #[serde(rename = "avgOverdrivePct")]
    pub avg_overdrive: Percentage,
    pub healthy_uploaders: u64,
    pub num_uploaders: u64,
    #[serde(rename = "uploadersStats")]
    pub uploaders: Vec<Uploader>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Uploader {
    #[serde(rename = "avgSectorUploadSpeedMbps")]
    #[serde(deserialize_with = "crate::deserialize_mbps_float")]
    pub avg_sector_upload_speed: Bandwidth,
    pub host_key: PublicKey,
}

#[cfg(test)]
mod tests {
    use crate::worker::stats::{download_req, upload_req, Download, Upload};
    use crate::{PublicKey, RequestType};
    use bandwidth::Bandwidth;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn download() -> anyhow::Result<()> {
        let req = download_req();
        assert_eq!(req.path, "./worker/stats/downloads");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "avgDownloadSpeedMbps": 277.89,
  "avgOverdrivePct": 2,
  "healthyDownloaders": 5,
  "numDownloaders": 5,
  "downloadersStats": [
    {
      "avgSectorDownloadSpeedMbps": 89.28,
      "hostKey": "ed25519:fd8a8fd8758a5001925c0cd96b601ad79fb612639ff6aa4950c7da3090a301a4",
      "numDownloads": 4405
    },
    {
      "avgSectorDownloadSpeedMbps": 66.1724,
      "hostKey": "ed25519:4b6bf45a867d2f664317fbe15ae036ca32fc32db118d8bcced5947fdb8664537",
      "numDownloads": 43
    },
    {
      "avgSectorDownloadSpeedMbps": 49.9636,
      "hostKey": "ed25519:090911c5182da4eb257807dc068c9fc4e3363b8b8208acdfb6a8b00ced08c45c",
      "numDownloads": 223
    },
    {
      "avgSectorDownloadSpeedMbps": 49.95,
      "hostKey": "ed25519:81a09fe85355baf606d87340edeb2c34f84cf7e35b4209f556da3bb5a72b92af",
      "numDownloads": 12
    },
    {
      "avgSectorDownloadSpeedMbps": 46.088,
      "hostKey": "ed25519:075f76fc20d9f6136b068463986ea63e36f069c83d9d8213c216cbf4a23ce761",
      "numDownloads": 1
    }
  ]
}
        "#;

        let download: Download = serde_json::from_str(&json)?;
        assert_eq!(
            download.avg_download_speed,
            Bandwidth::from_gbps_f64(0.27789)
        );
        assert_eq!(
            download.avg_overdrive.as_big_decimal(),
            &BigDecimal::from_str("0.02")?
        );
        assert_eq!(download.healthy_downloaders, 5);
        assert_eq!(download.num_downloaders, 5);

        assert_eq!(download.downloaders.len(), 5);
        assert_eq!(
            download
                .downloaders
                .get(0)
                .unwrap()
                .avg_sector_download_speed,
            Bandwidth::from_gbps_f64(0.08928)
        );
        assert_eq!(
            download.downloaders.get(1).unwrap().host_key,
            PublicKey::try_from(
                "ed25519:4b6bf45a867d2f664317fbe15ae036ca32fc32db118d8bcced5947fdb8664537"
            )?,
        );
        assert_eq!(download.downloaders.get(2).unwrap().num_downloads, 223);

        Ok(())
    }

    #[test]
    fn upload() -> anyhow::Result<()> {
        let req = upload_req();
        assert_eq!(req.path, "./worker/stats/uploads");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "avgSlabUploadSpeedMbps": 15.05,
  "avgOverdrivePct": 47.09,
  "healthyUploaders": 5,
  "numUploaders": 5,
  "uploadersStats": [
    {
      "hostKey": "ed25519:fd8a8fd8758a5001925c0cd96b601ad79fb612639ff6aa4950c7da3090a301a4",
      "avgSectorUploadSpeedMbps": 57.052
    },
    {
      "hostKey": "ed25519:b8c2d68bf993ec48908f120b8bd7fff03dd1c055b6920002d157261d82367431",
      "avgSectorUploadSpeedMbps": 22.6412
    },
    {
      "hostKey": "ed25519:075f76fc20d9f6136b068463986ea63e36f069c83d9d8213c216cbf4a23ce761",
      "avgSectorUploadSpeedMbps": 20.4524
    },
    {
      "hostKey": "ed25519:6c69db376b5a401fa2821ceb56458369824773b31b8e66ec213513b72946e280",
      "avgSectorUploadSpeedMbps": 17.6088
    },
    {
      "hostKey": "ed25519:a90d3c26a22d66903c06a1bf869e14e829e95cfa25b6bf08189c98713fc92449",
      "avgSectorUploadSpeedMbps": 17.4656
    }
  ]
}
        "#;

        let upload: Upload = serde_json::from_str(&json)?;
        assert_eq!(upload.avg_upload_speed, Bandwidth::from_gbps_f64(0.01505));
        assert_eq!(
            upload.avg_overdrive.as_big_decimal(),
            &BigDecimal::from_str("0.4709")?
        );
        assert_eq!(upload.healthy_uploaders, 5);
        assert_eq!(upload.num_uploaders, 5);

        assert_eq!(upload.uploaders.len(), 5);
        assert_eq!(
            upload.uploaders.get(0).unwrap().avg_sector_upload_speed,
            Bandwidth::from_gbps_f64(0.057052)
        );
        assert_eq!(
            upload.uploaders.get(1).unwrap().host_key,
            PublicKey::try_from(
                "ed25519:b8c2d68bf993ec48908f120b8bd7fff03dd1c055b6920002d157261d82367431"
            )?,
        );

        Ok(())
    }
}
