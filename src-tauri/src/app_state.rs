use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;

use crate::process_manager::ProcessRegistry;
use crate::profile_store::ProfileStore;
use crate::runtime_manager::RuntimeManager;
use crate::settings::{self, Locale, Settings};

#[derive(Clone)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub registry: Arc<RwLock<ProcessRegistry>>,
    locale: Arc<Mutex<Locale>>,
    exit_requested: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let locale = settings::load(&data_dir).locale;
        Self {
            data_dir,
            registry: Arc::new(RwLock::new(ProcessRegistry::default())),
            locale: Arc::new(Mutex::new(locale)),
            exit_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Active UI language. Read by the tray menu, which is built before the
    /// webview has a chance to report its own preference.
    pub fn locale(&self) -> Locale {
        self.locale
            .lock()
            .map(|locale| *locale)
            .unwrap_or_default()
    }

    pub fn set_locale(&self, next: Locale) {
        if let Ok(mut locale) = self.locale.lock() {
            *locale = next;
        }
    }

    pub fn settings(&self) -> Settings {
        Settings {
            locale: self.locale(),
        }
    }

    pub fn request_exit(&self) -> bool {
        !self.exit_requested.swap(true, Ordering::SeqCst)
    }

    pub fn is_exiting(&self) -> bool {
        self.exit_requested.load(Ordering::SeqCst)
    }

    pub fn profile_store(&self) -> ProfileStore {
        ProfileStore::new(self.data_dir.clone())
    }

    pub fn runtime_manager(&self) -> RuntimeManager {
        RuntimeManager::new(self.data_dir.clone())
    }
}
