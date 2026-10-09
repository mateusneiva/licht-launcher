use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha1::{Digest, Sha1};
use tokio::sync::mpsc;

use crate::download::DownloadProgress;
use crate::{
    AssetIndexFile, CoreError, DEFAULT_CONCURRENCY, DEFAULT_RETRY, DownloadTask, LaunchEnvironment,
    Result, SharedCache, Version, applicable_libraries, download_all,
};

pub const ASSET_OBJECT_BASE: &str = "https://resources.download.minecraft.net";

pub struct InstallPlan<'a> {
    pub version_id: &'a str,
    pub version: &'a Version,
    pub asset_index_json: &'a str,
    pub assets: &'a AssetIndexFile,
    pub environment: &'a LaunchEnvironment,
    pub asset_base: &'a str,
}

pub async fn install_version(
    client: &reqwest::Client,
    cache: &SharedCache,
    plan: InstallPlan<'_>,
    progress: mpsc::Sender<DownloadProgress>,
) -> Result<()> {
    cache.create()?;
    write_asset_index(cache, plan.version, plan.asset_index_json)?;

    let mut pending = Vec::new();
    for task in download_tasks(cache, &plan)? {
        if file_matches(&task.destination, &task.sha1)? {
            continue;
        }
        if let Some(parent) = task.destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        pending.push(task);
    }

    download_all(
        client,
        &pending,
        DEFAULT_CONCURRENCY,
        DEFAULT_RETRY,
        progress,
    )
    .await
}

fn write_asset_index(cache: &SharedCache, version: &Version, json: &str) -> Result<()> {
    let expected = &version.asset_index.sha1;
    let actual = hex_encode(Sha1::digest(json.as_bytes()).as_slice());
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(CoreError::Sha1Mismatch {
            expected: expected.clone(),
            actual,
        });
    }

    let path = cache.asset_index(&version.asset_index.id)?;
    if file_matches(&path, expected)? {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

fn download_tasks(cache: &SharedCache, plan: &InstallPlan<'_>) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();
    tasks.push(DownloadTask {
        url: plan.version.downloads.client.url.clone(),
        destination: cache.version_jar(plan.version_id)?,
        sha1: plan.version.downloads.client.sha1.clone(),
    });

    for library in applicable_libraries(plan.version, plan.environment) {
        let Some(downloads) = &library.downloads else {
            continue;
        };
        if let Some(artifact) = &downloads.artifact {
            tasks.push(task_for_artifact(cache, artifact)?);
        }
        let Some(natives) = &library.natives else {
            continue;
        };
        let Some(classifier_name) = natives.get(plan.environment.os.mojang_name()) else {
            continue;
        };
        let Some(classifiers) = &downloads.classifiers else {
            continue;
        };
        let Some(artifact) = classifiers.get(classifier_name) else {
            continue;
        };
        tasks.push(task_for_artifact(cache, artifact)?);
    }

    let base = plan.asset_base.trim_end_matches('/');
    for object in plan.assets.objects.values() {
        let hash = object.hash.to_ascii_lowercase();
        let destination = cache.asset_object(&hash)?;
        let prefix = &hash[..2];
        tasks.push(DownloadTask {
            url: format!("{base}/{prefix}/{hash}"),
            destination,
            sha1: hash,
        });
    }

    Ok(tasks)
}

fn task_for_artifact(cache: &SharedCache, artifact: &crate::Artifact) -> Result<DownloadTask> {
    Ok(DownloadTask {
        url: artifact.url.clone(),
        destination: cache.library(&artifact.path)?,
        sha1: artifact.sha1.clone(),
    })
}

fn file_matches(path: &Path, expected_sha1: &str) -> Result<bool> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_encode(hasher.finalize().as_slice()).eq_ignore_ascii_case(expected_sha1))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
