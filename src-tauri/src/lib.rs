use anyhow::Context;

mod commands;
mod logging;
mod progress;

pub fn run() -> anyhow::Result<()> {
    logging::init()?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Licht Launcher"
    );

    tauri::Builder::default()
        .manage(commands::OperationLock::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_instances,
            commands::create_instance,
            commands::rename_instance,
            commands::duplicate_instance,
            commands::delete_instance,
            commands::get_settings,
            commands::set_download_concurrency,
            commands::list_versions,
            commands::install_version,
            commands::launch
        ])
        .run(tauri::generate_context!())
        .context("Failed to run Licht Launcher")
}
