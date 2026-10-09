use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::TryStreamExt;
use serde::Serialize;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, mpsc};
use tokio_util::io::StreamReader;
use tracing::warn;
use ts_rs::TS;

use crate::{CoreError, Result};

const CHUNK_SIZE: usize = 64 * 1024;

pub const DEFAULT_CONCURRENCY: usize = 8;

pub const DEFAULT_RETRY: Retry = Retry {
    attempts: 3,
    initial_backoff: Duration::from_millis(200),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    pub attempts: u32,
    pub initial_backoff: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
    pub sha1: String,
    /// Size published by Mojang. The progress total is the sum of these sizes.
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub finished: u32,
    pub failed: u32,
    pub total: u32,
    // JSON numbers. A download size fits in a JavaScript number, unlike `bigint`.
    #[ts(type = "number")]
    pub bytes_done: u64,
    #[ts(type = "number")]
    pub bytes_total: u64,
}

pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    expected_sha1: &str,
    retry: Retry,
) -> Result<()> {
    if retry.attempts == 0 {
        return Err(CoreError::DownloadAttempts);
    }

    let part = part_path(destination)?;
    let mut backoff = retry.initial_backoff;
    let mut attempt = 1;
    loop {
        match write_verified_part(client, url, destination, &part, expected_sha1, None).await {
            Ok(()) => return Ok(()),
            Err(error) => {
                remove_part(&part).await?;
                if attempt == retry.attempts {
                    return Err(error);
                }
                warn!(attempt, url, "download attempt failed");
                tokio::time::sleep(backoff).await;
                backoff = backoff.saturating_mul(2);
                attempt += 1;
            }
        }
    }
}

pub async fn download_all(
    client: &reqwest::Client,
    tasks: &[DownloadTask],
    concurrency: usize,
    retry: Retry,
    progress: mpsc::Sender<DownloadProgress>,
) -> Result<()> {
    if tasks.is_empty() {
        return Ok(());
    }
    if concurrency == 0 {
        return Err(CoreError::DownloadConcurrency);
    }

    let total = u32::try_from(tasks.len())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "too many downloads"))?;
    let bytes = Arc::new(ByteProgress {
        done: AtomicU64::new(0),
        total: tasks
            .iter()
            .fold(0, |sum, task| sum.saturating_add(task.size)),
        files: total,
        sink: progress.clone(),
    });
    let _ = bytes.sink.try_send(bytes.snapshot(0, 0));
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let (results, mut incoming) = mpsc::channel(tasks.len());

    for task in tasks {
        let results = results.clone();
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let bytes = Arc::clone(&bytes);
        let url = task.url.clone();
        let destination = task.destination.clone();
        let sha1 = task.sha1.clone();
        tokio::spawn(async move {
            let result = async {
                let _permit = semaphore
                    .acquire_owned()
                    .await
                    .map_err(|_| std::io::Error::other("download queue closed"))?;
                let part = part_path(&destination)?;
                let mut backoff = retry.initial_backoff;
                let mut attempt = 1;
                loop {
                    match write_verified_part(
                        &client,
                        &url,
                        &destination,
                        &part,
                        &sha1,
                        Some(&bytes),
                    )
                    .await
                    {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            remove_part(&part).await?;
                            if attempt == retry.attempts {
                                return Err(error);
                            }
                            warn!(attempt, url, "download attempt failed");
                            tokio::time::sleep(backoff).await;
                            backoff = backoff.saturating_mul(2);
                            attempt += 1;
                        }
                    }
                }
            }
            .await;
            let _ = results.send(result).await;
        });
    }
    drop(results);

    let mut finished = 0;
    let mut failed = 0;
    let mut first_error = None;
    while let Some(result) = incoming.recv().await {
        match result {
            Ok(()) => finished += 1,
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
                failed += 1;
            }
        }
        let _ = progress.try_send(bytes.snapshot(finished, failed));
    }

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct ByteProgress {
    done: AtomicU64,
    total: u64,
    files: u32,
    sink: mpsc::Sender<DownloadProgress>,
}

impl ByteProgress {
    fn add(&self, bytes: u64) {
        let done = self.done.fetch_add(bytes, Ordering::Relaxed) + bytes;
        let _ = self.sink.try_send(DownloadProgress {
            finished: 0,
            failed: 0,
            total: self.files,
            bytes_done: done,
            bytes_total: self.total,
        });
    }

    fn snapshot(&self, finished: u32, failed: u32) -> DownloadProgress {
        DownloadProgress {
            finished,
            failed,
            total: self.files,
            bytes_done: self.done.load(Ordering::Relaxed),
            bytes_total: self.total,
        }
    }
}

/// Bytes of one attempt. A failed attempt is removed again so a retry does not count twice.
struct Attempt<'a> {
    progress: &'a ByteProgress,
    added: u64,
    committed: bool,
}

impl Attempt<'_> {
    fn add(&mut self, bytes: u64) {
        self.added = self.added.saturating_add(bytes);
        self.progress.add(bytes);
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        if self.committed || self.added == 0 {
            return;
        }
        let done = self.progress.done.fetch_sub(self.added, Ordering::Relaxed) - self.added;
        let _ = self.progress.sink.try_send(DownloadProgress {
            finished: 0,
            failed: 0,
            total: self.progress.files,
            bytes_done: done,
            bytes_total: self.progress.total,
        });
    }
}

async fn write_verified_part(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    part: &Path,
    expected_sha1: &str,
    progress: Option<&ByteProgress>,
) -> Result<()> {
    let mut attempt = progress.map(|progress| Attempt {
        progress,
        added: 0,
        committed: false,
    });
    let response = client.get(url).send().await?.error_for_status()?;
    let stream = response.bytes_stream().map_err(std::io::Error::other);
    let mut reader = StreamReader::new(stream);
    let mut file = tokio::fs::File::create(part).await?;
    let mut hasher = Sha1::new();
    let mut buffer = vec![0_u8; CHUNK_SIZE];

    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read]).await?;
        if let Some(attempt) = attempt.as_mut() {
            attempt.add(read as u64);
        }
    }
    file.flush().await?;
    drop(file);

    let actual = hex_encode(hasher.finalize().as_slice());
    if !actual.eq_ignore_ascii_case(expected_sha1) {
        return Err(CoreError::Sha1Mismatch {
            expected: expected_sha1.to_string(),
            actual,
        });
    }

    if tokio::fs::try_exists(destination).await? {
        tokio::fs::remove_file(destination).await?;
    }
    tokio::fs::rename(part, destination).await?;
    if let Some(attempt) = attempt.as_mut() {
        attempt.commit();
    }
    Ok(())
}

fn part_path(destination: &Path) -> Result<PathBuf> {
    let name = destination.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "download destination has no file name",
        )
    })?;
    let mut part_name = name.to_os_string();
    part_name.push(".part");
    Ok(destination.with_file_name(part_name))
}

async fn remove_part(part: &Path) -> Result<()> {
    match tokio::fs::remove_file(part).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
