use toml_edit::{value, DocumentMut, Item, Table};

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

pub fn ensure_admin_web_server(
    input: &str,
    port: u16,
    user: &str,
    password: &str,
) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    doc["webServer"]["addr"] = value("127.0.0.1");
    doc["webServer"]["port"] = value(i64::from(port));
    doc["webServer"]["user"] = value(user);
    doc["webServer"]["password"] = value(password);
    Ok(doc.to_string())
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
        current = match current.as_table() {
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

fn optional_table<'a>(item: Option<&'a Item>, label: &str) -> AppResult<Option<&'a Table>> {
    match item {
        Some(item) => item
            .as_table()
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
