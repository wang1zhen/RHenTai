use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use eh_client::{Category, Site, Tag};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{IpcError, Result};
use crate::image::sniff_content_type;
use crate::model::{LocalGallery, LocalKind, LocalPages};
use crate::store::{Store, current_time_secs};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "avif", "bmp"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RhentaiMetadata {
    pub version: u32,
    pub site: Site,
    pub gid: u64,
    pub token: String,
    pub title: String,
    pub title_jpn: Option<String>,
    pub category: Category,
    pub uploader: Option<String>,
    pub posted: i64,
    pub tags: Vec<Tag>,
    pub page_count: u32,
    pub pages: Vec<String>,
    pub downloaded_at: i64,
    pub original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanWarning {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone)]
pub struct Library {
    store: Store,
}

impl Library {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<LocalGallery>> {
        self.store.list_local_galleries()
    }

    pub fn rescan(&self, roots: &[PathBuf]) -> Result<Vec<LocalGallery>> {
        let (galleries, warnings) = self.rescan_report(roots)?;
        for w in warnings {
            tracing::warn!("skipped {}: {}", w.path.display(), w.reason);
        }
        Ok(galleries)
    }

    pub fn rescan_report(
        &self,
        roots: &[PathBuf],
    ) -> Result<(Vec<LocalGallery>, Vec<ScanWarning>)> {
        let mut warnings = Vec::new();
        for root in roots {
            if root.exists() {
                self.scan_root(root, &mut warnings)?;
            }
        }
        let galleries = self.store.list_local_galleries()?;
        Ok((galleries, warnings))
    }

    fn scan_root(&self, root: &Path, warnings: &mut Vec<ScanWarning>) -> Result<()> {
        self.scan_dir(root, 0, warnings)?;
        Ok(())
    }

    fn scan_dir(&self, dir: &Path, depth: usize, warnings: &mut Vec<ScanWarning>) -> Result<()> {
        if depth > 3 {
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        let mut subdirs = Vec::new();
        let mut image_files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Ignore hidden files and .part files
            if file_name.starts_with('.') || file_name.ends_with(".part") {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            // Symlinks not followed
            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                subdirs.push(path);
            } else if file_type.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                    image_files.push(file_name);
                } else if ext == "zip" || ext == "cbz" {
                    self.check_and_register_archive(&path, warnings);
                }
            }
        }

        if !image_files.is_empty() {
            // This directory is a folder gallery
            self.register_folder_gallery(dir, image_files.len() as u32);
        }

        // Recurse into subdirectories
        if depth < 3 {
            for sub in subdirs {
                self.scan_dir(&sub, depth + 1, warnings)?;
            }
        }

        Ok(())
    }

    fn register_folder_gallery(&self, dir: &Path, image_count: u32) {
        let canonical = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let id = compute_local_id(&canonical);
        let mtime = fs::metadata(dir)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
            })
            .unwrap_or_else(|_| current_time_secs());

        let metadata_file = dir.join("rhentai.json");
        let (title, gid, token, category, tags, downloaded) = if metadata_file.is_file() {
            if let Ok(content) = fs::read_to_string(&metadata_file) {
                if let Ok(meta) = serde_json::from_str::<RhentaiMetadata>(&content) {
                    (
                        meta.title,
                        Some(meta.gid),
                        Some(meta.token),
                        Some(meta.category),
                        meta.tags,
                        true,
                    )
                } else {
                    fallback_folder_info(dir)
                }
            } else {
                fallback_folder_info(dir)
            }
        } else {
            fallback_folder_info(dir)
        };

        let gallery = LocalGallery {
            id,
            title,
            path: canonical.to_string_lossy().to_string(),
            kind: LocalKind::Folder,
            page_count: image_count,
            gid,
            token,
            category,
            tags,
            downloaded,
            added_at: current_time_secs(),
            last_read_page: None,
        };

        let _ = self.store.upsert_local_gallery(&gallery, mtime);
    }

    fn check_and_register_archive(&self, path: &Path, warnings: &mut Vec<ScanWarning>) {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                warnings.push(ScanWarning {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                });
                return;
            }
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                warnings.push(ScanWarning {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                });
                return;
            }
        };

        let mut image_count = 0u32;
        for i in 0..archive.len() {
            let Ok(entry) = archive.by_index(i) else {
                continue;
            };
            if !entry.is_file() {
                continue;
            }
            let Ok(name) = entry.name() else {
                continue;
            };
            if name.starts_with('.') || name.contains("__MACOSX") {
                continue;
            }
            let Some(ext) = Path::new(name.as_ref())
                .extension()
                .and_then(|e| e.to_str())
            else {
                continue;
            };
            if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                image_count += 1;
            }
        }

        if image_count == 0 {
            return;
        }

        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let id = compute_local_id(&canonical);
        let mtime = fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
            })
            .unwrap_or_else(|_| current_time_secs());

        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());

        let gallery = LocalGallery {
            id,
            title,
            path: canonical.to_string_lossy().to_string(),
            kind: LocalKind::Archive,
            page_count: image_count,
            gid: None,
            token: None,
            category: None,
            tags: Vec::new(),
            downloaded: false,
            added_at: current_time_secs(),
            last_read_page: None,
        };

        let _ = self.store.upsert_local_gallery(&gallery, mtime);
    }

    pub fn pages(&self, id: &str) -> Result<LocalPages> {
        let gallery = self.store.get_local_gallery(id)?.ok_or_else(|| {
            IpcError::not_found(format!("Local gallery with id '{id}' not found"))
        })?;

        let names = match gallery.kind {
            LocalKind::Folder => {
                let dir = Path::new(&gallery.path);
                let mut list = Vec::new();
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') || name.ends_with(".part") || !path.is_file() {
                            continue;
                        }
                        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                            continue;
                        };
                        if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                            list.push(name);
                        }
                    }
                }
                list.sort_by(|a, b| natord::compare(a, b));
                list
            }
            LocalKind::Archive => {
                let file = File::open(&gallery.path)?;
                let mut archive = zip::ZipArchive::new(file)
                    .map_err(|e| IpcError::new(eh_client::ErrorKind::Parse, e.to_string()))?;
                let mut list = Vec::new();
                for i in 0..archive.len() {
                    let Ok(entry) = archive.by_index(i) else {
                        continue;
                    };
                    if !entry.is_file() {
                        continue;
                    }
                    let Ok(name) = entry.name() else {
                        continue;
                    };
                    let name = name.to_string();
                    if name.starts_with('.') || name.contains("__MACOSX") {
                        continue;
                    }
                    let Some(ext) = Path::new(&name).extension().and_then(|e| e.to_str()) else {
                        continue;
                    };
                    if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                        list.push(name);
                    }
                }
                list.sort_by(|a, b| natord::compare(a, b));
                list
            }
        };

        Ok(LocalPages {
            count: names.len() as u32,
            names,
        })
    }

    pub fn page(&self, id: &str, index: u32) -> Result<(Bytes, String)> {
        let gallery = self.store.get_local_gallery(id)?.ok_or_else(|| {
            IpcError::not_found(format!("Local gallery with id '{id}' not found"))
        })?;

        let pages = self.pages(id)?;
        let name = pages
            .names
            .get(index as usize)
            .ok_or_else(|| IpcError::not_found(format!("Page index {index} not found")))?;

        match gallery.kind {
            LocalKind::Folder => {
                let file_path = Path::new(&gallery.path).join(name);
                let bytes = fs::read(&file_path)?;
                let content_type = sniff_content_type(&bytes).to_string();
                Ok((Bytes::from(bytes), content_type))
            }
            LocalKind::Archive => {
                let file = File::open(&gallery.path)?;
                let mut archive = zip::ZipArchive::new(file)
                    .map_err(|e| IpcError::new(eh_client::ErrorKind::Parse, e.to_string()))?;
                let mut entry = archive
                    .by_name(name)
                    .map_err(|e| IpcError::new(eh_client::ErrorKind::Parse, e.to_string()))?;
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                let content_type = sniff_content_type(&bytes).to_string();
                Ok((Bytes::from(bytes), content_type))
            }
        }
    }

    pub fn remove(&self, id: &str, delete_files: bool, roots: &[PathBuf]) -> Result<()> {
        let gallery = self.store.get_local_gallery(id)?.ok_or_else(|| {
            IpcError::not_found(format!("Local gallery with id '{id}' not found"))
        })?;

        if delete_files {
            let path = Path::new(&gallery.path);
            let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

            let inside_any_root = roots.iter().any(|r| {
                if let Ok(cr) = fs::canonicalize(r) {
                    canonical.starts_with(cr)
                } else {
                    canonical.starts_with(r)
                }
            });

            if !inside_any_root {
                return Err(IpcError::invalid_input(
                    "Cannot delete files: gallery path is outside configured library roots",
                ));
            }

            if canonical.is_dir() {
                fs::remove_dir_all(&canonical)?;
            } else if canonical.is_file() {
                fs::remove_file(&canonical)?;
            }
        }

        self.store.remove_local_gallery(id)?;
        Ok(())
    }
}

pub fn compute_local_id(path: &Path) -> String {
    let s = path.to_string_lossy();
    let hash = Sha256::digest(s.as_bytes());
    let hex_str: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    hex_str[..16].to_string()
}

fn fallback_folder_info(
    dir: &Path,
) -> (
    String,
    Option<u64>,
    Option<String>,
    Option<Category>,
    Vec<Tag>,
    bool,
) {
    let title = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string());
    (title, None, None, None, Vec::new(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_local_id() {
        let p1 = Path::new("/some/path/to/gallery");
        let id1 = compute_local_id(p1);
        assert_eq!(id1.len(), 16);
        let id2 = compute_local_id(p1);
        assert_eq!(id1, id2);
    }
}
