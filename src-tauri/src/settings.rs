//! App-wide preferences.
//!
//! These live in their own file rather than the frontend's storage because the
//! tray menu is built before the webview loads and still needs the language.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::profile_store::atomic_write;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Locale {
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-CN")]
    ZhCn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub locale: Locale,
}

pub fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SETTINGS_FILE)
}

/// Read stored settings, falling back to defaults when the file is missing or
/// unreadable. A broken settings file must never stop the app from starting.
pub fn load(data_dir: &Path) -> Settings {
    fs::read_to_string(settings_path(data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save(data_dir: &Path, settings: &Settings) -> AppResult<()> {
    let json = serde_json::to_string_pretty(settings)
        .map_err(|err| AppError::Validation(err.to_string()))?;
    atomic_write(&settings_path(data_dir), &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), Settings::default());
        assert_eq!(load(dir.path()).locale, Locale::En);
    }

    #[test]
    fn saved_locale_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        save(
            dir.path(),
            &Settings {
                locale: Locale::ZhCn,
            },
        )
        .unwrap();

        assert_eq!(load(dir.path()).locale, Locale::ZhCn);
    }

    #[test]
    fn the_locale_serializes_as_bcp47() {
        let dir = tempfile::tempdir().unwrap();
        save(
            dir.path(),
            &Settings {
                locale: Locale::ZhCn,
            },
        )
        .unwrap();

        let raw = fs::read_to_string(settings_path(dir.path())).unwrap();
        assert!(raw.contains("\"zh-CN\""), "unexpected settings file: {raw}");
    }

    #[test]
    fn a_corrupt_file_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(settings_path(dir.path()), "{ not json").unwrap();

        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn an_unknown_locale_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(settings_path(dir.path()), r#"{"locale":"fr"}"#).unwrap();

        assert_eq!(load(dir.path()), Settings::default());
    }
}
