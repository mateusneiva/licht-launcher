use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub arguments: GameArguments,
    pub libraries: Vec<Library>,
    pub asset_index: AssetIndex,
    pub main_class: String,
    pub downloads: VersionDownloads,
    pub java_version: Option<JavaVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameArguments {
    Legacy(String),
    Modern {
        game: Vec<Argument>,
        jvm: Vec<Argument>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Literal(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<BTreeMap<String, bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub natives: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub extract: Option<LibraryExtract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<BTreeMap<String, Artifact>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LibraryExtract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionDownloads {
    pub client: Download,
    #[serde(default)]
    pub server: Option<Download>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
struct VersionFile {
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<String>,
    arguments: Option<ModernArguments>,
    libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    #[serde(rename = "mainClass")]
    main_class: String,
    downloads: VersionDownloads,
    #[serde(rename = "javaVersion")]
    java_version: Option<JavaVersion>,
}

#[derive(Debug, Deserialize)]
struct ModernArguments {
    #[serde(default)]
    game: Vec<Argument>,
    #[serde(default)]
    jvm: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchEnvironment {
    pub os: OsName,
    pub arch: Arch,
    pub os_version: String,
    pub features: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsName {
    Windows,
    Linux,
    Osx,
}

impl OsName {
    fn mojang_name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Osx => "osx",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86,
    X86_64,
}

impl Arch {
    fn mojang_name(self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }
}

pub fn rules_allow(rules: Option<&[Rule]>, env: &LaunchEnvironment) -> bool {
    let Some(rules) = rules else {
        return true;
    };
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, env) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

pub fn applicable_libraries<'a>(version: &'a Version, env: &LaunchEnvironment) -> Vec<&'a Library> {
    version
        .libraries
        .iter()
        .filter(|library| rules_allow(library.rules.as_deref(), env))
        .collect()
}

fn rule_matches(rule: &Rule, env: &LaunchEnvironment) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name
            && name != env.os.mojang_name()
        {
            return false;
        }
        if let Some(arch) = &os.arch
            && arch != env.arch.mojang_name()
        {
            return false;
        }
        if let Some(version) = &os.version {
            let Ok(pattern) = regex::Regex::new(version) else {
                return false;
            };
            if !pattern.is_match(&env.os_version) {
                return false;
            }
        }
    }

    if let Some(features) = &rule.features {
        for (name, expected) in features {
            let actual = env.features.get(name).copied().unwrap_or(false);
            if actual != *expected {
                return false;
            }
        }
    }

    true
}

pub fn parse_version(json: &str) -> Result<Version> {
    let file: VersionFile = serde_json::from_str(json).map_err(CoreError::Version)?;
    let arguments = match (file.minecraft_arguments, file.arguments) {
        (Some(legacy), None) => GameArguments::Legacy(legacy),
        (None, Some(modern)) => GameArguments::Modern {
            game: modern.game,
            jvm: modern.jvm,
        },
        _ => return Err(CoreError::VersionArguments),
    };

    Ok(Version {
        arguments,
        libraries: file.libraries,
        asset_index: file.asset_index,
        main_class: file.main_class,
        downloads: file.downloads,
        java_version: file.java_version,
    })
}
