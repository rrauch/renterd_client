use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
use bigdecimal::BigDecimal;
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

    pub async fn recommended_fee(&self) -> Result<BigDecimal, Error> {
        Ok(serde_json::from_value(
            self.inner
                .get_json("./bus/txpool/recommendedfee", None)
                .await?,
        )
        .map_err(|e| InvalidDataError(e.into()))?)
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
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn deserialize_fee() -> anyhow::Result<()> {
        let json = r#"
        "30000000000000000000""#;
        let fee: BigDecimal = serde_json::from_str(&json)?;
        assert_eq!(fee, BigDecimal::from_str("30000000000000000000")?);
        Ok(())
    }
}
