use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::process_manager::ProcessRegistry;
use crate::profile_store::ProfileStore;
use crate::runtime_manager::RuntimeManager;

#[derive(Clone)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub registry: Arc<RwLock<ProcessRegistry>>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            registry: Arc::new(RwLock::new(ProcessRegistry::default())),
        }
    }

    pub fn profile_store(&self) -> ProfileStore {
        ProfileStore::new(self.data_dir.clone())
    }

    pub fn runtime_manager(&self) -> RuntimeManager {
        RuntimeManager::new(self.data_dir.clone())
    }
}
