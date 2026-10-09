use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use md5::Digest;

use crate::{
    CoreError, LaunchEnvironment, Result, SharedCache, Version, create_natives_directory,
    extract_natives, launch_command,
};

#[derive(Debug)]
pub struct OfflineAccount {
    pub username: String,
    pub uuid: String,
}

/// Offline player id used by the game: version 3 UUID of `OfflinePlayer:<name>`,
/// written without hyphens. A namespace UUID would not match.
pub fn offline_account(username: &str) -> Result<OfflineAccount> {
    if username.is_empty() {
        return Err(CoreError::OfflineName);
    }
    let digest = md5::Md5::digest(format!("OfflinePlayer:{username}"));
    let bytes: [u8; 16] = digest.into();
    let uuid = uuid::Builder::from_md5_bytes(bytes)
        .into_uuid()
        .as_simple()
        .to_string();
    Ok(OfflineAccount {
        username: username.to_string(),
        uuid,
    })
}

#[derive(Debug)]
pub struct LaunchArgs {
    pub version_id: String,
    pub username: String,
    pub java: Option<PathBuf>,
    pub game_directory: PathBuf,
    pub cache: Option<PathBuf>,
}

pub fn parse_launch_args(args: &[String]) -> Result<LaunchArgs> {
    if args.first().map(String::as_str) != Some("launch") {
        return Err(CoreError::LaunchArgs);
    }

    let mut version_id = None;
    let mut username = None;
    let mut java = None;
    let mut game_directory = None;
    let mut cache = None;
    let mut rest = args[1..].iter();
    while let Some(flag) = rest.next() {
        let Some(value) = rest.next() else {
            return Err(CoreError::LaunchArgs);
        };
        if value.starts_with("--") {
            return Err(CoreError::LaunchArgs);
        }
        match flag.as_str() {
            "--version" => version_id = Some(value.clone()),
            "--username" => username = Some(value.clone()),
            "--java" => java = Some(PathBuf::from(value)),
            "--game-dir" => game_directory = Some(PathBuf::from(value)),
            "--cache" => cache = Some(PathBuf::from(value)),
            _ => return Err(CoreError::LaunchArgs),
        }
    }

    let Some(version_id) = version_id.filter(|id| !id.is_empty()) else {
        return Err(CoreError::LaunchArgs);
    };
    let Some(username) = username else {
        return Err(CoreError::LaunchArgs);
    };
    if username.is_empty() {
        return Err(CoreError::OfflineName);
    }
    let Some(game_directory) = game_directory else {
        return Err(CoreError::LaunchArgs);
    };

    Ok(LaunchArgs {
        version_id,
        username,
        java,
        game_directory,
        cache,
    })
}

/// Extracts natives, then builds the command for an offline account.
/// Does not start the process and does not print the command.
pub fn prepare_offline_launch(
    java: &Path,
    cache: &SharedCache,
    version_id: &str,
    version: &Version,
    environment: &LaunchEnvironment,
    account: &OfflineAccount,
    game_directory: &Path,
) -> Result<Vec<String>> {
    let natives_directory = create_natives_directory(cache, version_id)?;
    extract_natives(cache, version, environment, &natives_directory)?;

    let mut values = BTreeMap::new();
    values.insert("auth_player_name".to_string(), account.username.clone());
    values.insert("auth_uuid".to_string(), account.uuid.clone());
    values.insert("auth_access_token".to_string(), "0".to_string());
    values.insert("user_type".to_string(), "legacy".to_string());
    values.insert("user_properties".to_string(), "{}".to_string());
    values.insert("auth_xuid".to_string(), String::new());
    values.insert("clientid".to_string(), String::new());
    values.insert("version_name".to_string(), version_id.to_string());
    values.insert("version_type".to_string(), "release".to_string());
    values.insert("launcher_name".to_string(), "Licht Launcher".to_string());
    values.insert(
        "launcher_version".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    values.insert(
        "game_directory".to_string(),
        game_directory.display().to_string(),
    );
    values.insert(
        "natives_directory".to_string(),
        natives_directory.display().to_string(),
    );
    values.insert(
        "assets_root".to_string(),
        cache.root().join("assets").display().to_string(),
    );
    values.insert(
        "assets_index_name".to_string(),
        version.asset_index.id.clone(),
    );

    launch_command(java, cache, version_id, version, environment, &values)
}
