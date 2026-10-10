use std::collections::BTreeMap;

use licht_core::{
    ASSET_OBJECT_BASE, Arch, CoreError, GameExit, GameInstall, InstanceEntry, LaunchEnvironment,
    OsName, Settings, SharedCache, VersionSummary, fetch_version_manifest, install_game,
    installed_java, load_settings, log_codec, offline_account, parse_version,
    prepare_offline_launch, required_java_major, run_game, save_settings, version_summaries,
};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{Mutex, MutexGuard, mpsc};

use crate::progress::ProgressThrottle;

/// One install or one launch at a time. Listing versions does not take the lock.
pub struct OperationLock {
    gate: Mutex<()>,
}

impl OperationLock {
    pub fn new() -> Self {
        Self {
            gate: Mutex::new(()),
        }
    }

    pub fn try_begin(&self) -> Result<MutexGuard<'_, ()>, String> {
        self.gate
            .try_lock()
            .map_err(|_| "another install or launch is already running".to_string())
    }
}

fn failure(error: CoreError) -> String {
    error.to_string()
}

fn host_environment() -> Result<LaunchEnvironment, CoreError> {
    let os = match std::env::consts::OS {
        "windows" => OsName::Windows,
        "linux" => OsName::Linux,
        "macos" => OsName::Osx,
        _ => return Err(CoreError::LaunchHost),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => Arch::X86_64,
        "x86" => Arch::X86,
        _ => return Err(CoreError::LaunchHost),
    };
    Ok(LaunchEnvironment {
        os,
        arch,
        os_version: String::new(),
        features: BTreeMap::new(),
    })
}

fn instances_root() -> Result<std::path::PathBuf, CoreError> {
    SharedCache::system()?.instances_dir()
}

#[tauri::command]
pub fn list_instances() -> Result<Vec<InstanceEntry>, String> {
    let root = instances_root().map_err(failure)?;
    licht_core::list_instances(&root).map_err(failure)
}

#[tauri::command]
pub fn create_instance(name: String, version_id: String) -> Result<InstanceEntry, String> {
    let root = instances_root().map_err(failure)?;
    licht_core::create_instance(&root, &name, &version_id).map_err(failure)
}

#[tauri::command]
pub fn rename_instance(folder: String, name: String) -> Result<InstanceEntry, String> {
    let root = instances_root().map_err(failure)?;
    licht_core::rename_instance(&root, &folder, &name).map_err(failure)
}

#[tauri::command]
pub fn duplicate_instance(folder: String, name: String) -> Result<InstanceEntry, String> {
    let root = instances_root().map_err(failure)?;
    licht_core::duplicate_instance(&root, &folder, &name).map_err(failure)
}

#[tauri::command]
pub fn delete_instance(folder: String) -> Result<(), String> {
    let root = instances_root().map_err(failure)?;
    licht_core::delete_instance(&root, &folder).map_err(failure)
}

#[tauri::command]
pub fn get_settings() -> Result<Settings, String> {
    let cache = SharedCache::system().map_err(failure)?;
    load_settings(&cache).map_err(failure)
}

#[tauri::command]
pub fn set_download_concurrency(download_concurrency: u32) -> Result<Settings, String> {
    let cache = SharedCache::system().map_err(failure)?;
    save_settings(&cache, download_concurrency).map_err(failure)
}

#[tauri::command]
pub async fn list_versions() -> Result<Vec<VersionSummary>, String> {
    let cache = SharedCache::system().map_err(failure)?;
    let manifest = fetch_version_manifest(&reqwest::Client::new())
        .await
        .map_err(failure)?;
    Ok(version_summaries(&manifest, &cache))
}

#[tauri::command]
pub async fn install_version(
    app: AppHandle,
    state: State<'_, OperationLock>,
    version_id: String,
) -> Result<String, String> {
    let _guard = state.try_begin()?;
    install_one(&app, &version_id).await.map_err(failure)
}

async fn install_one(app: &AppHandle, version_id: &str) -> Result<String, CoreError> {
    let cache = SharedCache::system()?;
    let client = reqwest::Client::new();
    let manifest = fetch_version_manifest(&client).await?;
    let Some(entry) = manifest
        .versions
        .iter()
        .find(|version| version.id == version_id)
    else {
        return Err(CoreError::VersionMissing);
    };

    let (progress, mut incoming) = mpsc::channel(4096);
    let version_url = entry.url.clone();
    let version_sha1 = entry.sha1.clone();
    let id = version_id.to_string();
    let environment = host_environment()?;
    let installing = tauri::async_runtime::spawn(async move {
        install_game(
            &client,
            &cache,
            GameInstall {
                version_id: &id,
                version_json_url: &version_url,
                version_sha1: &version_sha1,
                environment: &environment,
                asset_base: ASSET_OBJECT_BASE,
                java: None,
            },
            progress,
        )
        .await
    });

    let mut throttle = ProgressThrottle::new();
    while let Some(progress) = incoming.recv().await {
        if throttle.should_emit(progress, std::time::Instant::now()) {
            let _ = app.emit("install-progress", progress);
        }
    }

    let java = installing
        .await
        .map_err(|_| CoreError::Io(std::io::Error::other("install task failed")))??;
    Ok(java.display().to_string())
}

#[tauri::command]
pub async fn launch(
    app: AppHandle,
    state: State<'_, OperationLock>,
    version_id: String,
    username: String,
) -> Result<GameExit, String> {
    let _guard = state.try_begin()?;
    launch_one(&app, &version_id, &username)
        .await
        .map_err(failure)
}

async fn launch_one(
    app: &AppHandle,
    version_id: &str,
    username: &str,
) -> Result<GameExit, CoreError> {
    let cache = SharedCache::system()?;
    let json = std::fs::read_to_string(cache.version_json(version_id)?)?;
    let version = parse_version(&json)?;
    let environment = host_environment()?;
    let java = installed_java(&cache, version_id, &version, &environment)?;
    let codec = log_codec(required_java_major(version_id, &version));
    let account = offline_account(username)?;
    let instance = cache.instance_dir(version_id)?;
    std::fs::create_dir_all(&instance)?;
    let prepared = prepare_offline_launch(
        &java,
        &cache,
        version_id,
        &version,
        &environment,
        &account,
        &instance,
    )?;

    let (output, mut incoming) = mpsc::channel(32);
    let appdata = prepared.appdata.map(|path| path.display().to_string());
    let command = prepared.command;
    let game_directory = prepared.game_directory;
    let running = tauri::async_runtime::spawn(async move {
        let mut env = Vec::new();
        if let Some(path) = &appdata {
            env.push(("APPDATA", path.as_str()));
        }
        run_game(&command, Some(&game_directory), &env, codec, output).await
    });
    while let Some(line) = incoming.recv().await {
        let _ = app.emit("game-log", line);
    }
    running
        .await
        .map_err(|_| CoreError::Io(std::io::Error::other("launch task failed")))?
}
