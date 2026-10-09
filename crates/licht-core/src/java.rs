use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha1::{Digest, Sha1};
use tokio::sync::mpsc;

use crate::{
    Arch, CoreError, DEFAULT_CONCURRENCY, DEFAULT_RETRY, DownloadProgress, DownloadTask,
    LaunchEnvironment, OsName, Result, SharedCache, Version, download_all,
};

pub const JAVA_RUNTIME_INDEX_URL: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

/// The runtime a version asks for, when the JSON includes `javaVersion`.
pub fn required_runtime(version: &Version) -> Option<&crate::JavaVersion> {
    version.java_version.as_ref()
}

const JAVA_SEARCH_DEPTH: u32 = 4;

pub fn java_binary_name(os: OsName) -> &'static str {
    match os {
        OsName::Windows => "java.exe",
        OsName::Linux | OsName::Osx => "java",
    }
}

/// Usual install directories. A missing directory is not an error.
pub fn default_java_roots(os: OsName) -> &'static [&'static str] {
    match os {
        OsName::Windows => &[
            r"C:\Program Files\Java",
            r"C:\Program Files\Eclipse Adoptium",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\BellSoft",
            r"C:\Program Files\Zulu",
        ],
        OsName::Linux => &["/usr/lib/jvm", "/usr/java"],
        OsName::Osx => &[
            "/Library/Java/JavaVirtualMachines",
            "/System/Library/Java/JavaVirtualMachines",
        ],
    }
}

/// Major version printed by `java -version`.
///
/// Java 8 reports `1.8.0_51`. Java 9 and later report `21.0.7`.
pub fn parse_java_version(output: &str) -> Result<u32> {
    for line in output.lines() {
        let Some(version) = quoted_version(line) else {
            continue;
        };
        return major_from_version(version);
    }
    Err(CoreError::JavaProbe)
}

pub fn probe_java_major(executable: &Path) -> Result<u32> {
    let output = std::process::Command::new(executable)
        .arg("-version")
        .output()?;
    if !output.status.success() {
        return Err(CoreError::JavaProbe);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stderr.trim().is_empty() {
        stdout
    } else {
        stderr
    };
    parse_java_version(&text)
}

pub fn validate_java(executable: &Path, required: &crate::JavaVersion) -> Result<u32> {
    let found = probe_java_major(executable)?;
    if found != required.major_version {
        return Err(CoreError::JavaMajor {
            found,
            required: required.major_version,
        });
    }
    Ok(found)
}

/// `java` / `java.exe` under each root, including the usual `bin` and macOS `.jdk` layouts.
pub fn discover_javas(roots: &[PathBuf], os: OsName) -> Vec<PathBuf> {
    let name = java_binary_name(os);
    let mut found = Vec::new();
    for root in roots {
        collect_javas(root, name, 0, &mut found);
    }
    found.sort();
    found.dedup();
    found
}

/// First executable whose major version equals `required`. Others are skipped.
pub fn matching_java(executables: &[PathBuf], required: &crate::JavaVersion) -> Option<PathBuf> {
    for executable in executables {
        match probe_java_major(executable) {
            Ok(major) if major == required.major_version => return Some(executable.clone()),
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(
                    path = %executable.display(),
                    "installed Java could not be read: {error}"
                );
            }
        }
    }
    None
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
                        size: files.raw.size,
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

pub fn installed_java(
    cache: &SharedCache,
    version: &Version,
    environment: &LaunchEnvironment,
) -> Result<PathBuf> {
    let Some(runtime) = required_runtime(version) else {
        return Err(CoreError::JavaRuntimeMissing);
    };
    let root = cache.runtime_dir(
        &runtime.component,
        runtime_platform(environment.os, environment.arch)?,
    )?;
    let path = java_executable(&root, environment.os);
    if path.is_file() {
        Ok(path)
    } else {
        Err(CoreError::JavaRuntimeMissing)
    }
}

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
    }
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(relative, link).is_err() {
            tracing::warn!("Java runtime symlink was copied because the link could not be created");
            let Some(parent) = link.parent() else {
                return Err(CoreError::CachePath);
            };
            std::fs::copy(parent.join(relative), link)?;
        }
    }
    Ok(())
}

fn quoted_version(line: &str) -> Option<&str> {
    let marker = "version \"";
    let start = line.find(marker)? + marker.len();
    let rest = line.get(start..)?;
    let end = rest.find('"')?;
    rest.get(..end)
}

fn major_from_version(version: &str) -> Result<u32> {
    let mut parts = version.split('.');
    let Some(first) = parts.next().filter(|part| !part.is_empty()) else {
        return Err(CoreError::JavaProbe);
    };
    let component = if first == "1" {
        parts.next().ok_or(CoreError::JavaProbe)?
    } else {
        first
    };
    let digits: String = component
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return Err(CoreError::JavaProbe);
    }
    digits.parse().map_err(|_| CoreError::JavaProbe)
}

fn collect_javas(dir: &Path, name: &str, depth: u32, found: &mut Vec<PathBuf>) {
    let candidate = dir.join(name);
    if candidate.is_file() {
        found.push(candidate);
    }
    if depth == JAVA_SEARCH_DEPTH {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            tracing::warn!(path = %dir.display(), "Java directory could not be read: {error}");
            return;
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            collect_javas(&path, name, depth + 1, found);
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
