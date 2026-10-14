use std::path::PathBuf;

use eh_client::GalleryKey;
use rhentai_core::{Core, EnqueueRequest, IpcError, LocalId, PathRequest};

fn parse_gallery_key(gallery: &str) -> Result<GalleryKey, IpcError> {
    let s = gallery.trim_end_matches('/');
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() >= 2 {
        let token = parts[parts.len() - 1];
        let gid_str = parts[parts.len() - 2];
        if let Ok(gid) = gid_str.parse::<u64>() {
            return Ok(GalleryKey {
                gid,
                token: token.to_string(),
            });
        }
    }
    Err(IpcError::invalid_input(format!(
        "Invalid gallery specifier: '{gallery}'. Expected full URL or '<gid>/<token>'"
    )))
}

pub async fn download(
    core: &Core,
    galleries: Vec<String>,
    original: bool,
    _concurrency: Option<u8>,
    _dir: Option<PathBuf>,
    json: bool,
) -> Result<(), IpcError> {
    let mut keys = Vec::new();
    for g in galleries {
        keys.push(parse_gallery_key(&g)?);
    }
    let req = EnqueueRequest {
        galleries: keys,
        original: Some(original),
    };
    let tasks = core.enqueue_downloads(req).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&tasks)?);
    } else {
        println!("Enqueued {} tasks", tasks.len());
    }
    Ok(())
}

pub async fn downloads(core: &Core, json: bool) -> Result<(), IpcError> {
    let tasks = core.list_downloads().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&tasks)?);
    } else {
        println!("Downloads count: {}", tasks.len());
    }
    Ok(())
}

pub async fn library_scan(core: &Core, dirs: Vec<PathBuf>, json: bool) -> Result<(), IpcError> {
    for d in dirs {
        core.add_library_dir(PathRequest {
            path: d.to_string_lossy().to_string(),
        })
        .await?;
    }
    let list = core.rescan_library().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        println!("Scanned {} galleries", list.len());
    }
    Ok(())
}

pub async fn library_pages(core: &Core, id: String, json: bool) -> Result<(), IpcError> {
    let pages = core.local_pages(LocalId { id }).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&pages)?);
    } else {
        println!("Local gallery has {} pages", pages.count);
    }
    Ok(())
}
