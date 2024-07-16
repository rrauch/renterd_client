use crate::Error::InvalidDataError;
use crate::{
    ApiRequest, ApiRequestBuilder, ClientInner, Error, PublicKey, RequestContent, RequestType,
};
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

    pub async fn get_all(&self) -> Result<Vec<Account>, Error> {
        Ok(self
            .inner
            .send_api_request(&get_all_req())
            .await?
            .json()
            .await?)
    }

    pub async fn get_or_add_account(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
    ) -> Result<Account, Error> {
        Ok(self
            .inner
            .send_api_request(&add_req(account_id, host_key)?)
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
        let req = lock_req(account_id, host_key, exclusive, duration)?;
        let resp: LockResponse = self.inner.send_api_request(&req).await?.json().await?;
        Ok((resp.account, resp.lock_id))
    }

    pub async fn unlock(&self, account_id: &PublicKey, lock_id: u64) -> Result<(), Error> {
        let req = unlock_req(account_id, lock_id)?;
        let _ = self.inner.send_api_request(&req).await?;
        Ok(())
    }

    pub async fn add_balance(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
        amount: u128,
    ) -> Result<(), Error> {
        let req = add_balance_req(account_id, host_key, amount)?;
        let _ = self.inner.send_api_request(&req).await?;
        Ok(())
    }

    pub async fn update_balance(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
        amount: u128,
    ) -> Result<(), Error> {
        let req = update_balance_req(account_id, host_key, amount)?;
        let _ = self.inner.send_api_request(&req).await?;
        Ok(())
    }

    pub async fn requires_sync(
        &self,
        account_id: &PublicKey,
        host_key: &PublicKey,
    ) -> Result<(), Error> {
        let req = requires_sync_req(account_id, host_key)?;
        let _ = self.inner.send_api_request(&req).await?;
        Ok(())
    }

    pub async fn reset_drift(&self, account_id: &PublicKey) -> Result<(), Error> {
        let url = format!("./bus/account/{}/resetdrift", account_id);
        let _ = self
            .inner
            .send_api_request(&ApiRequestBuilder::post(url).build())
            .await?;
        Ok(())
    }
}

fn get_all_req() -> ApiRequest {
    ApiRequestBuilder::get("./bus/accounts").build()
}

fn add_req(account_id: &PublicKey, host_key: &PublicKey) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(AddRequest { host_key }).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddRequest<'a> {
    host_key: &'a PublicKey,
}

fn update_balance_req(
    account_id: &PublicKey,
    host_key: &PublicKey,
    amount: u128,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}/update", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(UpdateBalanceRequest { host_key, amount })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateBalanceRequest<'a> {
    host_key: &'a PublicKey,
    amount: u128,
}

fn add_balance_req(
    account_id: &PublicKey,
    host_key: &PublicKey,
    amount: u128,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}/add", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(AddBalanceRequest { host_key, amount })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddBalanceRequest<'a> {
    host_key: &'a PublicKey,
    amount: u128,
}

fn requires_sync_req(account_id: &PublicKey, host_key: &PublicKey) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}/requiressync", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(RequiresSyncRequest { host_key })
            .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequiresSyncRequest<'a> {
    host_key: &'a PublicKey,
}

fn lock_req(
    account_id: &PublicKey,
    host_key: &PublicKey,
    exclusive: bool,
    duration: Duration,
) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}/lock", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(LockRequest {
            host_key,
            exclusive,
            duration,
        })
        .map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
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

fn unlock_req(account_id: &PublicKey, lock_id: u64) -> Result<ApiRequest, Error> {
    let url = format!("./bus/account/{}/unlock", account_id);
    let content = Some(RequestContent::Json(
        serde_json::to_value(UnlockRequest { lock_id }).map_err(|e| InvalidDataError(e.into()))?,
    ));
    Ok(ApiRequestBuilder::post(url).content(content).build())
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
    fn get_all() -> anyhow::Result<()> {
        let req = get_all_req();
        assert_eq!(req.path, "./bus/accounts");
        assert_eq!(req.request_type, RequestType::Get);
        assert_eq!(req.params, None);
        assert_eq!(req.content, None);

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
        let req = lock_req(
            &"ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?,
            &"ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?,
            false,
            Duration::from_millis(1000),
        )?;

        assert_eq!(req.path, "./bus/account/ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75/lock");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

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
        let req = unlock_req(
            &"ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?,
            13874228167312385374,
        )?;
        assert_eq!(req.path, "./bus/account/ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75/unlock");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

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
        let req = requires_sync_req(
            &"ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?,
            &host_key,
        )?;
        assert_eq!(req.path, "./bus/account/ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75/requiressync");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));
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
        let req = add_balance_req(
            &"ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?,
            &host_key,
            1000000,
        )?;
        assert_eq!(req.path, "./bus/account/ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75/add");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

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
        let req = update_balance_req(
            &"ed25519:ee51dac3daae45b63179b7a325443354489d4434f64652bbc30d7e1a3bd8003e"
                .try_into()?,
            &host_key,
            22221111,
        )?;
        assert_eq!(req.path, "./bus/account/ed25519:ee51dac3daae45b63179b7a325443354489d4434f64652bbc30d7e1a3bd8003e/update");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

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

        let req = add_req(
            &"ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75"
                .try_into()?,
            &"ed25519:0c920d0254011f1065eeb99aa909c644b991780c1155ce0aa34cce09e6eabdc9"
                .try_into()?,
        )?;
        assert_eq!(req.path, "./bus/account/ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75");
        assert_eq!(req.request_type, RequestType::Post);
        assert_eq!(req.params, None);
        assert_eq!(req.content, Some(RequestContent::Json(expected)));

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
