use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, U128Wrapper};
use chrono::{DateTime, FixedOffset};
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

    pub async fn state(&self) -> Result<State, Error> {
        Ok(self
            .inner
            .send_api_request(state_req())
            .await?
            .json()
            .await?)
    }

    pub async fn network(&self) -> Result<State, Error> {
        Ok(self
            .inner
            .send_api_request(network_req())
            .await?
            .json()
            .await?)
    }

    pub async fn sia_fund_fee(&self, payout: u128) -> Result<u128, Error> {
        let resp: U128Wrapper = self
            .inner
            .send_api_request(sia_fund_fee_req(payout))
            .await?
            .json()
            .await?;

        Ok(resp.0)
    }

    //todo: implement `accept_block` function
}

fn sia_fund_fee_req(payout: u128) -> ApiRequest {
    ApiRequestBuilder::get(format!("./bus/consensus/siafundfee/{}", payout)).build()
}

fn network_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/consensus/network").build()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Network {
    #[serde(rename = "Name")]
    pub name: String,
}

fn state_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/consensus/state").build()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    pub block_height: u64,
    pub last_block_time: DateTime<FixedOffset>,
    pub synced: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestType;

    #[test]
    fn state() -> anyhow::Result<()> {
        let req = state_req();
        assert_eq!(req.path, "./bus/consensus/state");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
  "blockHeight": 436326,
  "lastBlockTime": "2023-09-22T14:37:32Z",
  "synced": true
}
        "#;

        let state: State = serde_json::from_str(&json)?;
        assert_eq!(state.block_height, 436326);
        assert_eq!(state.synced, true);
        assert_eq!(
            state.last_block_time,
            DateTime::parse_from_rfc3339("2023-09-22T14:37:32Z")?
        );

        Ok(())
    }

    #[test]
    fn network() -> anyhow::Result<()> {
        let req = network_req();
        assert_eq!(req.path, "./bus/consensus/network");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        {
	"Name": "zen"
}
        "#;

        let network: Network = serde_json::from_str(&json)?;
        assert_eq!(network.name, "zen");

        Ok(())
    }

    #[test]
    fn sia_fund_fee() -> anyhow::Result<()> {
        let req = sia_fund_fee_req(900000);
        assert_eq!(req.path, "./bus/consensus/siafundfee/900000");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        "30000"
        "#;

        let resp: U128Wrapper = serde_json::from_str(&json)?;
        assert_eq!(resp.0, 30000);

        Ok(())
    }
}
