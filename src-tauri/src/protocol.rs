use base64::Engine;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use eh_client::ErrorKind;
use percent_encoding::percent_decode_str;
use rhentai_core::{Core, IpcError};
use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

fn resp_200(bytes: Vec<u8>, content_type: &str, is_thumb: bool) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*");
    if is_thumb {
        builder = builder.header("Cache-Control", "max-age=31536000");
    }
    builder.body(bytes).unwrap()
}

fn resp_404() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .header("Access-Control-Allow-Origin", "*")
        .body(b"Not Found".to_vec())
        .unwrap()
}

fn resp_502(err: &IpcError) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .header("Content-Type", "text/plain")
        .header("Access-Control-Allow-Origin", "*")
        .body(format!("{}: {}", err.kind.as_str(), err.message).into_bytes())
        .unwrap()
}

async fn handle_rhimg_request(
    app: AppHandle,
    request: Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let Some(core) = app.try_state::<Core>() else {
        responder.respond(resp_502(&IpcError::internal("Core state not initialized")));
        return;
    };
    let core = core.inner().clone();

    let uri = request.uri();
    if let Some(host) = uri.host()
        && host != "localhost"
        && host != "rhimg.localhost"
        && !host.is_empty()
    {
        responder.respond(resp_404());
        return;
    }

    let raw_path = uri.path();
    let mut path = raw_path;
    if let Some(rest) = path.strip_prefix("//localhost/") {
        path = rest;
    } else if let Some(rest) = path.strip_prefix("//rhimg.localhost/") {
        path = rest;
    } else {
        path = path.strip_prefix('/').unwrap_or(path);
    }

    // Ignore ?v= or any query component if it was embedded in path
    let clean_path = path.split('?').next().unwrap_or(path);

    let decoded = match percent_decode_str(clean_path).decode_utf8() {
        Ok(d) => d,
        Err(_) => {
            responder.respond(resp_404());
            return;
        }
    };

    let parts: Vec<&str> = decoded.split('/').collect();
    if parts.is_empty() {
        responder.respond(resp_404());
        return;
    }

    log::debug!("rhimg request: {}", uri);

    match parts[0] {
        "thumb" => {
            if parts.len() != 2 || parts[1].is_empty() {
                responder.respond(resp_404());
                return;
            }

            let b64 = parts[1].trim_end_matches('=');
            let decoded_bytes = match URL_SAFE_NO_PAD
                .decode(b64)
                .or_else(|_| URL_SAFE.decode(b64))
            {
                Ok(b) => b,
                Err(_) => {
                    responder.respond(resp_404());
                    return;
                }
            };

            let remote_url = match String::from_utf8(decoded_bytes) {
                Ok(u) => u,
                Err(_) => {
                    responder.respond(resp_404());
                    return;
                }
            };

            match core.images.thumb(&remote_url).await {
                Ok((bytes, content_type)) => {
                    responder.respond(resp_200(bytes.to_vec(), &content_type, true));
                }
                Err(err) => {
                    if err.kind == ErrorKind::NotFound {
                        responder.respond(resp_404());
                    } else {
                        responder.respond(resp_502(&err));
                    }
                }
            }
        }
        "page" => {
            if parts.len() != 4 {
                log::debug!("rhimg page: parts.len != 4: {:?}", parts);
                responder.respond(resp_404());
                return;
            }

            let Ok(gid) = parts[1].parse::<u64>() else {
                log::debug!("rhimg page: gid parse failed: {}", parts[1]);
                responder.respond(resp_404());
                return;
            };

            let token = parts[2];
            if token.is_empty() {
                log::debug!("rhimg page: empty token");
                responder.respond(resp_404());
                return;
            }

            let Ok(index) = parts[3].parse::<u32>() else {
                responder.respond(resp_404());
                return;
            };

            match core.images.page(gid, token, index).await {
                Ok((bytes, content_type)) => {
                    responder.respond(resp_200(bytes.to_vec(), &content_type, false));
                }
                Err(err) => {
                    if err.kind == ErrorKind::NotFound {
                        responder.respond(resp_404());
                    } else {
                        responder.respond(resp_502(&err));
                    }
                }
            }
        }
        "local" => {
            if parts.len() != 3 {
                responder.respond(resp_404());
                return;
            }

            let id = parts[1];
            if id.is_empty() {
                responder.respond(resp_404());
                return;
            }

            let Ok(index) = parts[2].parse::<u32>() else {
                responder.respond(resp_404());
                return;
            };

            let id_owned = id.to_string();
            let res = tokio::task::spawn_blocking(move || core.library.page(&id_owned, index))
                .await
                .map_err(|e| IpcError::internal(e.to_string()))
                .and_then(|r| r);

            match res {
                Ok((bytes, content_type)) => {
                    responder.respond(resp_200(bytes.to_vec(), &content_type, false));
                }
                Err(err) => {
                    if err.kind == ErrorKind::NotFound {
                        responder.respond(resp_404());
                    } else {
                        responder.respond(resp_502(&err));
                    }
                }
            }
        }
        _ => {
            responder.respond(resp_404());
        }
    }
}

pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.register_asynchronous_uri_scheme_protocol("rhimg", |ctx, request, responder| {
        let app_handle = ctx.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            handle_rhimg_request(app_handle, request, responder).await;
        });
    })
}
