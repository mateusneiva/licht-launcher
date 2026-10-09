use std::sync::atomic::{AtomicUsize, Ordering};

use licht_core::{
    Arch, CoreError, JavaChoice, JavaRuntimeEntry, OsName, SharedCache, install_java,
    parse_java_runtime_index, parse_java_runtime_manifest, parse_version, required_runtime,
    select_runtime,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

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

#[test]
fn the_saved_runtime_index_points_at_java_21_and_java_8() {
    let index = parse_java_runtime_index(include_str!("fixtures/java-runtime-index.json"))
        .expect("the saved runtime index should parse");

    let modern = select_runtime(&index, OsName::Windows, Arch::X86_64, "java-runtime-delta")
        .expect("Windows x64 should have Java 21");
    assert_eq!(modern.sha1, "cb4394a27089d19f65d5baa6cf0482c27c3c7865");

    let legacy = select_runtime(&index, OsName::Linux, Arch::X86, "jre-legacy")
        .expect("32-bit Linux should have the legacy runtime");
    assert_eq!(legacy.sha1.len(), 40);
    assert!(matches!(
        select_runtime(&index, OsName::Osx, Arch::X86, "jre-legacy"),
        Err(CoreError::JavaRuntimeMissing)
    ));
}

#[test]
fn the_saved_runtime_manifest_lists_java_exe() {
    let manifest = parse_java_runtime_manifest(include_str!("fixtures/java-runtime-delta.json"))
        .expect("the saved runtime manifest should parse");
    assert!(matches!(
        manifest.files.get("bin"),
        Some(JavaRuntimeEntry::Directory)
    ));
    assert!(matches!(
        manifest.files.get("bin/java.exe"),
        Some(JavaRuntimeEntry::File {
            executable: true,
            ..
        })
    ));
}

#[tokio::test]
async fn a_custom_java_is_used_without_downloading() {
    let directory = std::env::temp_dir().join(format!("licht-java-custom-{}", std::process::id()));
    let executable = directory.join("java.exe");
    std::fs::create_dir_all(&directory).expect("temp dir");
    std::fs::write(&executable, b"custom").expect("java");
    let cache = SharedCache::at(directory.join("cache"));
    let (progress, _incoming) = mpsc::channel(1);

    let installed = install_java(
        &reqwest::Client::new(),
        &cache,
        JavaChoice::Custom(&executable),
        progress,
    )
    .await
    .expect("the custom java should be accepted");

    assert_eq!(installed, executable);
    let missing_path = directory.join("missing.exe");
    let missing = install_java(
        &reqwest::Client::new(),
        &cache,
        JavaChoice::Custom(&missing_path),
        mpsc::channel(1).0,
    )
    .await
    .expect_err("a missing java should fail");
    assert!(matches!(missing, CoreError::JavaPath));
    let _ = std::fs::remove_dir_all(directory);
}

#[tokio::test]
async fn the_mojang_runtime_is_installed_once() {
    let directory = std::env::temp_dir().join(format!("licht-java-mojang-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let cache = SharedCache::at(&directory);
    let server = serve().await;
    let manifest = parse_java_runtime_manifest(&manifest_json(&server.base))
        .expect("the synthetic manifest should parse");
    let (progress, mut incoming) = mpsc::channel(4);

    let installed = install_java(
        &reqwest::Client::new(),
        &cache,
        JavaChoice::Mojang {
            component: "java-runtime-delta",
            os: OsName::Windows,
            arch: Arch::X86_64,
            manifest: &manifest,
        },
        progress,
    )
    .await
    .expect("the runtime should install");

    let java = cache
        .runtime_dir("java-runtime-delta", "windows-x64")
        .expect("runtime dir")
        .join("bin")
        .join("java.exe");
    assert_eq!(installed, java);
    assert_eq!(std::fs::read(&java).expect("java.exe"), b"hello world");
    assert_eq!(
        std::fs::read(java.parent().expect("bin").join("java")).expect("link"),
        b"hello world"
    );
    while incoming.recv().await.is_some() {}
    let requests = server.requests.load(Ordering::SeqCst);
    assert!(requests >= 1);

    let (progress, _incoming) = mpsc::channel(4);
    install_java(
        &reqwest::Client::new(),
        &cache,
        JavaChoice::Mojang {
            component: "java-runtime-delta",
            os: OsName::Windows,
            arch: Arch::X86_64,
            manifest: &manifest,
        },
        progress,
    )
    .await
    .expect("the second install should skip the download");
    assert_eq!(server.requests.load(Ordering::SeqCst), requests);
    let _ = std::fs::remove_dir_all(directory);
}

fn manifest_json(base: &str) -> String {
    format!(
        r#"{{
            "files": {{
                "bin": {{ "type": "directory" }},
                "bin/java.exe": {{
                    "type": "file",
                    "executable": true,
                    "downloads": {{
                        "raw": {{
                            "sha1": "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed",
                            "size": 11,
                            "url": "{base}/java.exe"
                        }}
                    }}
                }},
                "bin/java": {{ "type": "link", "target": "java.exe" }}
            }}
        }}"#
    )
}

struct Server {
    base: String,
    requests: std::sync::Arc<AtomicUsize>,
}

async fn serve() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("the test server should bind");
    let address = listener
        .local_addr()
        .expect("the test server should have an address");
    let requests = std::sync::Arc::new(AtomicUsize::new(0));
    let counted = std::sync::Arc::clone(&requests);
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let counted = std::sync::Arc::clone(&counted);
            tokio::spawn(async move {
                let mut incoming = Vec::new();
                let mut buffer = [0_u8; 256];
                loop {
                    let Ok(read) = socket.read(&mut buffer).await else {
                        return;
                    };
                    if read == 0 {
                        return;
                    }
                    incoming.extend_from_slice(&buffer[..read]);
                    if incoming.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                counted.fetch_add(1, Ordering::SeqCst);
                let body = b"hello world";
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                if socket.write_all(header.as_bytes()).await.is_err() {
                    return;
                }
                let _ = socket.write_all(body).await;
            });
        }
    });
    Server {
        base: format!("http://{address}"),
        requests,
    }
}
