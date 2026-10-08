use anyhow::Context;

mod logging;

pub fn run() -> anyhow::Result<()> {
    logging::init()?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting Licht Launcher"
    );

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .context("Failed to run Licht Launcher")
}
