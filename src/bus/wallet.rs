use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, Hash};
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

    pub async fn list(&self) -> Result<Wallet, Error> {
        Ok(self
            .inner
            .send_api_request(&list_req())
            .await?
            .json()
            .await?)
    }

    pub async fn outputs(&self) -> Result<Vec<Output>, Error> {
        Ok(self
            .inner
            .send_api_request(&outputs_req())
            .await?
            .json()
            .await?)
    }

    //todo: implement missing wallet functions
}

fn list_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/wallet").build()
}

fn outputs_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/wallet/outputs").build()
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Wallet {
    pub scan_height: u64,
    pub address: String, //todo
    #[serde(with = "crate::number_as_string")]
    pub spendable: u128,
    #[serde(with = "crate::number_as_string")]
    pub confirmed: u128,
    #[serde(with = "crate::number_as_string")]
    pub unconfirmed: u128,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Output {
    #[serde(with = "crate::number_as_string")]
    pub value: u128,
    pub address: String, //todo
    pub id: Hash,
    pub maturity_height: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;

    #[test]
    fn wallet() -> anyhow::Result<()> {
        let req = list_req();
        assert_eq!(req.path, "./bus/wallet");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "scanHeight": 436326,
  "address": "addr:9e5c7ee27eae74e278e7470d44163b08db21d8137ed04e476b742cd76f0b6deb1c7f6f10dcfe",
  "spendable": "78424071338002381489614636705",
  "confirmed": "78424071338002381489614636705",
  "unconfirmed": "0"
}
"#;
        let wallet: Wallet = serde_json::from_str(&json)?;
        assert_eq!(wallet.scan_height, 436326);
        assert_eq!(wallet.spendable, 78424071338002381489614636705);
        assert_eq!(wallet.confirmed, 78424071338002381489614636705);
        assert_eq!(wallet.unconfirmed, 0);
        Ok(())
    }

    #[test]
    fn outputs() -> anyhow::Result<()> {
        let req = outputs_req();
        assert_eq!(req.path, "./bus/wallet/outputs");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        [
  {
    "value": "130303561734677732679493600",
    "address": "addr:9e5c7ee27eae74e278e7470d44163b08db21d8137ed04e476b742cd76f0b6deb1c7f6f10dcfe",
    "id": "h:59d605fd783cfd4d0511f00d9e569cf71d416e68cb4f17424d85995c4c7674ab",
    "maturityHeight": 1122
  },
  {
    "value": "6840004092910992448143033193",
    "address": "addr:9e5c7ee27eae74e278e7470d44163b08db21d8137ed04e476b742cd76f0b6deb1c7f6f10dcfe",
    "id": "h:9918606661349b56fcb75786f719563dbc4170594bbe56b9c557b60c2d5776e1",
    "maturityHeight": 11223
  },
  {
    "value": "437078594556495665233100000",
    "address": "addr:9e5c7ee27eae74e278e7470d44163b08db21d8137ed04e476b742cd76f0b6deb1c7f6f10dcfe",
    "id": "h:d1e61b964297ab9e45c4829d42de0712b96d67d265024141851f4c7b94f3d6ee",
    "maturityHeight": 112233
  }
]
        "#;
        let outputs: Vec<Output> = serde_json::from_str(&json)?;
        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs.get(0).unwrap().value, 130303561734677732679493600);
        assert_eq!(outputs.get(0).unwrap().maturity_height, 1122);
        assert_eq!(
            outputs.get(1).unwrap().id,
            "h:9918606661349b56fcb75786f719563dbc4170594bbe56b9c557b60c2d5776e1".try_into()?
        );
        //todo: address

        Ok(())
    }
}
