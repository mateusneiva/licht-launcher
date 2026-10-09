use std::path::Path;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLine {
    pub stream: OutputStream,
    pub line: String,
}

/// `code` is missing when the process was killed by a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameExit {
    pub code: Option<i32>,
}

impl GameExit {
    pub fn succeeded(self) -> bool {
        self.code == Some(0)
    }
}

/// Runs `command[0]` with the remaining strings as arguments. No shell.
///
/// Stdin is closed. Stdout and stderr are read together so neither pipe can
/// fill up and stall the process. A dropped `output` receiver does not stop
/// the process. Cancelling this task kills the child.
pub async fn run_game(
    command: &[String],
    current_dir: Option<&Path>,
    output: mpsc::Sender<GameLine>,
) -> Result<GameExit> {
    let Some(program) = command.first() else {
        return Err(CoreError::GameCommand);
    };
    if program.is_empty() {
        return Err(CoreError::GameCommand);
    }

    let mut process = Command::new(program);
    process
        .args(&command[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    if let Some(directory) = current_dir {
        process.current_dir(directory);
    }

    let mut child = process.spawn()?;
    let stdout = child.stdout.take().ok_or_else(missing_pipe)?;
    let stderr = child.stderr.take().ok_or_else(missing_pipe)?;
    let (stdout_result, stderr_result) = futures_util::future::join(
        read_pipe(stdout, OutputStream::Stdout, output.clone()),
        read_pipe(stderr, OutputStream::Stderr, output),
    )
    .await;
    stdout_result?;
    stderr_result?;

    let status = child.wait().await?;
    Ok(GameExit {
        code: status.code(),
    })
}

async fn read_pipe(
    pipe: impl tokio::io::AsyncRead + Unpin,
    stream: OutputStream,
    output: mpsc::Sender<GameLine>,
) -> Result<()> {
    let mut lines = BufReader::new(pipe).lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        let _ = output.send(GameLine { stream, line }).await;
    }
}

fn missing_pipe() -> CoreError {
    CoreError::Io(std::io::Error::other("process pipe was not captured"))
}
