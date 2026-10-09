//! Minecraft launcher core, independent of Tauri.

mod error;
mod version_manifest;

pub use error::CoreError;
pub use version_manifest::{
    LatestVersions, ManifestVersion, VERSION_MANIFEST_URL, VersionManifest, VersionType,
    fetch_version_manifest, parse_version_manifest,
};

pub type Result<T> = std::result::Result<T, CoreError>;
