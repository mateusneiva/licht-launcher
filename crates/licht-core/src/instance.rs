use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CoreError, Result};

const SCHEMA: u32 = 1;
const DEFAULT_MIN_MEMORY_MB: u32 = 512;
const DEFAULT_MAX_MEMORY_MB: u32 = 2048;
const FILE_NAME: &str = "instance.json";

/// Profile stored at `instances/<folder>/instance.json`.
///
/// LaunchWrapper keeps the game in a child `.minecraft` folder, so this file
/// stays beside that folder instead of inside the directory the old client opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub schema: u32,
    pub name: String,
    pub version_id: String,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub jvm_arguments: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// An instance folder and the profile stored in it.
///
/// `folder` is chosen once and does not change when the profile is renamed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InstanceEntry {
    pub folder: String,
    pub name: String,
    pub version_id: String,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub jvm_arguments: Vec<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Deserialize)]
struct SchemaHeader {
    schema: u32,
}

pub fn parse_instance(json: &str) -> Result<Instance> {
    let header: SchemaHeader = serde_json::from_str(json).map_err(CoreError::Instance)?;
    if header.schema != SCHEMA {
        return Err(CoreError::InstanceSchema {
            schema: header.schema,
        });
    }
    let instance: Instance = serde_json::from_str(json).map_err(CoreError::Instance)?;
    validate(&instance)?;
    Ok(instance)
}

/// Reads `instance.json` in `dir`. A missing file is written as schema 1.
/// The directory itself is not created.
pub fn load_instance(dir: &Path) -> Result<Instance> {
    if !dir.is_dir() {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "instance directory is missing",
        )));
    }
    let path = dir.join(FILE_NAME);
    if path.is_file() {
        let json = fs::read_to_string(&path)?;
        return parse_instance(&json);
    }
    let name = directory_name(dir)?;
    let instance = default_instance(name.clone(), name);
    write_instance(dir, &instance)?;
    Ok(instance)
}

/// Loads every direct child directory. A missing `instances` directory is an empty list.
pub fn list_instances(instances_dir: &Path) -> Result<Vec<InstanceEntry>> {
    if !instances_dir.exists() {
        return Ok(Vec::new());
    }
    let mut instances = Vec::new();
    for entry in fs::read_dir(instances_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let folder = directory_name(&entry.path())?;
            let instance = load_instance(&entry.path())?;
            instances.push(entry_from(folder, instance));
        }
    }
    instances.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.folder.cmp(&right.folder))
    });
    Ok(instances)
}

pub fn create_instance(
    instances_dir: &Path,
    name: &str,
    version_id: &str,
) -> Result<InstanceEntry> {
    let name = display_name(name)?;
    single_component(version_id)?;
    let folder = folder_id(&name)?;
    fs::create_dir_all(instances_dir)?;
    let dir = instances_dir.join(&folder);
    if dir.exists() {
        return Err(CoreError::InstanceExists);
    }
    fs::create_dir(&dir)?;
    let instance = default_instance(name, version_id.to_string());
    write_instance(&dir, &instance)?;
    Ok(entry_from(folder, instance))
}

pub fn rename_instance(instances_dir: &Path, folder: &str, name: &str) -> Result<InstanceEntry> {
    let name = display_name(name)?;
    let dir = instance_dir(instances_dir, folder)?;
    let mut instance = load_instance(&dir)?;
    instance.name = name;
    write_instance(&dir, &instance)?;
    Ok(entry_from(folder.to_string(), instance))
}

pub fn duplicate_instance(instances_dir: &Path, folder: &str, name: &str) -> Result<InstanceEntry> {
    let name = display_name(name)?;
    let source = instance_dir(instances_dir, folder)?;
    let _ = load_instance(&source)?;
    let new_folder = folder_id(&name)?;
    let destination = instances_dir.join(&new_folder);
    if destination.exists() {
        return Err(CoreError::InstanceExists);
    }
    if let Err(error) = copy_tree(&source, &destination) {
        let _ = fs::remove_dir_all(&destination);
        return Err(error);
    }
    let mut instance = load_instance(&destination)?;
    instance.name = name;
    if let Err(error) = write_instance(&destination, &instance) {
        let _ = fs::remove_dir_all(&destination);
        return Err(error);
    }
    Ok(entry_from(new_folder, instance))
}

pub fn delete_instance(instances_dir: &Path, folder: &str) -> Result<()> {
    let dir = instance_dir(instances_dir, folder)?;
    fs::remove_dir_all(dir)?;
    Ok(())
}

fn default_instance(name: String, version_id: String) -> Instance {
    Instance {
        schema: SCHEMA,
        name,
        version_id,
        min_memory_mb: DEFAULT_MIN_MEMORY_MB,
        max_memory_mb: DEFAULT_MAX_MEMORY_MB,
        jvm_arguments: Vec::new(),
        width: None,
        height: None,
    }
}

fn entry_from(folder: String, instance: Instance) -> InstanceEntry {
    InstanceEntry {
        folder,
        name: instance.name,
        version_id: instance.version_id,
        min_memory_mb: instance.min_memory_mb,
        max_memory_mb: instance.max_memory_mb,
        jvm_arguments: instance.jvm_arguments,
        width: instance.width,
        height: instance.height,
    }
}

fn write_instance(dir: &Path, instance: &Instance) -> Result<()> {
    let body = serde_json::to_string_pretty(instance).map_err(CoreError::Instance)?;
    fs::write(dir.join(FILE_NAME), body)?;
    Ok(())
}

fn instance_dir(instances_dir: &Path, folder: &str) -> Result<PathBuf> {
    single_component(folder)?;
    let dir = instances_dir.join(folder);
    if !dir.is_dir() {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "instance directory is missing",
        )));
    }
    Ok(dir)
}

fn display_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(CoreError::InstanceName);
    }
    Ok(name.to_string())
}

fn folder_id(name: &str) -> Result<String> {
    let mut folder = String::new();
    let mut hyphen = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '.' || character == '_' {
            if hyphen && !folder.is_empty() {
                folder.push('-');
            }
            hyphen = false;
            folder.push(character);
        } else if character.is_whitespace() || character == '-' {
            hyphen = true;
        }
    }
    single_component(&folder).map_err(|_| CoreError::InstanceName)?;
    Ok(folder)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target)?;
        } else {
            return Err(CoreError::CachePath);
        }
    }
    Ok(())
}

fn validate(instance: &Instance) -> Result<()> {
    display_name(&instance.name)?;
    single_component(&instance.version_id)?;
    if instance.min_memory_mb == 0
        || instance.max_memory_mb == 0
        || instance.min_memory_mb > instance.max_memory_mb
    {
        return Err(CoreError::InstanceMemory);
    }
    Ok(())
}

fn directory_name(dir: &Path) -> Result<String> {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(CoreError::CachePath)?;
    single_component(name)?;
    Ok(name.to_string())
}

fn single_component(name: &str) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err(CoreError::CachePath);
    }
    Ok(())
}
