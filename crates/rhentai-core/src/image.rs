use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bytes::Bytes;
use eh_client::{GalleryKey, PageImage, PageRef};
use sha1::{Digest, Sha1};
use tokio::sync::{RwLock, Semaphore};

use crate::error::{IpcError, Result};
use crate::model::CacheCleared;
use crate::source::PageSource;
use crate::store::Store;

pub const DEFAULT_CACHE_CAP_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

#[derive(Clone)]
pub struct ImageService {
    _cache_dir: PathBuf,
    thumbs_dir: PathBuf,
    pages_dir: PathBuf,
    source: Arc<dyn PageSource>,
    store: Store,
    resolution_semaphore: Arc<Semaphore>,
    page_refs: Arc<RwLock<HashMap<u64, Vec<PageRef>>>>,
    page_images: Arc<RwLock<HashMap<(u64, u32), PageImage>>>,
    reload_keys: Arc<RwLock<HashMap<(u64, u32), String>>>,
    thumbs_per_page: Arc<RwLock<HashMap<u64, u32>>>,
}

impl ImageService {
    pub fn new(cache_dir: &Path, source: Arc<dyn PageSource>, store: Store) -> Self {
        let thumbs_dir = cache_dir.join("thumbs");
        let pages_dir = cache_dir.join("pages");
        let _ = fs::create_dir_all(&thumbs_dir);
        let _ = fs::create_dir_all(&pages_dir);

        let service = Self {
            _cache_dir: cache_dir.to_path_buf(),
            thumbs_dir,
            pages_dir,
            source,
            store,
            resolution_semaphore: Arc::new(Semaphore::new(4)),
            page_refs: Arc::new(RwLock::new(HashMap::new())),
            page_images: Arc::new(RwLock::new(HashMap::new())),
            reload_keys: Arc::new(RwLock::new(HashMap::new())),
            thumbs_per_page: Arc::new(RwLock::new(HashMap::new())),
        };

        let _ = service.enforce_cache_cap();
        service
    }

    pub async fn thumb(&self, url: &str) -> Result<(Bytes, String)> {
        let mut hasher = Sha1::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let thumb_path = self.thumbs_dir.join(&hash);

        if let Ok(bytes) = fs::read(&thumb_path) {
            let content_type = sniff_content_type(&bytes).to_string();
            return Ok((Bytes::from(bytes), content_type));
        }

        let fetched = self.source.fetch_image(url, None).await?;
        let _ = fs::write(&thumb_path, &fetched.bytes);
        let _ = self.enforce_cache_cap();

        let content_type = fetched
            .content_type
            .unwrap_or_else(|| sniff_content_type(&fetched.bytes).to_string());
        Ok((fetched.bytes, content_type))
    }

    pub async fn page(&self, gid: u64, token: &str, index: u32) -> Result<(Bytes, String)> {
        // 1. If gallery is downloaded and file exists, serve from disk
        if let Ok(Some(task)) = self.store.get_download_by_gid(gid) {
            let dir = Path::new(&task.dir);
            if dir.is_dir() {
                let prefix = format!("{:04}.", index + 1);
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with(&prefix)
                            && !name.ends_with(".part")
                            && let Ok(bytes) = fs::read(entry.path())
                        {
                            let content_type = sniff_content_type(&bytes).to_string();
                            return Ok((Bytes::from(bytes), content_type));
                        }
                    }
                }
            }
        }

        // 2. Check disk cache
        let page_cache_path = self.pages_dir.join(gid.to_string()).join(index.to_string());
        if let Ok(bytes) = fs::read(&page_cache_path) {
            let content_type = sniff_content_type(&bytes).to_string();
            return Ok((Bytes::from(bytes), content_type));
        }

        // 3. Resolve and fetch with at most 4 concurrent resolutions
        let _permit = self
            .resolution_semaphore
            .acquire()
            .await
            .map_err(|e| IpcError::internal(e.to_string()))?;

        let key = GalleryKey {
            gid,
            token: token.to_string(),
        };

        // Ensure PageRef
        let page_ref = self.get_or_fetch_page_ref(&key, index).await?;

        // Check reload key
        let reload_key = self.reload_keys.write().await.remove(&(gid, index));

        let page_img = if reload_key.is_none() {
            self.page_images.read().await.get(&(gid, index)).cloned()
        } else {
            None
        };

        let page_image = match page_img {
            Some(pi) => pi,
            None => {
                let pi = self
                    .source
                    .page_image(&key, &page_ref, reload_key.as_deref())
                    .await?;

                if is_509_url(&pi.url) {
                    return Err(IpcError::new(
                        eh_client::ErrorKind::ImageLimit,
                        "Image viewing limit exceeded (509)",
                    ));
                }

                self.page_images
                    .write()
                    .await
                    .insert((gid, index), pi.clone());
                pi
            }
        };

        let fetched = match self.source.fetch_image(&page_image.url, None).await {
            Ok(f) => f,
            Err(e) => {
                if e.kind == eh_client::ErrorKind::ImageLimit {
                    return Err(e);
                }
                return Err(e);
            }
        };

        if let Some(parent) = page_cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&page_cache_path, &fetched.bytes);
        let _ = self.enforce_cache_cap();

        let content_type = fetched
            .content_type
            .unwrap_or_else(|| sniff_content_type(&fetched.bytes).to_string());
        Ok((fetched.bytes, content_type))
    }

    pub async fn reload(&self, gid: u64, _token: &str, index: u32) -> Result<()> {
        let page_cache_path = self.pages_dir.join(gid.to_string()).join(index.to_string());
        if page_cache_path.exists() {
            let _ = fs::remove_file(page_cache_path);
        }

        let mut images = self.page_images.write().await;
        if let Some(old) = images.remove(&(gid, index))
            && let Some(rk) = old.reload_key
        {
            self.reload_keys.write().await.insert((gid, index), rk);
        }
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<CacheCleared> {
        let mut total_freed = 0u64;
        if let Ok(entries) = fs::read_dir(&self.thumbs_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total_freed += meta.len();
                }
                let _ = fs::remove_file(entry.path());
            }
        }
        if self.pages_dir.exists() {
            total_freed += get_dir_size(&self.pages_dir);
            let _ = fs::remove_dir_all(&self.pages_dir);
            let _ = fs::create_dir_all(&self.pages_dir);
        }
        Ok(CacheCleared { bytes: total_freed })
    }

    pub fn enforce_cache_cap(&self) -> Result<()> {
        let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
        collect_cache_files(&self.thumbs_dir, &mut files);
        collect_cache_files(&self.pages_dir, &mut files);

        let total_size: u64 = files.iter().map(|(_, size, _)| *size).sum();
        if total_size > DEFAULT_CACHE_CAP_BYTES {
            // Sort oldest first
            files.sort_by_key(|(_, _, mtime)| *mtime);
            let mut current_size = total_size;
            let target_size = (DEFAULT_CACHE_CAP_BYTES as f64 * 0.9) as u64; // down to 90%
            for (path, size, _) in files {
                if current_size <= target_size {
                    break;
                }
                if fs::remove_file(&path).is_ok() {
                    current_size = current_size.saturating_sub(size);
                }
            }
        }
        Ok(())
    }

    async fn get_or_fetch_page_ref(&self, key: &GalleryKey, index: u32) -> Result<PageRef> {
        if let Some(refs) = self.page_refs.read().await.get(&key.gid)
            && let Some(r) = refs.iter().find(|p| p.index == index)
        {
            return Ok(r.clone());
        }

        // Fetch detail if thumbs_per_page is not known
        let tpp = match self.thumbs_per_page.read().await.get(&key.gid).copied() {
            Some(n) => n,
            None => {
                let detail = self.source.gallery_detail(key).await?;
                let n = detail.thumbs_per_page.max(1);
                self.thumbs_per_page.write().await.insert(key.gid, n);
                self.page_refs
                    .write()
                    .await
                    .entry(key.gid)
                    .or_default()
                    .extend(detail.thumbnails.pages);
                n
            }
        };

        if let Some(refs) = self.page_refs.read().await.get(&key.gid)
            && let Some(r) = refs.iter().find(|p| p.index == index)
        {
            return Ok(r.clone());
        }

        let p = index / tpp;
        let thumbs = self.source.thumbnails(key, p).await?;
        let res = thumbs
            .pages
            .iter()
            .find(|p| p.index == index)
            .cloned()
            .ok_or_else(|| {
                IpcError::not_found(format!("Page {index} not found for gallery {}", key.gid))
            });

        self.page_refs
            .write()
            .await
            .entry(key.gid)
            .or_default()
            .extend(thumbs.pages);

        res
    }
}

fn is_509_url(url: &str) -> bool {
    url.contains("509.gif") || url.contains("509.png") || url.ends_with("/509.jpg")
}

pub fn sniff_content_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\xFF\xD8\xFF") {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.len() >= 12
        && (&bytes[4..12] == b"ftypavif"
            || &bytes[4..12] == b"ftypavis"
            || &bytes[4..8] == b"ftyp" && bytes[8..12].starts_with(b"avif"))
    {
        "image/avif"
    } else if bytes.starts_with(b"BM") {
        "image/bmp"
    } else {
        "application/octet-stream"
    }
}

fn collect_cache_files(dir: &Path, out: &mut Vec<(PathBuf, u64, std::time::SystemTime)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    out.push((path, meta.len(), mtime));
                }
            } else if path.is_dir() {
                collect_cache_files(&path, out);
            }
        }
    }
}

fn get_dir_size(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            } else if path.is_dir() {
                total += get_dir_size(&path);
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_content_type() {
        assert_eq!(sniff_content_type(b"\xFF\xD8\xFF\xE0"), "image/jpeg");
        assert_eq!(
            sniff_content_type(b"\x89PNG\r\n\x1a\n\x00\x00\x00\r"),
            "image/png"
        );
        assert_eq!(sniff_content_type(b"GIF89a\x01\x00"), "image/gif");
        assert_eq!(
            sniff_content_type(b"RIFF\x00\x00\x00\x00WEBP"),
            "image/webp"
        );
        assert_eq!(
            sniff_content_type(b"\x00\x00\x00\x20ftypavif\x00\x00\x00\x00"),
            "image/avif"
        );
        assert_eq!(sniff_content_type(b"BM\x00\x00\x00\x00"), "image/bmp");
        assert_eq!(
            sniff_content_type(b"random bytes"),
            "application/octet-stream"
        );
    }
}
