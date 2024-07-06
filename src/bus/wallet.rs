use crate::Error::InvalidDataError;
use crate::{ClientInner, Error};
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

    pub async fn list(&self) -> Result<Wallet, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/wallet", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Wallet {
    pub scan_height: u64,
    pub address: String,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub spendable: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub confirmed: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub unconfirmed: BigDecimal,
}

#[cfg(test)]
mod tests {
    use crate::bus::wallet::Wallet;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn deserialize_wallet() -> anyhow::Result<()> {
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
        assert_eq!(
            wallet.spendable,
            BigDecimal::from_str("78424071338002381489614636705")?
        );
        assert_eq!(
            wallet.confirmed,
            BigDecimal::from_str("78424071338002381489614636705")?
        );
        assert_eq!(wallet.unconfirmed, BigDecimal::from_str("0")?);
        Ok(())
    }
}
