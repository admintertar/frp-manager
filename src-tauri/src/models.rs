use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    pub id: String,
    pub display_name: String,
    pub server_addr: String,
    pub server_port: u16,
    pub proxy_count: usize,
    pub runtime_state: RuntimeState,
    pub runtime_pid: Option<u32>,
    pub runtime_started_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub display_name: String,
    pub server_addr: String,
    pub server_port: u16,
    pub auth_method: Option<String>,
    pub auth_token: Option<String>,
    pub admin_port: Option<u16>,
    pub proxies: Vec<ProxyConfig>,
    pub raw_toml: String,
    pub meta: ProfileMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub name: String,
    pub proxy_type: ProxyType,
    pub enabled: bool,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub subdomain: Option<String>,
    pub custom_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMeta {
    pub id: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub auto_start: bool,
    pub last_runtime_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeState {
    Stopped,
    Starting,
    Running,
    Reloading,
    Degraded,
    Failed,
}
