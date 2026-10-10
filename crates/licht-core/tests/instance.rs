use std::fs;
use std::path::{Path, PathBuf};

use licht_core::{CoreError, list_instances, load_instance, parse_instance};

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("licht-instance-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("scratch directory");
    root
}

fn write(dir: &Path, name: &str, body: &str) {
    fs::create_dir_all(dir).expect("directory");
    fs::write(dir.join(name), body).expect("file");
}

#[test]
fn a_folder_without_a_file_gains_schema_one() {
    let root = scratch("migrate");
    let dir = root.join("instances").join("1.20.1");
    fs::create_dir_all(&dir).expect("instance folder");

    let instance = load_instance(&dir).expect("migration");
    assert_eq!(instance.schema, 1);
    assert_eq!(instance.name, "1.20.1");
    assert_eq!(instance.version_id, "1.20.1");
    assert_eq!(instance.min_memory_mb, 512);
    assert_eq!(instance.max_memory_mb, 2048);
    assert!(instance.jvm_arguments.is_empty());
    assert_eq!(instance.width, None);
    assert_eq!(instance.height, None);

    let json = fs::read_to_string(dir.join("instance.json")).expect("written file");
    let again = load_instance(&dir).expect("second read");
    assert_eq!(again, instance);
    assert_eq!(
        fs::read_to_string(dir.join("instance.json")).expect("unchanged file"),
        json
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn schema_one_is_read_back_unchanged() {
    let root = scratch("read");
    let dir = root.join("Survival");
    write(
        &dir,
        "instance.json",
        r#"{
  "schema": 1,
  "name": "Survival",
  "versionId": "1.20.1",
  "minMemoryMb": 1024,
  "maxMemoryMb": 4096,
  "jvmArguments": ["-XX:+UseG1GC"],
  "width": 1280,
  "height": 720
}"#,
    );
    let before = fs::read_to_string(dir.join("instance.json")).expect("original");
    let instance = load_instance(&dir).expect("read");
    assert_eq!(instance.name, "Survival");
    assert_eq!(instance.version_id, "1.20.1");
    assert_eq!(instance.min_memory_mb, 1024);
    assert_eq!(instance.max_memory_mb, 4096);
    assert_eq!(instance.jvm_arguments, vec!["-XX:+UseG1GC".to_string()]);
    assert_eq!(instance.width, Some(1280));
    assert_eq!(instance.height, Some(720));
    assert_eq!(
        fs::read_to_string(dir.join("instance.json")).expect("untouched"),
        before
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_broken_file_and_a_newer_schema_are_left_in_place() {
    let root = scratch("reject");
    let broken = root.join("broken");
    write(&broken, "instance.json", "{ not json");
    let broken_body = fs::read_to_string(broken.join("instance.json")).expect("broken");
    assert!(matches!(
        load_instance(&broken),
        Err(CoreError::Instance(_))
    ));
    assert_eq!(
        fs::read_to_string(broken.join("instance.json")).expect("still broken"),
        broken_body
    );

    let future = root.join("future");
    write(
        &future,
        "instance.json",
        r#"{"schema": 2, "name": "Later"}"#,
    );
    let future_body = fs::read_to_string(future.join("instance.json")).expect("future");
    assert!(matches!(
        load_instance(&future),
        Err(CoreError::InstanceSchema { schema: 2 })
    ));
    assert_eq!(
        fs::read_to_string(future.join("instance.json")).expect("still future"),
        future_body
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn memory_must_be_a_positive_range() {
    let json = r#"{
  "schema": 1,
  "name": "1.20.1",
  "versionId": "1.20.1",
  "minMemoryMb": 2048,
  "maxMemoryMb": 512,
  "jvmArguments": [],
  "width": null,
  "height": null
}"#;
    assert!(matches!(
        parse_instance(json),
        Err(CoreError::InstanceMemory)
    ));
}

#[test]
fn a_minecraft_child_is_not_its_own_instance() {
    let root = scratch("list");
    let instances = root.join("instances");
    let legacy = instances.join("1.5.2");
    fs::create_dir_all(legacy.join(".minecraft").join("saves")).expect("legacy game dir");
    fs::create_dir_all(instances.join("1.20.1")).expect("modern instance");

    let listed = list_instances(&instances).expect("list");
    let ids: Vec<&str> = listed
        .iter()
        .map(|instance| instance.version_id.as_str())
        .collect();
    assert_eq!(ids, vec!["1.20.1", "1.5.2"]);
    assert!(legacy.join("instance.json").is_file());
    assert!(!legacy.join(".minecraft").join("instance.json").exists());
    assert!(
        list_instances(&root.join("missing"))
            .expect("absent")
            .is_empty()
    );
    let _ = fs::remove_dir_all(root);
}
