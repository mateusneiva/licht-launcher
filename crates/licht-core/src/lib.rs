//! Minecraft launcher core, independent of Tauri.

mod asset_index;
mod cache;
mod download;
mod error;
mod install;
mod java;
mod launch;
mod natives;
mod offline;
mod run;
mod version;
mod version_manifest;

pub use asset_index::{
    AssetIndexFile, AssetObject, fetch_asset_index, parse_asset_index, reconstruct_assets,
};
pub use cache::SharedCache;
pub use download::{
    DEFAULT_CONCURRENCY, DEFAULT_RETRY, DownloadProgress, DownloadTask, Retry, download_all,
    download_file,
};
pub use error::CoreError;
pub use install::{
    ASSET_OBJECT_BASE, GameInstall, InstallArgs, InstallPlan, install_game, install_version,
    parse_install_args,
};
pub use java::{
    ADOPTIUM_API, JAVA_RUNTIME_INDEX_URL, JavaChoice, JavaRuntimeBuild, JavaRuntimeEntry,
    JavaRuntimeIndex, JavaRuntimeManifest, JavaRuntimeManifestRef, default_java_roots,
    discover_javas, fetch_java_runtime_index, install_java, install_temurin, installed_java,
    matching_java, parse_java_runtime_index, parse_java_runtime_manifest, parse_java_version,
    probe_java_major, required_java_major, required_runtime, runtime_platform, select_runtime,
    validate_java,
};
pub use launch::{classpath, launch_command};
pub use natives::{NativeLibrary, create_natives_directory, extract_natives, native_libraries};
pub use offline::{
    LaunchArgs, OfflineAccount, OfflineLaunch, offline_account, parse_launch_args,
    prepare_offline_launch,
};
pub use run::{GameExit, GameLine, OutputStream, run_game};
pub use version::{
    Arch, Argument, ArgumentValue, Artifact, AssetIndex, Download, GameArguments, JavaVersion,
    LaunchEnvironment, Library, LibraryDownloads, LibraryExtract, OsName, OsRule, Rule, RuleAction,
    Version, VersionDownloads, applicable_libraries, parse_version, rules_allow,
};
pub use version_manifest::{
    LatestVersions, ManifestVersion, VERSION_MANIFEST_URL, VersionManifest, VersionType,
    VersionsArgs, fetch_version_manifest, parse_version_manifest, parse_versions_args,
    version_lines, version_type_name,
};

pub type Result<T> = std::result::Result<T, CoreError>;
