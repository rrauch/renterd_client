use crate::Error::InvalidDataError;
use crate::{Address, ClientInner, Error, PublicKey};
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

    pub async fn list(&self) -> Result<Vec<Account>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/accounts", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Account {
    pub id: Address,
    pub clean_shutdown: bool,
    pub host_key: PublicKey,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub balance: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub drift: BigDecimal,
    pub requires_sync: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
        [
  {
    "id": "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75",
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
    "balance": 1e+24,
    "drift": 1e+24,
    "requiresSync": false,
    "cleanShutdown": true
  },
  {
    "id": "ed25519:ac4c45c00fec02272f6f63aa015606d7fdd7a6c91669b6bb06930796d68ea293",
    "hostKey": "ed25519:70b75b1acff1f80f9ace0c048ce8651586254e23d19ba405dc6f226e81d08ca2",
    "balance": 9.353633845598274e+23,
    "drift": 9.3538858455984e+23,
    "requiresSync": false,
    "cleanShutdown": false
  },
  {
    "id": "ed25519:24c36bd8c237827a467d06ba616df3fa9a22e111c33f4803059f80719f22efc0",
    "hostKey": "ed25519:fe9cee676b1a6c92ebe430e88f10bd97fef7bf444d8519b5f23a34cee808447b",
    "balance": 5.7933767945738696e+23,
    "drift": 5.7947627945745646e+23,
    "requiresSync": false,
    "cleanShutdown": true
  }
  ]
        "#;

        let accounts: Vec<Account> = serde_json::from_str(&json)?;
        assert_eq!(3, accounts.len());

        let account = accounts.get(0).unwrap();
        assert_eq!(
            account.id,
            "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?
        );
        assert_eq!(account.balance, BigDecimal::from_str("1e+24")?);
        assert_eq!(account.requires_sync, false);

        let account = accounts.get(2).unwrap();
        assert_eq!(
            account.host_key,
            "ed25519:fe9cee676b1a6c92ebe430e88f10bd97fef7bf444d8519b5f23a34cee808447b"
                .try_into()?
        );
        assert_eq!(
            account.drift,
            BigDecimal::from_str("5.7947627945745646e+23")?
        );
        assert_eq!(account.clean_shutdown, true);

        Ok(())
    }
}
