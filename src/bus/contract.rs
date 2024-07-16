use crate::Error::InvalidDataError;
use crate::{
    ApiRequest, ApiRequestBuilder, ClientInner, Error, FileContractId, Hash, PublicKey,
    RequestContent,
};
use serde::{Deserialize, Serialize};
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

    pub async fn get_all(&self, contract_set: Option<String>) -> Result<Vec<Contract>, Error> {
        Ok(self
            .inner
            .send_api_request(&get_all_req(contract_set))
            .await?
            .json()
            .await?)
    }

    pub async fn get_by_id(&self, contract_id: &FileContractId) -> Result<Contract, Error> {
        Ok(self
            .inner
            .send_api_request(&get_by_id_req(contract_id))
            .await?
            .json()
            .await?)
    }

    //todo: add support for missing `add_contract` function

    pub async fn delete(&self, contract_id: &FileContractId) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&delete_req(contract_id))
            .await?;
        Ok(())
    }

    pub async fn acquire(
        &self,
        contract_id: &FileContractId,
        duration: Duration,
        priority: i32,
    ) -> Result<u64, Error> {
        let resp: AcquireResponse = self
            .inner
            .send_api_request(&acquire_req(contract_id, duration, priority)?)
            .await?
            .json()
            .await?;
        Ok(resp.lock_id)
    }

    pub async fn ancestors(
        &self,
        contract_id: &FileContractId,
        min_start_height: Option<u64>,
    ) -> Result<Vec<ArchivedContract>, Error> {
        Ok(self
            .inner
            .send_api_request(&ancestors_req(contract_id, min_start_height))
            .await?
            .json()
            .await?)
    }

    pub async fn prunable(&self) -> Result<Prunable, Error> {
        Ok(self
            .inner
            .send_api_request(&prunable_req())
            .await?
            .json()
            .await?)
    }

    pub async fn contract_sets(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .inner
            .send_api_request(&contract_sets_req())
            .await?
            .json()
            .await?)
    }

    pub async fn delete_all(&self) -> Result<(), Error> {
        let _ = self.inner.send_api_request(&delete_all_req()).await?;
        Ok(())
    }

    pub async fn archive<S: AsRef<str>>(
        &self,
        contract_ids: &Vec<(FileContractId, S)>,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&archive_req(contract_ids)?)
            .await?;
        Ok(())
    }

    pub async fn renewed(&self, contract_id: &FileContractId) -> Result<Contract, Error> {
        Ok(self
            .inner
            .send_api_request(&renewed_req(contract_id))
            .await?
            .json()
            .await?)
    }

    pub async fn create_contract_set<S: AsRef<str>>(
        &self,
        name: S,
        contract_ids: &Vec<FileContractId>,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&create_contract_set_req(name, contract_ids)?)
            .await?;
        Ok(())
    }

    pub async fn delete_contract_set<S: AsRef<str>>(&self, name: S) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&delete_contract_set_req(name))
            .await?;
        Ok(())
    }

    pub async fn update_spending(
        &self,
        contract_id: &FileContractId,
        revision_number: u64,
        size: u64,
        spending: &Spending,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&update_spending_req(
                contract_id,
                revision_number,
                size,
                spending,
            )?)
            .await?;
        Ok(())
    }

    pub async fn keep_alive(
        &self,
        contract_id: &FileContractId,
        duration: Duration,
        lock_id: u64,
    ) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&keep_alive_req(contract_id, duration, lock_id)?)
            .await?;
        Ok(())
    }

    //todo: implement missing `renewed` function

    pub async fn release(&self, contract_id: &FileContractId, lock_id: u64) -> Result<(), Error> {
        let _ = self
            .inner
            .send_api_request(&release_req(contract_id, lock_id)?)
            .await?;
        Ok(())
    }

    pub async fn roots(
        &self,
        contract_id: &FileContractId,
    ) -> Result<(Option<Vec<Hash>>, Option<Vec<Hash>>), Error> {
        let resp: RootsResponse = self
            .inner
            .send_api_request(&roots_req(contract_id))
            .await?
            .json()
            .await?;
        Ok((resp.roots, resp.uploading))
    }

    pub async fn size(&self, contract_id: &FileContractId) -> Result<(u64, u64), Error> {
        let resp: SizeResponse = self
            .inner
            .send_api_request(&size_req(contract_id))
            .await?
            .json()
            .await?;
        Ok((resp.prunable, resp.size))
    }
}

fn size_req(contract_id: &FileContractId) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/contract/{}/size", contract_id)).build()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SizeResponse {
    prunable: u64,
    size: u64,
}

fn roots_req(contract_id: &FileContractId) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/contract/{}/roots", contract_id)).build()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RootsResponse {
    roots: Option<Vec<Hash>>,
    uploading: Option<Vec<Hash>>,
}

fn release_req(contract_id: &FileContractId, lock_id: u64) -> Result<ApiRequest, Error> {
    let url = format!("./bus/contract/{}/release", contract_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(ReleaseRequest { lock_id }).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseRequest {
    #[serde(rename = "lockID")]
    lock_id: u64,
}

fn keep_alive_req(
    contract_id: &FileContractId,
    duration: Duration,
    lock_id: u64,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/contract/{}/keepalive", contract_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(KeepAliveRequest { duration, lock_id })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeepAliveRequest {
    #[serde(with = "crate::duration_ms")]
    duration: Duration,
    #[serde(rename = "lockID")]
    lock_id: u64,
}

fn ancestors_req(contract_id: &FileContractId, min_start_height: Option<u64>) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/contract/{}/ancestors", contract_id))
        .params(min_start_height.map(|msh| vec![("minStartHeight", msh.to_string())]))
        .build()
}

fn acquire_req(
    contract_id: &FileContractId,
    duration: Duration,
    priority: i32,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/contract/{}/acquire", contract_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(AcquireRequest { duration, priority })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AcquireRequest {
    #[serde(with = "crate::duration_ms")]
    duration: Duration,
    priority: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AcquireResponse {
    #[serde(rename = "lockID")]
    lock_id: u64,
}

fn delete_req(contract_id: &FileContractId) -> ApiRequest {
    ApiRequestBuilder::delete(format!("./bus/contract/{}", contract_id)).build()
}

fn get_by_id_req(contract_id: &FileContractId) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/contract/{}", contract_id)).build()
}

fn update_spending_req(
    contract_id: &FileContractId,
    revision_number: u64,
    size: u64,
    spending: &Spending,
) -> Result<ApiRequest, Error> {
    //todo: clarify what needs to be done with `missedHostPayout` and `validRenterPayout`
    let content = Some(RequestContent::Json(
        serde_json::to_value(vec![UpdateSpendingRequest {
            contract_id,
            revision_number,
            size,
            spending,
        }])
        .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/contracts/spending")
        .content(content)
        .build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSpendingRequest<'a> {
    #[serde(rename = "contractID")]
    contract_id: &'a FileContractId,
    revision_number: u64,
    size: u64,
    #[serde(flatten)]
    spending: &'a Spending,
}

fn delete_contract_set_req<S: AsRef<str>>(name: S) -> ApiRequest {
    ApiRequestBuilder::delete(format!("./bus/contracts/set/{}", name.as_ref())).build()
}

fn create_contract_set_req<S: AsRef<str>>(
    name: S,
    contract_ids: &Vec<FileContractId>,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/contracts/set/{}", name.as_ref());
    let content = Some(RequestContent::Json(
        serde_json::to_value(contract_ids).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::put(url).content(content).build())
}

fn renewed_req(contract_id: &FileContractId) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/contracts/renewed/{}", contract_id)).build()
}

fn archive_req<S: AsRef<str>>(
    contract_ids: &Vec<(FileContractId, S)>,
) -> Result<ApiRequest, Error> {
    let map: BTreeMap<String, &str> = contract_ids
        .iter()
        .map(|(id, reason)| (id.to_string(), reason.as_ref()))
        .collect();
    let content = Some(RequestContent::Json(
        serde_json::to_value(map).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post("./bus/contracts/archive")
        .content(content)
        .build())
}

fn delete_all_req() -> ApiRequest {
    ApiRequestBuilder::delete("./bus/contracts/all").build()
}

fn contract_sets_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/contracts/sets").build()
}

fn prunable_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/contracts/prunable").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Prunable {
    pub contracts: Vec<PrunableContract>,
    pub total_prunable: u64,
    pub total_size: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PrunableContract {
    pub id: FileContractId,
    pub prunable: u64,
    pub size: u64,
}

fn get_all_req(contract_set: Option<String>) -> ApiRequest {
    let params = if let Some(contract_set) = contract_set {
        Some(vec![("contractset", contract_set)])
    } else {
        None
    };

    ApiRequestBuilder::get("./bus/contracts")
        .params(params)
        .build()
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
#[serde(rename_all = "camelCase")]
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
    #[serde(with = "crate::number_as_string")]
    pub contract_price: u128,
    pub renewed_from: FileContractId,
    pub spending: Spending,
    #[serde(with = "crate::number_as_string")]
    pub total_cost: u128,
    pub contract_sets: Option<Vec<String>>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedContract {
    pub id: FileContractId,
    pub host_key: PublicKey,
    pub renewed_to: FileContractId,
    pub spending: Spending,
    pub proof_height: u64,
    pub revision_height: u64,
    pub revision_number: u64,
    pub size: u64,
    pub start_height: u64,
    pub state: State,
    pub window_start: u64,
    pub window_end: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Spending {
    #[serde(with = "crate::number_as_string")]
    uploads: u128,
    #[serde(with = "crate::number_as_string")]
    downloads: u128,
    #[serde(with = "crate::number_as_string")]
    fund_account: u128,
    #[serde(with = "crate::number_as_string")]
    deletions: u128,
    #[serde(with = "crate::number_as_string")]
    sector_roots: u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;
    use serde_json::Value;

    #[test]
    fn get_all() -> anyhow::Result<()> {
        let req = get_all_req(None);
        assert_eq!(req.path, "./bus/contracts");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let req = get_all_req(Some("foo_id".to_string()));
        assert_eq!(
            req.params,
            Some(vec![("contractset".into(), "foo_id".into())])
        );

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
            14400000000000000000000000
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
            529353231686279158451232
        );
        assert_eq!(
            contracts.get(3).unwrap().spending.fund_account,
            1000000000000000000000001
        );
        assert_eq!(contracts.get(3).unwrap().spending.deletions, 100);

        assert_eq!(
            contracts
                .get(1)
                .unwrap()
                .contract_sets
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            contracts
                .get(3)
                .unwrap()
                .contract_sets
                .as_ref()
                .unwrap()
                .len(),
            2
        );

        assert_eq!(
            contracts
                .get(0)
                .unwrap()
                .contract_sets
                .as_ref()
                .unwrap()
                .get(0)
                .unwrap(),
            "autopilot"
        );

        Ok(())
    }

    #[test]
    fn prunable() -> anyhow::Result<()> {
        let req = prunable_req();
        assert_eq!(req.path, "./bus/contracts/prunable");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
{
  "contracts": [
    {
      "id": "fcid:f5be2457ad1e16ce1f54a50a3c09643532f51fabab14974c2de376d14d981067",
      "prunable": 2159022178304,
      "size": 4203732795392
    },
    {
      "id": "fcid:5a40eba5f91a266a02d43b05de25be1150c2013d17b1962a2c450d864b8ba2e7",
      "prunable": 1927324631040,
      "size": 2774909583360
    }
  ],
  "totalPrunable": 72393854812160,
  "totalSize": 250864786735104
}
        "#;

        let prunable: Prunable = serde_json::from_str(&json)?;
        assert_eq!(prunable.contracts.len(), 2);
        assert_eq!(
            prunable.contracts.get(0).unwrap().id,
            "fcid:f5be2457ad1e16ce1f54a50a3c09643532f51fabab14974c2de376d14d981067".try_into()?
        );
        assert_eq!(prunable.contracts.get(0).unwrap().prunable, 2159022178304);
        assert_eq!(prunable.contracts.get(1).unwrap().size, 2774909583360);
        assert_eq!(prunable.total_prunable, 72393854812160);
        assert_eq!(prunable.total_size, 250864786735104);
        Ok(())
    }

    #[test]
    fn contract_sets() -> anyhow::Result<()> {
        let req = contract_sets_req();
        assert_eq!(req.path, "./bus/contracts/sets");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
[
	"autopilot"
]
        "#;

        let sets: Vec<String> = serde_json::from_str(&json)?;
        assert_eq!(sets.len(), 1);
        assert_eq!(sets.get(0).unwrap(), "autopilot");
        Ok(())
    }

    #[test]
    fn delete_all() -> anyhow::Result<()> {
        let req = delete_all_req();
        assert_eq!(req.path, "./bus/contracts/all");
        assert_eq!(req.request_type, RequestType::Delete);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);
        Ok(())
    }

    #[test]
    fn archive() -> anyhow::Result<()> {
        let req = archive_req(&vec![(
            "fcid:a0b28586a59457d0a8f7c3d06bcc1c45470c95a02d5e4ff9c1ee9972f712d1f0".try_into()?,
            "Some reason for the archival",
        )])?;
        let json = r#"
        {
    "fcid:a0b28586a59457d0a8f7c3d06bcc1c45470c95a02d5e4ff9c1ee9972f712d1f0": "Some reason for the archival"
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contracts/archive");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }

    #[test]
    fn renewed() -> anyhow::Result<()> {
        let req = renewed_req(
            &"fcid:a0b28586a59457d0a8f7c3d06bcc1c45470c95a02d5e4ff9c1ee9972f712d1f0".try_into()?,
        );
        assert_eq!(req.path, "./bus/contracts/renewed/fcid:a0b28586a59457d0a8f7c3d06bcc1c45470c95a02d5e4ff9c1ee9972f712d1f0");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
 {
	"id": "fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e",
	"hostIP": "justanotherhost.ddns.net:9882",
	"hostKey": "ed25519:e156553dd877e99f24a5c02b72d1c1edd75dce76c663f6b939f20d3f7e9f01d9",
	"siamuxAddr": "justanotherhost.ddns.net:9883",
	"proofHeight": 0,
	"revisionHeight": 76518,
	"revisionNumber": 5,
	"size": 0,
	"startHeight": 75509,
	"state": "active",
	"windowStart": 83573,
	"windowEnd": 83717,
	"contractPrice": "200000000000000000000000",
	"renewedFrom": "fcid:e26dcafdbddcede53cb9d24a03caa4917a8196d3733790389b638af6c9b5564b",
	"spending": {
		"uploads": "0",
		"downloads": "0",
		"fundAccount": "1000000000000000000000001",
		"deletions": "0",
		"sectorRoots": "0"
	},
	"totalCost": "2614400000000000000000000",
	"contractSets": null
}
        "#;
        let contract: Contract = serde_json::from_str(&json)?;
        assert_eq!(
            contract.id,
            "fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e".try_into()?
        );
        assert_eq!(contract.size, 0);
        assert_eq!(contract.total_cost, 2614400000000000000000000);
        assert_eq!(contract.contract_sets, None);

        Ok(())
    }

    #[test]
    fn create_contract_set() -> anyhow::Result<()> {
        let req = create_contract_set_req(
            "foo_set",
            &vec![
                "fcid:93c26cb56eb1048da7582f0f929415389a8352ca91cece7b2885297e5d5703a7"
                    .try_into()?,
                "fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434"
                    .try_into()?,
            ],
        )?;

        let json = r#"
        ["fcid:93c26cb56eb1048da7582f0f929415389a8352ca91cece7b2885297e5d5703a7", "fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434"]
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contracts/set/foo_set");
        assert_eq!(req.request_type, RequestType::Put);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }

    #[test]
    fn delete_contract_set() -> anyhow::Result<()> {
        let req = delete_contract_set_req("foobar");
        assert_eq!(req.path, "./bus/contracts/set/foobar");
        assert_eq!(req.request_type, RequestType::Delete);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);
        Ok(())
    }

    #[test]
    fn update_spending() -> anyhow::Result<()> {
        let req = update_spending_req(
            &"fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434".try_into()?,
            1,
            4194304,
            &Spending {
                downloads: 0,
                uploads: 100,
                deletions: 0,
                fund_account: 0,
                sector_roots: 0,
            },
        )?;

        let json = r#"
        [
    {
        "contractID": "fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434",
        "revisionNumber": 1,
        "size": 4194304,
        "uploads": "100",
        "deletions": "0",
        "downloads": "0",
        "fundAccount": "0",
        "sectorRoots": "0"
    }
]
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contracts/spending");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
        Ok(())
    }

    #[test]
    fn get_by_id() -> anyhow::Result<()> {
        let req = get_by_id_req(
            &"fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434".try_into()?,
        );
        assert_eq!(
            req.path,
            "./bus/contract/fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434"
        );
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
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
        "#;
        let contract: Contract = serde_json::from_str(&json)?;
        assert_eq!(contract.size, 65812824064);
        assert_eq!(contract.spending.deletions, 100);
        assert_eq!(contract.window_end, 452130);

        Ok(())
    }

    #[test]
    fn delete() -> anyhow::Result<()> {
        let req = delete_req(
            &"fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434".try_into()?,
        );
        assert_eq!(
            req.path,
            "./bus/contract/fcid:76db85736f888e8d5715124de37d0bcef81b2ae2cac2155aa8b8c64103e5a434"
        );
        assert_eq!(req.request_type, RequestType::Delete);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);
        Ok(())
    }

    #[test]
    fn acquire() -> anyhow::Result<()> {
        let req = acquire_req(
            &"fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b".try_into()?,
            Duration::from_millis(10000),
            10,
        )?;

        let json = r#"
        {
    "duration": 10000,
    "priority": 10
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contract/fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b/acquire");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        let json = r#"
        {
  "lockID": 609920465282217500
}
        "#;
        let resp: AcquireResponse = serde_json::from_str(&json)?;
        assert_eq!(resp.lock_id, 609920465282217500);

        Ok(())
    }

    #[test]
    fn ancestors() -> anyhow::Result<()> {
        let req = ancestors_req(
            &"fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b".try_into()?,
            Some(10101),
        );

        assert_eq!(req.path, "./bus/contract/fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b/ancestors");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(
            req.params,
            Some(vec![("minStartHeight".into(), "10101".into())])
        );
        assert_eq!(req.content, None);

        let json = r#"
 [
  {
    "id": "fcid:2485444e9b4086cc8299c200cb0ed3dca2107c5735f94f76cc2c99d5033f5134",
    "hostKey": "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16",
    "renewedTo": "fcid:8759504666fda45730bec2b97655035d8fd57c825da1cff37224b3ad76cca44f",
    "spending": {
      "uploads": "1090546428887263463997440",
      "downloads": "0",
      "fundAccount": "42978359151727049434292394",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "proofHeight": 0,
    "revisionHeight": 441490,
    "revisionNumber": 1844674407370955200,
    "size": 0,
    "startHeight": 436131,
    "state": "complete",
    "windowStart": 443094,
    "windowEnd": 443238
  },
  {
    "id": "fcid:c75ecb905e90f6ee20ee592e9d86aa3d8374a38c28f649a24d5e4c4962a1e406",
    "hostKey": "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16",
    "renewedTo": "fcid:2485444e9b4086cc8299c200cb0ed3dca2107c5735f94f76cc2c99d5033f5134",
    "spending": {
      "uploads": "3110447958017610172727296",
      "downloads": "0",
      "fundAccount": "17990344568438185069915138",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "proofHeight": 0,
    "revisionHeight": 436131,
    "revisionNumber": 1844674407370955200,
    "size": 0,
    "startHeight": 429019,
    "state": "complete",
    "windowStart": 437046,
    "windowEnd": 437190
  },
  {
    "id": "fcid:d4178f2f3003a67b14ea677c87ce556ce9f17cf8e14c749e6cc1b7b43e9aef67",
    "hostKey": "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16",
    "renewedTo": "fcid:c75ecb905e90f6ee20ee592e9d86aa3d8374a38c28f649a24d5e4c4962a1e406",
    "spending": {
      "uploads": "12873175201530040418304",
      "downloads": "0",
      "fundAccount": "0",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "proofHeight": 0,
    "revisionHeight": 429020,
    "revisionNumber": 1844674407370955200,
    "size": 0,
    "startHeight": 428823,
    "state": "complete",
    "windowStart": 430998,
    "windowEnd": 431142
  },
  {
    "id": "fcid:3fb5e0b9c3526b0998a16eab9863b16452a87de319d88696c5153c04838215f3",
    "hostKey": "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16",
    "renewedTo": "fcid:d4178f2f3003a67b14ea677c87ce556ce9f17cf8e14c749e6cc1b7b43e9aef67",
    "spending": {
      "uploads": "13094433078586754937257984",
      "downloads": "0",
      "fundAccount": "1000000000000000000000001",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "proofHeight": 0,
    "revisionHeight": 428823,
    "revisionNumber": 1844674407370955200,
    "size": 0,
    "startHeight": 422776,
    "state": "complete",
    "windowStart": 430839,
    "windowEnd": 430983
  },
  {
    "id": "fcid:c92cee85afbf62e6905be22100b47ed5f0db5ad17df49d579053cf69ca217352",
    "hostKey": "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16",
    "renewedTo": "fcid:3fb5e0b9c3526b0998a16eab9863b16452a87de319d88696c5153c04838215f3",
    "spending": {
      "uploads": "666479193656818139791360",
      "downloads": "0",
      "fundAccount": "4500778226686476151712781",
      "deletions": "0",
      "sectorRoots": "0"
    },
    "proofHeight": 0,
    "revisionHeight": 422777,
    "revisionNumber": 1844674407370955200,
    "size": 0,
    "startHeight": 416731,
    "state": "complete",
    "windowStart": 424792,
    "windowEnd": 424936
  }
]
        "#;
        let resp: Vec<ArchivedContract> = serde_json::from_str(&json)?;
        assert_eq!(resp.len(), 5);
        assert_eq!(
            resp.get(0).unwrap().id,
            "fcid:2485444e9b4086cc8299c200cb0ed3dca2107c5735f94f76cc2c99d5033f5134".try_into()?
        );
        assert_eq!(
            resp.get(1).unwrap().host_key,
            "ed25519:dfb16d76de07c537ad62647b39cba0497a6e339dfb1644bd8f8cda95893b1f16"
                .try_into()?
        );
        assert_eq!(
            resp.get(2).unwrap().renewed_to,
            "fcid:c75ecb905e90f6ee20ee592e9d86aa3d8374a38c28f649a24d5e4c4962a1e406".try_into()?
        );
        assert_eq!(resp.get(3).unwrap().revision_height, 428823);
        assert_eq!(resp.get(4).unwrap().state, State::Complete);

        Ok(())
    }

    #[test]
    fn keep_alive() -> anyhow::Result<()> {
        let req = keep_alive_req(
            &"fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b".try_into()?,
            Duration::from_millis(10000),
            609920465282217447,
        )?;

        let json = r#"
        {
    "duration": 10000,
    "lockID": 609920465282217447
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contract/fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b/keepalive");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn release() -> anyhow::Result<()> {
        let req = release_req(
            &"fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b".try_into()?,
            609920465282217447,
        )?;

        let json = r#"
        {
    "lockID": 609920465282217447
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        assert_eq!(req.path, "./bus/contract/fcid:06025daad00bb361df5a897b33a82ec24f61499757a3a4b7053a921314b9099b/release");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

        Ok(())
    }

    #[test]
    fn roots() -> anyhow::Result<()> {
        let req = roots_req(
            &"fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e".try_into()?,
        );

        assert_eq!(req.path, "./bus/contract/fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e/roots");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
{
  "roots": [
    "h:910c1669ef60f4f2ae6d47f736bb5b4268a6326adae0cba2cffaee62d9c27443",
    "h:10f91c26e84bea5882e02e8bd14697ccd3f8513dc58a65eab8a7295d53b6d47c",
    "h:fda69eaaab99f5b7bb7f4100da4499548901550041ae3b05fe43f1894054c408"
  ],
  "uploading": null
}
        "#;
        let resp: RootsResponse = serde_json::from_str(&json)?;
        assert!(resp.roots.is_some());
        assert!(resp.uploading.is_none());
        let roots = resp.roots.unwrap();
        assert_eq!(roots.len(), 3);
        assert_eq!(
            roots.get(0).unwrap(),
            &"h:910c1669ef60f4f2ae6d47f736bb5b4268a6326adae0cba2cffaee62d9c27443".try_into()?
        );
        assert_eq!(
            roots.get(1).unwrap(),
            &"h:10f91c26e84bea5882e02e8bd14697ccd3f8513dc58a65eab8a7295d53b6d47c".try_into()?
        );
        assert_eq!(
            roots.get(2).unwrap(),
            &"h:fda69eaaab99f5b7bb7f4100da4499548901550041ae3b05fe43f1894054c408".try_into()?
        );

        Ok(())
    }

    #[test]
    fn size() -> anyhow::Result<()> {
        let req = size_req(
            &"fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e".try_into()?,
        );

        assert_eq!(req.path, "./bus/contract/fcid:9573152b5a294ef910f08a3f18af8bf7b51a4c6ae108c0bd7c3d973db7d6c89e/size");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
{
  "prunable": 144149839872,
  "size": 377936150528
}
        "#;
        let resp: SizeResponse = serde_json::from_str(&json)?;
        assert_eq!(resp.prunable, 144149839872);
        assert_eq!(resp.size, 377936150528);

        Ok(())
    }
}
