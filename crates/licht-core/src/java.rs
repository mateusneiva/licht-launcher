use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha1::{Digest, Sha1};
use tokio::sync::mpsc;

use crate::{
    Arch, CoreError, DEFAULT_CONCURRENCY, DEFAULT_RETRY, DownloadProgress, DownloadTask, OsName,
    Result, SharedCache, Version, download_all,
};

pub const JAVA_RUNTIME_INDEX_URL: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// The runtime a version asks for, when the JSON includes `javaVersion`.
pub fn required_runtime(version: &Version) -> Option<&crate::JavaVersion> {
    version.java_version.as_ref()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeIndex {
    #[serde(flatten)]
    pub platforms: BTreeMap<String, BTreeMap<String, Vec<JavaRuntimeBuild>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeBuild {
    pub availability: JavaRuntimeAvailability,
    pub manifest: JavaRuntimeManifestRef,
    pub version: JavaRuntimeVersionName,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeAvailability {
    pub group: u32,
    pub progress: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeManifestRef {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeVersionName {
    pub name: String,
    pub released: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeManifest {
    pub files: BTreeMap<String, JavaRuntimeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JavaRuntimeEntry {
    File {
        #[serde(default)]
        executable: bool,
        downloads: JavaRuntimeDownloads,
    },
    Directory,
    Link {
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeDownloads {
    pub raw: JavaRuntimeDownload,
    #[serde(default)]
    pub lzma: Option<JavaRuntimeDownload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaRuntimeDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

pub enum JavaChoice<'a> {
    Mojang {
        component: &'a str,
        os: OsName,
        arch: Arch,
        manifest: &'a JavaRuntimeManifest,
    },
    Custom(&'a Path),
}

pub fn parse_java_runtime_index(json: &str) -> Result<JavaRuntimeIndex> {
    serde_json::from_str(json).map_err(CoreError::JavaRuntime)
}

pub fn parse_java_runtime_manifest(json: &str) -> Result<JavaRuntimeManifest> {
    serde_json::from_str(json).map_err(CoreError::JavaRuntime)
}

pub async fn fetch_java_runtime_index(client: &reqwest::Client) -> Result<JavaRuntimeIndex> {
    let body = client
        .get(JAVA_RUNTIME_INDEX_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    parse_java_runtime_index(&body)
}

pub fn runtime_platform(os: OsName, arch: Arch) -> Result<&'static str> {
    match (os, arch) {
        (OsName::Windows, Arch::X86_64) => Ok("windows-x64"),
        (OsName::Windows, Arch::X86) => Ok("windows-x86"),
        (OsName::Linux, Arch::X86_64) => Ok("linux"),
        (OsName::Linux, Arch::X86) => Ok("linux-i386"),
        (OsName::Osx, Arch::X86_64) => Ok("mac-os"),
        (OsName::Osx, Arch::X86) => Err(CoreError::JavaRuntimeMissing),
    }
}

pub fn select_runtime<'a>(
    index: &'a JavaRuntimeIndex,
    os: OsName,
    arch: Arch,
    component: &str,
) -> Result<&'a JavaRuntimeManifestRef> {
    let platform = runtime_platform(os, arch)?;
    let builds = index
        .platforms
        .get(platform)
        .and_then(|components| components.get(component))
        .filter(|builds| !builds.is_empty())
        .ok_or(CoreError::JavaRuntimeMissing)?;
    let ready = builds
        .iter()
        .rev()
        .find(|build| build.availability.progress == 100)
        .or_else(|| builds.last());
    let Some(build) = ready else {
        return Err(CoreError::JavaRuntimeMissing);
    };
    Ok(&build.manifest)
}

pub async fn install_java(
    client: &reqwest::Client,
    cache: &SharedCache,
    choice: JavaChoice<'_>,
    progress: mpsc::Sender<DownloadProgress>,
) -> Result<PathBuf> {
    let JavaChoice::Mojang {
        component,
        os,
        arch,
        manifest,
    } = choice
    else {
        let JavaChoice::Custom(executable) = choice else {
            return Err(CoreError::JavaPath);
        };
        if !executable.is_file() {
            return Err(CoreError::JavaPath);
        }
        return Ok(executable.to_path_buf());
    };

    let platform = runtime_platform(os, arch)?;
    let root = cache.runtime_dir(component, platform)?;
    std::fs::create_dir_all(&root)?;

    let mut downloads = Vec::new();
    let mut links = Vec::new();
    for (relative, entry) in &manifest.files {
        let path = under(&root, relative)?;
        match entry {
            JavaRuntimeEntry::Directory => std::fs::create_dir_all(path)?,
            JavaRuntimeEntry::File {
                executable,
                downloads: files,
            } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                if !file_matches(&path, &files.raw.sha1)? {
                    downloads.push(DownloadTask {
                        url: files.raw.url.clone(),
                        destination: path.clone(),
                        sha1: files.raw.sha1.clone(),
                    });
                }
                if *executable {
                    links.push(Executable(path));
                }
            }
            JavaRuntimeEntry::Link { target } => links.push(Link {
                path,
                target: target.clone(),
            }),
        }
    }

    download_all(
        client,
        &downloads,
        DEFAULT_CONCURRENCY,
        DEFAULT_RETRY,
        progress,
    )
    .await?;

    for item in links {
        match item {
            Executable(path) => mark_executable(&path)?,
            Link { path, target } => place_link(&path, &target)?,
        }
    }

    Ok(java_executable(&root, os))
}

enum Placed {
    Executable(PathBuf),
    Link { path: PathBuf, target: String },
}

use Placed::{Executable, Link};

fn java_executable(root: &Path, os: OsName) -> PathBuf {
    match os {
        OsName::Windows => root.join("bin").join("java.exe"),
        OsName::Linux | OsName::Osx => root.join("bin").join("java"),
    }
}

fn under(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() {
        return Err(CoreError::CachePath);
    }
    let mut full = root.to_path_buf();
    for component in relative.split(['/', '\\']) {
        if component.is_empty() || component == "." || component == ".." {
            return Err(CoreError::CachePath);
        }
        full.push(component);
    }
    Ok(full)
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

fn mark_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        std::fs::set_permissions(path, permissions)?;
    }
    let _ = path;
    Ok(())
}

fn place_link(link: &Path, target: &str) -> Result<()> {
    if link.exists() {
        std::fs::remove_file(link)?;
    }
    let relative = Path::new(target);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(relative, link)?;
        return Ok(());
    }
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(relative, link).is_ok() {
            return Ok(());
        }
        tracing::warn!("Java runtime symlink was copied because the link could not be created");
        let Some(parent) = link.parent() else {
            return Err(CoreError::CachePath);
        };
        std::fs::copy(parent.join(relative), link)?;
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
