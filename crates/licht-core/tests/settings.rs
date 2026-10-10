use std::path::PathBuf;

use licht_core::{CoreError, SharedCache, load_settings, save_settings};

fn cache(name: &str) -> SharedCache {
    let root = std::env::temp_dir().join(format!("licht-settings-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    SharedCache::at(root)
}

fn settings_file(cache: &SharedCache) -> PathBuf {
    cache.root().join("settings.json")
}

#[test]
fn a_missing_file_returns_eight_and_is_not_created() {
    let cache = cache("missing");

    let settings = load_settings(&cache).expect("default");

    assert_eq!(settings.schema, 1);
    assert_eq!(settings.download_concurrency, 8);
    assert!(!cache.root().exists());
}

#[test]
fn a_saved_value_roundtrips() {
    let cache = cache("roundtrip");

    let saved = save_settings(&cache, 4).expect("save");
    let loaded = load_settings(&cache).expect("load");

    assert_eq!(saved.schema, 1);
    assert_eq!(saved.download_concurrency, 4);
    assert_eq!(loaded, saved);
    assert!(settings_file(&cache).is_file());
}

#[test]
fn values_outside_one_to_sixteen_are_rejected_without_writing() {
    let cache = cache("range");
    let path = settings_file(&cache);

    let low = save_settings(&cache, 0).expect_err("zero");
    assert!(matches!(low, CoreError::SettingsConcurrency));
    assert!(!path.exists());

    save_settings(&cache, 8).expect("valid");
    let before = std::fs::read_to_string(&path).expect("file");
    let high = save_settings(&cache, 17).expect_err("seventeen");
    assert!(matches!(high, CoreError::SettingsConcurrency));
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), before);
}

#[test]
fn an_unknown_schema_is_left_in_place() {
    let cache = cache("schema");
    let path = settings_file(&cache);
    std::fs::create_dir_all(cache.root()).expect("root");
    let original = "{\"schema\":2,\"downloadConcurrency\":8}\n";
    std::fs::write(&path, original).expect("write");

    let loaded = load_settings(&cache).expect_err("schema");
    let saved = save_settings(&cache, 4).expect_err("save");

    assert!(matches!(loaded, CoreError::SettingsSchema { schema: 2 }));
    assert!(matches!(saved, CoreError::SettingsSchema { schema: 2 }));
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), original);
}

#[test]
fn invalid_json_is_left_in_place() {
    let cache = cache("invalid");
    let path = settings_file(&cache);
    std::fs::create_dir_all(cache.root()).expect("root");
    let original = "{ not json";
    std::fs::write(&path, original).expect("write");

    let loaded = load_settings(&cache).expect_err("json");
    let saved = save_settings(&cache, 4).expect_err("save");

    assert!(matches!(loaded, CoreError::Settings(_)));
    assert!(matches!(saved, CoreError::Settings(_)));
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), original);
}
