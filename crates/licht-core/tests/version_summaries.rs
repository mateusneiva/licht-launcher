use licht_core::{SharedCache, parse_version_manifest, version_summaries};

fn manifest() -> licht_core::VersionManifest {
    parse_version_manifest(
        r#"{
            "latest": {"release": "1.2", "snapshot": "s"},
            "versions": [
                {
                    "id": "1.2",
                    "type": "release",
                    "url": "https://example.invalid/1.2.json",
                    "time": "t",
                    "releaseTime": "t",
                    "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "complianceLevel": 0
                },
                {
                    "id": "bad/id",
                    "type": "snapshot",
                    "url": "https://example.invalid/bad.json",
                    "time": "t",
                    "releaseTime": "t",
                    "sha1": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "complianceLevel": 0
                }
            ]
        }"#,
    )
    .expect("manifest")
}

#[test]
fn a_saved_version_json_is_installed_and_a_bad_id_is_not() {
    let root = std::env::temp_dir().join(format!("licht-summaries-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let cache = SharedCache::at(&root);
    let json = cache.version_json("1.2").expect("version path");
    std::fs::create_dir_all(json.parent().expect("parent")).expect("version dir");
    std::fs::write(&json, "{}").expect("version json");

    let summaries = version_summaries(&manifest(), &cache);
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].id, "1.2");
    assert!(summaries[0].installed);
    assert_eq!(summaries[1].id, "bad/id");
    assert!(!summaries[1].installed);

    let _ = std::fs::remove_dir_all(root);
}
