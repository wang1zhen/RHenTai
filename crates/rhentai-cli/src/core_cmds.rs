use std::collections::HashMap;
use std::path::PathBuf;

use eh_client::GalleryKey;
use rhentai_core::{
    Core, CoreEvent, DownloadState, DownloadTask, EnqueueRequest, IpcError, LocalId, TaskId,
};

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
    concurrency: Option<u8>,
    dir: Option<PathBuf>,
    json: bool,
) -> Result<(), IpcError> {
    let mut keys = Vec::new();
    for g in galleries {
        keys.push(parse_gallery_key(&g)?);
    }

    core.downloads.set_run_overrides(concurrency, dir);

    let mut rx = core.subscribe();

    let req = EnqueueRequest {
        galleries: keys,
        original: Some(original),
    };
    let tasks = core.enqueue_downloads(req).await?;
    if tasks.is_empty() {
        return Ok(());
    }

    let task_ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();

    for t in &tasks {
        if t.state == DownloadState::Paused || t.state == DownloadState::Failed {
            core.resume_download(TaskId { id: t.id }).await?;
        }
        let line = format_task_start_message(t);
        if json {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }

    core.start();

    let mut finished_states: HashMap<i64, DownloadTask> = HashMap::new();

    loop {
        let current_tasks = core.list_downloads().await?;
        let mut all_done = true;
        for t in &current_tasks {
            if task_ids.contains(&t.id) {
                if t.state == DownloadState::Completed
                    || t.state == DownloadState::Failed
                    || t.state == DownloadState::Paused
                {
                    finished_states.insert(t.id, t.clone());
                } else {
                    all_done = false;
                }
            }
        }

        if all_done && finished_states.len() == task_ids.len() {
            break;
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                let _ = core.pause_all().await;
                return Err(IpcError::new(
                    eh_client::ErrorKind::Cancelled,
                    "Download interrupted by Ctrl-C; tasks paused",
                ));
            }
            ev = rx.recv() => {
                match ev {
                    Ok(CoreEvent::DownloadProgress(p)) => {
                        if task_ids.contains(&p.id) {
                            let line = format!(
                                "[task {}] {:?}: {}/{} (failed: {})",
                                p.id, p.state, p.done, p.total, p.failed
                            );
                            if json {
                                eprintln!("{line}");
                            } else {
                                println!("{line}");
                            }
                            if (p.state == DownloadState::Completed
                                || p.state == DownloadState::Failed
                                || p.state == DownloadState::Paused)
                                && let Ok(Some(task)) = core.store.get_download(p.id)
                            {
                                finished_states.insert(p.id, task);
                            }
                        }
                    }
                    Ok(CoreEvent::DownloadsChanged) => {}
                    _ => {}
                }
            }
        }
    }

    let all_completed = finished_states
        .values()
        .all(|t| t.state == DownloadState::Completed);

    if json {
        let results: Vec<&DownloadTask> = task_ids
            .iter()
            .filter_map(|id| finished_states.get(id))
            .collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
    }

    if all_completed {
        Ok(())
    } else if let Some(failed_task) = finished_states
        .values()
        .find(|t| t.state == DownloadState::Failed || t.state == DownloadState::Paused)
    {
        let kind = match failed_task.state {
            DownloadState::Paused => {
                if let Some(ref err) = failed_task.error {
                    if err.contains("ImageLimit") {
                        eh_client::ErrorKind::ImageLimit
                    } else if err.contains("Banned") {
                        eh_client::ErrorKind::Banned
                    } else {
                        eh_client::ErrorKind::Internal
                    }
                } else {
                    eh_client::ErrorKind::Internal
                }
            }
            _ => eh_client::ErrorKind::Internal,
        };
        let msg = failed_task
            .error
            .clone()
            .unwrap_or_else(|| format!("Task {} {:?}", failed_task.id, failed_task.state));
        Err(IpcError::new(kind, msg))
    } else {
        Err(IpcError::internal("Download finished with errors"))
    }
}

pub async fn downloads(core: &Core, json: bool) -> Result<(), IpcError> {
    let tasks = core.list_downloads().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&tasks)?);
    } else {
        println!("Downloads count: {}", tasks.len());
        for t in &tasks {
            println!(
                "  [{}] gid: {}, state: {:?}, progress: {}/{}, dir: {}",
                t.id, t.gid, t.state, t.done, t.total, t.dir
            );
        }
    }
    Ok(())
}

pub async fn library_scan(core: &Core, dirs: Vec<PathBuf>, json: bool) -> Result<(), IpcError> {
    let s = core.settings.get();
    let mut roots = vec![PathBuf::from(&s.download_dir)];
    roots.extend(s.library_dirs.iter().map(PathBuf::from));
    roots.extend(dirs);

    let (list, warnings) = core.library.rescan_report(&roots)?;
    for w in warnings {
        eprintln!("warning: skipped {}: {}", w.path.display(), w.reason);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        println!("Scanned {} galleries", list.len());
        for g in &list {
            println!(
                "  [{}] {} ({} pages, {:?})",
                g.id, g.title, g.page_count, g.kind
            );
        }
    }
    Ok(())
}

pub async fn library_pages(core: &Core, id: String, json: bool) -> Result<(), IpcError> {
    let pages = core.local_pages(LocalId { id }).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&pages)?);
    } else {
        println!("Local gallery has {} pages", pages.count);
        for (i, name) in pages.names.iter().enumerate() {
            println!("  {:04}: {}", i + 1, name);
        }
    }
    Ok(())
}

pub(crate) fn format_task_start_message(t: &DownloadTask) -> String {
    if t.done > 0 || t.state == DownloadState::Paused || t.state == DownloadState::Failed {
        format!(
            "[task {}] resuming {}: {}/{} pages already present",
            t.id, t.gid, t.done, t.total
        )
    } else {
        format!("[task {}] starting {}", t.id, t.gid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhentai_core::Category;

    #[test]
    fn test_n1_message_selection_restored_queued_task() {
        let restored_queued_task = DownloadTask {
            id: 4,
            gid: 4253608,
            token: "06c93f5210".to_string(),
            site: eh_client::Site::Eh,
            title: "Test Gallery".to_string(),
            thumb: String::new(),
            category: Category::Manga,
            total: 41,
            done: 8,
            failed: 0,
            state: DownloadState::Queued,
            error: None,
            dir: "/tmp/test".to_string(),
            original: false,
            created_at: 0,
            updated_at: 0,
        };

        let msg = format_task_start_message(&restored_queued_task);
        assert_eq!(msg, "[task 4] resuming 4253608: 8/41 pages already present");
    }

    #[test]
    fn test_n1_message_selection_fresh_task() {
        let fresh_task = DownloadTask {
            id: 1,
            gid: 123456,
            token: "token123".to_string(),
            site: eh_client::Site::Eh,
            title: "Fresh Gallery".to_string(),
            thumb: String::new(),
            category: Category::Manga,
            total: 0,
            done: 0,
            failed: 0,
            state: DownloadState::Queued,
            error: None,
            dir: "/tmp/fresh".to_string(),
            original: false,
            created_at: 0,
            updated_at: 0,
        };

        let msg = format_task_start_message(&fresh_task);
        assert_eq!(msg, "[task 1] starting 123456");
    }
}
