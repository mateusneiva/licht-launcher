use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CoreError, DEFAULT_CONCURRENCY, Result, SharedCache};

const SCHEMA: u32 = 1;
const MIN_CONCURRENCY: u32 = 1;
const MAX_CONCURRENCY: u32 = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub schema: u32,
    pub download_concurrency: u32,
}

fn settings_path(cache: &SharedCache) -> PathBuf {
    cache.root().join("settings.json")
}

fn default_concurrency() -> Result<u32> {
    u32::try_from(DEFAULT_CONCURRENCY).map_err(|_| CoreError::SettingsConcurrency)
}

fn validated(download_concurrency: u32) -> Result<Settings> {
    if !(MIN_CONCURRENCY..=MAX_CONCURRENCY).contains(&download_concurrency) {
        return Err(CoreError::SettingsConcurrency);
    }
    Ok(Settings {
        schema: SCHEMA,
        download_concurrency,
    })
}

fn read_settings(path: &Path) -> Result<Settings> {
    let text = std::fs::read_to_string(path)?;
    let settings: Settings = serde_json::from_str(&text).map_err(CoreError::Settings)?;
    if settings.schema != SCHEMA {
        return Err(CoreError::SettingsSchema {
            schema: settings.schema,
        });
    }
    validated(settings.download_concurrency)
}

/// Missing `settings.json` means the default of 8. The file is created only when saved.
pub fn load_settings(cache: &SharedCache) -> Result<Settings> {
    let path = settings_path(cache);
    if !path.is_file() {
        return validated(default_concurrency()?);
    }
    read_settings(&path)
}

pub fn save_settings(cache: &SharedCache, download_concurrency: u32) -> Result<Settings> {
    let path = settings_path(cache);
    if path.is_file() {
        let _existing = read_settings(&path)?;
    }
    let settings = validated(download_concurrency)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(CoreError::Settings)?;
    std::fs::write(path, format!("{json}\n"))?;
    Ok(settings)
}
