use crate::{ApiRequest, ApiRequestBuilder, ClientInner, Error, U128Wrapper};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn recommended_fee(&self) -> Result<u128, Error> {
        let wrapper: U128Wrapper = self
            .inner
            .send_api_request(&fee_req())
            .await?
            .json()
            .await?;
        Ok(wrapper.0)
    }

    //todo: implement transactions
    pub async fn transactions(&self) -> Result<Vec<Value>, Error> {
        Ok(self.inner.send_api_request(&tx_req()).await?.json().await?)
    }
}

fn fee_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/txpool/recommendedfee").build()
}

fn tx_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/txpool/transactions").build()
}

#[cfg(test)]
mod tests {
    use crate::bus::txpool::fee_req;
    use crate::{RequestType, U128Wrapper};

    #[test]
    fn fee() -> anyhow::Result<()> {
        let req = fee_req();
        assert_eq!(req.path, "./bus/txpool/recommendedfee");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

        let json = r#"
        "30000000000000000000""#;
        let fee: U128Wrapper = serde_json::from_str(&json)?;
        assert_eq!(fee.0, 30000000000000000000);
        Ok(())
    }
}
