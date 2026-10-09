use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetIndexFile {
    pub objects: BTreeMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub virtual_assets: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub fn parse_asset_index(json: &str) -> Result<AssetIndexFile> {
    serde_json::from_str(json).map_err(CoreError::AssetIndex)
}

pub async fn fetch_asset_index(client: &reqwest::Client, url: &str) -> Result<AssetIndexFile> {
    let body = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    parse_asset_index(&body)
}
