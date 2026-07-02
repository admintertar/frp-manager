use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::error::AppResult;

pub fn app_log_path(data_dir: &Path) -> PathBuf {
    data_dir.join("logs").join("app.log")
}

pub fn append_app_log(data_dir: &Path, scope: &str, message: &str) -> AppResult<()> {
    let path = app_log_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let message = single_line(message);
    let line = format!(
        "{} [{}] {}\n",
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        single_line(scope),
        message
    );
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(line.as_bytes())?;
    Ok(())
}

pub fn read_app_log(data_dir: &Path) -> AppResult<String> {
    let path = app_log_path(data_dir);
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(path)?)
}

fn single_line(input: &str) -> String {
    input.replace(['\r', '\n'], " ")
}
