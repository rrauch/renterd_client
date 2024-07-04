use crate::Error::InvalidDataError;
use crate::{ClientInner, Error, Hash};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Api {
    inner: Arc<ClientInner>,
}

impl Api {
    pub(super) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    pub async fn list(
        &self,
        offset: Option<NonZeroUsize>,
        limit: Option<NonZeroUsize>,
    ) -> Result<(Vec<Alert>, bool), Error> {
        let offset = offset.map(|o| o.to_string()).unwrap_or("0".to_string());
        let limit = limit.map(|l| l.to_string()).unwrap_or("-1".to_string());
        let mut params = Vec::with_capacity(2);
        params.push(("offset", offset));
        params.push(("limit", limit));

        let response: ListResponse =
            serde_json::from_value(self.inner.get_json("./bus/alerts", Some(params)).await?)
                .map_err(|e| InvalidDataError(e.into()))?;

        Ok((response.alerts.unwrap_or(vec![]), response.has_more))
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Alert {
    pub id: Hash,
    pub severity: Severity,
    pub message: String,
    pub data: Option<BTreeMap<String, Value>>,
    pub timestamp: DateTime<FixedOffset>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct ListResponse {
    alerts: Option<Vec<Alert>>,
    has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list() -> anyhow::Result<()> {
        let json = r#"
{
	"alerts":
        [
  {
    "id": "h:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d7c",
    "severity": "error",
    "message": "failed to refill account: couldn't fund account: unable to fetch revision with contract: LatestRevision: DialStream: could not dial transport: dial tcp 47.187.112.34:9983: connect: no route to host (2.245964157s)\n",
    "data": {
      "accountID": "ed25519:80b1f19914f46b24334f27e713b95b1b2f8db2c1fcb6a46bdf220f0a2898ba81",
      "contractID": "fcid:e4e00f9de8b61ed6d372c908d986ea30bb2ccf2a08c73291ebc7eaa872c271c2",
      "hostKey": "ed25519:5c42512594e19e8a31395163de1877b29d15ce03b2de5c2a59e91d67f7c24383",
      "origin": "autopilot.autopilot"
    },
    "timestamp": "2023-08-30T14:48:37.500057361Z"
  },
  {
    "id": "h:95e6a83685b5007bb7b080740d508a881d26aefdbb3bb78701584ff6576aeacc",
    "severity": "error",
    "message": "failed to refill account: couldn't fund account: unable to fetch revision with contract: LatestRevision: failed to fetch pricetable, err: host price table gouging: {{   } {  MaxCollateral is below minimum: ~20.06 uS < 100 SC }}\n",
    "timestamp": "2023-08-30T14:48:36.195164983Z"
  },
  {
    "id": "h:ff24699354782e8cf58d7074f3fa63c030ac0d81d674e141542006de99ecfa36",
    "severity": "info",
    "message": "wallet is low on funds",
    "data": {
      "address": "addr:a9adb468928455e381f8468fff2e5d0dc95e0755aef27daa9d845ed40565bf696f2637c7b19e",
      "balance": "141738724911491675264573908846",
      "origin": "autopilot.autopilot"
    },
    "timestamp": "2023-08-30T14:45:19.922778399Z"
  },
    {
    "id": "h:94e6a83685b5007bb7b080740d508a881d26aefdbb3bb78701584ff6576aeacb",
    "severity": "warning",
    "message": "this is a test",
    "data": {
      "setAdditions": {
				"fcid:xxxxxxxxxxxxxxxxxxxx": {
					"additions": [
						{
							"size": 0,
							"time": "2024-06-21T03:11:02.841188843Z"
						}
					],
					"hostKey": "ed25519:xxxxxxxxxxxxxxxxxxxxxxxxx"
				}
			},
      "setRemovals": {}
    },
    "timestamp": "2023-08-30T14:45:19.922778399Z"
  }
  ],
	"hasMore": false,
	"totals": {
		"info": 1,
		"warning": 1,
		"error": 2,
		"critical": 0
	}
}

        "#;

        let alerts_response: ListResponse = serde_json::from_str(&json)?;
        assert_eq!(alerts_response.has_more, false);
        let alerts: Vec<Alert> = alerts_response.alerts.unwrap();
        assert_eq!(4, alerts.len());

        let alert = alerts.get(0).unwrap();
        assert_eq!(
            alert.id,
            "h:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d7c".try_into()?
        );
        assert_eq!(alert.message, "failed to refill account: couldn't fund account: unable to fetch revision with contract: LatestRevision: DialStream: could not dial transport: dial tcp 47.187.112.34:9983: connect: no route to host (2.245964157s)\n");
        assert_eq!(alert.severity, Severity::Error);
        assert_eq!(
            alert.timestamp,
            DateTime::parse_from_rfc3339("2023-08-30T14:48:37.500057361Z")?
        );
        let data = alert.data.as_ref().unwrap();
        assert_eq!(4, data.len());
        assert_eq!(
            data.get("contractID").unwrap(),
            "fcid:e4e00f9de8b61ed6d372c908d986ea30bb2ccf2a08c73291ebc7eaa872c271c2"
        );
        assert_eq!(
            data.last_key_value()
                .map(|(k, v)| (k, v.as_str().unwrap()))
                .unwrap(),
            (&"origin".to_string(), "autopilot.autopilot")
        );

        assert!(alerts.get(1).unwrap().data.is_none());

        let alert = alerts.get(2).unwrap();
        assert_eq!(alert.message, "wallet is low on funds");
        assert_eq!(alert.severity, Severity::Info);
        assert_eq!(
            alert.timestamp,
            DateTime::parse_from_rfc3339("2023-08-30T14:45:19.922778399Z")?
        );
        let data = alert.data.as_ref().unwrap();
        assert_eq!(3, data.len());
        assert_eq!(
            data.get("balance").unwrap(),
            "141738724911491675264573908846"
        );
        assert_eq!(
            data.first_key_value()
                .map(|(k, v)| (k, v.as_str().unwrap()))
                .unwrap(),
            (
                &"address".to_string(),
                "addr:a9adb468928455e381f8468fff2e5d0dc95e0755aef27daa9d845ed40565bf696f2637c7b19e"
            )
        );

        let data = alerts.get(3).unwrap().data.as_ref().unwrap();
        assert_eq!(data.len(), 2);

        let json = r#"
        {
	"alerts": null,
	"hasMore": false,
	"totals": {
		"info": 0,
		"warning": 0,
		"error": 0,
		"critical": 0
	}
}
"#;

        let alerts_response: ListResponse = serde_json::from_str(&json)?;
        assert!(alerts_response.alerts.is_none());

        Ok(())
    }
}
