use toml_edit::{value, DocumentMut, Item, Table, TableLike};

use crate::admin_api::AdminEndpoint;
use crate::error::{AppError, AppResult};
use crate::models::{Profile, ProfileMeta, ProxyConfig, ProxyType};

pub fn normalize_profile_toml(input: &str) -> AppResult<String> {
    let doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    Ok(doc.to_string())
}

pub fn parse_profile_toml(id: &str, display_name: &str, input: &str) -> AppResult<Profile> {
    let doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;

    let server_addr = doc
        .get("serverAddr")
        .and_then(|item| item.as_str())
        .ok_or_else(|| AppError::Validation("serverAddr is required".into()))?
        .to_string();
    let server_port = doc
        .get("serverPort")
        .and_then(|item| item.as_integer())
        .ok_or_else(|| AppError::Validation("serverPort is required".into()))?;
    let server_port = u16::try_from(server_port)
        .map_err(|_| AppError::Validation("serverPort must be between 0 and 65535".into()))?;

    let auth_table = optional_table(doc.get("auth"), "auth")?;
    let auth_method = optional_string(
        auth_table.and_then(|table| table.get("method")),
        "auth.method",
    )?;
    let auth_token = optional_string(
        auth_table.and_then(|table| table.get("token")),
        "auth.token",
    )?;
    let admin_port = optional_port(&doc, &["webServer", "port"], "webServer.port")?;

    let mut proxies = Vec::new();
    let proxy_array = match doc.get("proxies") {
        Some(item) => Some(
            item.as_array_of_tables()
                .ok_or_else(|| AppError::Validation("proxies must be an array of tables".into()))?,
        ),
        None => None,
    };

    if let Some(proxy_array) = proxy_array {
        for table in proxy_array {
            let name = table
                .get("name")
                .and_then(|item| item.as_str())
                .ok_or_else(|| AppError::Validation("proxy name is required".into()))?
                .to_string();
            let proxy_type = match table
                .get("type")
                .and_then(|item| item.as_str())
                .ok_or_else(|| AppError::Validation(format!("proxy {name} type is required")))?
            {
                "http" => ProxyType::Http,
                "https" => ProxyType::Https,
                "tcp" => ProxyType::Tcp,
                "udp" => ProxyType::Udp,
                other => {
                    return Err(AppError::Validation(format!(
                        "proxy {name} type {other} is not supported in v1"
                    )));
                }
            };
            let enabled = match table.get("enabled") {
                Some(item) => item.as_bool().ok_or_else(|| {
                    AppError::Validation(format!("proxy {name} enabled must be a boolean"))
                })?,
                None => true,
            };
            let local_ip = optional_string(table.get("localIP"), &format!("proxy {name} localIP"))?;
            let local_port =
                optional_table_port(table, "localPort", &format!("proxy {name} localPort"))?;
            let remote_port =
                optional_table_port(table, "remotePort", &format!("proxy {name} remotePort"))?;
            let subdomain =
                optional_string(table.get("subdomain"), &format!("proxy {name} subdomain"))?;
            let custom_domains = match table.get("customDomains") {
                Some(item) => {
                    let array = item.as_array().ok_or_else(|| {
                        AppError::Validation(format!("proxy {name} customDomains must be an array"))
                    })?;
                    let mut domains = Vec::new();
                    for domain in array {
                        domains.push(
                            domain
                                .as_str()
                                .ok_or_else(|| {
                                    AppError::Validation(format!(
                                        "proxy {name} customDomains must contain only strings"
                                    ))
                                })?
                                .to_string(),
                        );
                    }
                    domains
                }
                None => Vec::new(),
            };

            proxies.push(ProxyConfig {
                name,
                proxy_type,
                enabled,
                local_ip,
                local_port,
                remote_port,
                subdomain,
                custom_domains,
            });
        }
    }

    validate_unique_proxy_names(&proxies)?;

    let now = chrono::Utc::now();
    Ok(Profile {
        id: id.to_string(),
        display_name: display_name.to_string(),
        server_addr,
        server_port,
        auth_method,
        auth_token,
        admin_port,
        proxies,
        raw_toml: input.to_string(),
        meta: ProfileMeta {
            id: id.to_string(),
            display_name: display_name.to_string(),
            created_at: now,
            updated_at: now,
            auto_start: false,
            last_runtime_version: None,
        },
    })
}

pub fn set_proxy_enabled(input: &str, proxy_name: &str, enabled: bool) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    let proxy_item = doc
        .get_mut("proxies")
        .ok_or_else(|| AppError::Validation("profile has no proxies".into()))?;
    let proxy_array = proxy_item
        .as_array_of_tables_mut()
        .ok_or_else(|| AppError::Validation("proxies must be an array of tables".into()))?;

    let mut found = false;
    for table in proxy_array.iter_mut() {
        if table.get("name").and_then(|item| item.as_str()) == Some(proxy_name) {
            table["enabled"] = value(enabled);
            found = true;
            break;
        }
    }

    if found {
        Ok(doc.to_string())
    } else {
        Err(AppError::Validation(format!(
            "proxy {proxy_name} not found"
        )))
    }
}

/// Write the loopback admin settings into a profile TOML.
///
/// The block is written as a real `[webServer]` table: a TOML header resets the
/// current table context, so it stays correct no matter where in the file it
/// lands. Dotted keys here would be swallowed by a preceding `[[proxies]]`
/// section instead.
pub fn ensure_admin_web_server(
    input: &str,
    port: u16,
    user: &str,
    password: &str,
) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;

    if !doc.contains_key("webServer") {
        doc.insert("webServer", Item::Table(Table::new()));
    }
    let table = doc
        .get_mut("webServer")
        .and_then(|item| item.as_table_mut())
        .ok_or_else(|| AppError::Validation("webServer must be a table".into()))?;

    table.insert("addr", value("127.0.0.1"));
    table.insert("port", value(i64::from(port)));
    table.insert("user", value(user));
    table.insert("password", value(password));

    Ok(doc.to_string())
}

/// Port of an existing `webServer` block, if the profile has one.
pub fn admin_port_from_toml(input: &str) -> AppResult<Option<u16>> {
    let doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    let port = optional_port(&doc, &["webServer", "port"], "webServer.port")?;
    Ok(port.filter(|port| *port > 0))
}

/// Connection details for a profile's frpc admin server, if it is configured.
pub fn admin_endpoint_from_toml(input: &str) -> AppResult<Option<AdminEndpoint>> {
    let Some(port) = admin_port_from_toml(input)? else {
        return Ok(None);
    };
    let doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    let table = optional_table(doc.get("webServer"), "webServer")?;

    Ok(Some(AdminEndpoint {
        addr: optional_string(table.and_then(|table| table.get("addr")), "webServer.addr")?
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        port,
        user: optional_string(table.and_then(|table| table.get("user")), "webServer.user")?,
        password: optional_string(
            table.and_then(|table| table.get("password")),
            "webServer.password",
        )?,
    }))
}

#[cfg(test)]
mod admin_web_server_tests {
    use super::*;

    const WITH_ADMIN: &str = r#"
serverAddr = "frp.example.com"
serverPort = 7000

webServer.addr = "127.0.0.1"
webServer.port = 17401
webServer.user = "local-admin"
webServer.password = "s3cret"
"#;

    #[test]
    fn reads_the_admin_endpoint_written_by_the_app() {
        let endpoint = admin_endpoint_from_toml(WITH_ADMIN).unwrap().unwrap();

        assert_eq!(endpoint.addr, "127.0.0.1");
        assert_eq!(endpoint.port, 17401);
        assert_eq!(endpoint.user.as_deref(), Some("local-admin"));
        assert_eq!(endpoint.password.as_deref(), Some("s3cret"));
    }

    #[test]
    fn a_profile_without_a_web_server_block_has_no_endpoint() {
        let raw = "serverAddr = \"frp.example.com\"\nserverPort = 7000\n";
        assert!(admin_endpoint_from_toml(raw).unwrap().is_none());
        assert!(admin_port_from_toml(raw).unwrap().is_none());
    }

    #[test]
    fn a_zero_port_counts_as_not_configured() {
        let raw = "serverAddr = \"a\"\nserverPort = 1\nwebServer.port = 0\n";
        assert!(admin_endpoint_from_toml(raw).unwrap().is_none());
    }

    #[test]
    fn missing_credentials_are_allowed() {
        let raw = "serverAddr = \"a\"\nserverPort = 1\nwebServer.port = 17401\n";
        let endpoint = admin_endpoint_from_toml(raw).unwrap().unwrap();
        assert_eq!(endpoint.user, None);
        assert_eq!(endpoint.password, None);
    }

    #[test]
    fn ensure_admin_web_server_is_idempotent_and_preserves_other_keys() {
        let raw = "serverAddr = \"frp.example.com\"\nserverPort = 7000\n";
        let once = ensure_admin_web_server(raw, 17402, "u", "p").unwrap();
        let twice = ensure_admin_web_server(&once, 17402, "u", "p").unwrap();

        assert_eq!(once, twice);
        assert!(twice.contains("serverAddr = \"frp.example.com\""));

        let endpoint = admin_endpoint_from_toml(&twice).unwrap().unwrap();
        assert_eq!(endpoint.port, 17402);
        assert_eq!(endpoint.addr, "127.0.0.1");
    }

    #[test]
    fn the_admin_block_tolerates_the_inline_table_form() {
        let raw = "serverAddr = \"a\"\nserverPort = 1\nwebServer = { addr = \"127.0.0.1\", port = 17403, user = \"u\", password = \"p\" }\n";
        let endpoint = admin_endpoint_from_toml(raw).unwrap().unwrap();

        assert_eq!(endpoint.port, 17403);
        assert_eq!(endpoint.user.as_deref(), Some("u"));
    }

    #[test]
    fn the_admin_block_reads_back_what_the_app_writes() {
        let raw = "serverAddr = \"frp.example.com\"\nserverPort = 7000\n";
        let written = ensure_admin_web_server(raw, 17404, "local-admin", "s3cret").unwrap();

        // Round-tripping matters: the profile is re-parsed on every load.
        let parsed = parse_profile_toml("demo", "demo", &written).unwrap();
        assert_eq!(parsed.admin_port, Some(17404));

        let endpoint = admin_endpoint_from_toml(&written).unwrap().unwrap();
        assert_eq!(endpoint.port, 17404);
        assert_eq!(endpoint.password.as_deref(), Some("s3cret"));
        assert!(written.contains("[webServer]"), "expected a real table: {written}");
    }

    #[test]
    fn the_admin_block_does_not_get_absorbed_by_a_proxies_section() {
        let raw = "serverAddr = \"a\"\nserverPort = 1\n\n[[proxies]]\nname = \"web\"\ntype = \"http\"\nlocalPort = 80\n";
        let written = ensure_admin_web_server(raw, 17405, "u", "p").unwrap();

        let parsed = parse_profile_toml("demo", "demo", &written).unwrap();
        assert_eq!(parsed.admin_port, Some(17405));
        assert_eq!(parsed.proxies.len(), 1);
        assert_eq!(parsed.proxies[0].name, "web");
        // The proxy must not have gained a stray nested webServer table.
        assert_eq!(parsed.proxies[0].proxy_type, ProxyType::Http);
    }

    #[test]
    fn ensure_admin_web_server_overwrites_a_stale_port() {
        let updated = ensure_admin_web_server(WITH_ADMIN, 17499, "u", "p").unwrap();
        assert_eq!(admin_port_from_toml(&updated).unwrap(), Some(17499));
    }
}

fn validate_unique_proxy_names(proxies: &[ProxyConfig]) -> AppResult<()> {
    let mut names = std::collections::HashSet::new();
    for proxy in proxies {
        if !names.insert(proxy.name.clone()) {
            return Err(AppError::Validation(format!(
                "duplicate proxy name {}",
                proxy.name
            )));
        }
    }
    Ok(())
}

fn optional_port(doc: &DocumentMut, path: &[&str], label: &str) -> AppResult<Option<u16>> {
    if path.is_empty() {
        return Ok(None);
    }

    let mut current = match doc.get(path[0]) {
        Some(item) => item,
        None => return Ok(None),
    };

    for key in &path[1..] {
        current = match current.as_table_like() {
            Some(table) => match table.get(key) {
                Some(item) => item,
                None => return Ok(None),
            },
            None => {
                return Err(AppError::Validation(format!(
                    "{} must be a table",
                    path[..path.len() - 1].join(".")
                )));
            }
        };
    }

    current
        .as_integer()
        .ok_or_else(|| AppError::Validation(format!("{label} must be an integer")))?
        .try_into()
        .map(Some)
        .map_err(|_| AppError::Validation(format!("{label} must be between 0 and 65535")))
}

fn optional_table<'a>(item: Option<&'a Item>, label: &str) -> AppResult<Option<&'a dyn TableLike>> {
    match item {
        Some(item) => item
            .as_table_like()
            .map(Some)
            .ok_or_else(|| AppError::Validation(format!("{label} must be a table"))),
        None => Ok(None),
    }
}

fn optional_string(item: Option<&Item>, label: &str) -> AppResult<Option<String>> {
    match item {
        Some(item) => item
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| AppError::Validation(format!("{label} must be a string"))),
        None => Ok(None),
    }
}

fn optional_table_port(table: &toml_edit::Table, key: &str, label: &str) -> AppResult<Option<u16>> {
    let Some(item) = table.get(key) else {
        return Ok(None);
    };
    item.as_integer()
        .ok_or_else(|| AppError::Validation(format!("{label} must be an integer")))?
        .try_into()
        .map(Some)
        .map_err(|_| AppError::Validation(format!("{label} must be between 0 and 65535")))
}
