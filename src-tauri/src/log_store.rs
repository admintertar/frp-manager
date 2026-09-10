//! Profile log storage.
//!
//! Each profile keeps one live log file (`current.log`). When it grows past a
//! size cap the file is renamed to a dated archive (`2026-07-01.log`, then
//! `2026-07-01.1.log`, ...) and a fresh live file is started. Only the newest
//! few archives are kept.
//!
//! Reads are tail-only: the log is polled from the UI every few seconds, so
//! returning the whole file would grow the IPC payload without bound.

use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Live log file name inside a profile's `logs/` directory.
pub const LOG_FILE_NAME: &str = "current.log";

/// Rotate once the live log reaches this size.
pub const DEFAULT_MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

/// How many dated archives to keep per profile.
pub const DEFAULT_KEEP_ARCHIVES: usize = 10;

/// Upper bound on how much of a log is returned to the UI.
pub const TAIL_READ_BYTES: u64 = 128 * 1024;

pub fn current_log_path(logs_dir: &Path) -> PathBuf {
    logs_dir.join(LOG_FILE_NAME)
}

fn is_log_file(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("log")
}

fn is_live_log(path: &Path) -> bool {
    path.file_name().and_then(|value| value.to_str()) == Some(LOG_FILE_NAME)
}

/// Dated archives in `logs_dir`, oldest first.
pub fn list_archives(logs_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(logs_dir) else {
        return Vec::new();
    };

    let mut archives = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_log_file(path) && !is_live_log(path))
        .collect::<Vec<_>>();
    archives.sort_by_key(|path| archive_sort_key(path));
    archives
}

/// Sort key that keeps `2026-07-01.log` before `2026-07-01.1.log` and both
/// before `2026-07-02.log`.
fn archive_sort_key(path: &Path) -> (String, u32) {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    match stem.split_once('.') {
        Some((date, suffix)) => (date.to_string(), suffix.parse().unwrap_or(0)),
        None => (stem, 0),
    }
}

fn next_archive_path(logs_dir: &Path, stamp: &str) -> PathBuf {
    let first = logs_dir.join(format!("{stamp}.log"));
    if !first.exists() {
        return first;
    }
    let mut suffix = 1u32;
    loop {
        let candidate = logs_dir.join(format!("{stamp}.{suffix}.log"));
        if !candidate.exists() {
            return candidate;
        }
        suffix += 1;
    }
}

/// Move the live log aside into a dated archive, regardless of size.
pub fn archive_current(log_path: &Path, keep: usize) -> io::Result<Option<PathBuf>> {
    if !log_path.exists() {
        return Ok(None);
    }
    let logs_dir = log_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "log path has no parent"))?;

    let stamp = chrono::Local::now().format("%Y-%m-%d").to_string();
    let target = next_archive_path(logs_dir, &stamp);
    fs::rename(log_path, &target)?;
    prune_archives(logs_dir, keep)?;
    Ok(Some(target))
}

/// Archive the live log only once it has reached `max_bytes`.
pub fn rotate_if_needed(
    log_path: &Path,
    max_bytes: u64,
    keep: usize,
) -> io::Result<Option<PathBuf>> {
    match fs::metadata(log_path) {
        Ok(metadata) if metadata.len() >= max_bytes => archive_current(log_path, keep),
        Ok(_) => Ok(None),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

/// Delete the oldest archives so at most `keep` remain.
pub fn prune_archives(logs_dir: &Path, keep: usize) -> io::Result<()> {
    let archives = list_archives(logs_dir);
    if archives.len() <= keep {
        return Ok(());
    }
    for path in &archives[..archives.len() - keep] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

/// Read at most `max_bytes` from the end of the log.
///
/// When the read starts mid-file the first (likely partial) line is dropped.
pub fn read_tail(log_path: &Path, max_bytes: u64) -> io::Result<String> {
    let mut file = fs::File::open(log_path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut text = String::from_utf8_lossy(&buffer).into_owned();
    if start > 0 {
        if let Some(index) = text.find('\n') {
            text.drain(..=index);
        }
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_log(dir: &Path, contents: &str) -> PathBuf {
        let path = current_log_path(dir);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn tail_read_returns_whole_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "one\ntwo\nthree\n");

        assert_eq!(read_tail(&path, 1024).unwrap(), "one\ntwo\nthree\n");
    }

    #[test]
    fn tail_read_drops_partial_first_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "aaaaaaaaaa\nbbbbbbbbbb\ncccccccccc\n");

        let tail = read_tail(&path, 14).unwrap();
        assert!(!tail.starts_with("aaaa"), "unexpected tail: {tail:?}");
        assert!(tail.ends_with("cccccccccc\n"), "unexpected tail: {tail:?}");
    }

    #[test]
    fn tail_read_of_missing_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_tail(&current_log_path(dir.path()), 1024).is_err());
    }

    #[test]
    fn rotation_is_skipped_below_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "small");

        let rotated = rotate_if_needed(&path, 1024, 3).unwrap();
        assert!(rotated.is_none());
        assert!(path.exists());
    }

    #[test]
    fn rotation_archives_the_live_log_and_frees_the_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "0123456789");

        let rotated = rotate_if_needed(&path, 4, 3).unwrap().unwrap();
        assert!(!path.exists(), "live log should have been moved aside");
        assert!(rotated.exists());
        assert_eq!(fs::read_to_string(&rotated).unwrap(), "0123456789");
        assert!(rotated
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".log"));
    }

    #[test]
    fn repeated_rotation_on_one_day_uses_suffixed_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "first");

        let first = rotate_if_needed(&path, 1, 5).unwrap().unwrap();
        fs::write(&path, "second").unwrap();
        let second = rotate_if_needed(&path, 1, 5).unwrap().unwrap();

        assert_ne!(first, second, "second rotation must not overwrite the first");
        assert_eq!(list_archives(dir.path()).len(), 2);
    }

    #[test]
    fn prune_keeps_only_the_newest_archives() {
        let dir = tempfile::tempdir().unwrap();
        for (name, body) in [
            ("2026-07-01.log", "a"),
            ("2026-07-02.log", "b"),
            ("2026-07-03.log", "c"),
        ] {
            fs::write(dir.path().join(name), body).unwrap();
        }

        prune_archives(dir.path(), 2).unwrap();

        let remaining = list_archives(dir.path())
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(remaining, vec!["2026-07-02.log", "2026-07-03.log"]);
    }

    #[test]
    fn list_archives_ignores_the_live_log_and_other_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(current_log_path(dir.path()), "live").unwrap();
        fs::write(dir.path().join("2026-07-01.log"), "archived").unwrap();
        fs::write(dir.path().join("notes.txt"), "not a log").unwrap();

        let archives = list_archives(dir.path());
        assert_eq!(archives.len(), 1);
        assert!(archives[0].ends_with("2026-07-01.log"));
    }

    #[test]
    fn archive_current_preserves_non_empty_logs() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_log(dir.path(), "previous run output\n");

        let archived = archive_current(&path, 3).unwrap().unwrap();
        assert!(!path.exists());
        assert_eq!(
            fs::read_to_string(&archived).unwrap(),
            "previous run output\n"
        );
    }
}
