use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, RequestContent, PublicKey, RequestType};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
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

    pub async fn list(&self) -> Result<Vec<Account>, Error> {
        Ok(
            serde_json::from_value(self.inner.get_json("./bus/accounts", None).await?)
                .map_err(|e| InvalidDataError(e.into()))?,
        )
    }

    pub async fn get_or_add_account(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
    ) -> Result<Account, Error> {
        let url = format!("./bus/account/{}", account_id);
        let req = add_req(host_key)?;
        Ok(self
            .inner
            .send_api_request(&url, &req)
            .await?
            .json()
            .await?)
    }

    pub async fn lock(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
        exclusive: bool,
        duration: Duration,
    ) -> Result<(Account, u64), Error> {
        let url = format!("./bus/account/{}/lock", account_id);
        let req = lock_req(host_key, exclusive, duration)?;
        let resp: LockResponse = self
            .inner
            .send_api_request(&url, &req)
            .await?
            .json()
            .await?;
        Ok((resp.account, resp.lock_id))
    }

    pub async fn unlock(&self, account_id: &PublicKey, lock_id: u64) -> Result<(), Error> {
        let url = format!("./bus/account/{}/unlock", account_id);
        let req = unlock_req(lock_id)?;
        let _ = self.inner.send_api_request(&url, &req).await?;
        Ok(())
    }

    pub async fn add_balance(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
        amount: u128,
    ) -> Result<(), Error> {
        let url = format!("./bus/account/{}/add", account_id);
        let req = add_balance_req(host_key, amount)?;
        let _ = self.inner.send_api_request(&url, &req).await?;
        Ok(())
    }

    pub async fn update_balance(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
        amount: u128,
    ) -> Result<(), Error> {
        let url = format!("./bus/account/{}/update", account_id);
        let req = update_balance_req(host_key, amount)?;
        let _ = self.inner.send_api_request(&url, &req).await?;
        Ok(())
    }

    pub async fn requires_sync(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
    ) -> Result<(), Error> {
        let url = format!("./bus/account/{}/requiressync", account_id);
        let req = requires_sync_req(host_key)?;
        let _ = self.inner.send_api_request(&url, &req).await?;
        Ok(())
    }

    pub async fn reset_drift(&self, account_id: &PublicKey) -> Result<(), Error> {
        let url = format!("./bus/account/{}/resetdrift", account_id);
        let _ = self
            .inner
            .send_api_request(&url, &RequestType::Post(None, None))
            .await?;
        Ok(())
    }
}

fn add_req(host_key: &PublicKey) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(AddRequest { host_key })
                .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddRequest<'a> {
    host_key: &'a PublicKey,
}

fn update_balance_req(host_key: &PublicKey, amount: u128) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(UpdateBalanceRequest { host_key, amount })
                .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateBalanceRequest<'a> {
    host_key: &'a PublicKey,
    amount: u128,
}

fn add_balance_req(host_key: &PublicKey, amount: u128) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(AddBalanceRequest { host_key, amount })
                .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddBalanceRequest<'a> {
    host_key: &'a PublicKey,
    amount: u128,
}

fn requires_sync_req(host_key: &PublicKey) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(RequiresSyncRequest { host_key })
                .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequiresSyncRequest<'a> {
    host_key: &'a PublicKey,
}

fn lock_req(
    host_key: &PublicKey,
    exclusive: bool,
    duration: Duration,
) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(LockRequest {
                host_key,
                exclusive,
                duration,
            })
            .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LockRequest<'a> {
    host_key: &'a PublicKey,
    exclusive: bool,
    #[serde(with = "crate::duration_ms")]
    duration: Duration,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct LockResponse {
    account: Account,
    #[serde(rename = "lockID")]
    lock_id: u64,
}

fn unlock_req(lock_id: u64) -> Result<RequestType<'static>, Error> {
    Ok(RequestType::Post(
        Some(RequestContent::Json(
            serde_json::to_value(UnlockRequest { lock_id })
                .map_err(|e| InvalidDataError(e.into()))?,
        )),
        None,
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnlockRequest {
    #[serde(rename = "lockID")]
    lock_id: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: PublicKey,
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
    use serde_json::Value;
    use std::str::FromStr;

    #[test]
    fn list() -> anyhow::Result<()> {
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

    #[test]
    fn lock() -> anyhow::Result<()> {
        let json = r#"
        {
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
    "exclusive": false,
    "duration": 1000
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        match lock_req(
            &"ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?,
            false,
            Duration::from_millis(1000),
        )? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        let json = r#"
        {
  "account": {
    "id": "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75",
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
    "balance": 1e+24,
    "drift": 1e+24,
    "requiresSync": false,
    "cleanShutdown": true
  },
  "lockID": 13874228167312386000
}
        "#;

        let resp: LockResponse = serde_json::from_str(&json)?;
        assert_eq!(
            resp.account.id,
            "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?
        );
        assert_eq!(
            resp.account.host_key,
            "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?
        );
        assert_eq!(resp.account.balance, BigDecimal::from_str("1e+24")?);
        assert_eq!(resp.account.drift, BigDecimal::from_str("1e+24")?);
        assert_eq!(resp.account.requires_sync, false);
        assert_eq!(resp.account.clean_shutdown, true);

        assert_eq!(resp.lock_id, 13874228167312386000);

        Ok(())
    }

    #[test]
    fn unlock() -> anyhow::Result<()> {
        let json = r#"
        {
    "lockID": 13874228167312385374
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        match unlock_req(13874228167312385374)? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }

    #[test]
    fn requires_sync() -> anyhow::Result<()> {
        let json = r#"
        {
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        let host_key: PublicKey =
            "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?;
        match requires_sync_req(&host_key)? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }

    #[test]
    fn add_balance() -> anyhow::Result<()> {
        let json = r#"{
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
    "amount": 1000000
}
"#;
        let expected: Value = serde_json::from_str(&json)?;

        let host_key: PublicKey =
            "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?;
        match add_balance_req(&host_key, 1000000)? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }

    #[test]
    fn update_balance() -> anyhow::Result<()> {
        let json = r#"{
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
    "amount": 22221111
}
"#;
        let expected: Value = serde_json::from_str(&json)?;

        let host_key: PublicKey =
            "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?;
        match update_balance_req(&host_key, 22221111)? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        Ok(())
    }

    #[test]
    fn add() -> anyhow::Result<()> {
        let json = r#"
        {
    "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
}
        "#;
        let expected: Value = serde_json::from_str(&json)?;

        match add_req(
            &"ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?,
        )? {
            RequestType::Post(Some(RequestContent::Json(json)), None) => {
                assert_eq!(json, expected)
            }
            _ => panic!("invalid request"),
        }

        let json = r#"
        {
  "id": "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75",
  "hostKey": "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9",
  "balance": 1e+24,
  "drift": 1e+24,
  "requiresSync": false,
  "cleanShutdown": true
}
        "#;

        let account: Account = serde_json::from_str(&json)?;
        assert_eq!(
            account.id,
            "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?
        );
        assert_eq!(
            account.host_key,
            "ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?
        );
        assert_eq!(account.balance, BigDecimal::from_str("1e+24")?);
        assert_eq!(account.drift, BigDecimal::from_str("1e+24")?);
        assert_eq!(account.requires_sync, false);
        assert_eq!(account.clean_shutdown, true);

        Ok(())
    }
}
