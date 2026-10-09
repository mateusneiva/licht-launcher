use std::collections::BTreeMap;

use licht_core::{
    ASSET_OBJECT_BASE, Arch, CoreError, GameInstall, LaunchEnvironment, OsName, OutputStream,
    SharedCache, fetch_version_manifest, install_game, offline_account, parse_install_args,
    parse_launch_args, parse_version, prepare_offline_launch, run_game,
};
use tokio::sync::mpsc;

fn main() {
    let code = match runtime() {
        Ok(runtime) => match runtime.block_on(run()) {
            Ok(code) => code,
            Err(error) => {
                report(&error);
                1
            }
        },
        Err(error) => {
            report(&error);
            1
        }
    };
    std::process::exit(code);
}

async fn run() -> licht_core::Result<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("launch") => launch(&args).await,
        Some("install") => {
            install(&args).await?;
            Ok(0)
        }
        _ => Err(CoreError::LaunchArgs),
    }
}

fn report(error: &CoreError) {
    eprintln!("{error}");
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        eprintln!("  {cause}");
        source = cause.source();
    }
}

fn runtime() -> licht_core::Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?)
}

async fn install(args: &[String]) -> licht_core::Result<()> {
    let args = parse_install_args(args)?;
    let cache = match &args.cache {
        Some(root) => SharedCache::at(root),
        None => SharedCache::system()?,
    };
    let client = reqwest::Client::new();
    let manifest = fetch_version_manifest(&client).await?;
    let Some(entry) = manifest
        .versions
        .iter()
        .find(|version| version.id == args.version_id)
    else {
        return Err(CoreError::VersionMissing);
    };

    let (progress, mut incoming) = mpsc::channel(64);
    let version_id = args.version_id.clone();
    let version_url = entry.url.clone();
    let version_sha1 = entry.sha1.clone();
    let java = args.java.clone();
    let environment = host_environment()?;
    let installing = tokio::spawn(async move {
        install_game(
            &client,
            &cache,
            GameInstall {
                version_id: &version_id,
                version_json_url: &version_url,
                version_sha1: &version_sha1,
                environment: &environment,
                asset_base: ASSET_OBJECT_BASE,
                java: java.as_deref(),
            },
            progress,
        )
        .await
    });
    while let Some(progress) = incoming.recv().await {
        if progress.finished % 100 == 0 || progress.finished + progress.failed == progress.total {
            println!(
                "{}/{} failed {}",
                progress.finished, progress.total, progress.failed
            );
        }
    }
    let java = installing
        .await
        .map_err(|_| CoreError::Io(std::io::Error::other("install task failed")))??;
    println!("java {}", java.display());
    Ok(())
}

async fn launch(args: &[String]) -> licht_core::Result<i32> {
    let args = parse_launch_args(args)?;
    let cache = match &args.cache {
        Some(root) => SharedCache::at(root),
        None => SharedCache::system()?,
    };
    let json = std::fs::read_to_string(cache.version_json(&args.version_id)?)?;
    let version = parse_version(&json)?;
    let account = offline_account(&args.username)?;
    let command = prepare_offline_launch(
        &args.java,
        &cache,
        &args.version_id,
        &version,
        &host_environment()?,
        &account,
        &args.game_directory,
    )?;

    let (output, mut incoming) = mpsc::channel(32);
    let game_directory = args.game_directory.clone();
    let running =
        tokio::spawn(async move { run_game(&command, Some(&game_directory), output).await });
    while let Some(line) = incoming.recv().await {
        match line.stream {
            OutputStream::Stdout => println!("{}", line.line),
            OutputStream::Stderr => eprintln!("{}", line.line),
        }
    }
    let exit = running
        .await
        .map_err(|_| CoreError::Io(std::io::Error::other("launch task failed")))??;
    Ok(exit.code.unwrap_or(1))
}

fn host_environment() -> licht_core::Result<LaunchEnvironment> {
    let os = match std::env::consts::OS {
        "windows" => OsName::Windows,
        "linux" => OsName::Linux,
        "macos" => OsName::Osx,
        _ => return Err(CoreError::LaunchHost),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => Arch::X86_64,
        "x86" => Arch::X86,
        _ => return Err(CoreError::LaunchHost),
    };
    Ok(LaunchEnvironment {
        os,
        arch,
        os_version: String::new(),
        features: BTreeMap::new(),
    })
}
