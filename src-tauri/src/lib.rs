pub mod commands;
pub mod events;
pub mod protocol;
pub mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            state::init_state(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_status,
            commands::login_password,
            commands::login_cookies,
            commands::login_web,
            commands::logout,
            commands::get_settings,
            commands::update_settings,
            commands::list_galleries,
            commands::gallery_detail,
            commands::gallery_thumbnails,
            commands::reload_page_image,
            commands::favorite_categories,
            commands::set_favorite,
            commands::batch_favorites,
            commands::enqueue_downloads,
            commands::list_downloads,
            commands::pause_download,
            commands::resume_download,
            commands::remove_download,
            commands::pause_all,
            commands::resume_all,
            commands::list_library,
            commands::rescan_library,
            commands::local_pages,
            commands::add_library_dir,
            commands::remove_local,
            commands::list_history,
            commands::record_progress,
            commands::clear_history,
            commands::open_path,
            commands::clear_image_cache,
        ]);

    let builder = protocol::register(builder);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
