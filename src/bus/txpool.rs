use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, U128Wrapper};
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
        let wrapper: U128Wrapper = serde_json::from_value(
            self.inner
                .get_json("./bus/txpool/recommendedfee", None)
                .await?,
        )
        .map_err(|e| InvalidDataError(e.into()))?;
        Ok(wrapper.0)
    }

    //todo: implement transactions
    pub async fn transactions(&self) -> Result<Vec<Value>, Error> {
        Ok(serde_json::from_value(
            self.inner
                .get_json("./bus/txpool/transactions", None)
                .await?,
        )
        .map_err(|e| InvalidDataError(e.into()))?)
    }
}

#[cfg(test)]
mod tests {
    use crate::U128Wrapper;

    #[test]
    fn deserialize_fee() -> anyhow::Result<()> {
        let json = r#"
        "30000000000000000000""#;
        let fee: U128Wrapper = serde_json::from_str(&json)?;
        assert_eq!(fee.0, 30000000000000000000);
        Ok(())
    }
}
