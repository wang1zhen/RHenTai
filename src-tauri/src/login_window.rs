#![cfg(desktop)]

use std::collections::HashSet;
use std::time::Duration;

use eh_client::model::RawCookie;
use rhentai_core::{Core, IpcError};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

pub async fn login_web(app: &AppHandle, core: &Core) -> Result<(), IpcError> {
    if let Some(win) = app.get_webview_window("login") {
        let _ = win.set_focus();
        return Ok(());
    }

    let url: Url = "https://forums.e-hentai.org/index.php?act=Login&CODE=00"
        .parse()
        .map_err(|e: url::ParseError| IpcError::internal(e.to_string()))?;

    let _win = WebviewWindowBuilder::new(app, "login", WebviewUrl::External(url))
        .title("E-Hentai Login")
        .inner_size(900.0, 720.0)
        .build()
        .map_err(|e| IpcError::internal(e.to_string()))?;

    let app_handle = app.clone();
    let core_clone = core.clone();

    tauri::async_runtime::spawn(async move {
        poll_login_flow(app_handle, core_clone).await;
    });

    Ok(())
}

async fn poll_login_flow(app: AppHandle, core: Core) {
    let forums_url = match Url::parse("https://forums.e-hentai.org/") {
        Ok(u) => u,
        Err(_) => return,
    };
    let eh_url = match Url::parse("https://e-hentai.org/") {
        Ok(u) => u,
        Err(_) => return,
    };
    let ex_url = match Url::parse("https://exhentai.org/") {
        Ok(u) => u,
        Err(_) => return,
    };

    // Phase 1: Poll cookies every second until `ipb_member_id` and `ipb_pass_hash` are present.
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let Some(win) = app.get_webview_window("login") else {
            // User closed the window
            return;
        };

        let mut has_member_id = false;
        let mut has_pass_hash = false;

        let forums_cookies = win.cookies_for_url(forums_url.clone()).unwrap_or_default();
        let eh_cookies = win.cookies_for_url(eh_url.clone()).unwrap_or_default();

        for c in forums_cookies.iter().chain(eh_cookies.iter()) {
            if c.name() == "ipb_member_id" && !c.value().is_empty() && c.value() != "-1" {
                has_member_id = true;
            } else if c.name() == "ipb_pass_hash" && !c.value().is_empty() {
                has_pass_hash = true;
            }
        }

        if has_member_id && has_pass_hash {
            break;
        }
    }

    // Phase 2: Navigate window to https://exhentai.org/ and wait up to 20 s for igneous.
    let Some(win) = app.get_webview_window("login") else {
        return;
    };

    if win.navigate(ex_url.clone()).is_err() {
        return;
    }

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let Some(win) = app.get_webview_window("login") else {
            return;
        };

        let ex_cookies = win.cookies_for_url(ex_url.clone()).unwrap_or_default();
        let has_igneous = ex_cookies
            .iter()
            .any(|c| c.name() == "igneous" && !c.value().is_empty() && c.value() != "mystery");

        if has_igneous {
            break;
        }
    }

    // Phase 3: Import cookies into core.client, verify, emit auth-changed, and close window.
    let Some(win) = app.get_webview_window("login") else {
        return;
    };

    let mut raw_cookies: Vec<RawCookie> = Vec::new();
    let mut seen = HashSet::new();

    let check_urls = [
        (&forums_url, "forums.e-hentai.org"),
        (&eh_url, "e-hentai.org"),
        (&ex_url, "exhentai.org"),
    ];

    for (url, default_domain) in check_urls {
        if let Ok(cookies) = win.cookies_for_url(url.clone()) {
            for c in cookies {
                let domain = c
                    .domain()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| default_domain.to_string());
                let path = c
                    .path()
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "/".to_string());
                let key = (c.name().to_string(), domain.clone(), path.clone());
                if seen.insert(key) {
                    raw_cookies.push(RawCookie {
                        name: c.name().to_string(),
                        value: c.value().to_string(),
                        domain,
                        path,
                    });
                }
            }
        }
    }

    if let Ok(all_cookies) = win.cookies() {
        for c in all_cookies {
            let domain = c.domain().map(|d| d.to_string()).unwrap_or_default();
            if domain.contains("e-hentai.org") || domain.contains("exhentai.org") {
                let path = c
                    .path()
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "/".to_string());
                let key = (c.name().to_string(), domain.clone(), path.clone());
                if seen.insert(key) {
                    raw_cookies.push(RawCookie {
                        name: c.name().to_string(),
                        value: c.value().to_string(),
                        domain,
                        path,
                    });
                }
            }
        }
    }

    if let Err(e) = core.client.import_cookies(&raw_cookies) {
        tracing::warn!("failed to import cookies from login window: {e}");
    }

    match core.client.auth_status(true).await {
        Ok(status) => {
            let _ = app.emit("auth-changed", status);
        }
        Err(e) => {
            tracing::warn!("failed to verify auth status online: {e}");
            if let Ok(status) = core.client.auth_status(false).await {
                let _ = app.emit("auth-changed", status);
            }
        }
    }

    let _ = win.close();
}
