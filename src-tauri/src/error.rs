use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(String),
    #[error("TOML write error: {0}")]
    TomlWrite(String),
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Process already running: {0}")]
    ProcessAlreadyRunning(String),
    #[error("Process not running: {0}")]
    ProcessNotRunning(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Update error: {0}")]
    Update(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Serialize)]
pub struct AppErrorResponse {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io_error",
            AppError::TomlParse(_) => "toml_parse_error",
            AppError::TomlWrite(_) => "toml_write_error",
            AppError::ProfileNotFound(_) => "profile_not_found",
            AppError::ProcessAlreadyRunning(_) => "process_already_running",
            AppError::ProcessNotRunning(_) => "process_not_running",
            AppError::Runtime(_) => "runtime_error",
            AppError::Update(_) => "update_error",
            AppError::Validation(_) => "validation_error",
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        AppErrorResponse {
            code: self.code(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

pub type AppResult<T> = Result<T, AppError>;
