use std::fs;
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::config_toml::{normalize_profile_toml, parse_profile_toml};
use crate::error::{AppError, AppResult};
use crate::models::{Profile, ProfileMeta, ProfileSummary, RuntimeState};

#[derive(Debug, Clone)]
pub struct ProfileStore {
    root: PathBuf,
}

impl ProfileStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }

    pub fn list(&self) -> AppResult<Vec<ProfileSummary>> {
        let dir = self.profiles_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut summaries = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().to_string();
            let profile = self.load(&id)?;
            summaries.push(ProfileSummary {
                id: profile.id,
                display_name: profile.display_name,
                server_addr: profile.server_addr,
                server_port: profile.server_port,
                proxy_count: profile.proxies.len(),
                runtime_state: RuntimeState::Stopped,
                runtime_pid: None,
                runtime_started_at: None,
            });
        }
        summaries.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(summaries)
    }

    pub fn load(&self, id: &str) -> AppResult<Profile> {
        validate_profile_id(id)?;
        let profile_dir = self.profile_dir(id);
        let meta_path = profile_dir.join("meta.json");
        let toml_path = profile_dir.join("profile.toml");
        if !toml_path.exists() {
            return Err(AppError::ProfileNotFound(id.to_string()));
        }

        let raw = fs::read_to_string(toml_path)?;
        let meta: ProfileMeta = serde_json::from_str(&fs::read_to_string(meta_path)?)
            .map_err(|err| AppError::Validation(err.to_string()))?;
        if meta.id != id {
            return Err(AppError::Validation(format!(
                "profile metadata id {} does not match directory id {id}",
                meta.id
            )));
        }
        validate_profile_id(&meta.id)?;
        let mut profile = parse_profile_toml(&meta.id, &meta.display_name, &raw)?;
        profile.meta = meta;
        Ok(profile)
    }

    pub fn import_from_text(&self, display_name: &str, input: &str) -> AppResult<ProfileSummary> {
        let normalized = normalize_profile_toml(input)?;
        let id = slugify(display_name);
        validate_profile_id(&id)?;
        let profile_dir = self.profile_dir(&id);
        let profile = parse_profile_toml(&id, display_name, &normalized)?;

        fs::create_dir_all(self.profiles_dir())?;
        let reservation = ProfileDirReservation::create(profile_dir)?;
        let profile_dir = reservation.path();

        fs::create_dir_all(profile_dir.join("logs"))?;

        let now = Utc::now();
        let meta = ProfileMeta {
            id: id.clone(),
            display_name: display_name.to_string(),
            created_at: now,
            updated_at: now,
            auto_start: false,
            last_runtime_version: None,
        };

        atomic_write(&profile_dir.join("profile.toml"), &normalized)?;
        atomic_write(
            &profile_dir.join("meta.json"),
            &serde_json::to_string_pretty(&meta)
                .map_err(|err| AppError::Validation(err.to_string()))?,
        )?;
        reservation.commit();

        Ok(ProfileSummary {
            id,
            display_name: display_name.to_string(),
            server_addr: profile.server_addr,
            server_port: profile.server_port,
            proxy_count: profile.proxies.len(),
            runtime_state: RuntimeState::Stopped,
            runtime_pid: None,
            runtime_started_at: None,
        })
    }

    pub fn save_raw_toml(&self, id: &str, raw_toml: &str) -> AppResult<()> {
        validate_profile_id(id)?;
        parse_profile_toml(id, id, raw_toml)?;
        let path = self.profile_dir(id).join("profile.toml");
        if !path.exists() {
            return Err(AppError::ProfileNotFound(id.to_string()));
        }
        atomic_write(&path, raw_toml)
    }

    pub fn update_display_name(&self, id: &str, display_name: &str) -> AppResult<()> {
        validate_profile_id(id)?;
        let meta_path = self.profile_dir(id).join("meta.json");
        if !meta_path.exists() {
            return Err(AppError::ProfileNotFound(id.to_string()));
        }
        let mut meta: ProfileMeta = serde_json::from_str(&fs::read_to_string(&meta_path)?)
            .map_err(|err| AppError::Validation(err.to_string()))?;
        if meta.id != id {
            return Err(AppError::Validation(format!(
                "profile metadata id {} does not match directory id {id}",
                meta.id
            )));
        }
        meta.display_name = display_name.to_string();
        meta.updated_at = Utc::now();
        atomic_write(
            &meta_path,
            &serde_json::to_string_pretty(&meta)
                .map_err(|err| AppError::Validation(err.to_string()))?,
        )
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        validate_profile_id(id)?;
        let profile_dir = self.profile_dir(id);
        if !profile_dir.exists() {
            return Err(AppError::ProfileNotFound(id.to_string()));
        }
        fs::remove_dir_all(profile_dir)?;
        Ok(())
    }

    pub fn profile_toml_path(&self, id: &str) -> AppResult<PathBuf> {
        validate_profile_id(id)?;
        Ok(self.profile_dir(id).join("profile.toml"))
    }

    fn profile_dir(&self, id: &str) -> PathBuf {
        self.profiles_dir().join(id)
    }
}

fn atomic_write(path: &Path, contents: &str) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation(format!("path {} has no parent", path.display())))?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(contents.as_bytes())?;
    tmp.as_file_mut().sync_all()?;
    tmp.persist(path).map_err(|err| AppError::Io(err.error))?;
    Ok(())
}

struct ProfileDirReservation {
    path: PathBuf,
    committed: bool,
}

impl ProfileDirReservation {
    fn create(path: PathBuf) -> AppResult<Self> {
        match fs::create_dir(&path) {
            Ok(()) => Ok(Self {
                path,
                committed: false,
            }),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                let id = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("profile");
                Err(AppError::Validation(format!("profile {id} already exists")))
            }
            Err(err) => Err(AppError::Io(err)),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for ProfileDirReservation {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn slugify(input: &str) -> String {
    let mut slug = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "profile".to_string()
    } else {
        slug
    }
}

fn validate_profile_id(id: &str) -> AppResult<()> {
    if is_valid_profile_id(id) {
        Ok(())
    } else {
        Err(AppError::Validation(format!("invalid profile id {id}")))
    }
}

fn is_valid_profile_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_dir_reservation_removes_uncommitted_directory() {
        let dir = tempfile::tempdir().unwrap();
        let profile_dir = dir.path().join("profiles").join("demo");
        fs::create_dir_all(profile_dir.parent().unwrap()).unwrap();

        {
            let reservation = ProfileDirReservation::create(profile_dir.clone()).unwrap();
            fs::write(reservation.path().join("partial"), "partial").unwrap();
        }

        assert!(!profile_dir.exists());
    }
}
