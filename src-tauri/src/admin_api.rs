//! Client for the frpc local admin API (`webServer` in a profile's TOML).
//!
//! Endpoints used here were verified against the frpc 0.69.1 binary and a live
//! client process:
//!
//! * `GET /api/status` returns `{"<proxy type>": [ { name, status, err, ... } ]}`
//! * `GET /api/config` returns the running configuration as TOML text
//! * `GET /api/reload` re-reads the configuration file frpc was started with
//!
//! Authentication is HTTP Basic and every response is `401` without it.

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use crate::error::{AppError, AppResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Proxy status values reported by frpc that mean the tunnel is up.
const ONLINE_STATUS: &str = "running";

/// Where a profile's frpc admin server is listening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminEndpoint {
    pub addr: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl AdminEndpoint {
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", host_for_url(&self.addr), self.port)
    }

    fn has_credentials(&self) -> bool {
        self.user
            .as_deref()
            .is_some_and(|user| !user.trim().is_empty())
    }
}

/// frpc reports the admin address as `0.0.0.0` when it binds all interfaces,
/// but that is not a connectable destination.
fn host_for_url(addr: &str) -> &str {
    match addr.trim() {
        "" | "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        other => other,
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProxyRuntimeStatus {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub err: String,
}

/// `GET /api/status` groups proxy entries by proxy type.
pub type StatusSnapshot = HashMap<String, Vec<ProxyRuntimeStatus>>;

/// Whether `name` is currently reported as a running proxy.
pub fn proxy_is_online(snapshot: &StatusSnapshot, name: &str) -> bool {
    snapshot
        .values()
        .flatten()
        .any(|entry| entry.name == name && entry.status == ONLINE_STATUS)
}

/// First error message reported for `name`, if any.
pub fn proxy_error<'a>(snapshot: &'a StatusSnapshot, name: &str) -> Option<&'a str> {
    snapshot
        .values()
        .flatten()
        .find(|entry| entry.name == name && !entry.err.is_empty())
        .map(|entry| entry.err.as_str())
}

fn client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|err| AppError::Runtime(format!("build admin client failed: {err}")))
}

fn request(
    method: reqwest::Method,
    endpoint: &AdminEndpoint,
    path: &str,
) -> AppResult<reqwest::RequestBuilder> {
    let request = client()?.request(method, format!("{}{path}", endpoint.base_url()));
    Ok(if endpoint.has_credentials() {
        request.basic_auth(
            endpoint.user.clone().unwrap_or_default(),
            endpoint.password.clone(),
        )
    } else {
        request
    })
}

async fn send(request: reqwest::RequestBuilder, what: &str) -> AppResult<reqwest::Response> {
    let response = request
        .send()
        .await
        .map_err(|err| AppError::Runtime(format!("frpc admin {what} failed: {err}")))?;

    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Runtime(
            "frpc admin rejected the stored credentials".into(),
        ));
    }
    Err(AppError::Runtime(format!(
        "frpc admin {what} returned HTTP {status}"
    )))
}

/// `GET /api/status`
pub async fn fetch_status(endpoint: &AdminEndpoint) -> AppResult<StatusSnapshot> {
    let response = send(request(reqwest::Method::GET, endpoint, "/api/status")?, "status").await?;
    response
        .json::<StatusSnapshot>()
        .await
        .map_err(|err| AppError::Runtime(format!("frpc admin status was not valid JSON: {err}")))
}

/// `GET /api/config`
pub async fn fetch_config(endpoint: &AdminEndpoint) -> AppResult<String> {
    let response = send(request(reqwest::Method::GET, endpoint, "/api/config")?, "config").await?;
    response
        .text()
        .await
        .map_err(|err| AppError::Runtime(format!("frpc admin config could not be read: {err}")))
}

/// `GET /api/reload` — makes frpc re-read the config file it was started with.
pub async fn reload(endpoint: &AdminEndpoint) -> AppResult<()> {
    let response = send(request(reqwest::Method::GET, endpoint, "/api/reload")?, "reload").await?;
    let body = response.text().await.unwrap_or_default();
    if body.to_ascii_lowercase().contains("fail") || body.to_ascii_lowercase().contains("error") {
        return Err(AppError::Runtime(format!(
            "frpc admin reload reported: {}",
            body.trim()
        )));
    }
    Ok(())
}

/// Poll `/api/status` until `name` reaches the expected online state.
pub async fn wait_for_proxy_state(
    endpoint: &AdminEndpoint,
    name: &str,
    expect_online: bool,
    attempts: u32,
    delay: Duration,
) -> AppResult<bool> {
    for attempt in 0..attempts {
        let snapshot = fetch_status(endpoint).await?;
        if proxy_is_online(&snapshot, name) == expect_online {
            return Ok(true);
        }
        if attempt + 1 < attempts {
            tokio::time::sleep(delay).await;
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(addr: &str) -> AdminEndpoint {
        AdminEndpoint {
            addr: addr.to_string(),
            port: 17401,
            user: Some("local-admin".into()),
            password: Some("secret".into()),
        }
    }

    fn snapshot(entries: &[(&str, &str, &str)]) -> StatusSnapshot {
        let mut map: StatusSnapshot = HashMap::new();
        for (kind, name, status) in entries {
            map.entry((*kind).to_string())
                .or_default()
                .push(ProxyRuntimeStatus {
                    name: (*name).to_string(),
                    status: (*status).to_string(),
                    err: String::new(),
                });
        }
        map
    }

    #[test]
    fn base_url_targets_loopback_for_wildcard_binds() {
        assert_eq!(endpoint("0.0.0.0").base_url(), "http://127.0.0.1:17401");
        assert_eq!(endpoint("").base_url(), "http://127.0.0.1:17401");
        assert_eq!(endpoint("::").base_url(), "http://127.0.0.1:17401");
        assert_eq!(endpoint("127.0.0.1").base_url(), "http://127.0.0.1:17401");
    }

    #[test]
    fn base_url_keeps_an_explicit_host() {
        assert_eq!(
            endpoint("192.168.1.10").base_url(),
            "http://192.168.1.10:17401"
        );
    }

    #[test]
    fn credentials_presence_follows_the_user_field() {
        assert!(endpoint("127.0.0.1").has_credentials());

        let mut anonymous = endpoint("127.0.0.1");
        anonymous.user = None;
        assert!(!anonymous.has_credentials());

        anonymous.user = Some("   ".into());
        assert!(!anonymous.has_credentials());
    }

    #[test]
    fn parses_the_status_shape_reported_by_frpc() {
        let raw = r#"{"tcp":[{"name":"probe-echo","type":"tcp","status":"running","err":"","local_addr":"127.0.0.1:19001","plugin":"","remote_addr":"127.0.0.1:19002"}]}"#;
        let parsed: StatusSnapshot = serde_json::from_str(raw).unwrap();

        assert!(proxy_is_online(&parsed, "probe-echo"));
        assert!(!proxy_is_online(&parsed, "missing"));
    }

    #[test]
    fn tolerates_status_entries_without_optional_fields() {
        let parsed: StatusSnapshot = serde_json::from_str(r#"{"http":[{"name":"zwd"}]}"#).unwrap();
        assert!(!proxy_is_online(&parsed, "zwd"));
    }

    #[test]
    fn online_check_spans_all_proxy_types() {
        let parsed = snapshot(&[
            ("http", "web", "running"),
            ("tcp", "ssh", "waiting"),
            ("udp", "dns", "start error"),
        ]);

        assert!(proxy_is_online(&parsed, "web"));
        assert!(!proxy_is_online(&parsed, "ssh"));
        assert!(!proxy_is_online(&parsed, "dns"));
    }

    #[test]
    fn proxy_error_is_reported_when_present() {
        let mut parsed = snapshot(&[("tcp", "ssh", "start error")]);
        parsed.get_mut("tcp").unwrap()[0].err = "port already used".into();

        assert_eq!(proxy_error(&parsed, "ssh"), Some("port already used"));
        assert_eq!(proxy_error(&parsed, "absent"), None);
    }
}
