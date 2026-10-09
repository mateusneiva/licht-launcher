use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    Arch, Artifact, CoreError, LaunchEnvironment, Library, OsName, Result, SharedCache, Version,
    applicable_libraries,
};

static NATIVES_RUNS: AtomicU64 = AtomicU64::new(0);

pub struct NativeLibrary<'a> {
    pub artifact: &'a Artifact,
    pub exclude: &'a [String],
}

/// Native jars for this OS and architecture.
///
/// Old versions name the jar in `natives`, and `${arch}` becomes `32` or `64`.
/// Current versions put `natives-windows` in the library name. The plain name is
/// 64-bit; `natives-windows-x86` and `natives-windows-arm64` are other machines.
pub fn native_libraries<'a>(
    version: &'a Version,
    env: &LaunchEnvironment,
) -> Vec<NativeLibrary<'a>> {
    let mut libraries = Vec::new();
    for library in applicable_libraries(version, env) {
        if let Some(native) = legacy_native(library, env) {
            libraries.push(native);
            continue;
        }
        if let Some(native) = modern_native(library, env) {
            libraries.push(native);
        }
    }
    libraries
}

pub fn extract_natives(
    cache: &SharedCache,
    version: &Version,
    env: &LaunchEnvironment,
    destination: &Path,
) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for library in native_libraries(version, env) {
        let path = cache.library(&library.artifact.path)?;
        extract_archive(&path, destination, library.exclude)?;
    }
    Ok(())
}

/// A new directory under the system temp folder. Each call is a separate run.
pub fn create_natives_directory() -> Result<PathBuf> {
    let run = NATIVES_RUNS.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("licht-natives-{}-{run}", std::process::id()));
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn legacy_native<'a>(library: &'a Library, env: &LaunchEnvironment) -> Option<NativeLibrary<'a>> {
    let template = library.natives.as_ref()?.get(env.os.mojang_name())?;
    let classifier = template.replace("${arch}", arch_bits(env.arch));
    let artifact = library
        .downloads
        .as_ref()?
        .classifiers
        .as_ref()?
        .get(&classifier)?;
    Some(NativeLibrary {
        artifact,
        exclude: exclude_list(library),
    })
}

fn modern_native<'a>(library: &'a Library, env: &LaunchEnvironment) -> Option<NativeLibrary<'a>> {
    let classifier = library.name.split(':').nth(3)?;
    if !classifier_matches(classifier, env.os, env.arch) {
        return None;
    }
    let artifact = library.downloads.as_ref()?.artifact.as_ref()?;
    Some(NativeLibrary {
        artifact,
        exclude: exclude_list(library),
    })
}

fn classifier_matches(classifier: &str, os: OsName, arch: Arch) -> bool {
    let Some(rest) = classifier.strip_prefix("natives-") else {
        return false;
    };
    let (platform, suffix) = match rest.split_once('-') {
        Some((platform, suffix)) => (platform, Some(suffix)),
        None => (rest, None),
    };
    let platform_matches = match os {
        OsName::Windows => platform == "windows",
        OsName::Linux => platform == "linux",
        OsName::Osx => platform == "osx" || platform == "macos",
    };
    if !platform_matches {
        return false;
    }
    match arch {
        Arch::X86_64 => suffix.is_none(),
        Arch::X86 => suffix == Some("x86"),
    }
}

fn arch_bits(arch: Arch) -> &'static str {
    match arch {
        Arch::X86_64 => "64",
        Arch::X86 => "32",
    }
}

fn exclude_list(library: &Library) -> &[String] {
    library
        .extract
        .as_ref()
        .map(|extract| extract.exclude.as_slice())
        .unwrap_or(&[])
}

fn extract_archive(jar: &Path, destination: &Path, exclude: &[String]) -> Result<()> {
    let file = File::open(jar)?;
    let mut archive = zip::ZipArchive::new(file).map_err(CoreError::NativeArchive)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(CoreError::NativeArchive)?;
        let name = entry.name().replace('\\', "/");
        if entry.is_dir() || is_excluded(&name, exclude) {
            continue;
        }
        let Some(relative) = entry.enclosed_name() else {
            return Err(CoreError::NativePath);
        };
        if relative.as_os_str().is_empty() {
            return Err(CoreError::NativePath);
        }
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut output = File::create(target)?;
        io::copy(&mut entry, &mut output)?;
    }
    Ok(())
}

fn is_excluded(name: &str, exclude: &[String]) -> bool {
    exclude.iter().any(|prefix| name.starts_with(prefix))
}
