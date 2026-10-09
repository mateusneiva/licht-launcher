use std::collections::BTreeMap;

use licht_core::{
    Arch, CoreError, LaunchEnvironment, OsName, OutputStream, SharedCache, offline_account,
    parse_launch_args, parse_version, prepare_offline_launch, run_game,
};
use tokio::sync::mpsc;

fn main() {
    let code = match runtime() {
        Ok(runtime) => match runtime.block_on(launch()) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("{error}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    std::process::exit(code);
}

fn runtime() -> licht_core::Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?)
}

async fn launch() -> licht_core::Result<i32> {
    let args = parse_launch_args(&std::env::args().skip(1).collect::<Vec<_>>())?;
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
