use crate::autopilot::Autopilot;
use crate::bus::Bus;
use crate::worker::Worker;
use bandwidth::Bandwidth;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, FixedOffset};
use reqwest::{Client as ReqwestClient, Response};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use url::Url;
use zeroize::Zeroize;

pub mod autopilot;
pub mod bus;
pub mod worker;

#[derive(Clone)]
pub struct Client {
    bus: Bus,
    autopilot: Autopilot,
    worker: Worker,
}

impl Client {
    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn autopilot(&self) -> &Autopilot {
        &self.autopilot
    }

    pub fn worker(&self) -> &Worker {
        &self.worker
    }
}

struct ClientInner {
    api_endpoint_url: Url,
    api_password: String,
    reqwest_client: ReqwestClient,
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        self.api_password.zeroize();
    }
}

enum RequestType<'a> {
    Get(Option<Vec<(&'a str, String)>>),
    Post(Option<RequestContent>, Option<Vec<(&'a str, String)>>),
    Put(Option<RequestContent>, Option<Vec<(&'a str, String)>>),
}

enum RequestContent {
    Json(Value),
}

impl ClientInner {
    pub(crate) async fn get_json<'a>(
        &self,
        endpoint: &str,
        params: Option<Vec<(&'a str, String)>>,
    ) -> Result<Value, Error> {
        let bytes = self
            .send_api_request(endpoint, &RequestType::Get(params))
            .await?
            .bytes()
            .await?;
        Ok(
            serde_json::from_slice(bytes.as_ref())
                .map_err(|e| Error::InvalidDataError(e.into()))?,
        )
    }

    async fn send_api_request(
        &self,
        endpoint: &str,
        request: &RequestType<'_>,
    ) -> Result<Response, Error> {
        let url = self
            .api_endpoint_url
            .join(endpoint)
            .expect("endpoint url join error");

        let (mut request_builder, content, params) = match request {
            RequestType::Get(params) => (self.reqwest_client.get(url), &None, params),
            RequestType::Post(content, params) => (self.reqwest_client.post(url), content, params),
            RequestType::Put(content, params) => (self.reqwest_client.put(url), content, params),
        };

        if let Some(params) = params {
            request_builder = request_builder.query(params);
        }

        if let Some(content) = content {
            match content {
                RequestContent::Json(json) => request_builder = request_builder.json(json),
            }
        }

        let req = request_builder
            .basic_auth("api", Some(&self.api_password))
            .build()?;

        let resp = self.reqwest_client.execute(req).await?;

        if resp.status().as_u16() == 401 {
            return Err(Error::AuthenticationError);
        }

        let _ = resp.error_for_status_ref()?;

        Ok(resp)
    }
}

#[derive(Error, Debug)]
pub enum ClientBuilderError {
    #[error("api endpoint is missing, you need to specify a valid url before building the client")]
    MissingApiEndpointUrl,
    #[error("api endpoint `{0}` is not invalid")]
    InvalidApiEndpoint(String),
    #[error("api password is missing, you need to specify a password before building the client")]
    MissingApiPassword,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid data")]
    InvalidDataError(#[from] InvalidDataError),
    #[error("incorrect api password")]
    AuthenticationError,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum InvalidDataError {
    #[error("invalid key {0}")]
    InvalidKey(String),
    #[error("unsupported public key {0}")]
    UnsupportedPublicKey(String),
    #[error(transparent)]
    InvalidJson(#[from] serde_json::Error),
    #[error("invalid hash {0}")]
    InvalidHash(String),
    #[error("unsupported hash {0}")]
    UnsupportedHash(String),
    #[error("invalid fcid {0}")]
    InvalidFileContractId(String),
    #[error("unsupported fcid {0}")]
    UnsupportedFileContractId(String),
    #[error("invalid settings id {0}")]
    InvalidSettingsId(String),
    #[error("invalid percentage {0}")]
    InvalidPercentage(String),
}

pub struct ClientBuilder {
    api_endpoint_url: Option<String>,
    api_password: Option<String>,
    accept_invalid_certs: bool,
    verbose_logging: bool,
}

impl Drop for ClientBuilder {
    fn drop(&mut self) {
        if let Some(mut password) = self.api_password.take() {
            password.zeroize();
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            api_endpoint_url: None,
            api_password: None,
            accept_invalid_certs: false,
            verbose_logging: false,
        }
    }

    pub fn api_endpoint_url<S: AsRef<str>>(mut self, api_endpoint_url: S) -> Self {
        self.api_endpoint_url = Some(api_endpoint_url.as_ref().to_string());
        self
    }

    pub fn api_password(mut self, api_password: String) -> Self {
        self.api_password = Some(api_password);
        self
    }

    pub fn danger_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    pub fn verbose_logging(mut self, enable_verbose_logging: bool) -> Self {
        self.verbose_logging = enable_verbose_logging;
        self
    }

    pub fn build(mut self) -> Result<Client, ClientBuilderError> {
        let api_endpoint_url = match self.api_endpoint_url.take() {
            Some(s) => {
                let url: Url = s
                    .as_str()
                    .try_into()
                    .map_err(|_| ClientBuilderError::InvalidApiEndpoint(s))?;

                let scheme = url.scheme();
                if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
                    return Err(ClientBuilderError::InvalidApiEndpoint(url.to_string()));
                }

                if !url.has_host() {
                    return Err(ClientBuilderError::InvalidApiEndpoint(url.to_string()));
                }

                url
            }
            None => {
                return Err(ClientBuilderError::MissingApiEndpointUrl);
            }
        };

        let api_password = self
            .api_password
            .take()
            .ok_or(ClientBuilderError::MissingApiPassword)?;

        let reqwest_client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(self.accept_invalid_certs)
            .connection_verbose(self.verbose_logging)
            .build()
            .map_err(|e| ClientBuilderError::ReqwestError(e))?;

        let inner = Arc::new(ClientInner {
            api_endpoint_url,
            api_password,
            reqwest_client,
        });

        Ok(Client {
            bus: Bus::new(inner.clone()),
            autopilot: Autopilot::new(inner.clone()),
            worker: Worker::new(inner),
        })
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct State {
    pub start_time: DateTime<FixedOffset>,
    pub network: String,
    pub version: String,
    pub commit: String,
    pub os: String,
    pub build_time: DateTime<FixedOffset>,
}

#[derive(PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub enum PublicKey {
    Ed25519([u8; 32]),
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyVisitor)
    }
}

struct PublicKeyVisitor;

impl<'de> Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.try_into().map_err(|e| serde::de::Error::custom(e))?)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicKey::Ed25519(bytes) => {
                f.write_fmt(format_args!("ed25519:{}", hex::encode(bytes)))
            }
        }
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<&str> for PublicKey {
    type Error = InvalidDataError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.strip_prefix("ed25519:") {
            Some(hex) => {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(hex, &mut bytes)
                    .map_err(|_| InvalidDataError::InvalidKey(s.to_string()))?;
                Ok(PublicKey::Ed25519(bytes))
            }
            None => Err(InvalidDataError::UnsupportedPublicKey(s.to_string())),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Hash {
    Hash256([u8; 32]),
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Hash::Hash256(bytes) => f.write_fmt(format_args!("h:{}", hex::encode(bytes))),
        }
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<&str> for Hash {
    type Error = InvalidDataError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.strip_prefix("h:") {
            Some(hex) => {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(hex, &mut bytes)
                    .map_err(|_| InvalidDataError::InvalidHash(s.to_string()))?;
                Ok(Hash::Hash256(bytes))
            }
            None => Err(InvalidDataError::UnsupportedHash(s.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.try_into().map_err(|e| serde::de::Error::custom(e))?)
    }
}

struct HashVisitor;

#[derive(PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub enum FileContractId {
    Hash256([u8; 32]),
}

impl Display for FileContractId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileContractId::Hash256(bytes) => {
                f.write_fmt(format_args!("fcid:{}", hex::encode(bytes)))
            }
        }
    }
}

impl Debug for FileContractId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<&str> for FileContractId {
    type Error = InvalidDataError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.strip_prefix("fcid:") {
            Some(hex) => {
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(hex, &mut bytes)
                    .map_err(|_| InvalidDataError::InvalidFileContractId(s.to_string()))?;
                Ok(FileContractId::Hash256(bytes))
            }
            None => Err(InvalidDataError::UnsupportedFileContractId(s.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for FileContractId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FileContractIdVisitor)
    }
}

impl<'de> Visitor<'de> for FileContractIdVisitor {
    type Value = FileContractId;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.try_into().map_err(|e| serde::de::Error::custom(e))?)
    }
}

struct FileContractIdVisitor;

pub(crate) mod duration_ns {
    use bigdecimal::ToPrimitive;
    use serde::de::Visitor;
    use serde::{Deserializer, Serializer};
    use std::fmt::Formatter;
    use std::time::Duration;

    pub fn serialize<S>(v: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(
            serializer.serialize_u64(v.as_nanos().to_u64().ok_or(serde::ser::Error::custom(
                "nanoseconds cannot be represented as u64",
            ))?)?,
        )
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(DurationVisitor)
    }

    struct DurationVisitor;
    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a nanosecond number")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Duration::from_nanos(v))
        }
    }
}

pub(crate) mod duration_ms {
    use bigdecimal::ToPrimitive;
    use serde::de::Visitor;
    use serde::{Deserializer, Serializer};
    use std::fmt::Formatter;
    use std::time::Duration;

    pub fn serialize<S>(v: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(
            serializer.serialize_u64(v.as_millis().to_u64().ok_or(
                serde::ser::Error::custom("milliseconds cannot be represented as u64"),
            )?)?,
        )
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(DurationVisitor)
    }

    struct DurationVisitor;
    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a millisecond number")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Duration::from_millis(v))
        }
    }
}

pub(crate) mod number_as_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn serialize<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ToString,
        S: Serializer,
    {
        Ok(serializer.serialize_str(v.to_string().as_str())?)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr + serde::Deserialize<'de>,
        <T as FromStr>::Err: Display,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt<T> {
            String(String),
            Number(T),
        }

        match StringOrInt::<T>::deserialize(deserializer)? {
            StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
            StringOrInt::Number(i) => Ok(i),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub struct SettingsId([u8; 16]);

impl Display for SettingsId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", hex::encode(self.0)))
    }
}

impl Debug for SettingsId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl TryFrom<&str> for SettingsId {
    type Error = InvalidDataError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; 16];
        hex::decode_to_slice(s, &mut bytes)
            .map_err(|_| InvalidDataError::InvalidSettingsId(s.to_string()))?;
        Ok(SettingsId(bytes))
    }
}

impl<'de> Deserialize<'de> for SettingsId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SettingsIdVisitor)
    }
}

struct SettingsIdVisitor;

impl<'de> Visitor<'de> for SettingsIdVisitor {
    type Value = SettingsId;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.try_into().map_err(|e| serde::de::Error::custom(e))?)
    }
}

fn deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<String>, E>
        where
            E: serde::de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_owned()))
            }
        }

        fn visit_none<E>(self) -> Result<Option<String>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }
    }

    deserializer.deserialize_option(StringVisitor)
}

fn deserialize_mbps_float<'de, D>(deserializer: D) -> Result<Bandwidth, D::Error>
where
    D: Deserializer<'de>,
{
    struct BandwidthVisitor;

    fn to_bandwidth(mbps: f64) -> Bandwidth {
        let gbps = mbps / 1_000.0;
        let full_gbps = gbps.trunc() as u64;
        let remaining_bps = ((mbps - (full_gbps as f64 * 1_000.0)) * 1_000_000.0) as u32;
        Bandwidth::new(full_gbps, remaining_bps)
    }

    impl<'de> Visitor<'de> for BandwidthVisitor {
        type Value = Bandwidth;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(to_bandwidth(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(to_bandwidth(v as f64))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            //let value = map.next_value::<Number>()?;
            match map.next_value::<String>()?.parse::<f64>() {
                Ok(v) => Ok(to_bandwidth(v)),
                Err(_) => Err(serde::de::Error::custom("Invalid number")),
            }
        }
    }

    deserializer.deserialize_any(BandwidthVisitor)
}

#[derive(Deserialize)]
pub(crate) struct U128Wrapper(#[serde(with = "crate::number_as_string")] u128);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Percentage {
    inner: BigDecimal,
}

impl Percentage {
    fn new(value: BigDecimal) -> Self {
        Self { inner: value / 100 }
    }

    pub fn as_big_decimal(&self) -> &BigDecimal {
        &self.inner
    }
}

impl TryFrom<&str> for Percentage {
    type Error = InvalidDataError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Percentage::new(BigDecimal::from_str(value).map_err(
            |_| InvalidDataError::InvalidPercentage(value.to_string()),
        )?))
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PercentageVisitor)
    }
}

struct PercentageVisitor;

impl<'de> Visitor<'de> for PercentageVisitor {
    type Value = Percentage;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.try_into().map_err(|e| serde::de::Error::custom(e))?)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Percentage::new(BigDecimal::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Percentage::new(
            BigDecimal::from_f64(v).ok_or(serde::de::Error::custom("failed to parse f64"))?,
        ))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        match BigDecimal::from_str(map.next_value::<String>()?.as_str()) {
            Ok(bd) => Ok(Percentage::new(bd)),
            Err(_) => Err(serde::de::Error::custom("Invalid number")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::Zero;

    #[test]
    fn public_key_handling() -> anyhow::Result<()> {
        let valid_str = "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75";
        let valid_key: PublicKey = valid_str.try_into()?;
        assert_eq!(valid_str, valid_key.to_string());

        match TryInto::<PublicKey>::try_into(
            "ed25519:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead7",
        ) {
            Err(InvalidDataError::InvalidKey(_)) => {}
            _ => panic!("invalid key error expected"),
        }

        match TryInto::<PublicKey>::try_into(
            "foo:99611c808ccb74402f0c80ea0b22cefe3b46a73abe1072c90687658d44dead75",
        ) {
            Err(InvalidDataError::UnsupportedPublicKey(_)) => {}
            _ => panic!("unsupported public key error expected"),
        }

        Ok(())
    }

    #[test]
    fn hash_handling() -> anyhow::Result<()> {
        let valid_str = "h:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d7c";
        let valid_hash: Hash = valid_str.try_into()?;
        assert_eq!(valid_str, valid_hash.to_string());

        match TryInto::<Hash>::try_into(
            "h:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d",
        ) {
            Err(InvalidDataError::InvalidHash(_)) => {}
            _ => panic!("invalid hash error expected"),
        }

        match TryInto::<Hash>::try_into(
            "foo:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d7c",
        ) {
            Err(InvalidDataError::UnsupportedHash(_)) => {}
            _ => panic!("unsupported hash error expected"),
        }

        Ok(())
    }

    #[test]
    fn fcid_handling() -> anyhow::Result<()> {
        let valid_str = "fcid:d41536902fedd6717e16839df5a6022c1d0663ebc2f44f8ad4a7bb743313dabd";
        let valid_fcid: FileContractId = valid_str.try_into()?;
        assert_eq!(valid_str, valid_fcid.to_string());

        match TryInto::<FileContractId>::try_into(
            "fcid:f78694e6db65d95389eb271a9239810701a7f1df199564f51b1fc6c1c7935d",
        ) {
            Err(InvalidDataError::InvalidFileContractId(_)) => {}
            _ => panic!("invalid fcid error expected"),
        }

        match TryInto::<FileContractId>::try_into(
            "foo:d41536902fedd6717e16839df5a6022c1d0663ebc2f44f8ad4a7bb743313dabd",
        ) {
            Err(InvalidDataError::UnsupportedFileContractId(_)) => {}
            _ => panic!("unsupported fcid error expected"),
        }

        Ok(())
    }

    #[test]
    fn settings_id_handling() -> anyhow::Result<()> {
        let valid_str = "defb754518682448a13b2e30fff7c2ae";
        let valid_id: SettingsId = valid_str.try_into()?;
        assert_eq!(valid_str, valid_id.to_string());

        match TryInto::<SettingsId>::try_into("defb754518682448a13b2e30fff7c2a") {
            Err(InvalidDataError::InvalidSettingsId(_)) => {}
            _ => panic!("invalid settings error expected"),
        }

        Ok(())
    }

    #[test]
    fn bandwidth_deserialization() -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "crate::deserialize_mbps_float")]
            bw: Bandwidth,
        }
        let json_int_zero = r#"{ "bw": 0 }"#;
        let test: Test = serde_json::from_str(json_int_zero)?;
        assert_eq!(test.bw, Bandwidth::from_mbps(0));

        let json_float_zero = r#"{ "bw": 0.0 }"#;
        let test: Test = serde_json::from_str(json_float_zero)?;
        assert_eq!(test.bw, Bandwidth::from_mbps(0));

        let json_int_1 = r#"{ "bw": 1 }"#;
        let test: Test = serde_json::from_str(json_int_1)?;
        assert_eq!(test.bw, Bandwidth::from_mbps(1));

        let json_float = r#"{ "bw": 1000.1 }"#;
        let test: Test = serde_json::from_str(json_float)?;
        assert_eq!(test.bw, Bandwidth::new(1, 100000));

        Ok(())
    }

    #[test]
    fn percentage_deserialization() -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct Test {
            p1: Percentage,
            p2: Percentage,
            p3: Percentage,
            p4: Percentage,
        }

        let json = r#"{
         "p1": 0,
         "p2": 2,
         "p3": 123,
         "p4": 10.25
        }
        "#;
        let test: Test = serde_json::from_str(&json)?;
        assert!(test.p1.as_big_decimal().is_zero());
        assert_eq!(test.p2.as_big_decimal(), &BigDecimal::from_str("0.02")?);
        assert_eq!(test.p3.as_big_decimal(), &BigDecimal::from_str("1.23")?);
        assert_eq!(test.p4.as_big_decimal(), &BigDecimal::from_str("0.1025")?);
        Ok(())
    }
}
