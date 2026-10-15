pub mod commands;
pub mod events;
#[cfg(desktop)]
pub mod login_window;
pub mod protocol;
pub mod state;

fn setup_main_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window_config = app
        .config()
        .app
        .windows
        .first()
        .ok_or("No window config found in tauri.conf.json")?;

    let mut builder = tauri::WebviewWindowBuilder::from_config(app, window_config)?;

    if let Ok(route) = std::env::var("RHENTAI_START_ROUTE")
        && !route.is_empty()
    {
        let r = route.strip_prefix('#').unwrap_or(&route);
        let r_json = serde_json::to_string(r).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            r##"(function() {{
                function setRoute() {{
                    if (!location.hash || location.hash === "" || location.hash === "#" || location.hash === "#/") {{
                        location.hash = "#" + {r_json};
                    }}
                }}
                setRoute();
                if (document.readyState === 'loading') {{
                    window.addEventListener('DOMContentLoaded', setRoute);
                }}
            }})();"##
        );
        builder = builder.initialization_script(&script);
    }

    #[cfg(debug_assertions)]
    if let Ok(qa_eval) = std::env::var("RHENTAI_QA_EVAL")
        && !qa_eval.is_empty()
    {
        let script = format!(
            r#"if (document.readyState === 'loading') {{
                window.addEventListener('DOMContentLoaded', () => {{
                    {qa_eval}
                }});
            }} else {{
                {qa_eval}
            }}"#
        );
        builder = builder.initialization_script(&script);
    }

    builder.build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .finish(),
    );

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            state::init_state(app)?;
            setup_main_window(app)?;
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
