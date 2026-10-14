use percent_encoding::percent_decode_str;
use tauri::http::{Response, StatusCode};

pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.register_asynchronous_uri_scheme_protocol("rhimg", |_ctx, request, responder| {
        let uri = request.uri();
        let path = uri.path();
        let stripped = path.strip_prefix('/').unwrap_or(path);
        let decoded = match percent_decode_str(stripped).decode_utf8() {
            Ok(d) => d,
            Err(_) => {
                responder.respond(
                    Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(b"Invalid UTF-8 in URI path".to_vec())
                        .unwrap(),
                );
                return;
            }
        };

        let parts: Vec<&str> = decoded.split('/').collect();
        if parts.is_empty() {
            responder.respond(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(b"Empty path".to_vec())
                    .unwrap(),
            );
            return;
        }

        match parts[0] {
            "thumb" => {
                if parts.len() == 2 && !parts[1].is_empty() {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .header("Content-Type", "text/plain")
                            .body(b"upstream: thumbnail not yet loaded".to_vec())
                            .unwrap(),
                    );
                } else {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(b"Not Found".to_vec())
                            .unwrap(),
                    );
                }
            }
            "page" => {
                if parts.len() == 4
                    && parts[1].parse::<u64>().is_ok()
                    && !parts[2].is_empty()
                    && parts[3].parse::<u32>().is_ok()
                {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .header("Content-Type", "text/plain")
                            .body(b"upstream: page image not yet loaded".to_vec())
                            .unwrap(),
                    );
                } else {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(b"Not Found".to_vec())
                            .unwrap(),
                    );
                }
            }
            "local" => {
                if parts.len() == 3 && !parts[1].is_empty() && parts[2].parse::<u32>().is_ok() {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "text/plain")
                            .body(b"local page not found".to_vec())
                            .unwrap(),
                    );
                } else {
                    responder.respond(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(b"Not Found".to_vec())
                            .unwrap(),
                    );
                }
            }
            _ => {
                responder.respond(
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(b"Route not found".to_vec())
                        .unwrap(),
                );
            }
        }
    })
}
