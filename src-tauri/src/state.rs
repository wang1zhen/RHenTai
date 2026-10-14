use std::sync::Arc;

use rhentai_core::{AppPaths, Core};
use tauri::{App, Manager};

use crate::events::TauriSink;

pub fn init_state(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let paths = app.path();
    let data_dir = paths.app_data_dir()?;
    let config_dir = paths.app_config_dir()?;
    let cache_dir = paths.app_cache_dir()?;
    let download_dir = paths
        .download_dir()
        .unwrap_or_else(|_| data_dir.clone())
        .join("RHenTai");

    let app_paths = AppPaths::new(data_dir, config_dir, cache_dir, download_dir);
    let sink = Arc::new(TauriSink(app_handle.clone()));
    let core = Core::open(app_paths, sink)?;

    app.manage(core);
    Ok(())
}
