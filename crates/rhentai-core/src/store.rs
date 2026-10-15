use std::path::Path;
use std::sync::{Arc, Mutex};

use eh_client::{Category, Site, Tag};
use rusqlite::{Connection, OptionalExtension, params};

use crate::error::Result;
use crate::model::{
    DownloadState, DownloadTask, HistoryEntry, LocalGallery, LocalKind, ReadTarget,
};

#[derive(Debug, Clone)]
pub struct DownloadPageRow {
    pub download_id: i64,
    pub idx: u32,
    pub page_token: Option<String>,
    pub file_name: Option<String>,
    pub state: String,
    pub error: Option<String>,
    pub bytes: Option<i64>,
}

#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn open(data_dir: &Path) -> Result<Self> {
        let db_path = data_dir.join("rhentai.db");
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&db_path)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version < 1 {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS downloads (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  gid INTEGER NOT NULL UNIQUE, token TEXT NOT NULL, site TEXT NOT NULL,
                  title TEXT NOT NULL, thumb TEXT NOT NULL, category TEXT NOT NULL,
                  total INTEGER NOT NULL DEFAULT 0, done INTEGER NOT NULL DEFAULT 0, failed INTEGER NOT NULL DEFAULT 0,
                  state TEXT NOT NULL, error TEXT, dir TEXT NOT NULL, original INTEGER NOT NULL DEFAULT 0,
                  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS download_pages (
                  download_id INTEGER NOT NULL REFERENCES downloads(id) ON DELETE CASCADE,
                  idx INTEGER NOT NULL, page_token TEXT, file_name TEXT, state TEXT NOT NULL, error TEXT, bytes INTEGER,
                  PRIMARY KEY (download_id, idx)
                );
                CREATE TABLE IF NOT EXISTS history (
                  key TEXT PRIMARY KEY,
                  gid INTEGER, token TEXT, local_id TEXT,
                  title TEXT NOT NULL, thumb TEXT, category TEXT,
                  page_count INTEGER NOT NULL, last_page INTEGER NOT NULL, read_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS local_galleries (
                  id TEXT PRIMARY KEY,
                  path TEXT NOT NULL UNIQUE, kind TEXT NOT NULL, title TEXT NOT NULL, page_count INTEGER NOT NULL,
                  gid INTEGER, token TEXT, category TEXT, tags_json TEXT NOT NULL DEFAULT '[]',
                  downloaded INTEGER NOT NULL DEFAULT 0, mtime INTEGER NOT NULL, added_at INTEGER NOT NULL
                );",
            )?;
            tx.commit()?;
            conn.pragma_update(None, "user_version", 1)?;
        }
        Ok(())
    }

    // --- Downloads ---

    pub fn insert_download(&self, task: &DownloadTask) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let site_str = match task.site {
            Site::Eh => "eh",
            Site::Ex => "ex",
        };
        let state_str = download_state_to_str(task.state);
        let cat_str = format!("{:?}", task.category);
        conn.execute(
            "INSERT INTO downloads (gid, token, site, title, thumb, category, total, done, failed, state, error, dir, original, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                task.gid as i64,
                task.token,
                site_str,
                task.title,
                task.thumb,
                cat_str,
                task.total,
                task.done,
                task.failed,
                state_str,
                task.error,
                task.dir,
                if task.original { 1 } else { 0 },
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_download(&self, id: i64) -> Result<Option<DownloadTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, gid, token, site, title, thumb, category, total, done, failed, state, error, dir, original, created_at, updated_at
             FROM downloads WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(params![id], row_to_download_task)
            .optional()?;
        Ok(row)
    }

    pub fn get_download_by_gid(&self, gid: u64) -> Result<Option<DownloadTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, gid, token, site, title, thumb, category, total, done, failed, state, error, dir, original, created_at, updated_at
             FROM downloads WHERE gid = ?1",
        )?;
        let row = stmt
            .query_row(params![gid as i64], row_to_download_task)
            .optional()?;
        Ok(row)
    }

    pub fn list_downloads(&self) -> Result<Vec<DownloadTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, gid, token, site, title, thumb, category, total, done, failed, state, error, dir, original, created_at, updated_at
             FROM downloads ORDER BY created_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([], row_to_download_task)?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn update_download_state(
        &self,
        id: i64,
        state: DownloadState,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        let state_str = download_state_to_str(state);
        conn.execute(
            "UPDATE downloads SET state = ?1, error = ?2, updated_at = ?3 WHERE id = ?4",
            params![state_str, error, now, id],
        )?;
        Ok(())
    }

    pub fn update_download_progress(
        &self,
        id: i64,
        done: u32,
        failed: u32,
        total: u32,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        conn.execute(
            "UPDATE downloads SET done = ?1, failed = ?2, total = ?3, updated_at = ?4 WHERE id = ?5",
            params![done, failed, total, now, id],
        )?;
        Ok(())
    }

    pub fn reconcile_download_progress(&self, download_id: i64) -> Result<(u32, u32, u32)> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        conn.execute(
            "UPDATE downloads
             SET done = (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = ?1 AND download_pages.state = 'done'),
                 failed = (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = ?1 AND download_pages.state = 'failed'),
                 total = CASE 
                     WHEN total = 0 AND (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = ?1) > 0 
                     THEN (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = ?1) 
                     ELSE total 
                 END,
                 updated_at = ?2
             WHERE id = ?1 AND EXISTS (SELECT 1 FROM download_pages WHERE download_pages.download_id = ?1)",
            params![download_id, now],
        )?;

        let mut stmt = conn.prepare("SELECT done, failed, total FROM downloads WHERE id = ?1")?;
        let res = stmt
            .query_row(params![download_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap_or((0, 0, 0));
        Ok(res)
    }

    pub fn reconcile_all_downloads_progress(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        conn.execute(
            "UPDATE downloads
             SET done = (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = downloads.id AND download_pages.state = 'done'),
                 failed = (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = downloads.id AND download_pages.state = 'failed'),
                 total = CASE 
                     WHEN total = 0 AND (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = downloads.id) > 0 
                     THEN (SELECT COUNT(*) FROM download_pages WHERE download_pages.download_id = downloads.id) 
                     ELSE total 
                 END,
                 updated_at = ?1
             WHERE EXISTS (SELECT 1 FROM download_pages WHERE download_pages.download_id = downloads.id)",
            params![now],
        )?;
        Ok(())
    }

    pub fn update_download_detail(
        &self,
        id: i64,
        title: &str,
        thumb: &str,
        category: &str,
        total: u32,
        dir: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        conn.execute(
            "UPDATE downloads SET title = ?1, thumb = ?2, category = ?3, total = ?4, dir = ?5, updated_at = ?6 WHERE id = ?7",
            params![title, thumb, category, total, dir, now, id],
        )?;
        Ok(())
    }

    pub fn reset_running_downloads_to_queued(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = current_time_secs();
        conn.execute(
            "UPDATE downloads SET state = 'queued', updated_at = ?1 WHERE state = 'running'",
            params![now],
        )?;
        Ok(())
    }

    pub fn remove_download(&self, id: i64) -> Result<Option<DownloadTask>> {
        let task = self.get_download(id)?;
        if task.is_some() {
            let conn = self.conn.lock().unwrap();
            conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        }
        Ok(task)
    }

    // --- Download Pages ---

    pub fn insert_download_pages_if_empty(
        &self,
        download_id: i64,
        pages: &[eh_client::PageRef],
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM download_pages WHERE download_id = ?1",
            params![download_id],
            |r| r.get(0),
        )?;
        if count == 0 {
            let mut stmt = conn.prepare(
                "INSERT OR REPLACE INTO download_pages (download_id, idx, page_token, file_name, state, error, bytes)
                 VALUES (?1, ?2, ?3, ?4, 'queued', NULL, NULL)",
            )?;
            for p in pages {
                stmt.execute(params![
                    download_id,
                    p.index,
                    &p.page_token,
                    p.file_name.as_deref()
                ])?;
            }
        }
        Ok(())
    }

    pub fn get_download_pages(&self, download_id: i64) -> Result<Vec<DownloadPageRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT download_id, idx, page_token, file_name, state, error, bytes
             FROM download_pages WHERE download_id = ?1 ORDER BY idx ASC",
        )?;
        let rows = stmt.query_map(params![download_id], |row| {
            Ok(DownloadPageRow {
                download_id: row.get(0)?,
                idx: row.get(1)?,
                page_token: row.get(2)?,
                file_name: row.get(3)?,
                state: row.get(4)?,
                error: row.get(5)?,
                bytes: row.get(6)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn update_download_page_state(
        &self,
        download_id: i64,
        idx: u32,
        state: &str,
        file_name: Option<&str>,
        error: Option<&str>,
        bytes: Option<i64>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE download_pages SET state = ?1, file_name = COALESCE(?2, file_name), error = ?3, bytes = COALESCE(?4, bytes)
             WHERE download_id = ?5 AND idx = ?6",
            params![state, file_name, error, bytes, download_id, idx],
        )?;
        Ok(())
    }

    pub fn reset_failed_pages_for_download(&self, download_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE download_pages SET state = 'queued', error = NULL WHERE download_id = ?1 AND state != 'done'",
            params![download_id],
        )?;
        Ok(())
    }

    // --- History ---

    pub fn record_history(&self, entry: &HistoryEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let (key, gid, token, local_id) = match &entry.target {
            ReadTarget::Online { gid, token } => (
                format!("online:{gid}"),
                Some(*gid as i64),
                Some(token.clone()),
                None,
            ),
            ReadTarget::Local { id } => (format!("local:{id}"), None, None, Some(id.clone())),
        };
        let cat_str = entry.category.map(|c| format!("{c:?}"));
        conn.execute(
            "INSERT INTO history (key, gid, token, local_id, title, thumb, category, page_count, last_page, read_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(key) DO UPDATE SET
               title = excluded.title,
               thumb = excluded.thumb,
               category = excluded.category,
               page_count = excluded.page_count,
               last_page = excluded.last_page,
               read_at = excluded.read_at",
            params![
                key,
                gid,
                token,
                local_id,
                entry.title,
                entry.thumb,
                cat_str,
                entry.page_count,
                entry.last_page,
                entry.read_at,
            ],
        )?;

        // Cap to 500 entries
        conn.execute(
            "DELETE FROM history WHERE key NOT IN (
               SELECT key FROM history ORDER BY read_at DESC LIMIT 500
             )",
            [],
        )?;
        Ok(())
    }

    pub fn list_history(&self) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, gid, token, local_id, title, thumb, category, page_count, last_page, read_at
             FROM history ORDER BY read_at DESC LIMIT 500",
        )?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let gid: Option<i64> = row.get(1)?;
            let token: Option<String> = row.get(2)?;
            let local_id: Option<String> = row.get(3)?;
            let title: String = row.get(4)?;
            let thumb: Option<String> = row.get(5)?;
            let cat_str: Option<String> = row.get(6)?;
            let page_count: u32 = row.get(7)?;
            let last_page: u32 = row.get(8)?;
            let read_at: i64 = row.get(9)?;

            let target = if let (Some(gid), Some(token)) = (gid, token) {
                ReadTarget::Online {
                    gid: gid as u64,
                    token,
                }
            } else if let Some(lid) = local_id {
                ReadTarget::Local { id: lid }
            } else if let Some(stripped) = key.strip_prefix("online:") {
                ReadTarget::Online {
                    gid: stripped.parse().unwrap_or(0),
                    token: String::new(),
                }
            } else {
                ReadTarget::Local {
                    id: key.strip_prefix("local:").unwrap_or(&key).to_string(),
                }
            };

            let category = cat_str.as_deref().map(Category::from_site);

            Ok(HistoryEntry {
                target,
                title,
                thumb,
                category,
                page_count,
                last_page,
                read_at,
            })
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn clear_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    pub fn get_history_entry(&self, key: &str) -> Result<Option<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, gid, token, local_id, title, thumb, category, page_count, last_page, read_at
             FROM history WHERE key = ?1",
        )?;
        let row = stmt
            .query_row(params![key], |row| {
                let key: String = row.get(0)?;
                let gid: Option<i64> = row.get(1)?;
                let token: Option<String> = row.get(2)?;
                let local_id: Option<String> = row.get(3)?;
                let title: String = row.get(4)?;
                let thumb: Option<String> = row.get(5)?;
                let cat_str: Option<String> = row.get(6)?;
                let page_count: u32 = row.get(7)?;
                let last_page: u32 = row.get(8)?;
                let read_at: i64 = row.get(9)?;

                let target = if let (Some(gid), Some(token)) = (gid, token) {
                    ReadTarget::Online {
                        gid: gid as u64,
                        token,
                    }
                } else if let Some(lid) = local_id {
                    ReadTarget::Local { id: lid }
                } else if let Some(stripped) = key.strip_prefix("online:") {
                    ReadTarget::Online {
                        gid: stripped.parse().unwrap_or(0),
                        token: String::new(),
                    }
                } else {
                    ReadTarget::Local {
                        id: key.strip_prefix("local:").unwrap_or(&key).to_string(),
                    }
                };

                let category = cat_str.as_deref().map(Category::from_site);

                Ok(HistoryEntry {
                    target,
                    title,
                    thumb,
                    category,
                    page_count,
                    last_page,
                    read_at,
                })
            })
            .optional()?;
        Ok(row)
    }

    // --- Local Galleries ---

    pub fn upsert_local_gallery(&self, gallery: &LocalGallery, mtime: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let kind_str = match gallery.kind {
            LocalKind::Folder => "folder",
            LocalKind::Archive => "archive",
        };
        let cat_str = gallery.category.map(|c| format!("{c:?}"));
        let tags_json = serde_json::to_string(&gallery.tags).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO local_galleries (id, path, kind, title, page_count, gid, token, category, tags_json, downloaded, mtime, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
               path = excluded.path,
               kind = excluded.kind,
               title = excluded.title,
               page_count = excluded.page_count,
               gid = excluded.gid,
               token = excluded.token,
               category = excluded.category,
               tags_json = excluded.tags_json,
               downloaded = excluded.downloaded,
               mtime = excluded.mtime",
            params![
                gallery.id,
                gallery.path,
                kind_str,
                gallery.title,
                gallery.page_count,
                gallery.gid.map(|g| g as i64),
                gallery.token,
                cat_str,
                tags_json,
                if gallery.downloaded { 1 } else { 0 },
                mtime,
                gallery.added_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_local_galleries(&self) -> Result<Vec<LocalGallery>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, kind, title, page_count, gid, token, category, tags_json, downloaded, added_at
             FROM local_galleries ORDER BY added_at DESC, title ASC",
        )?;
        let rows = stmt.query_map([], row_to_local_gallery)?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn get_local_gallery(&self, id: &str) -> Result<Option<LocalGallery>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, kind, title, page_count, gid, token, category, tags_json, downloaded, added_at
             FROM local_galleries WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(params![id], row_to_local_gallery)
            .optional()?;
        Ok(row)
    }

    pub fn get_local_gallery_by_path(&self, path: &str) -> Result<Option<LocalGallery>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, kind, title, page_count, gid, token, category, tags_json, downloaded, added_at
             FROM local_galleries WHERE path = ?1",
        )?;
        let row = stmt
            .query_row(params![path], row_to_local_gallery)
            .optional()?;
        Ok(row)
    }

    pub fn remove_local_gallery(&self, id: &str) -> Result<Option<LocalGallery>> {
        let gallery = self.get_local_gallery(id)?;
        if gallery.is_some() {
            let conn = self.conn.lock().unwrap();
            conn.execute("DELETE FROM local_galleries WHERE id = ?1", params![id])?;
        }
        Ok(gallery)
    }
}

fn row_to_download_task(row: &rusqlite::Row) -> rusqlite::Result<DownloadTask> {
    let id: i64 = row.get(0)?;
    let gid_i64: i64 = row.get(1)?;
    let token: String = row.get(2)?;
    let site_str: String = row.get(3)?;
    let title: String = row.get(4)?;
    let thumb: String = row.get(5)?;
    let cat_str: String = row.get(6)?;
    let total: u32 = row.get(7)?;
    let done: u32 = row.get(8)?;
    let failed: u32 = row.get(9)?;
    let state_str: String = row.get(10)?;
    let error: Option<String> = row.get(11)?;
    let dir: String = row.get(12)?;
    let orig_i32: i32 = row.get(13)?;
    let created_at: i64 = row.get(14)?;
    let updated_at: i64 = row.get(15)?;

    let site = if site_str == "ex" { Site::Ex } else { Site::Eh };
    let category = Category::from_site(&cat_str);
    let state = str_to_download_state(&state_str);

    Ok(DownloadTask {
        id,
        gid: gid_i64 as u64,
        token,
        site,
        title,
        thumb,
        category,
        total,
        done,
        failed,
        state,
        error,
        dir,
        original: orig_i32 != 0,
        created_at,
        updated_at,
    })
}

fn row_to_local_gallery(row: &rusqlite::Row) -> rusqlite::Result<LocalGallery> {
    let id: String = row.get(0)?;
    let path: String = row.get(1)?;
    let kind_str: String = row.get(2)?;
    let title: String = row.get(3)?;
    let page_count: u32 = row.get(4)?;
    let gid_i64: Option<i64> = row.get(5)?;
    let token: Option<String> = row.get(6)?;
    let cat_str: Option<String> = row.get(7)?;
    let tags_json: String = row.get(8)?;
    let downloaded_i32: i32 = row.get(9)?;
    let added_at: i64 = row.get(10)?;

    let kind = if kind_str == "archive" {
        LocalKind::Archive
    } else {
        LocalKind::Folder
    };
    let category = cat_str.as_deref().map(Category::from_site);
    let tags: Vec<Tag> = serde_json::from_str(&tags_json).unwrap_or_default();

    Ok(LocalGallery {
        id,
        title,
        path,
        kind,
        page_count,
        gid: gid_i64.map(|g| g as u64),
        token,
        category,
        tags,
        downloaded: downloaded_i32 != 0,
        added_at,
        last_read_page: None,
    })
}

fn download_state_to_str(s: DownloadState) -> &'static str {
    match s {
        DownloadState::Queued => "queued",
        DownloadState::Running => "running",
        DownloadState::Paused => "paused",
        DownloadState::Completed => "completed",
        DownloadState::Failed => "failed",
    }
}

fn str_to_download_state(s: &str) -> DownloadState {
    match s {
        "running" => DownloadState::Running,
        "paused" => DownloadState::Paused,
        "completed" => DownloadState::Completed,
        "failed" => DownloadState::Failed,
        _ => DownloadState::Queued,
    }
}

pub fn current_time_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_idempotent() {
        let store = Store::in_memory().unwrap();
        // Running init again must not error
        assert!(store.init().is_ok());
        assert!(store.init().is_ok());

        // Check user_version
        let conn = store.conn.lock().unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_download_crud() {
        let store = Store::in_memory().unwrap();
        let task = DownloadTask {
            id: 0,
            gid: 12345,
            token: "abcdef".to_string(),
            site: Site::Eh,
            title: "Test Gallery".to_string(),
            thumb: "https://example.com/thumb.jpg".to_string(),
            category: Category::Manga,
            total: 10,
            done: 0,
            failed: 0,
            state: DownloadState::Queued,
            error: None,
            dir: "/downloads/12345".to_string(),
            original: false,
            created_at: 1000,
            updated_at: 1000,
        };
        let id = store.insert_download(&task).unwrap();
        assert!(id > 0);

        let fetched = store.get_download(id).unwrap().unwrap();
        assert_eq!(fetched.gid, 12345);
        assert_eq!(fetched.state, DownloadState::Queued);

        store
            .update_download_state(id, DownloadState::Running, None)
            .unwrap();
        let running = store.get_download(id).unwrap().unwrap();
        assert_eq!(running.state, DownloadState::Running);

        store.reset_running_downloads_to_queued().unwrap();
        let reset = store.get_download(id).unwrap().unwrap();
        assert_eq!(reset.state, DownloadState::Queued);

        let list = store.list_downloads().unwrap();
        assert_eq!(list.len(), 1);

        let removed = store.remove_download(id).unwrap();
        assert!(removed.is_some());
        assert!(store.get_download(id).unwrap().is_none());
    }

    #[test]
    fn test_history_cap_500() {
        let store = Store::in_memory().unwrap();
        for i in 0..520 {
            let entry = HistoryEntry {
                target: ReadTarget::Online {
                    gid: i,
                    token: format!("token_{i}"),
                },
                title: format!("Title {i}"),
                thumb: None,
                category: None,
                page_count: 20,
                last_page: 1,
                read_at: i as i64,
            };
            store.record_history(&entry).unwrap();
        }
        let list = store.list_history().unwrap();
        assert_eq!(list.len(), 500);
        // Newest should be 519
        if let ReadTarget::Online { gid, .. } = list[0].target {
            assert_eq!(gid, 519);
        } else {
            panic!("Expected online target");
        }
    }
}
