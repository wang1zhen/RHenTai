# RHenTai IPC Contract

This is the single source of truth for every type and command that crosses the Tauri IPC boundary, for
the `rhimg://` image protocol, and for the CLI surface used in QA. Change this file first, then the code.

## 0. Conventions (mandatory)

- Every IPC type derives `Debug, Clone, Serialize, Deserialize, ts_rs::TS` and has
  `#[serde(rename_all = "camelCase")]` and `#[ts(export, export_to = "<path to repo>/src/bindings/")]`.
  Set the export dir once with the `TS_RS_EXPORT_DIR` env var in `.cargo/config.toml`
  (`[env] TS_RS_EXPORT_DIR = { value = "src/bindings", relative = true }`) and use plain `#[ts(export)]`.
- **Integers:** ts-rs maps `u64`/`i64` to `bigint`. Every `u64`/`i64` field must carry
  `#[ts(type = "number")]`. GIDs and unix timestamps fit in 2^53.
- Enums without data serialize as camelCase strings. Enums with data use `#[serde(tag = "kind")]`.
- `Option<T>` maps to `T | null`. Use `#[ts(optional = nullable)]` only if the TS side needs `?:`; it does
  not need it here.
- Regenerate bindings with `cargo test -p eh-client -p rhentai-core export_bindings` (ts-rs generates
  `export_bindings_*` tests). Commit the generated `src/bindings/*.ts`.
- **Commands take zero or one argument, always named `req`.** The JS call is `invoke("cmd", { req })`.
  This sidesteps Tauri's camelCase renaming of argument names.
- Every command returns `Result<T, IpcError>`. The frontend wrapper rethrows an `IpcError` object.

## 1. Shared types (crate `eh-client`, module `model`)

```rust
#[serde(rename_all = "lowercase")]
pub enum Site { Eh, Ex }                                         // "eh" | "ex"

pub enum Category {                                              // camelCase strings
    Doujinshi, Manga, ArtistCg, GameCg, Western, NonH, ImageSet, Cosplay, AsianPorn, Misc, Private, Unknown,
}
// impl Category { fn from_site(s: &str) -> Self  /* accepts "Doujinshi","Artist CG","artistcg","Non-H","Image Set","Asian Porn","Misc","Private", case-insensitive, also gdata spellings */ ;
//                 fn f_cats_bit(self) -> u32 /* EH_PROTOCOL §1.3 */ }

pub struct GalleryKey { #[ts(type = "number")] pub gid: u64, pub token: String }

pub struct Tag { pub namespace: String, pub name: String }        // namespace "" when unnamespaced
pub struct TagGroup { pub namespace: String, pub tags: Vec<String> }

pub struct GallerySummary {
    #[ts(type = "number")] pub gid: u64,
    pub token: String,
    pub title: String,
    pub title_jpn: Option<String>,
    pub category: Category,
    pub thumb: String,                       // absolute remote URL (load through rhimg thumb route)
    pub uploader: Option<String>,
    #[ts(type = "number")] pub posted: i64,  // unix seconds
    pub file_count: u32,
    pub rating: f32,                         // 0.0..=5.0
    pub tags: Vec<Tag>,
    pub expunged: bool,
    pub favorite_slot: Option<u8>,           // 0..=9 when the page shows it (logged in)
    pub favorite_note: Option<String>,
}

#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ListSource {
    Front,                                   // {"kind":"front"}
    Watched,                                 // {"kind":"watched"}  (login required)
    Popular,                                 // {"kind":"popular"}  (single page, no cursor)
    Toplist { period: ToplistPeriod },       // {"kind":"toplist","period":"day"}  (EH only)
    Favorites { slot: Option<u8> },          // {"kind":"favorites","slot":null}  (login required)
}
pub enum ToplistPeriod { Day, Month, Year, All }  // tl = 15, 13, 12, 11

pub struct SearchFilter {
    pub keyword: String,                     // raw f_search text
    pub excluded_categories: Vec<Category>,  // OR-ed into f_cats
    pub min_rating: Option<u8>,              // 2..=5 -> f_srdd   (EH_PROTOCOL 1.2)
    pub min_pages: Option<u32>,              // f_spf
    pub max_pages: Option<u32>,              // f_spt (only when >= min_pages)
    pub show_expunged: bool,                 // f_sh=on
    pub require_torrent: bool,               // f_sto=on
}

pub struct ListRequest { pub source: ListSource, pub filter: SearchFilter, pub cursor: Option<String> }

pub struct GalleryListPage {
    pub items: Vec<GallerySummary>,
    pub next: Option<String>,                // opaque cursor for the next page (None = end)
    pub prev: Option<String>,
}
// Cursor format (opaque to the UI, produced only by eh-client): "next:<gid>" | "prev:<gid>" | "page:<n>" (toplist)

pub struct ThumbSprite {                     // draw as background-image with background-position
    pub url: String, pub width: u32, pub height: u32, pub offset_x: i32, pub offset_y: i32,
}
pub struct PageRef {
    pub index: u32,                          // 0-based page index within the gallery
    pub page_token: String,                  // 10 hex chars from /s/<page_token>/<gid>-<index+1>
    pub thumb: ThumbSprite,
    pub file_name: Option<String>,           // from the thumbnail title "Page N: name.jpg"
}
pub struct ThumbnailPage { pub page: u32, pub pages: Vec<PageRef> }

pub struct Comment {
    pub author: String, pub posted: String, pub score: Option<String>, pub body: String, pub is_uploader: bool,
}

pub struct GalleryDetail {
    pub summary: GallerySummary,             // favorite_slot / favorite_note from the detail page
    pub language: Option<String>,
    pub file_size: Option<String>,           // site text, e.g. "209.9 MiB"
    pub favorited_count: Option<u32>,
    pub rating_count: Option<u32>,
    pub parent: Option<GalleryKey>,
    pub tag_groups: Vec<TagGroup>,
    pub thumbnails: ThumbnailPage,           // detail page p=0
    pub thumbnail_page_count: u32,           // number of ?p= pages (>= 1)
    pub thumbs_per_page: u32,                // number of thumbnails on p=0 (e.g. 20 or 40)
    pub comments: Vec<Comment>,
}

pub struct PageImage {
    pub index: u32,
    pub url: String,
    pub width: Option<u32>, pub height: Option<u32>,
    pub reload_key: Option<String>,          // value for nl=
    pub original_url: Option<String>,        // fullimg link when present (login)
    pub file_name: Option<String>,
}

pub struct FavoriteCategory { pub slot: u8, pub name: String, pub count: u32 }

pub struct AuthStatus {
    pub logged_in: bool,
    pub member_id: Option<String>,
    pub username: Option<String>,            // best effort
    pub ex_access: bool,                     // a valid igneous cookie is present
}
pub struct CookieLogin { pub member_id: String, pub pass_hash: String, pub igneous: Option<String> }
pub struct PasswordLogin { pub username: String, pub password: String }

pub enum ErrorKind {
    Network, Http, Parse, Cloudflare, Banned, LoginRequired, LoginFailed, NoExAccess, ImageLimit,
    NotFound, InvalidInput, Io, Db, Cancelled, Internal,
}
pub struct IpcError { pub kind: ErrorKind, pub message: String }   // defined in rhentai-core, re-used by src-tauri
```

## 2. App types (crate `rhentai-core`)

```rust
pub enum Theme { System, Light, Dark }
#[serde(rename = ...)] pub enum UiLanguage { #[serde(rename = "zh-CN")] ZhCn, #[serde(rename = "en")] En }
pub enum ReaderMode { Vertical, Paged }
pub enum ReadingDirection { Ltr, Rtl }
pub enum FitMode { Width, Height, Original }
pub struct ReaderSettings { pub mode: ReaderMode, pub direction: ReadingDirection, pub fit: FitMode, pub preload: u8 }

pub struct Settings {
    pub site: Site, pub proxy: Option<String>, pub download_dir: String, pub download_concurrency: u8,
    pub download_original: bool, pub library_dirs: Vec<String>, pub reader: ReaderSettings,
    pub theme: Theme, pub language: UiLanguage,
}

pub enum DownloadState { Queued, Running, Paused, Completed, Failed }
pub struct DownloadTask {
    #[ts(type = "number")] pub id: i64,
    #[ts(type = "number")] pub gid: u64,
    pub token: String, pub site: Site, pub title: String, pub thumb: String, pub category: Category,
    pub total: u32, pub done: u32, pub failed: u32, pub state: DownloadState, pub error: Option<String>,
    pub dir: String, pub original: bool,
    #[ts(type = "number")] pub created_at: i64,
    #[ts(type = "number")] pub updated_at: i64,
}
pub struct DownloadProgress {
    #[ts(type = "number")] pub id: i64,
    pub done: u32, pub total: u32, pub failed: u32, pub state: DownloadState, pub error: Option<String>,
    #[ts(type = "number")] pub bytes_per_sec: u64,
}

pub enum LocalKind { Folder, Archive }
pub struct LocalGallery {
    pub id: String, pub title: String, pub path: String, pub kind: LocalKind, pub page_count: u32,
    #[ts(type = "number | null")] pub gid: Option<u64>,
    pub token: Option<String>, pub category: Option<Category>, pub tags: Vec<Tag>, pub downloaded: bool,
    #[ts(type = "number")] pub added_at: i64,
    pub last_read_page: Option<u32>,
}
pub struct LocalPages { pub count: u32, pub names: Vec<String> }

#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ReadTarget { Online { #[ts(type = "number")] gid: u64, token: String }, Local { id: String } }
pub struct HistoryEntry {
    pub target: ReadTarget, pub title: String, pub thumb: Option<String>, pub category: Option<Category>,
    pub page_count: u32, pub last_page: u32,
    #[ts(type = "number")] pub read_at: i64,
}
pub struct ReadProgress {
    pub target: ReadTarget, pub page: u32, pub page_count: u32,
    pub title: String, pub thumb: Option<String>, pub category: Option<Category>,
}

// small request wrappers
pub struct ThumbnailRequest { #[ts(type = "number")] pub gid: u64, pub token: String, pub page: u32 }
pub struct PageImageKey { #[ts(type = "number")] pub gid: u64, pub token: String, pub index: u32 }
pub struct FavoriteUpdate { #[ts(type = "number")] pub gid: u64, pub token: String, pub slot: Option<u8>, pub note: String } // slot None = remove
pub struct BatchFavorite { #[ts(type = "number[]")] pub gids: Vec<u64>, pub slot: Option<u8> }                         // slot None = remove
pub struct EnqueueRequest { pub galleries: Vec<GalleryKey>, pub original: Option<bool> }                               // None = settings default
pub struct TaskId { #[ts(type = "number")] pub id: i64 }
pub struct RemoveDownload { #[ts(type = "number")] pub id: i64, pub delete_files: bool }
pub struct LocalId { pub id: String }
pub struct RemoveLocal { pub id: String, pub delete_files: bool }
pub struct PathRequest { pub path: String }
pub struct CacheCleared { #[ts(type = "number")] pub bytes: u64 }
```

## 3. Commands (`src-tauri`, all `async`)

| Command | `req` type | Returns | Notes |
|---|---|---|---|
| `auth_status` | — | `AuthStatus` | local check (cookies present + igneous validity) |
| `login_password` | `PasswordLogin` | `AuthStatus` | `cloudflare` error if forums is challenged |
| `login_cookies` | `CookieLogin` | `AuthStatus` | verifies online; `loginFailed` if invalid |
| `login_web` | — | `()` | desktop: opens `login` window; result via `auth-changed` event |
| `logout` | — | `AuthStatus` | clears cookies (keeps `nw=1`) |
| `get_settings` | — | `Settings` | |
| `update_settings` | `Settings` | `Settings` | validates + applies (proxy, site, dirs) |
| `list_galleries` | `ListRequest` | `GalleryListPage` | |
| `gallery_detail` | `GalleryKey` | `GalleryDetail` | |
| `gallery_thumbnails` | `ThumbnailRequest` | `ThumbnailPage` | |
| `reload_page_image` | `PageImageKey` | `()` | invalidates the cached image; next fetch uses `nl` |
| `favorite_categories` | — | `FavoriteCategory[]` | login required |
| `set_favorite` | `FavoriteUpdate` | `()` | add / move / edit note / remove |
| `batch_favorites` | `BatchFavorite` | `()` | move many to slot or remove |
| `enqueue_downloads` | `EnqueueRequest` | `DownloadTask[]` | already-queued galleries are returned unchanged |
| `list_downloads` | — | `DownloadTask[]` | newest first |
| `pause_download` | `TaskId` | `()` | |
| `resume_download` | `TaskId` | `()` | also retries failed |
| `remove_download` | `RemoveDownload` | `()` | |
| `pause_all` / `resume_all` | — | `()` | |
| `list_library` | — | `LocalGallery[]` | from DB (fast) |
| `rescan_library` | — | `LocalGallery[]` | rescans roots, emits `library-changed` |
| `local_pages` | `LocalId` | `LocalPages` | |
| `add_library_dir` | `PathRequest` | `Settings` | then rescan |
| `remove_local` | `RemoveLocal` | `()` | |
| `list_history` | — | `HistoryEntry[]` | newest first, max 500 |
| `record_progress` | `ReadProgress` | `()` | |
| `clear_history` | — | `()` | |
| `open_path` | `PathRequest` | `()` | desktop only (reveal folder) |
| `clear_image_cache` | — | `CacheCleared` | bytes freed |

## 4. Events (Rust → JS, `app.emit`)

| Event | Payload |
|---|---|
| `download-progress` | `DownloadProgress` |
| `downloads-changed` | `null` |
| `library-changed` | `null` |
| `auth-changed` | `AuthStatus` |

## 5. `rhimg://` image protocol

The frontend builds URLs only with `rhimgUrl(route)`, which is `convertFileSrc(route, "rhimg")` from
`@tauri-apps/api/core`. It may append `?v=<n>` (outside the encoded route) for cache busting. The Rust
handler takes `request.uri().path()`, strips the leading `/`, percent-decodes it, and matches:

| Route | Meaning |
|---|---|
| `thumb/<base64url-no-pad(remote url)>` | thumbnail or sprite sheet (disk cached) |
| `page/<gid>/<token>/<index>` | online page image, 0-based index (resolved + cached by ImageService) |
| `local/<local_id>/<index>` | local gallery page (folder file or zip entry) |

Success: `200` with the correct `Content-Type` (sniffed from magic bytes). Errors: `404` for an unknown
route or index, and `502` with body `<errorKind>: <message>` for upstream failures. The webview shows a
broken image, and the reader offers a retry.

On Windows and Android, `convertFileSrc` yields `http://rhimg.localhost/<route>`; elsewhere it yields
`rhimg://localhost/<route>`. The handler accepts both.

## 6. CLI (`rhentai-cli`) — QA surface

```
rhentai-cli [--data-dir DIR] [--site eh|ex] [--json] <command>
  status                                       # AuthStatus
  login-cookies --member-id ID --pass-hash HASH [--igneous V]
  login-password --username U --password P
  logout
  list [--source front|popular|watched|toplist-day|toplist-month|toplist-year|toplist-all|favorites[:SLOT]]
       [--query Q] [--exclude-cats doujinshi,manga,...] [--cursor C]
  detail <GALLERY>                             # GALLERY = full URL or "<gid>/<token>"
  thumbs <GALLERY> --page N
  image <GALLERY> --index I [--out FILE] [--reload]   # resolve + download one page image
  favcats
  fav-set <GALLERY> (--slot N [--note TEXT] | --remove)
  download <GALLERY>... [--original] [--concurrency N] [--dir DIR]  # runs DownloadManager until all tasks finish; prints progress lines
  downloads                                    # list tasks
  library scan [--dir DIR]...                  # add dirs to library roots for this run, scan, print LocalGallery list
  library pages <ID>
```

The CLI exits 0 on success. On failure it exits 1 and prints `error[<kind>]: <message>` to stderr. With
`--json` it prints exactly one JSON document to stdout.
