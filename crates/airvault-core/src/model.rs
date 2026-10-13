use crate::storage::SecretRecord;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Offline,
    Proxy,
    Combined,
}
impl Mode {
    pub fn can_sign(self) -> bool {
        self != Self::Proxy
    }
    pub fn can_network(self) -> bool {
        self != Self::Offline && cfg!(feature = "network")
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    Bitcoin,
    Testnet,
    Testnet4,
    Signet,
    Regtest,
}
impl Network {
    pub fn native(self) -> ::bitcoin::Network {
        match self {
            Self::Bitcoin => ::bitcoin::Network::Bitcoin,
            Self::Testnet => ::bitcoin::Network::Testnet,
            Self::Testnet4 => ::bitcoin::Network::Testnet4,
            Self::Signet => ::bitcoin::Network::Signet,
            Self::Regtest => ::bitcoin::Network::Regtest,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Root {
    pub id: Uuid,
    pub label: String,
    pub description: String,
    pub created_at: u64,
    pub kind: String,
    pub custom_password: bool,
    pub reauthenticate: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Identity {
    pub id: Uuid,
    pub root_id: Uuid,
    pub label: String,
    pub passphrase_record: Option<Uuid>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Account {
    pub id: Uuid,
    pub identity_id: Option<Uuid>,
    pub label: String,
    pub network: Network,
    pub path: String,
    pub address: String,
    pub public_key: String,
    pub fingerprint: String,
    pub watch_only: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct History {
    pub id: Uuid,
    pub request_id: Uuid,
    pub action: String,
    pub at: u64,
    pub digest: String,
    pub txid: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Settings {
    pub mode: Mode,
    pub timeout_seconds: u64,
    pub lock_on_blur: bool,
    pub rpc_url: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Offline,
            timeout_seconds: 300,
            lock_on_blur: false,
            rpc_url: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VaultData {
    pub settings: Settings,
    pub roots: Vec<Root>,
    pub identities: Vec<Identity>,
    pub accounts: Vec<Account>,
    pub records: Vec<SecretRecord>,
    pub history: Vec<History>,
    pub pending: Vec<crate::protocol::Request>,
    pub responses: Vec<crate::protocol::Response>,
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn label(value: &str) -> crate::Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 120 || value.chars().any(char::is_control) {
        return Err(crate::Error::Invalid(
            "label must contain 1–120 printable characters",
        ));
    }
    Ok(value.to_owned())
}
