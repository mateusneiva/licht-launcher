//! Minecraft launcher core, independent of Tauri.

mod asset_index;
mod error;
mod version;
mod version_manifest;

pub use asset_index::{AssetIndexFile, AssetObject, fetch_asset_index, parse_asset_index};
pub use error::CoreError;
pub use version::{
    Arch, Argument, ArgumentValue, Artifact, AssetIndex, Download, GameArguments, JavaVersion,
    LaunchEnvironment, Library, LibraryDownloads, LibraryExtract, OsName, OsRule, Rule, RuleAction,
    Version, VersionDownloads, applicable_libraries, parse_version, rules_allow,
};
pub use version_manifest::{
    LatestVersions, ManifestVersion, VERSION_MANIFEST_URL, VersionManifest, VersionType,
    fetch_version_manifest, parse_version_manifest,
};

pub type Result<T> = std::result::Result<T, CoreError>;
