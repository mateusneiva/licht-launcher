use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures_util::TryStreamExt;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Semaphore, mpsc};
use tokio_util::io::StreamReader;
use tracing::warn;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub finished: u32,
    pub failed: u32,
    pub total: u32,
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
        match write_verified_part(client, url, destination, &part, expected_sha1).await {
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
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let (results, mut incoming) = mpsc::channel(tasks.len());

    for task in tasks {
        let results = results.clone();
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let url = task.url.clone();
        let destination = task.destination.clone();
        let sha1 = task.sha1.clone();
        tokio::spawn(async move {
            let result = async {
                let _permit = semaphore
                    .acquire_owned()
                    .await
                    .map_err(|_| std::io::Error::other("download queue closed"))?;
                download_file(&client, &url, &destination, &sha1, retry).await
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
        let _ = progress
            .send(DownloadProgress {
                finished,
                failed,
                total,
            })
            .await;
    }

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

async fn write_verified_part(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    part: &Path,
    expected_sha1: &str,
) -> Result<()> {
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
