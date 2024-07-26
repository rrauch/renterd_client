use crate::Error::InvalidDataError;
use crate::{empty_string_as_none, ApiRequest, ApiRequestBuilder, RequestContent};
use crate::{ClientInner, Error, PublicKey, SettingsId};
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
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

    pub async fn get_all(
        &self,
        offset: Option<NonZeroUsize>,
        limit: Option<NonZeroUsize>,
    ) -> Result<Vec<Host>, Error> {
        Ok(self
            .inner
            .send_api_request(get_all_req(offset, limit))
            .await?
            .json()
            .await?)
    }

    pub async fn get_by_key(&self, key: &PublicKey) -> Result<Host, Error> {
        Ok(self
            .inner
            .send_api_request(get_by_key_req(key))
            .await?
            .json()
            .await?)
    }

    pub async fn allowlist(&self) -> Result<Vec<PublicKey>, Error> {
        Ok(self
            .inner
            .send_api_request(allowlist_req())
            .await?
            .json()
            .await?)
    }

    pub async fn modify_allowlist(&self, action: ModifyAction<PublicKey>) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(modify_allowlist_req(action)?)
            .await?;
        Ok(())
    }

    pub async fn blocklist(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .inner
            .send_api_request(blocklist_req())
            .await?
            .json()
            .await?)
    }

    pub async fn modify_blocklist(&self, action: ModifyAction<String>) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(modify_blocklist_req(action)?)
            .await?;
        Ok(())
    }

    //todo: implement missing `pricetables` function

    pub async fn remove(
        &self,
        min_recent_scan_failures: u64,
        max_downtime_hours: u64,
        bucket: Option<String>,
    ) -> Result<u64, Error> {
        Ok(self
            .inner
            .send_api_request(remove_req(
                min_recent_scan_failures,
                max_downtime_hours,
                bucket,
            )?)
            .await?
            .json()
            .await?)
    }

    //todo: implement `scans` function

    pub async fn scanning(
        &self,
        offset: Option<NonZeroUsize>,
        limit: Option<NonZeroUsize>,
        last_scan: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<HostAddress>, Error> {
        Ok(self
            .inner
            .send_api_request(scanning_req(offset, limit, last_scan))
            .await?
            .json()
            .await?)
    }

    pub async fn reset_lost_sectors(&self, key: &PublicKey) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(reset_lost_sectors_req(key))
            .await?;
        Ok(())
    }
}

fn reset_lost_sectors_req(key: &PublicKey) -> ApiRequest {
    ApiRequestBuilder::post(format!("./bus/host/{}/resetlostsectors", key)).build()
}

fn get_by_key_req(key: &PublicKey) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/host/{}", key)).build()
}

fn scanning_req(
    offset: Option<NonZeroUsize>,
    limit: Option<NonZeroUsize>,
    last_scan: Option<DateTime<FixedOffset>>,
) -> ApiRequest {
    let mut params = Vec::with_capacity(3);
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(last_scan) = last_scan {
        params.push(("lastScan", last_scan.to_rfc3339()));
    }
    let params = if params.is_empty() {
        None
    } else {
        Some(params)
    };
    ApiRequestBuilder::get("./bus/hosts/scanning")
        .params(params)
        .build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HostAddress {
    pub public_key: PublicKey,
    pub net_address: String,
}

fn remove_req(
    min_recent_scan_failures: u64,
    max_downtime_hours: u64,
    bucket: Option<String>,
) -> Result<ApiRequest, Error> {
    let params = bucket.map(|b| vec![("bucket", b)]);
    let content = Some(RequestContent::Json(
        serde_json::to_value(RemoveRequest {
            min_recent_scan_failures,
            max_downtime_hours,
        })
        .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/hosts/remove")
        .content(content)
        .params(params)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoveRequest {
    min_recent_scan_failures: u64,
    max_downtime_hours: u64,
}

fn blocklist_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/hosts/blocklist").build()
}

fn modify_blocklist_req(action: ModifyAction<String>) -> Result<ApiRequest, Error> {
    let req: ModifyRequest<String> = action.into();
    let content = Some(RequestContent::Json(
        serde_json::to_value(req).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::put("./bus/hosts/blocklist")
        .content(content)
        .build())
}

fn modify_allowlist_req(action: ModifyAction<PublicKey>) -> Result<ApiRequest, Error> {
    let req: ModifyRequest<PublicKey> = action.into();
    let content = Some(RequestContent::Json(
        serde_json::to_value(req).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::put("./bus/hosts/allowlist")
        .content(content)
        .build())
}

pub enum ModifyAction<T: Serialize> {
    AddRemove {
        add: Option<Vec<T>>,
        remove: Option<Vec<T>>,
    },
    Clear,
}

impl<T: Serialize> From<ModifyAction<T>> for ModifyRequest<T> {
    fn from(value: ModifyAction<T>) -> Self {
        match value {
            ModifyAction::AddRemove { add, remove } => ModifyRequest {
                add: add.unwrap_or(vec![]),
                remove: remove.unwrap_or(vec![]),
                clear: false,
            },
            ModifyAction::Clear => ModifyRequest {
                add: vec![],
                remove: vec![],
                clear: true,
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModifyRequest<T: Serialize> {
    add: Vec<T>,
    remove: Vec<T>,
    clear: bool,
}

fn allowlist_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/hosts/allowlist").build()
}

fn get_all_req(offset: Option<NonZeroUsize>, limit: Option<NonZeroUsize>) -> ApiRequest {
    let mut params = Vec::with_capacity(2);
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    let params = if params.is_empty() {
        None
    } else {
        Some(params)
    };

    ApiRequestBuilder::get("./bus/hosts").params(params).build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Host {
    pub known_since: DateTime<FixedOffset>,
    pub last_announcement: DateTime<FixedOffset>,
    pub public_key: PublicKey,
    pub net_address: String,
    pub price_table: PriceTable,
    pub settings: Settings,
    pub interactions: Interactions,
    pub scanned: bool,
    pub blocked: bool,
    pub checks: BTreeMap<String, Check>,
    pub stored_data: u64,
    pub subnets: Option<Vec<String>>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PriceTable {
    pub uid: SettingsId,
    #[serde(with = "crate::duration_ns")]
    pub validity: Duration,
    #[serde(rename = "hostblockheight")]
    pub host_block_height: u64,
    #[serde(rename = "updatepricetablecost")]
    #[serde(with = "crate::number_as_string")]
    pub update_price_table_cost: u128,
    #[serde(rename = "accountbalancecost")]
    #[serde(with = "crate::number_as_string")]
    pub account_balance_cost: u128,
    #[serde(rename = "fundaccountcost")]
    #[serde(with = "crate::number_as_string")]
    pub fund_account_cost: u128,
    #[serde(rename = "latestrevisioncost")]
    #[serde(with = "crate::number_as_string")]
    pub latest_revision_cost: u128,
    #[serde(rename = "subscriptionmemorycost")]
    #[serde(with = "crate::number_as_string")]
    pub subscription_memory_cost: u128,
    #[serde(rename = "subscriptionnotificationcost")]
    #[serde(with = "crate::number_as_string")]
    pub subscription_notification_cost: u128,
    #[serde(rename = "initbasecost")]
    #[serde(with = "crate::number_as_string")]
    pub init_base_cost: u128,
    #[serde(rename = "memorytimecost")]
    #[serde(with = "crate::number_as_string")]
    pub memory_time_cost: u128,
    #[serde(rename = "downloadbandwidthcost")]
    #[serde(with = "crate::number_as_string")]
    pub download_bandwidth_cost: u128,
    #[serde(rename = "uploadbandwidthcost")]
    #[serde(with = "crate::number_as_string")]
    pub upload_bandwidth_cost: u128,
    #[serde(rename = "dropsectorsbasecost")]
    #[serde(with = "crate::number_as_string")]
    pub drop_sector_base_cost: u128,
    #[serde(rename = "dropsectorsunitcost")]
    #[serde(with = "crate::number_as_string")]
    pub drop_sector_unit_cost: u128,
    #[serde(rename = "hassectorbasecost")]
    #[serde(with = "crate::number_as_string")]
    pub has_sector_base_cost: u128,
    #[serde(rename = "readbasecost")]
    #[serde(with = "crate::number_as_string")]
    pub read_base_cost: u128,
    #[serde(rename = "readlengthcost")]
    #[serde(with = "crate::number_as_string")]
    pub read_length_cost: u128,
    #[serde(rename = "renewcontractcost")]
    #[serde(with = "crate::number_as_string")]
    pub renew_contract_cost: u128,
    #[serde(rename = "revisionbasecost")]
    #[serde(with = "crate::number_as_string")]
    pub revision_base_cost: u128,
    #[serde(rename = "swapsectorcost")]
    #[serde(with = "crate::number_as_string")]
    pub swap_sector_base_cost: u128,
    #[serde(rename = "writebasecost")]
    #[serde(with = "crate::number_as_string")]
    pub write_base_cost: u128,
    #[serde(rename = "writelengthcost")]
    #[serde(with = "crate::number_as_string")]
    pub write_length_cost: u128,
    #[serde(rename = "writestorecost")]
    #[serde(with = "crate::number_as_string")]
    pub write_store_cost: u128,
    #[serde(rename = "txnfeeminrecommended")]
    #[serde(with = "crate::number_as_string")]
    pub txn_fee_min_recommended: u128,
    #[serde(rename = "txnfeemaxrecommended")]
    #[serde(with = "crate::number_as_string")]
    pub txn_fee_max_recommended: u128,
    #[serde(rename = "contractprice")]
    #[serde(with = "crate::number_as_string")]
    pub contract_price: u128,
    #[serde(rename = "collateralcost")]
    #[serde(with = "crate::number_as_string")]
    pub collateral_cost: u128,
    #[serde(rename = "maxcollateral")]
    #[serde(with = "crate::number_as_string")]
    pub max_collateral: u128,
    #[serde(rename = "maxduration")]
    pub max_duration: u64, //todo: clarify if `Duration` can be used or if this is in blocks
    #[serde(rename = "windowsize")]
    pub window_size: u64,
    #[serde(rename = "registryentriesleft")]
    pub registry_entries_left: u64,
    #[serde(rename = "registryentriestotal")]
    pub registry_entries_total: u64,
    pub expiry: DateTime<FixedOffset>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    #[serde(rename = "acceptingcontracts")]
    pub accepting_contracts: bool,
    #[serde(rename = "maxdownloadbatchsize")]
    pub max_download_batch_size: u64,
    #[serde(rename = "maxduration")]
    pub max_duration: u64, //todo: clarify
    #[serde(rename = "maxrevisebatchsize")]
    pub max_revise_batch_size: u64,
    #[serde(rename = "netaddress")]
    pub net_address: String,
    #[serde(rename = "remainingstorage")]
    pub remaining_storage: u64,
    #[serde(rename = "sectorsize")]
    pub sector_size: u64,
    #[serde(rename = "totalstorage")]
    pub total_storage: u64,
    #[serde(rename = "unlockhash")]
    pub address: String, //todo
    #[serde(rename = "windowsize")]
    pub window_size: u64,
    #[serde(with = "crate::number_as_string")]
    pub collateral: u128,
    #[serde(rename = "maxcollateral")]
    #[serde(with = "crate::number_as_string")]
    pub max_collateral: u128,
    #[serde(rename = "baserpcprice")]
    #[serde(with = "crate::number_as_string")]
    pub base_rpc_price: u128,
    #[serde(rename = "contractprice")]
    #[serde(with = "crate::number_as_string")]
    pub contract_price: u128,
    #[serde(rename = "downloadbandwidthprice")]
    #[serde(with = "crate::number_as_string")]
    pub download_bandwidth_price: u128,
    #[serde(rename = "sectoraccessprice")]
    #[serde(with = "crate::number_as_string")]
    pub sector_access_price: u128,
    #[serde(rename = "storageprice")]
    #[serde(with = "crate::number_as_string")]
    pub storage_price: u128,
    #[serde(rename = "uploadbandwidthprice")]
    #[serde(with = "crate::number_as_string")]
    pub upload_bandwidth_price: u128,
    #[serde(rename = "ephemeralaccountexpiry")]
    #[serde(with = "crate::duration_ns")]
    pub ephemeral_account_expiry: Duration,
    #[serde(rename = "maxephemeralaccountbalance")]
    #[serde(with = "crate::number_as_string")]
    pub max_ephemeral_account_balance: u128,
    #[serde(rename = "revisionnumber")]
    pub revision_number: u64,
    pub version: String,
    pub release: String,
    #[serde(rename = "siamuxport")]
    pub sia_mux_port: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Interactions {
    pub total_scans: u64,
    pub last_scan: DateTime<FixedOffset>,
    pub last_scan_success: bool,
    pub lost_sectors: u64,
    pub second_to_last_scan_success: bool,
    #[serde(with = "crate::duration_ns")]
    pub uptime: Duration,
    #[serde(with = "crate::duration_ns")]
    pub downtime: Duration,
    pub successful_interactions: usize,
    pub failed_interactions: usize,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Check {
    pub gouging: GougingBreakdown,
    pub score: ScoreBreakdown,
    pub usability: UsabilityBreakDown,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GougingBreakdown {
    #[serde(rename = "contractErr")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub contract_error: Option<String>,
    #[serde(rename = "downloadErr")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub download_error: Option<String>,
    #[serde(rename = "gougingErr")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub gouging_error: Option<String>,
    #[serde(rename = "pruneErr")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub prune_error: Option<String>,
    #[serde(rename = "uploadErr")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub upload_error: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ScoreBreakdown {
    #[serde(with = "bigdecimal::serde::json_num")]
    pub age: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub collateral: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub interactions: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub storage_remaining: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub uptime: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub version: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub prices: BigDecimal,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UsabilityBreakDown {
    pub blocked: bool,
    pub offline: bool,
    pub low_score: bool,
    #[serde(rename = "redundantIP")]
    pub redundant_ip: bool,
    pub gouging: bool,
    pub not_accepting_contracts: bool,
    pub not_announced: bool,
    pub not_completing_scan: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use serde_json::Value;

    #[test]
    fn get_all() -> anyhow::Result<()> {
        let req = get_all_req(None, None);
        assert_eq!(req.path, "./bus/hosts");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let req = get_all_req(Some(10.try_into()?), None);
        assert_eq!(req.params, Some(vec![("offset".into(), "10".into())]));

        let req = get_all_req(Some(10.try_into()?), Some(20.try_into()?));
        assert_eq!(
            req.params,
            Some(vec![
                ("offset".into(), "10".into()),
                ("limit".into(), "20".into())
            ])
        );

        let json = r#"
[
  {
    "knownSince": "2023-02-20T19:05:14.419+01:00",
    "lastAnnouncement": "2024-03-01T13:57:41Z",
    "publicKey": "ed25519:b050c0c63f9f3b4d5a89acadf628e8d8c6f8768e38fbe731e429334e0fd2cece",
    "netAddress": "78.197.237.216:9982",
    "priceTable": {
      "uid": "224738b6f0b77080c186cf47a12c4645",
      "validity": 600000000000,
      "hostblockheight": 410801,
      "updatepricetablecost": "1",
      "accountbalancecost": "1",
      "fundaccountcost": "1",
      "latestrevisioncost": "407200000000000000",
      "subscriptionmemorycost": "1",
      "subscriptionnotificationcost": "0",
      "initbasecost": "0",
      "memorytimecost": "1",
      "downloadbandwidthcost": "150000000000000",
      "uploadbandwidthcost": "50000000000000",
      "dropsectorsbasecost": "1",
      "dropsectorsunitcost": "1",
      "hassectorbasecost": "1",
      "readbasecost": "0",
      "readlengthcost": "1",
      "renewcontractcost": "100000000000000000",
      "revisionbasecost": "0",
      "swapsectorcost": "1",
      "writebasecost": "1",
      "writelengthcost": "1",
      "writestorecost": "81018518518",
      "txnfeeminrecommended": "10000000000000000000",
      "txnfeemaxrecommended": "30000000000000000000",
      "contractprice": "150000000000000000000000",
      "collateralcost": "81018518518",
      "maxcollateral": "5000000000000000000000000000",
      "maxduration": 25920,
      "windowsize": 144,
      "registryentriesleft": 0,
      "registryentriestotal": 0,
      "expiry": "2024-07-04T12:19:01.025014279Z"
    },
    "settings": {
      "acceptingcontracts": true,
      "baserpcprice": "0",
      "collateral": "81018518518",
      "contractprice": "150000000000000000000000",
      "downloadbandwidthprice": "150000000000000",
      "ephemeralaccountexpiry": 604800000000000,
      "maxcollateral": "5000000000000000000000000000",
      "maxdownloadbatchsize": 17825792,
      "maxduration": 25920,
      "maxephemeralaccountbalance": "1000000000000000000000000",
      "maxrevisebatchsize": 17825792,
      "netaddress": "78.197.237.216:9982",
      "remainingstorage": 2877292544,
      "revisionnumber": 44846666,
      "sectoraccessprice": "0",
      "sectorsize": 4194304,
      "siamuxport": "9983",
      "storageprice": "81018518518",
      "totalstorage": 3999956729856,
      "unlockhash": "50344392179ea814d5b98f281dd459894171ca5e9064ab04596363031cddd886f16409aceed1",
      "uploadbandwidthprice": "50000000000000",
      "version": "1.5.4",
      "release": "hostd 58db87c",
      "windowsize": 144
    },
    "interactions": {
      "totalScans": 36,
      "lastScan": "2023-03-29T15:42:34.501324171+02:00",
      "lastScanSuccess": true,
      "lostSectors": 0,
      "secondToLastScanSuccess": true,
      "uptime": 3163048968672609,
      "downtime": 0,
      "successfulInteractions": 72,
      "failedInteractions": 0
    },
    "scanned": true,
    "blocked": false,
    "checks": {
      "autopilot": {
        "gouging": {
          "contractErr": "",
          "downloadErr": "",
          "gougingErr": "rpc price too high, 100 nS \u003e 40 nS",
          "pruneErr": "",
          "uploadErr": ""
        },
        "score": {
          "age": 0,
          "collateral": 0,
          "interactions": 0,
          "storageRemaining": 0,
          "uptime": 0,
          "version": 0,
          "prices": 0
        },
        "usability": {
          "blocked": false,
          "offline": false,
          "lowScore": false,
          "redundantIP": false,
          "gouging": true,
          "notAcceptingContracts": false,
          "notAnnounced": false,
          "notCompletingScan": false
        }
      }
    },
    "storedData": 0,
    "subnets": [
      "foo",
      "bar"
    ]
  },
  {
    "knownSince": "2023-02-20T19:05:14.419+01:00",
    "lastAnnouncement": "2024-03-01T13:57:41Z",
    "publicKey": "ed25519:652653b752985af12bdbb8fa8d18b0bb01fc1dc3d4737d96fe54cdf1fb38d23f",
    "netAddress": "siahost.lvr26.eu:9982",
    "priceTable": {
      "uid": "e3a9371d148b0372b43b066e3841cbf1",
      "validity": 600000000000,
      "hostblockheight": 410827,
      "updatepricetablecost": "1",
      "accountbalancecost": "1",
      "fundaccountcost": "1",
      "latestrevisioncost": "612000000000000000",
      "subscriptionmemorycost": "1",
      "subscriptionnotificationcost": "1",
      "initbasecost": "0",
      "memorytimecost": "1",
      "downloadbandwidthcost": "250000000000000",
      "uploadbandwidthcost": "200000000000000",
      "dropsectorsbasecost": "1",
      "dropsectorsunitcost": "1",
      "hassectorbasecost": "1",
      "readbasecost": "0",
      "readlengthcost": "1",
      "renewcontractcost": "100000000000000000",
      "revisionbasecost": "0",
      "swapsectorcost": "1",
      "writebasecost": "0",
      "writelengthcost": "1",
      "writestorecost": "46296296296",
      "txnfeeminrecommended": "10000000000000000000",
      "txnfeemaxrecommended": "30000000000000000000",
      "contractprice": "1000000000000000000000000",
      "collateralcost": "69444444444",
      "maxcollateral": "2000000000000000000000000000",
      "maxduration": 25920,
      "windowsize": 144,
      "registryentriesleft": 30625751,
      "registryentriestotal": 31250048,
      "expiry": "2024-07-04T12:19:01.025014279Z"
    },
    "settings": {
      "acceptingcontracts": true,
      "baserpcprice": "0",
      "collateral": "69444444444",
      "contractprice": "1000000000000000000000000",
      "downloadbandwidthprice": "250000000000000",
      "ephemeralaccountexpiry": 604800000000000,
      "maxcollateral": "2000000000000000000000000000",
      "maxdownloadbatchsize": 524288000,
      "maxduration": 25920,
      "maxephemeralaccountbalance": "1000000000000000000000000",
      "maxrevisebatchsize": 104857600,
      "netaddress": "siahost.lvr26.eu:9982",
      "remainingstorage": 3802597949440,
      "revisionnumber": 57930250,
      "sectoraccessprice": "0",
      "sectorsize": 4194304,
      "siamuxport": "9983",
      "storageprice": "46296296296",
      "totalstorage": 4799894388736,
      "unlockhash": "a3d1a1fc55279e0df01d7fdf8524774d0111447161768519a1b29e82e74c19be647dd9daad11",
      "uploadbandwidthprice": "200000000000000",
      "version": "1.5.9",
      "release": "hostd 58db87c",
      "windowsize": 144
    },
    "interactions": {
      "totalScans": 37,
      "lastScan": "2023-03-29T17:02:33.031823404+02:00",
      "lastScanSuccess": true,
      "lostSectors": 0,
      "secondToLastScanSuccess": true,
      "uptime": 3167842220713472,
      "downtime": 0,
      "successfulInteractions": 105,
      "failedInteractions": 4
    },
    "scanned": true,
    "blocked": false,
    "checks": {
      "autopilot": {
        "gouging": {
          "contractErr": "",
          "downloadErr": "",
          "gougingErr": "rpc price too high, 100 nS \u003e 40 nS",
          "pruneErr": "",
          "uploadErr": ""
        },
        "score": {
          "age": 0,
          "collateral": 0,
          "interactions": 0,
          "storageRemaining": 0,
          "uptime": 0,
          "version": 0,
          "prices": 0
        },
        "usability": {
          "blocked": false,
          "offline": false,
          "lowScore": false,
          "redundantIP": false,
          "gouging": true,
          "notAcceptingContracts": false,
          "notAnnounced": false,
          "notCompletingScan": false
        }
      }
    },
    "storedData": 0
  },
  {
    "knownSince": "2023-02-20T19:11:29.392+01:00",
    "lastAnnouncement": "2024-03-01T13:57:41Z",
    "publicKey": "ed25519:3de2398c1dbfd04b2e0392f3ddad1e227eccd6c920ae5f931afd7f86576fc838",
    "netAddress": "91.214.242.11:9982",
    "priceTable": {
      "uid": "f65364dac4da3c8d104fac4991eb28a9",
      "validity": 600000000000,
      "hostblockheight": 410818,
      "updatepricetablecost": "1",
      "accountbalancecost": "1",
      "fundaccountcost": "1",
      "latestrevisioncost": "1121952000000000000",
      "subscriptionmemorycost": "1",
      "subscriptionnotificationcost": "1",
      "initbasecost": "0",
      "memorytimecost": "1",
      "downloadbandwidthcost": "499000000000000",
      "uploadbandwidthcost": "49000000000000",
      "dropsectorsbasecost": "1",
      "dropsectorsunitcost": "1",
      "hassectorbasecost": "1",
      "readbasecost": "0",
      "readlengthcost": "1",
      "renewcontractcost": "100000000000000000",
      "revisionbasecost": "0",
      "swapsectorcost": "1",
      "writebasecost": "0",
      "writelengthcost": "1",
      "writestorecost": "34722222222",
      "txnfeeminrecommended": "10000000000000000000",
      "txnfeemaxrecommended": "30000000000000000000",
      "contractprice": "200000000000000000000000",
      "collateralcost": "69444444444",
      "maxcollateral": "7000000000000000000000000000",
      "maxduration": 51840,
      "windowsize": 144,
      "registryentriesleft": 17582116,
      "registryentriestotal": 19839104,
      "expiry": "2024-07-04T12:19:01.025014279Z"
    },
    "settings": {
      "acceptingcontracts": true,
      "baserpcprice": "0",
      "collateral": "69444444444",
      "contractprice": "200000000000000000000000",
      "downloadbandwidthprice": "499000000000000",
      "ephemeralaccountexpiry": 604800000000000,
      "maxcollateral": "7000000000000000000000000000",
      "maxdownloadbatchsize": 433000000,
      "maxduration": 51840,
      "maxephemeralaccountbalance": "1000000000000000000000000",
      "maxrevisebatchsize": 268435456,
      "netaddress": "91.214.242.11:9982",
      "remainingstorage": 30768503980032,
      "revisionnumber": 62462582,
      "sectoraccessprice": "0",
      "sectorsize": 4194304,
      "siamuxport": "9983",
      "storageprice": "34722222222",
      "totalstorage": 39946953949184,
      "unlockhash": "b6efa70622fb77e4b30170d4609decfc5ec4b893cda7705bf48d8b0f5c142d280bac57f716d6",
      "uploadbandwidthprice": "49000000000000",
      "version": "1.5.9",
      "release": "hostd 58db87c",
      "windowsize": 144
    },
    "interactions": {
      "totalScans": 36,
      "lastScan": "2023-03-29T15:42:34.65766984+02:00",
      "lastScanSuccess": true,
      "lostSectors": 0,
      "secondToLastScanSuccess": true,
      "uptime": 3163000438080921,
      "downtime": 0,
      "successfulInteractions": 230,
      "failedInteractions": 2
    },
    "scanned": true,
    "blocked": false,
    "checks": {
      "autopilot": {
        "gouging": {
          "contractErr": "",
          "downloadErr": "",
          "gougingErr": "rpc price too high, 100 nS \u003e 40 nS",
          "pruneErr": "",
          "uploadErr": ""
        },
        "score": {
          "age": 0,
          "collateral": 0,
          "interactions": 0,
          "storageRemaining": 0,
          "uptime": 0,
          "version": 0,
          "prices": 0
        },
        "usability": {
          "blocked": false,
          "offline": false,
          "lowScore": false,
          "redundantIP": false,
          "gouging": true,
          "notAcceptingContracts": false,
          "notAnnounced": false,
          "notCompletingScan": false
        }
      }
    },
    "storedData": 0
  },
  {
    "knownSince": "2023-02-20T19:15:26.84+01:00",
    "lastAnnouncement": "2024-03-01T13:57:41Z",
    "publicKey": "ed25519:e14888420f7df8001990283b44e469e922c0dc14dc2d0156a31fbb6524d08008",
    "netAddress": "shawnhomesc.no-ip.org:9982",
    "priceTable": {
      "uid": "daab22a125d9fa4ffbfb099cbba2e06b",
      "validity": 600000000000,
      "hostblockheight": 410818,
      "updatepricetablecost": "1",
      "accountbalancecost": "1",
      "fundaccountcost": "1",
      "latestrevisioncost": "253600000000000000",
      "subscriptionmemorycost": "1",
      "subscriptionnotificationcost": "1",
      "initbasecost": "0",
      "memorytimecost": "1",
      "downloadbandwidthcost": "75000000000000",
      "uploadbandwidthcost": "150000000000000",
      "dropsectorsbasecost": "1",
      "dropsectorsunitcost": "1",
      "hassectorbasecost": "1",
      "readbasecost": "0",
      "readlengthcost": "1",
      "renewcontractcost": "100000000000000000",
      "revisionbasecost": "0",
      "swapsectorcost": "1",
      "writebasecost": "0",
      "writelengthcost": "1",
      "writestorecost": "92592592592",
      "txnfeeminrecommended": "10000000000000000000",
      "txnfeemaxrecommended": "30000000000000000000",
      "contractprice": "150000000000000000000000",
      "collateralcost": "23148148148",
      "maxcollateral": "2500000000000000000000000000",
      "maxduration": 27216,
      "windowsize": 144,
      "registryentriesleft": 15578716,
      "registryentriestotal": 15625024,
      "expiry": "2024-07-04T12:19:01.025014279Z"
    },
    "settings": {
      "acceptingcontracts": true,
      "baserpcprice": "0",
      "collateral": "23148148148",
      "contractprice": "150000000000000000000000",
      "downloadbandwidthprice": "75000000000000",
      "ephemeralaccountexpiry": 604800000000000,
      "maxcollateral": "2500000000000000000000000000",
      "maxdownloadbatchsize": 17825792,
      "maxduration": 27216,
      "maxephemeralaccountbalance": "1000000000000000000000000",
      "maxrevisebatchsize": 17825792,
      "netaddress": "shawnhomesc.no-ip.org:9982",
      "remainingstorage": 2051270508544,
      "revisionnumber": 35680855,
      "sectoraccessprice": "0",
      "sectorsize": 4194304,
      "siamuxport": "9983",
      "storageprice": "92592592592",
      "totalstorage": 4748354781184,
      "unlockhash": "326c36421b0ad2deee9f54fa98d1829e2a917cfc9febeeaa4f20b9121b434a7e8dad6b2f9578",
      "uploadbandwidthprice": "150000000000000",
      "version": "1.5.9",
      "release": "hostd 58db87c",
      "windowsize": 144
    },
    "interactions": {
      "totalScans": 36,
      "lastScan": "2023-03-29T15:42:35.974329629+02:00",
      "lastScanSuccess": true,
      "lostSectors": 0,
      "secondToLastScanSuccess": true,
      "uptime": 3072358865133168,
      "downtime": 90631369071825,
      "successfulInteractions": 207,
      "failedInteractions": 21
    },
    "scanned": true,
    "blocked": false,
    "checks": {
      "autopilot": {
        "gouging": {
          "contractErr": "",
          "downloadErr": "",
          "gougingErr": "rpc price too high, 100 nS \u003e 40 nS",
          "pruneErr": "",
          "uploadErr": ""
        },
        "score": {
          "age": 0,
          "collateral": 0,
          "interactions": 0,
          "storageRemaining": 0,
          "uptime": 0,
          "version": 0,
          "prices": 0
        },
        "usability": {
          "blocked": false,
          "offline": false,
          "lowScore": false,
          "redundantIP": false,
          "gouging": true,
          "notAcceptingContracts": false,
          "notAnnounced": false,
          "notCompletingScan": false
        }
      }
    },
    "storedData": 0
  }
]
        "#;
        let hosts: Vec<Host> = serde_json::from_str(&json)?;
        assert_eq!(hosts.len(), 4);

        assert_eq!(
            hosts.get(0).unwrap().known_since,
            DateTime::parse_from_rfc3339("2023-02-20T19:05:14.419+01:00")?
        );
        assert_eq!(hosts.get(0).unwrap().net_address, "78.197.237.216:9982");
        assert_eq!(hosts.get(0).unwrap().scanned, true);
        assert_eq!(hosts.get(0).unwrap().blocked, false);
        assert_eq!(hosts.get(0).unwrap().checks.len(), 1);
        assert_eq!(hosts.get(0).unwrap().stored_data, 0);
        assert_eq!(hosts.get(0).unwrap().subnets.as_ref().unwrap().len(), 2);

        let price_table = &hosts.get(1).unwrap().price_table;
        assert_eq!(
            price_table.uid,
            "e3a9371d148b0372b43b066e3841cbf1".try_into()?
        );
        assert_eq!(price_table.validity, Duration::from_nanos(600000000000));
        assert_eq!(price_table.upload_bandwidth_cost, 200000000000000);
        assert_eq!(price_table.window_size, 144);
        assert_eq!(
            price_table.expiry,
            DateTime::parse_from_rfc3339("2024-07-04T12:19:01.025014279Z")?
        );

        let settings = &hosts.get(2).unwrap().settings;
        assert_eq!(settings.accepting_contracts, true);
        assert_eq!(settings.base_rpc_price, 0);
        assert_eq!(settings.window_size, 144);
        assert_eq!(
            settings.ephemeral_account_expiry,
            Duration::from_nanos(604800000000000)
        );
        assert_eq!(settings.sector_size, 4194304);
        assert_eq!(settings.net_address, "91.214.242.11:9982");

        let interactions = &hosts.get(2).unwrap().interactions;
        assert_eq!(interactions.total_scans, 36);
        assert_eq!(
            interactions.last_scan,
            DateTime::parse_from_rfc3339("2023-03-29T15:42:34.65766984+02:00")?
        );
        assert_eq!(interactions.last_scan_success, true);
        assert_eq!(interactions.uptime, Duration::from_nanos(3163000438080921));
        assert_eq!(interactions.failed_interactions, 2);

        let checks = hosts.get(3).unwrap().checks.get("autopilot").unwrap();
        assert_eq!(checks.gouging.contract_error, None);
        assert_eq!(checks.gouging.download_error, None);
        assert_eq!(
            checks.gouging.gouging_error,
            Some("rpc price too high, 100 nS \u{003e} 40 nS".to_string())
        );
        assert_eq!(checks.score.interactions, BigDecimal::from(0));
        assert_eq!(checks.score.uptime, BigDecimal::from(0));
        assert_eq!(checks.usability.blocked, false);
        assert_eq!(checks.usability.redundant_ip, false);
        assert_eq!(checks.usability.gouging, true);
        assert_eq!(checks.usability.not_accepting_contracts, false);

        Ok(())
    }

    #[test]
    fn allowlist() -> anyhow::Result<()> {
        let req = allowlist_req();
        assert_eq!(req.path, "./bus/hosts/allowlist");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
["ed25519:6f7ac63891fa2eadeb3031b75817a4beaae91070f485c3d139f1ffd3107d6aa8"]
        "#;

        let resp: Vec<PublicKey> = serde_json::from_str(&json)?;
        assert_eq!(resp.len(), 1);
        assert_eq!(
            resp.get(0).unwrap(),
            &"ed25519:6f7ac63891fa2eadeb3031b75817a4beaae91070f485c3d139f1ffd3107d6aa8"
                .try_into()?
        );

        Ok(())
    }

    #[test]
    fn modify_allowlist() -> anyhow::Result<()> {
        let json = r#"
        {
    "add": [],
    "remove": ["ed25519:6f7ac63891fa2eadeb3031b75817a4beaae91070f485c3d139f1ffd3107d6aa8"],
    "clear": false
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = modify_allowlist_req(ModifyAction::AddRemove {
            add: None,
            remove: Some(vec![
                "ed25519:6f7ac63891fa2eadeb3031b75817a4beaae91070f485c3d139f1ffd3107d6aa8"
                    .try_into()?,
            ]),
        })?;
        assert_eq!(req.path, "./bus/hosts/allowlist");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn blocklist() -> anyhow::Result<()> {
        let req = blocklist_req();
        assert_eq!(req.path, "./bus/hosts/blocklist");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
["siacentral.ddnsfree.com","siacentral.mooo.com"]
        "#;

        let resp: Vec<String> = serde_json::from_str(&json)?;
        assert_eq!(resp.len(), 2);
        assert_eq!(resp.get(0).unwrap(), "siacentral.ddnsfree.com");
        assert_eq!(resp.get(1).unwrap(), "siacentral.mooo.com");

        Ok(())
    }

    #[test]
    fn modify_blocklist() -> anyhow::Result<()> {
        let json = r#"
        {
    "add": [],
    "remove": ["siacentral.ddnsfree.com","siacentral.mooo.com","51.158.108.244", "45.148.30.56"],
    "clear": false
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = modify_blocklist_req(ModifyAction::AddRemove {
            add: None,
            remove: Some(vec![
                "siacentral.ddnsfree.com".to_string(),
                "siacentral.mooo.com".to_string(),
                "51.158.108.244".to_string(),
                "45.148.30.56".to_string(),
            ]),
        })?;
        assert_eq!(req.path, "./bus/hosts/blocklist");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn remove() -> anyhow::Result<()> {
        let json = r#"
        {
    "minRecentScanFailures": 3,
    "maxDowntimeHours": 1000
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let req = remove_req(3, 1000, Some("foo".to_string()))?;
        assert_eq!(req.path, "./bus/hosts/remove");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, Some(vec![("bucket".into(), "foo".into())]));
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        let json = "0";
        let resp: u64 = serde_json::from_str(&json)?;
        assert_eq!(resp, 0);

        Ok(())
    }

    #[test]
    fn scanning() -> anyhow::Result<()> {
        let req = scanning_req(
            None,
            Some(10.try_into()?),
            Some(DateTime::parse_from_rfc3339("2023-03-30T15:12:15+00:00")?),
        );
        assert_eq!(req.path, "./bus/hosts/scanning");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(
            req.params,
            Some(vec![
                ("limit".into(), "10".into()),
                ("lastScan".into(), "2023-03-30T15:12:15+00:00".into())
            ])
        );
        assert_eq!(req.content, None);

        let json = "0";
        let resp: u64 = serde_json::from_str(&json)?;
        assert_eq!(resp, 0);

        let json = r#"
        [
  {
    "publicKey": "ed25519:de9e1fd0e7c19b23ac2271a3a4bceed161108d16ab708922c4573cf53fa82dfa",
    "netAddress": "87.255.6.177:9982"
  }
]
        "#;

        let resp: Vec<HostAddress> = serde_json::from_str(&json)?;
        assert_eq!(resp.len(), 1);
        assert_eq!(
            resp.get(0).unwrap().public_key,
            "ed25519:de9e1fd0e7c19b23ac2271a3a4bceed161108d16ab708922c4573cf53fa82dfa"
                .try_into()?
        );
        assert_eq!(resp.get(0).unwrap().net_address, "87.255.6.177:9982");

        Ok(())
    }

    #[test]
    fn get_by_key() -> anyhow::Result<()> {
        let req = get_by_key_req(
            &"ed25519:b050c0c63f9f3b4d5a89acadf628e8d8c6f8768e38fbe731e429334e0fd2cece"
                .try_into()?,
        );
        assert_eq!(
            req.path,
            "./bus/host/ed25519:b050c0c63f9f3b4d5a89acadf628e8d8c6f8768e38fbe731e429334e0fd2cece"
        );
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = "0";
        let resp: u64 = serde_json::from_str(&json)?;
        assert_eq!(resp, 0);

        let json = r#"
        {
    "knownSince": "2023-02-20T19:15:26.84+01:00",
    "lastAnnouncement": "2024-03-01T13:57:41Z",
    "publicKey": "ed25519:e14888420f7df8001990283b44e469e922c0dc14dc2d0156a31fbb6524d08008",
    "netAddress": "shawnhomesc.no-ip.org:9982",
    "priceTable": {
      "uid": "daab22a125d9fa4ffbfb099cbba2e06b",
      "validity": 600000000000,
      "hostblockheight": 410818,
      "updatepricetablecost": "1",
      "accountbalancecost": "1",
      "fundaccountcost": "1",
      "latestrevisioncost": "253600000000000000",
      "subscriptionmemorycost": "1",
      "subscriptionnotificationcost": "1",
      "initbasecost": "0",
      "memorytimecost": "1",
      "downloadbandwidthcost": "75000000000000",
      "uploadbandwidthcost": "150000000000000",
      "dropsectorsbasecost": "1",
      "dropsectorsunitcost": "1",
      "hassectorbasecost": "1",
      "readbasecost": "0",
      "readlengthcost": "1",
      "renewcontractcost": "100000000000000000",
      "revisionbasecost": "0",
      "swapsectorcost": "1",
      "writebasecost": "0",
      "writelengthcost": "1",
      "writestorecost": "92592592592",
      "txnfeeminrecommended": "10000000000000000000",
      "txnfeemaxrecommended": "30000000000000000000",
      "contractprice": "150000000000000000000000",
      "collateralcost": "23148148148",
      "maxcollateral": "2500000000000000000000000000",
      "maxduration": 27216,
      "windowsize": 144,
      "registryentriesleft": 15578716,
      "registryentriestotal": 15625024,
      "expiry": "2024-07-04T12:19:01.025014279Z"
    },
    "settings": {
      "acceptingcontracts": true,
      "baserpcprice": "0",
      "collateral": "23148148148",
      "contractprice": "150000000000000000000000",
      "downloadbandwidthprice": "75000000000000",
      "ephemeralaccountexpiry": 604800000000000,
      "maxcollateral": "2500000000000000000000000000",
      "maxdownloadbatchsize": 17825792,
      "maxduration": 27216,
      "maxephemeralaccountbalance": "1000000000000000000000000",
      "maxrevisebatchsize": 17825792,
      "netaddress": "shawnhomesc.no-ip.org:9982",
      "remainingstorage": 2051270508544,
      "revisionnumber": 35680855,
      "sectoraccessprice": "0",
      "sectorsize": 4194304,
      "siamuxport": "9983",
      "storageprice": "92592592592",
      "totalstorage": 4748354781184,
      "unlockhash": "326c36421b0ad2deee9f54fa98d1829e2a917cfc9febeeaa4f20b9121b434a7e8dad6b2f9578",
      "uploadbandwidthprice": "150000000000000",
      "version": "1.5.9",
      "release": "hostd 58db87c",
      "windowsize": 144
    },
    "interactions": {
      "totalScans": 36,
      "lastScan": "2023-03-29T15:42:35.974329629+02:00",
      "lastScanSuccess": true,
      "lostSectors": 0,
      "secondToLastScanSuccess": true,
      "uptime": 3072358865133168,
      "downtime": 90631369071825,
      "successfulInteractions": 207,
      "failedInteractions": 21
    },
    "scanned": true,
    "blocked": false,
    "checks": {
      "autopilot": {
        "gouging": {
          "contractErr": "",
          "downloadErr": "",
          "gougingErr": "rpc price too high, 100 nS \u003e 40 nS",
          "pruneErr": "",
          "uploadErr": ""
        },
        "score": {
          "age": 0,
          "collateral": 0,
          "interactions": 0,
          "storageRemaining": 0,
          "uptime": 0,
          "version": 0,
          "prices": 0
        },
        "usability": {
          "blocked": false,
          "offline": false,
          "lowScore": false,
          "redundantIP": false,
          "gouging": true,
          "notAcceptingContracts": false,
          "notAnnounced": false,
          "notCompletingScan": false
        }
      }
    },
    "storedData": 0
  }
        "#;

        let resp: Host = serde_json::from_str(&json)?;
        assert_eq!(
            resp.public_key,
            "ed25519:e14888420f7df8001990283b44e469e922c0dc14dc2d0156a31fbb6524d08008"
                .try_into()?
        );
        assert_eq!(resp.net_address, "shawnhomesc.no-ip.org:9982");
        assert_eq!(resp.stored_data, 0);

        Ok(())
    }

    #[test]
    fn reset_lost_sectors() -> anyhow::Result<()> {
        let req = reset_lost_sectors_req(
            &"ed25519:5150ae4bed4a2da68211243ded72a6fc166c860560023fd6f7221e54f5f478da"
                .try_into()?,
        );
        assert_eq!(req.path, "./bus/host/ed25519:5150ae4bed4a2da68211243ded72a6fc166c860560023fd6f7221e54f5f478da/resetlostsectors");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);
        Ok(())
    }
}
