use std::fs;
use std::time::Duration;

use frp_manager_lib::error::AppError;
use frp_manager_lib::process_manager::{ProcessRegistry, ProfileProcessState};
use tempfile::tempdir;
use tokio::time::sleep;

#[tokio::test]
async fn starts_and_stops_fake_frpc_process() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    fs::write(
        &fake,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 0.69.1; exit 0; fi\nsleep 30\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(&fake, permissions).unwrap();

    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();
    let snapshot = registry.refresh_snapshot("profile-a").unwrap();
    assert_eq!(snapshot.state, ProfileProcessState::Running);
    assert!(snapshot.pid.is_some());
    assert!(snapshot.started_at.is_some());
    registry.stop("profile-a").await.unwrap();
    assert_eq!(registry.state("profile-a"), ProfileProcessState::Stopped);
}

#[tokio::test]
async fn does_not_record_immediately_exited_process_as_running() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(&fake, "#!/bin/sh\nexit 7\n");
    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    let result = registry
        .start("profile-a", &fake, &config, dir.path())
        .await;

    assert!(matches!(result, Err(AppError::Runtime(_))));
    assert_eq!(registry.state("profile-a"), ProfileProcessState::Stopped);
    assert_eq!(registry.running_count(), 0);
}

#[tokio::test]
async fn drains_child_stdout_and_stderr() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        r#"#!/bin/sh
i=0
while [ "$i" -lt 20000 ]; do
  echo "stdout line $i xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  echo "stderr line $i xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" >&2
  i=$((i + 1))
done
touch noisy-ready
sleep 30
"#,
    );
    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();

    let marker = dir.path().join("noisy-ready");
    for _ in 0..40 {
        if marker.exists() {
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }

    assert!(marker.exists());
    registry.stop("profile-a").await.unwrap();
}

#[tokio::test]
async fn writes_child_stdout_and_stderr_to_current_log() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        r#"#!/bin/sh
echo "stdout connected"
echo "stderr connected" >&2
touch log-ready
sleep 30
"#,
    );
    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();

    let log_path = dir.path().join("logs/current.log");
    let mut contents = String::new();
    for _ in 0..40 {
        contents = fs::read_to_string(&log_path).unwrap_or_default();
        if contents.contains("stdout connected") && contents.contains("stderr connected") {
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }

    assert!(
        contents.contains("stdout connected"),
        "expected stdout in log, got {contents:?}"
    );
    assert!(
        contents.contains("stderr connected"),
        "expected stderr in log, got {contents:?}"
    );
    registry.stop("profile-a").await.unwrap();
}

#[tokio::test]
async fn restart_replaces_running_process() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        r#"#!/bin/sh
count=0
if [ -f starts ]; then
  count=$(cat starts)
fi
count=$((count + 1))
echo "$count" > starts
touch "started-$count"
sleep 30
"#,
    );
    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();
    wait_for_path(&dir.path().join("started-1")).await;

    registry
        .restart("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();
    wait_for_path(&dir.path().join("started-2")).await;

    let snapshot = registry.refresh_snapshot("profile-a").unwrap();
    assert_eq!(snapshot.state, ProfileProcessState::Running);
    assert_eq!(registry.running_count(), 1);
    registry.stop("profile-a").await.unwrap();
}

#[tokio::test]
async fn stop_all_stops_every_running_process() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        r#"#!/bin/sh
touch "$2.started"
sleep 30
"#,
    );
    let config_a = dir.path().join("profile-a.toml");
    let config_b = dir.path().join("profile-b.toml");
    fs::write(
        &config_a,
        "serverAddr = \"example.com\"\nserverPort = 7000\n",
    )
    .unwrap();
    fs::write(
        &config_b,
        "serverAddr = \"example.com\"\nserverPort = 7000\n",
    )
    .unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config_a, dir.path())
        .await
        .unwrap();
    registry
        .start("profile-b", &fake, &config_b, dir.path())
        .await
        .unwrap();

    assert_eq!(registry.running_count(), 2);
    registry.stop_all().await.unwrap();
    assert_eq!(registry.running_count(), 0);
    assert_eq!(registry.state("profile-a"), ProfileProcessState::Stopped);
    assert_eq!(registry.state("profile-b"), ProfileProcessState::Stopped);
}

#[tokio::test]
async fn refresh_state_marks_later_exited_process_as_failed() {
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        "#!/bin/sh\ntouch ready\nwhile [ ! -f exit-now ]; do sleep 0.05; done\nexit 9\n",
    );
    let config = dir.path().join("profile.toml");
    fs::write(&config, "serverAddr = \"example.com\"\nserverPort = 7000\n").unwrap();

    let mut registry = ProcessRegistry::default();
    registry
        .start("profile-a", &fake, &config, dir.path())
        .await
        .unwrap();
    assert_eq!(
        registry.refresh_state("profile-a").unwrap(),
        ProfileProcessState::Running
    );

    wait_for_path(&dir.path().join("ready")).await;
    fs::write(dir.path().join("exit-now"), "").unwrap();
    sleep(Duration::from_millis(150)).await;

    assert_eq!(
        registry.refresh_state("profile-a").unwrap(),
        ProfileProcessState::Failed
    );
    assert_eq!(registry.running_count(), 0);
}

async fn wait_for_path(path: &std::path::Path) {
    for _ in 0..40 {
        if path.exists() {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {}", path.display());
}

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(path, permissions).unwrap();
}
