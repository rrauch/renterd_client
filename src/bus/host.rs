use crate::deserialize_option_string;
use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, PublicKey, SettingsId};
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::collections::BTreeMap;
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

    pub async fn list(&self) -> Result<Vec<Host>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/hosts", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
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
    #[serde(with = "bigdecimal::serde::json_num")]
    pub update_price_table_cost: BigDecimal,
    #[serde(rename = "accountbalancecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub account_balance_cost: BigDecimal,
    #[serde(rename = "fundaccountcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub fund_account_cost: BigDecimal,
    #[serde(rename = "latestrevisioncost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub latest_revision_cost: BigDecimal,
    #[serde(rename = "subscriptionmemorycost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub subscription_memory_cost: BigDecimal,
    #[serde(rename = "subscriptionnotificationcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub subscription_notification_cost: BigDecimal,
    #[serde(rename = "initbasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub init_base_cost: BigDecimal,
    #[serde(rename = "memorytimecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub memory_time_cost: BigDecimal,
    #[serde(rename = "downloadbandwidthcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub download_bandwidth_cost: BigDecimal,
    #[serde(rename = "uploadbandwidthcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub upload_bandwidth_cost: BigDecimal,
    #[serde(rename = "dropsectorsbasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub drop_sector_base_cost: BigDecimal,
    #[serde(rename = "dropsectorsunitcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub drop_sector_unit_cost: BigDecimal,
    #[serde(rename = "hassectorbasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub has_sector_base_cost: BigDecimal,
    #[serde(rename = "readbasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub read_base_cost: BigDecimal,
    #[serde(rename = "readlengthcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub read_length_cost: BigDecimal,
    #[serde(rename = "renewcontractcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub renew_contract_cost: BigDecimal,
    #[serde(rename = "revisionbasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub revision_base_cost: BigDecimal,
    #[serde(rename = "swapsectorcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub swap_sector_base_cost: BigDecimal,
    #[serde(rename = "writebasecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub write_base_cost: BigDecimal,
    #[serde(rename = "writelengthcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub write_length_cost: BigDecimal,
    #[serde(rename = "writestorecost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub write_store_cost: BigDecimal,
    #[serde(rename = "txnfeeminrecommended")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub txn_fee_min_recommended: BigDecimal,
    #[serde(rename = "txnfeemaxrecommended")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub txn_fee_max_recommended: BigDecimal,
    #[serde(rename = "contractprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub contract_price: BigDecimal,
    #[serde(rename = "collateralcost")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub collateral_cost: BigDecimal,
    #[serde(rename = "maxcollateral")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub max_collateral: BigDecimal,
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
    #[serde(with = "bigdecimal::serde::json_num")]
    pub collateral: BigDecimal,
    #[serde(rename = "maxcollateral")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub max_collateral: BigDecimal,
    #[serde(rename = "baserpcprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub base_rpc_price: BigDecimal,
    #[serde(rename = "contractprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub contract_price: BigDecimal,
    #[serde(rename = "downloadbandwidthprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub download_bandwidth_price: BigDecimal,
    #[serde(rename = "sectoraccessprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub sector_access_price: BigDecimal,
    #[serde(rename = "storageprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub storage_price: BigDecimal,
    #[serde(rename = "uploadbandwidthprice")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub upload_bandwidth_price: BigDecimal,
    #[serde(rename = "ephemeralaccountexpiry")]
    #[serde(with = "crate::duration_ns")]
    pub ephemeral_account_expiry: Duration,
    #[serde(rename = "maxephemeralaccountbalance")]
    #[serde(with = "bigdecimal::serde::json_num")]
    pub max_ephemeral_account_balance: BigDecimal,
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
    #[serde(deserialize_with = "deserialize_option_string")]
    pub contract_error: Option<String>,
    #[serde(rename = "downloadErr")]
    #[serde(deserialize_with = "deserialize_option_string")]
    pub download_error: Option<String>,
    #[serde(rename = "gougingErr")]
    #[serde(deserialize_with = "deserialize_option_string")]
    pub gouging_error: Option<String>,
    #[serde(rename = "pruneErr")]
    #[serde(deserialize_with = "deserialize_option_string")]
    pub prune_error: Option<String>,
    #[serde(rename = "uploadErr")]
    #[serde(deserialize_with = "deserialize_option_string")]
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
    use std::str::FromStr;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
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
        assert_eq!(
            price_table.upload_bandwidth_cost,
            BigDecimal::from_str("200000000000000")?
        );
        assert_eq!(price_table.window_size, 144);
        assert_eq!(
            price_table.expiry,
            DateTime::parse_from_rfc3339("2024-07-04T12:19:01.025014279Z")?
        );

        let settings = &hosts.get(2).unwrap().settings;
        assert_eq!(settings.accepting_contracts, true);
        assert_eq!(settings.base_rpc_price, BigDecimal::from_str("0")?);
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
}
