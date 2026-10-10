use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{CoreError, Result};

const SCHEMA: u32 = 1;
const DEFAULT_MIN_MEMORY_MB: u32 = 512;
const DEFAULT_MAX_MEMORY_MB: u32 = 2048;
const FILE_NAME: &str = "instance.json";

/// Profile stored at `instances/<id>/instance.json`.
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
    let instance = Instance {
        schema: SCHEMA,
        name: name.clone(),
        version_id: name,
        min_memory_mb: DEFAULT_MIN_MEMORY_MB,
        max_memory_mb: DEFAULT_MAX_MEMORY_MB,
        jvm_arguments: Vec::new(),
        width: None,
        height: None,
    };
    let body = serde_json::to_string_pretty(&instance).map_err(CoreError::Instance)?;
    fs::write(path, body)?;
    Ok(instance)
}

/// Loads every direct child directory. A missing `instances` directory is an empty list.
pub fn list_instances(instances_dir: &Path) -> Result<Vec<Instance>> {
    if !instances_dir.exists() {
        return Ok(Vec::new());
    }
    let mut instances = Vec::new();
    for entry in fs::read_dir(instances_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            instances.push(load_instance(&entry.path())?);
        }
    }
    instances.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version_id.cmp(&right.version_id))
    });
    Ok(instances)
}

fn validate(instance: &Instance) -> Result<()> {
    single_component(&instance.name)?;
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
