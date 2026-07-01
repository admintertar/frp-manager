use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::process_manager::ProcessRegistry;
use crate::profile_store::ProfileStore;
use crate::runtime_manager::RuntimeManager;

#[derive(Clone)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub registry: Arc<RwLock<ProcessRegistry>>,
    exit_requested: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            registry: Arc::new(RwLock::new(ProcessRegistry::default())),
            exit_requested: Arc::new(AtomicBool::new(false)),
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
