use serde::Deserialize;

use crate::Result;

pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<ManifestVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

pub fn parse_version_manifest(json: &str) -> Result<VersionManifest> {
    Ok(serde_json::from_str(json)?)
}

pub async fn fetch_version_manifest(client: &reqwest::Client) -> Result<VersionManifest> {
    let body = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    parse_version_manifest(&body)
}
