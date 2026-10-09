use std::path::{Path, PathBuf};

use licht_core::{CoreError, GameLine, OutputStream, run_game};
use tokio::sync::mpsc;

#[tokio::test]
async fn a_finished_process_reports_both_streams_and_success() {
    let directory = temp_dir("streams");
    let program = write_stub(&directory, "streams", Stub::Streams { code: 0 });
    let (output, mut incoming) = mpsc::channel(8);

    let exit = run_game(&[program.display().to_string()], None, output)
        .await
        .expect("the stub should start");

    assert!(exit.succeeded());
    let lines = collect(&mut incoming);
    assert!(lines.contains(&GameLine {
        stream: OutputStream::Stdout,
        line: "hello-out".to_string(),
    }));
    assert!(lines.contains(&GameLine {
        stream: OutputStream::Stderr,
        line: "hello-err".to_string(),
    }));
}

#[tokio::test]
async fn a_failing_process_still_delivers_its_output() {
    let directory = temp_dir("fail");
    let program = write_stub(&directory, "fail", Stub::Streams { code: 1 });
    let (output, mut incoming) = mpsc::channel(8);

    let exit = run_game(&[program.display().to_string()], None, output)
        .await
        .expect("the stub should start");

    assert!(!exit.succeeded());
    assert_eq!(exit.code, Some(1));
    let lines = collect(&mut incoming);
    assert!(lines.iter().any(|line| line.line == "hello-out"));
    assert!(lines.iter().any(|line| line.line == "hello-err"));
}

#[tokio::test]
async fn an_empty_command_is_rejected() {
    let (output, _incoming) = mpsc::channel(1);
    let missing = run_game(&[], None, output).await.expect_err("empty");
    assert!(matches!(missing, CoreError::GameCommand));

    let (output, _incoming) = mpsc::channel(1);
    let blank = run_game(&[String::new()], None, output)
        .await
        .expect_err("blank");
    assert!(matches!(blank, CoreError::GameCommand));
}

#[tokio::test]
async fn the_working_directory_is_the_one_the_caller_passes() {
    let directory = temp_dir("workdir");
    let program = write_stub(&directory, "workdir", Stub::Marker);
    let work = directory.join("work");
    std::fs::create_dir_all(&work).expect("work directory");
    let (output, _incoming) = mpsc::channel(1);

    let exit = run_game(&[program.display().to_string()], Some(&work), output)
        .await
        .expect("the stub should start");

    assert!(exit.succeeded());
    assert!(work.join("marker.txt").is_file());
    assert!(!directory.join("marker.txt").is_file());
}

fn temp_dir(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("licht-run-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("temp directory");
    directory
}

fn collect(incoming: &mut mpsc::Receiver<GameLine>) -> Vec<GameLine> {
    let mut lines = Vec::new();
    while let Ok(line) = incoming.try_recv() {
        lines.push(line);
    }
    lines
}

enum Stub {
    Streams { code: i32 },
    Marker,
}

fn write_stub(directory: &Path, name: &str, stub: Stub) -> PathBuf {
    #[cfg(windows)]
    {
        let path = directory.join(format!("{name}.cmd"));
        let body = match stub {
            Stub::Streams { code: 0 } => {
                "@echo off\r\necho hello-out\r\n>&2 echo hello-err\r\n".to_string()
            }
            Stub::Streams { code } => {
                format!("@echo off\r\necho hello-out\r\n>&2 echo hello-err\r\nexit /b {code}\r\n")
            }
            Stub::Marker => "@echo off\r\necho seen> marker.txt\r\n".to_string(),
        };
        std::fs::write(&path, body).expect("stub");
        path
    }
    #[cfg(unix)]
    {
        let path = directory.join(name);
        let body = match stub {
            Stub::Streams { code: 0 } => {
                "#!/bin/sh\necho hello-out\necho hello-err >&2\n".to_string()
            }
            Stub::Streams { code } => {
                format!("#!/bin/sh\necho hello-out\necho hello-err >&2\nexit {code}\n")
            }
            Stub::Marker => "#!/bin/sh\necho seen > marker.txt\n".to_string(),
        };
        std::fs::write(&path, body).expect("stub");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        std::fs::set_permissions(&path, permissions).expect("executable");
        path
    }
}
