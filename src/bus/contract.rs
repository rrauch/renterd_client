use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, FileContractId, PublicKey};
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

    pub async fn list(&self) -> Result<Vec<Contract>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/contracts", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub enum State {
    Invalid,
    Unknown,
    Pending,
    Active,
    Complete,
    Failed,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub enum ArchivalReason {
    #[serde(rename = "hostpruned")]
    HostPruned,
    Removed,
    Renewed,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Contract {
    pub id: FileContractId,
    #[serde(rename = "hostIP")]
    pub host_ip: String,
    pub host_key: PublicKey,
    pub siamux_addr: String,
    pub proof_height: u64,
    pub revision_height: u64,
    pub revision_number: u64,
    pub size: u64,
    pub start_height: u64,
    pub state: State,
    pub window_start: u64,
    pub window_end: u64,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub contract_price: BigDecimal,
    pub renewed_from: FileContractId,
    pub spending: Spending,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub total_cost: BigDecimal,
    pub contract_sets: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Spending {
    #[serde(with = "bigdecimal::serde::json_num")]
    uploads: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    downloads: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    fund_account: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    deletions: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    sector_roots: BigDecimal,
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
    "id": "fcid:d41536902fedd6717e16839df5a6022c1d0663ebc2f44f8ad4a7bb743313dabd",
    "hostIP": "90.188.3.70:9982",
    "hostKey": "ed25519:d129f18cdfb12fa426fe60f5f2f77e2498d173977e88a037328d1a0fa8b56d68",
    "siamuxAddr": "90.188.3.70:9983",
    "proofHeight": 0,
    "revisionHeight": 448237,
    "revisionNumber": 1536,
    "size": 52131004416,
    "startHeight": 448236,
    "state": "active",
    "windowStart": 451986,
    "windowEnd": 452130,
    "contractPrice": "150000000000000000000000",
    "renewedFrom": "fcid:f61b41b930b162e88cb325f04c3ee8b214da247a362fd683901709820e073798",
    "spending": {
      "uploads": "529353231686279158451232",
      "downloads": "0",
      "fundAccount": "0",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "totalCost": "14400000000000000000000000",
    "contractSets": [
      "autopilot"
    ]
  },
  {
    "id": "fcid:7836fd93b7560322b9bf3848a818f95055f34a9153c035189b9431038f3a701a",
    "hostIP": "aliensstorj1.ddns.net:9982",
    "hostKey": "ed25519:a71661d9f854a4d6f93e9b120f07efc75facfd9bd2cb26de4c3559b74316eb75",
    "siamuxAddr": "aliensstorj1.ddns.net:9983",
    "proofHeight": 0,
    "revisionHeight": 448001,
    "revisionNumber": 1743,
    "size": 2705326080,
    "startHeight": 443944,
    "state": "active",
    "windowStart": 451986,
    "windowEnd": 452130,
    "contractPrice": "0",
    "renewedFrom": "fcid:0000000000000000000000000000000000000000000000000000000000000000",
    "spending": {
      "uploads": "228067790528778513022976",
      "downloads": "0",
      "fundAccount": "0",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "totalCost": "10000000000000000000000000",
    "contractSets": [
      "autopilot"
    ]
  },
  {
    "id": "fcid:3fb286004e515545c1c78e9578ef691776f12d952808cc4190710f5eb43f3c7f",
    "hostIP": "radar-storj.ddns.net:9982",
    "hostKey": "ed25519:6b1e236a60b73a647af694c99b6c7b9e4b55368ead1a81a119e4616522d8632e",
    "siamuxAddr": "radar-storj.ddns.net:9983",
    "proofHeight": 0,
    "revisionHeight": 447984,
    "revisionNumber": 10291,
    "size": 21189623808,
    "startHeight": 443944,
    "state": "pending",
    "windowStart": 451986,
    "windowEnd": 452130,
    "contractPrice": "0",
    "renewedFrom": "fcid:0000000000000000000000000000000000000000000000000000000000000000",
    "spending": {
      "uploads": "2907457802022127790325760",
      "downloads": "0",
      "fundAccount": "3000000000000000000000003",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "totalCost": "10000000000000000000000000",
    "contractSets": [
      "autopilot"
    ]
  },
  {
    "id": "fcid:c2d8326f7fde113cd31c10f7076cf1752ae9a8aa34fd8736c34023468fc598a1",
    "hostIP": "90.188.9.144:8982",
    "hostKey": "ed25519:607c893eab14fdc17fc9ee173a40d17121f54a4f1e65c009e45c7840c06c464f",
    "siamuxAddr": "90.188.9.144:8983",
    "proofHeight": 0,
    "revisionHeight": 448681,
    "revisionNumber": 10939,
    "size": 65812824064,
    "startHeight": 447670,
    "state": "complete",
    "windowStart": 451986,
    "windowEnd": 452130,
    "contractPrice": "160000000000000000000000",
    "renewedFrom": "fcid:679f6eb91de592fdc617bdac9608986e957342e88c00b98e4e15207512cb1c53",
    "spending": {
      "uploads": "723651521685372841099264",
      "downloads": "0",
      "fundAccount": "1000000000000000000000001",
      "deletions": "100",
      "sectorRoots": "0"
    },
    "totalCost": "12000000000000000000000000",
    "contractSets": [
      "autopilot",
      "foo"
    ]
  }
  ]
        "#;

        let contracts: Vec<Contract> = serde_json::from_str(&json)?;
        assert_eq!(contracts.len(), 4);

        assert_eq!(State::Active, contracts.get(0).unwrap().state);
        assert_eq!(State::Pending, contracts.get(2).unwrap().state);
        assert_eq!(State::Complete, contracts.get(3).unwrap().state);

        assert_eq!(
            contracts.get(0).unwrap().id,
            "fcid:d41536902fedd6717e16839df5a6022c1d0663ebc2f44f8ad4a7bb743313dabd".try_into()?
        );
        assert_eq!(
            contracts.get(2).unwrap().renewed_from,
            "fcid:0000000000000000000000000000000000000000000000000000000000000000".try_into()?
        );
        assert_eq!(
            contracts.get(3).unwrap().renewed_from,
            "fcid:679f6eb91de592fdc617bdac9608986e957342e88c00b98e4e15207512cb1c53".try_into()?
        );

        assert_eq!(
            contracts.get(0).unwrap().total_cost,
            BigDecimal::from_str("14400000000000000000000000")?
        );

        assert_eq!(
            contracts.get(2).unwrap().host_key,
            "ed25519:6b1e236a60b73a647af694c99b6c7b9e4b55368ead1a81a119e4616522d8632e"
                .try_into()?
        );

        assert_eq!(contracts.get(3).unwrap().host_ip, "90.188.9.144:8982");
        assert_eq!(
            contracts.get(1).unwrap().siamux_addr,
            "aliensstorj1.ddns.net:9983"
        );

        assert_eq!(
            contracts.get(0).unwrap().spending.uploads,
            BigDecimal::from_str("529353231686279158451232")?
        );
        assert_eq!(
            contracts.get(3).unwrap().spending.fund_account,
            BigDecimal::from_str("1000000000000000000000001")?
        );
        assert_eq!(
            contracts.get(3).unwrap().spending.deletions,
            BigDecimal::from_str("100")?
        );

        assert_eq!(contracts.get(1).unwrap().contract_sets.len(), 1);
        assert_eq!(contracts.get(3).unwrap().contract_sets.len(), 2);

        assert_eq!(
            contracts.get(0).unwrap().contract_sets.get(0).unwrap(),
            "autopilot"
        );

        Ok(())
    }
}
