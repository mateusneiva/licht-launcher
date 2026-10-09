use licht_core::{parse_version, required_runtime};

#[test]
fn saved_versions_name_their_runtime() {
    for (json, component, major) in [
        (include_str!("fixtures/version-1.8.9.json"), "jre-legacy", 8),
        (
            include_str!("fixtures/version-1.12.2.json"),
            "jre-legacy",
            8,
        ),
        (
            include_str!("fixtures/version-1.20.6.json"),
            "java-runtime-delta",
            21,
        ),
        (
            include_str!("fixtures/version-1.21.11.json"),
            "java-runtime-delta",
            21,
        ),
    ] {
        let version = parse_version(json).expect("the saved version should parse");
        let runtime = required_runtime(&version).expect("the saved version names a runtime");
        assert_eq!(runtime.component, component);
        assert_eq!(runtime.major_version, major);
    }
}

#[test]
fn a_version_without_java_version_has_no_required_runtime() {
    let version = parse_version(
        r#"{
            "minecraftArguments": "--username",
            "libraries": [],
            "assetIndex": {
                "id": "1.8",
                "sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "size": 1,
                "totalSize": 2,
                "url": "https://example.invalid/index.json"
            },
            "mainClass": "net.minecraft.client.main.Main",
            "downloads": {
                "client": {
                    "sha1": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "size": 3,
                    "url": "https://example.invalid/client.jar"
                }
            }
        }"#,
    )
    .expect("a legacy version without javaVersion should parse");

    assert!(required_runtime(&version).is_none());
}
