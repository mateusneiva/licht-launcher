use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use licht_core::{
    Arch, Artifact, AssetIndex, AssetIndexFile, AssetObject, CoreError, Download, DownloadProgress,
    GameArguments, InstallPlan, LaunchEnvironment, Library, LibraryDownloads, OsName, SharedCache,
    Version, VersionDownloads, install_version, offline_account, parse_launch_args,
    prepare_offline_launch,
};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};

const CLIENT: &[u8] = b"client-jar";
const LIBRARY: &[u8] = b"library-jar";
const ASSET: &[u8] = b"asset-bytes";

#[test]
fn steve_has_the_offline_uuid() {
    let steve = offline_account("Steve").expect("Steve");
    assert_eq!(steve.uuid, "5627dd98e6be3c21b8a8e92344183641");
    assert_eq!(steve.uuid.len(), 32);
    assert_eq!(steve.uuid.chars().nth(12), Some('3'));
    assert_eq!(offline_account("Steve").expect("again").uuid, steve.uuid);
    assert_ne!(offline_account("Alex").expect("Alex").uuid, steve.uuid);
    assert!(matches!(offline_account(""), Err(CoreError::OfflineName)));
}

#[test]
fn a_missing_launch_flag_fails_before_any_process() {
    let error = parse_launch_args(&[
        "launch".to_string(),
        "--version".to_string(),
        "demo".to_string(),
        "--username".to_string(),
        "Steve".to_string(),
    ])
    .expect_err("java and game dir are required");
    assert!(matches!(error, CoreError::LaunchArgs));
}

#[tokio::test]
async fn an_installed_version_launches_offline_with_its_client_jar() {
    let directory = std::env::temp_dir().join(format!("licht-offline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let cache = SharedCache::at(&directory);
    let base = serve().await;
    let asset_hash = sha1_hex(ASSET);
    let index_json = index_json(&asset_hash);
    let version = version(&base, &sha1_hex(index_json.as_bytes()));
    let assets = assets(&asset_hash);
    let environment = environment();

    let (progress, mut incoming) = mpsc::channel::<DownloadProgress>(8);
    install_version(
        &reqwest::Client::new(),
        &cache,
        InstallPlan {
            version_id: "demo",
            version: &version,
            asset_index_json: &index_json,
            assets: &assets,
            environment: &environment,
            asset_base: &base,
        },
        progress,
    )
    .await
    .expect("install");
    while incoming.recv().await.is_some() {}

    let jar = cache.version_jar("demo").expect("jar path");
    assert_eq!(std::fs::read(&jar).expect("installed jar"), CLIENT);

    let account = offline_account("Steve").expect("account");
    let command = prepare_offline_launch(
        PathBuf::from("java").as_path(),
        &cache,
        "demo",
        &version,
        &environment,
        &account,
        &directory.join("game"),
    )
    .expect("command");

    let jar_text = jar.display().to_string();
    assert!(command.iter().any(|part| part.contains(&jar_text)));
    assert!(command.iter().any(|part| part == "--username"));
    assert!(command.iter().any(|part| part == "Steve"));
    assert!(command.iter().any(|part| part == &account.uuid));
    assert!(
        command
            .iter()
            .all(|part| !part.contains("${auth_player_name}"))
    );
    assert_eq!(std::fs::read(&jar).expect("jar still there"), CLIENT);

    let _ = std::fs::remove_dir_all(directory);
}

fn version(base: &str, index_sha1: &str) -> Version {
    Version {
        arguments: GameArguments::Legacy(
            "--username ${auth_player_name} --uuid ${auth_uuid}".to_string(),
        ),
        libraries: vec![library(
            &format!("{base}/library"),
            "com/example/demo/1.0/demo-1.0.jar",
            &sha1_hex(LIBRARY),
        )],
        asset_index: AssetIndex {
            id: "test".to_string(),
            sha1: index_sha1.to_string(),
            size: 1,
            total_size: 1,
            url: format!("{base}/index"),
        },
        main_class: "a.B".to_string(),
        downloads: VersionDownloads {
            client: Download {
                sha1: sha1_hex(CLIENT),
                size: CLIENT.len() as u64,
                url: format!("{base}/client"),
            },
            server: None,
        },
        java_version: None,
    }
}

fn library(url: &str, path: &str, sha1: &str) -> Library {
    Library {
        name: "com.example:demo:1.0".to_string(),
        downloads: Some(LibraryDownloads {
            artifact: Some(Artifact {
                path: path.to_string(),
                sha1: sha1.to_string(),
                size: 1,
                url: url.to_string(),
            }),
            classifiers: None,
        }),
        natives: None,
        rules: None,
        extract: None,
    }
}

fn assets(hash: &str) -> AssetIndexFile {
    AssetIndexFile {
        objects: BTreeMap::from([(
            "icons/icon.png".to_string(),
            AssetObject {
                hash: hash.to_string(),
                size: ASSET.len() as u64,
            },
        )]),
        virtual_assets: false,
        map_to_resources: false,
    }
}

fn index_json(asset_hash: &str) -> String {
    format!(
        r#"{{"objects":{{"icons/icon.png":{{"hash":"{asset_hash}","size":{}}}}}}}"#,
        ASSET.len()
    )
}

fn environment() -> LaunchEnvironment {
    LaunchEnvironment {
        os: OsName::Linux,
        arch: Arch::X86_64,
        os_version: String::new(),
        features: BTreeMap::new(),
    }
}

fn sha1_hex(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

async fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("the test server should bind");
    let address = listener
        .local_addr()
        .expect("the test server should have an address");
    let files = Arc::new(Mutex::new(BTreeMap::from([
        ("/client".to_string(), CLIENT.to_vec()),
        ("/library".to_string(), LIBRARY.to_vec()),
    ])));

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let files = Arc::clone(&files);
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
                let path = request_path(&incoming);
                let body = {
                    let mut files = files.lock().await;
                    if let Some(hash) = path.strip_prefix('/')
                        && let Some((prefix, hash)) = hash.split_once('/')
                        && hash.starts_with(prefix)
                    {
                        files.entry(path.clone()).or_insert_with(|| ASSET.to_vec());
                    }
                    files.get(&path).cloned()
                };
                let payload = body.unwrap_or_default();
                let status = if payload.is_empty() { 500 } else { 200 };
                let header = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    payload.len()
                );
                if socket.write_all(header.as_bytes()).await.is_err() {
                    return;
                }
                let _ = socket.write_all(&payload).await;
            });
        }
    });

    format!("http://{address}")
}

fn request_path(request: &[u8]) -> String {
    let text = String::from_utf8_lossy(request);
    text.split_whitespace().nth(1).unwrap_or("/").to_string()
}
