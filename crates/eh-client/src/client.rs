use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use url::Url;

use crate::auth::{is_valid_igneous, parse_username_from_profile};
use crate::error::{EhError, Result};
use crate::favorites::{build_batch_favorites_form, build_fav_popup_url, build_favorite_form};
use crate::list::build_list_url;
use crate::model::{
    AuthStatus, CookieLogin, FavoriteCategory, FetchedImage, GalleryDetail, GalleryKey,
    GalleryListPage, GallerySummary, ListRequest, PageImage, PageRef, PasswordLogin, RawCookie,
    Site, ThumbnailPage,
};
use crate::parse::{
    detect_special_page, is_content_warning, parse_favorite_categories, parse_gallery_detail,
    parse_gallery_list, parse_gdata_response, parse_image_page, parse_login_response,
    parse_thumbnail_page,
};

pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
    pub cookie_path: Option<PathBuf>,
    pub site: Site,
    pub min_request_interval_ms: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            user_agent: None,
            cookie_path: None,
            site: Site::Eh,
            min_request_interval_ms: 350,
        }
    }
}

/// Thread-safe rate limiter: at least min_interval_ms between request starts and at most 3 in flight.
#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<tokio::sync::Semaphore>,
    last_start: Arc<tokio::sync::Mutex<tokio::time::Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        let interval = Duration::from_millis(min_interval_ms);
        let start_time = tokio::time::Instant::now()
            .checked_sub(interval)
            .unwrap_or_else(tokio::time::Instant::now);

        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(3)),
            last_start: Arc::new(tokio::sync::Mutex::new(start_time)),
            min_interval: interval,
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| EhError::internal(e.to_string()))?;

        let mut last = self.last_start.lock().await;
        let now = tokio::time::Instant::now();
        if now < *last {
            tokio::time::sleep(*last - now).await;
            let actual_wake = tokio::time::Instant::now();
            *last = actual_wake + self.min_interval;
        } else {
            *last = now + self.min_interval;
        }
        drop(last);

        Ok(permit)
    }
}

pub struct PersistentCookieStore {
    pub store: Arc<RwLock<cookie_store::CookieStore>>,
    pub cookie_path: Option<PathBuf>,
}

impl reqwest::cookie::CookieStore for PersistentCookieStore {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
        url: &Url,
    ) {
        let mut store = self.store.write().unwrap();
        let cookies = cookie_headers.filter_map(|val| {
            std::str::from_utf8(val.as_bytes())
                .map_err(cookie_store::RawCookieParseError::from)
                .and_then(cookie_store::RawCookie::parse)
                .map(|c| c.into_owned())
                .ok()
        });
        store.store_response_cookies(cookies, url);
        if let Some(ref path) = self.cookie_path {
            let _ = save_cookies(&store, path);
        }
    }

    fn cookies(&self, url: &Url) -> Option<reqwest::header::HeaderValue> {
        let store = self.store.read().unwrap();
        let s = store
            .get_request_values(url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        if s.is_empty() {
            None
        } else {
            reqwest::header::HeaderValue::from_maybe_shared(bytes::Bytes::from(s)).ok()
        }
    }
}

fn save_cookies(store: &cookie_store::CookieStore, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    #[cfg(unix)]
    let mut options = std::fs::OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.write(true).create(true).truncate(true).mode(0o600);
    }
    #[cfg(not(unix))]
    let mut options = std::fs::OpenOptions::new();
    #[cfg(not(unix))]
    options.write(true).create(true).truncate(true);

    let file = options.open(path).map_err(|e| EhError::io(e.to_string()))?;
    let mut writer = std::io::BufWriter::new(file);
    cookie_store::serde::json::save(store, &mut writer).map_err(|e| EhError::io(e.to_string()))?;
    drop(writer);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }

    Ok(())
}

fn load_cookies(path: &Path) -> Option<cookie_store::CookieStore> {
    if !path.exists() {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    cookie_store::serde::json::load(reader).ok()
}

fn ensure_default_cookies(store: &mut cookie_store::CookieStore) {
    if let Ok(eh_url) = Url::parse("https://e-hentai.org/") {
        let _ = store.parse(
            "nw=1; Domain=.e-hentai.org; Path=/; Max-Age=31536000",
            &eh_url,
        );
    }
    if let Ok(ex_url) = Url::parse("https://exhentai.org/") {
        let _ = store.parse(
            "nw=1; Domain=.exhentai.org; Path=/; Max-Age=31536000",
            &ex_url,
        );
    }
}

pub(crate) fn remove_credential_cookies_from_store(store: &mut cookie_store::CookieStore) {
    let credential_names = ["ipb_member_id", "ipb_pass_hash", "igneous"];
    let domains = [
        "e-hentai.org",
        ".e-hentai.org",
        "exhentai.org",
        ".exhentai.org",
        "forums.e-hentai.org",
        ".forums.e-hentai.org",
    ];

    for domain in domains {
        for path in ["/", ""] {
            for name in credential_names {
                store.remove(domain, path, name);
            }
        }
    }

    let to_remove: Vec<(String, String, String)> = store
        .iter_any()
        .filter(|c| {
            let n = c.name();
            n == "ipb_member_id" || n == "ipb_pass_hash" || n == "igneous"
        })
        .map(|c| {
            (
                String::from(&c.domain),
                c.path.to_string(),
                c.name().to_string(),
            )
        })
        .collect();

    for (d, p, n) in to_remove {
        store.remove(&d, &p, &n);
    }
}

fn build_reqwest_client(
    proxy: Option<&str>,
    user_agent: &str,
    cookie_store: Arc<PersistentCookieStore>,
    request_timeout: Duration,
    follow_redirects: bool,
) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent(user_agent)
        .connect_timeout(Duration::from_secs(15))
        .timeout(request_timeout)
        .gzip(true)
        .brotli(true)
        .cookie_provider(cookie_store);

    if !follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    if let Some(proxy_str) = proxy
        && !proxy_str.trim().is_empty()
    {
        let p =
            reqwest::Proxy::all(proxy_str).map_err(|e| EhError::invalid_input(e.to_string()))?;
        builder = builder.proxy(p);
    }

    builder.build().map_err(|e| EhError::network(e.to_string()))
}

struct Inner {
    site: RwLock<Site>,
    config: RwLock<ClientConfig>,
    cookie_store: Arc<RwLock<cookie_store::CookieStore>>,
    persistent_jar: Arc<PersistentCookieStore>,
    http_client: RwLock<reqwest::Client>,
    no_redirect_client: RwLock<reqwest::Client>,
    image_client: RwLock<reqwest::Client>,
    rate_limiter: RateLimiter,
}

#[derive(Clone)]
pub struct EhClient {
    inner: Arc<Inner>,
}

impl EhClient {
    pub fn new(cfg: ClientConfig) -> Result<Self> {
        let mut store = if let Some(ref path) = cfg.cookie_path {
            load_cookies(path).unwrap_or_default()
        } else {
            cookie_store::CookieStore::default()
        };

        ensure_default_cookies(&mut store);
        if let Some(ref path) = cfg.cookie_path {
            let _ = save_cookies(&store, path);
        }

        let cookie_store = Arc::new(RwLock::new(store));
        let persistent_jar = Arc::new(PersistentCookieStore {
            store: cookie_store.clone(),
            cookie_path: cfg.cookie_path.clone(),
        });

        let ua = cfg.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT);

        let http_client = build_reqwest_client(
            cfg.proxy.as_deref(),
            ua,
            persistent_jar.clone(),
            Duration::from_secs(30),
            true,
        )?;

        let no_redirect_client = build_reqwest_client(
            cfg.proxy.as_deref(),
            ua,
            persistent_jar.clone(),
            Duration::from_secs(30),
            false,
        )?;

        let image_client = build_reqwest_client(
            cfg.proxy.as_deref(),
            ua,
            persistent_jar.clone(),
            Duration::from_secs(60),
            true,
        )?;

        let rate_limiter = RateLimiter::new(cfg.min_request_interval_ms);
        let site = cfg.site;

        Ok(Self {
            inner: Arc::new(Inner {
                site: RwLock::new(site),
                config: RwLock::new(cfg),
                cookie_store,
                persistent_jar,
                http_client: RwLock::new(http_client),
                no_redirect_client: RwLock::new(no_redirect_client),
                image_client: RwLock::new(image_client),
                rate_limiter,
            }),
        })
    }

    pub fn site(&self) -> Site {
        *self.inner.site.read().unwrap()
    }

    pub fn set_site(&self, site: Site) {
        *self.inner.site.write().unwrap() = site;
    }

    /// Replace the proxy used by all subsequent requests. The HTTP client is rebuilt and the cookie
    /// jar is kept.
    pub fn set_proxy(&self, proxy: Option<String>) -> Result<()> {
        let ua = {
            let mut cfg = self.inner.config.write().unwrap();
            cfg.proxy = proxy.clone();
            cfg.user_agent
                .clone()
                .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string())
        };

        let http = build_reqwest_client(
            proxy.as_deref(),
            &ua,
            self.inner.persistent_jar.clone(),
            Duration::from_secs(30),
            true,
        )?;

        let no_redirect = build_reqwest_client(
            proxy.as_deref(),
            &ua,
            self.inner.persistent_jar.clone(),
            Duration::from_secs(30),
            false,
        )?;

        let image = build_reqwest_client(
            proxy.as_deref(),
            &ua,
            self.inner.persistent_jar.clone(),
            Duration::from_secs(60),
            true,
        )?;

        *self.inner.http_client.write().unwrap() = http;
        *self.inner.no_redirect_client.write().unwrap() = no_redirect;
        *self.inner.image_client.write().unwrap() = image;

        Ok(())
    }

    pub async fn request_html(&self, url: &str, referer: Option<&str>) -> Result<String> {
        self.request_html_inner(url, referer, false).await
    }

    async fn request_html_inner(
        &self,
        url: &str,
        referer: Option<&str>,
        retried_warning: bool,
    ) -> Result<String> {
        let _permit = self.inner.rate_limiter.acquire().await?;
        let client = self.inner.http_client.read().unwrap().clone();

        let mut req = client.get(url);
        if let Some(ref_url) = referer {
            req = req.header(reqwest::header::REFERER, ref_url);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let final_url = resp.url().to_string();
        let body = resp
            .text()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        detect_special_page(status, &headers, &final_url, &body)?;

        // Content warning retry: ?nw=session
        if is_content_warning(&body) {
            if retried_warning {
                return Err(EhError::parse(
                    "Content warning bypassed with nw=session, but warning page persisted",
                ));
            }
            drop(_permit);
            let separator = if url.contains('?') { '&' } else { '?' };
            let retry_url = format!("{url}{separator}nw=session");
            return Box::pin(self.request_html_inner(&retry_url, referer, true)).await;
        }

        Ok(body)
    }

    // auth (docs/EH_PROTOCOL.md §2)
    pub async fn auth_status(&self, verify_online: bool) -> Result<AuthStatus> {
        let (member_id, has_pass, igneous) = {
            let store = self.inner.cookie_store.read().unwrap();
            let mut mid = None;
            let mut pass = false;
            let mut ign = None;

            if let Ok(eh_url) = Url::parse("https://e-hentai.org/") {
                for (name, val) in store.get_request_values(&eh_url) {
                    if name == "ipb_member_id" {
                        mid = Some(val.to_string());
                    } else if name == "ipb_pass_hash" {
                        pass = true;
                    }
                }
            }

            if let Ok(ex_url) = Url::parse("https://exhentai.org/") {
                for (name, val) in store.get_request_values(&ex_url) {
                    if name == "ipb_member_id" && mid.is_none() {
                        mid = Some(val.to_string());
                    } else if name == "ipb_pass_hash" {
                        pass = true;
                    } else if name == "igneous" {
                        ign = Some(val.to_string());
                    }
                }
            }

            (mid, pass, ign)
        };

        let ex_access_cookie = igneous.as_deref().map(is_valid_igneous).unwrap_or(false);

        if !verify_online {
            let logged_in = member_id.is_some() && has_pass;
            return Ok(AuthStatus {
                logged_in,
                member_id,
                username: None,
                ex_access: logged_in && ex_access_cookie,
            });
        }

        if member_id.is_none() || !has_pass {
            return Ok(AuthStatus {
                logged_in: false,
                member_id: None,
                username: None,
                ex_access: false,
            });
        }

        // Verify online: GET https://e-hentai.org/home.php without following redirects
        let client_no_redir = self.inner.no_redirect_client.read().unwrap().clone();
        let resp = {
            let _permit = self.inner.rate_limiter.acquire().await?;
            client_no_redir
                .get("https://e-hentai.org/home.php")
                .send()
                .await
                .map_err(|e| EhError::network(e.to_string()))?
        };

        if resp.status().is_redirection()
            && let Some(loc) = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
            && loc.contains("bounce_login.php")
        {
            return Ok(AuthStatus {
                logged_in: false,
                member_id: None,
                username: None,
                ex_access: false,
            });
        }

        if !resp.status().is_success() {
            return Ok(AuthStatus {
                logged_in: false,
                member_id: None,
                username: None,
                ex_access: false,
            });
        }

        let logged_in = true;

        // Check EX access online
        let mut ex_access = ex_access_cookie;
        let http_client = self.inner.http_client.read().unwrap().clone();
        {
            let _permit = self.inner.rate_limiter.acquire().await?;
            if let Ok(ex_resp) = http_client.get("https://exhentai.org/").send().await {
                let status = ex_resp.status();
                let body = ex_resp.text().await.unwrap_or_default();
                ex_access = status.is_success()
                    && detect_special_page(
                        status,
                        &reqwest::header::HeaderMap::new(),
                        "https://exhentai.org/",
                        &body,
                    )
                    .is_ok();
            }
        }

        // Fetch username (best effort)
        let mut username = None;
        if let Some(ref mid) = member_id {
            let profile_url = Site::forums_profile_url(mid);
            let _permit = self.inner.rate_limiter.acquire().await?;
            if let Ok(u_resp) = http_client.get(&profile_url).send().await
                && let Ok(u_html) = u_resp.text().await
            {
                username = parse_username_from_profile(&u_html);
            }
        }

        Ok(AuthStatus {
            logged_in,
            member_id,
            username,
            ex_access,
        })
    }

    pub async fn login_password(&self, req: &PasswordLogin) -> Result<AuthStatus> {
        let client = self.inner.http_client.read().unwrap().clone();
        let _permit = self.inner.rate_limiter.acquire().await?;

        let form = [
            ("referer", "https://forums.e-hentai.org/index.php"),
            ("b", ""),
            ("bt", ""),
            ("UserName", &req.username),
            ("PassWord", &req.password),
            ("CookieDate", "365"),
        ];

        let form_body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&form)
            .finish();

        let resp = client
            .post(Site::forums_login_url())
            .header(reqwest::header::ORIGIN, "https://forums.e-hentai.org")
            .header(
                reqwest::header::REFERER,
                "https://forums.e-hentai.org/index.php?act=Login&CODE=00",
            )
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(form_body)
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;
        drop(_permit);

        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp
            .text()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        let login_res = parse_login_response(status, &headers, &body)?;

        // Store cookies
        self.persist_login_cookies(&login_res.member_id, &login_res.pass_hash, None)?;

        self.auth_status(true).await
    }

    pub async fn login_cookies(&self, req: &CookieLogin) -> Result<AuthStatus> {
        let snapshot = self.snapshot_credential_cookies();
        self.persist_login_cookies(&req.member_id, &req.pass_hash, req.igneous.as_deref())?;

        let verify_res = self.verify_login_request().await;
        match verify_res {
            Ok(()) => self.auth_status(true).await,
            Err(e) => {
                let _ = self.restore_credential_cookies(&snapshot);
                Err(e)
            }
        }
    }

    async fn verify_login_request(&self) -> Result<()> {
        let client_no_redir = self.inner.no_redirect_client.read().unwrap().clone();
        let _permit = self.inner.rate_limiter.acquire().await?;
        let resp = client_no_redir
            .get("https://e-hentai.org/home.php")
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;
        drop(_permit);

        let status = resp.status();
        let headers = resp.headers().clone();
        let final_url = resp.url().to_string();

        if status.is_redirection() {
            if let Some(loc) = headers
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                && loc.contains("bounce_login.php")
            {
                return Err(EhError::login_failed(
                    "Invalid credentials: redirected to bounce_login.php",
                ));
            }
            return Err(EhError::login_failed(
                "Login verification redirected unexpectedly",
            ));
        }

        if !status.is_success() {
            return Err(EhError::login_failed(format!(
                "Login verification returned HTTP {}",
                status
            )));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        detect_special_page(status, &headers, &final_url, &body)?;

        Ok(())
    }

    pub(crate) fn snapshot_credential_cookies(&self) -> Vec<RawCookie> {
        let store = self.inner.cookie_store.read().unwrap();
        let credential_names = ["ipb_member_id", "ipb_pass_hash", "igneous"];
        store
            .iter_any()
            .filter(|c| credential_names.contains(&c.name()))
            .map(|c| RawCookie {
                name: c.name().to_string(),
                value: c.value().to_string(),
                domain: String::from(&c.domain),
                path: c.path.to_string(),
            })
            .collect()
    }

    pub(crate) fn restore_credential_cookies(&self, snapshot: &[RawCookie]) -> Result<()> {
        let eh_url =
            Url::parse("https://e-hentai.org/").map_err(|e| EhError::internal(e.to_string()))?;
        {
            let mut store = self.inner.cookie_store.write().unwrap();
            remove_credential_cookies_from_store(&mut store);
            for c in snapshot {
                let path = if c.path.is_empty() { "/" } else { &c.path };
                let cookie_str = format!(
                    "{}={}; Domain={}; Path={}; Max-Age=31536000",
                    c.name, c.value, c.domain, path
                );
                let domain_clean = c.domain.trim_start_matches('.');
                let target_url = Url::parse(&format!("https://{domain_clean}/"))
                    .unwrap_or_else(|_| eh_url.clone());
                let _ = store.parse(&cookie_str, &target_url);
            }
            ensure_default_cookies(&mut store);

            if let Some(ref path) = self.inner.persistent_jar.cookie_path {
                save_cookies(&store, path)?;
            }
        }
        Ok(())
    }

    pub fn remove_credential_cookies(&self) -> Result<()> {
        {
            let mut store = self.inner.cookie_store.write().unwrap();
            remove_credential_cookies_from_store(&mut store);
            ensure_default_cookies(&mut store);

            if let Some(ref path) = self.inner.persistent_jar.cookie_path {
                save_cookies(&store, path)?;
            }
        }
        Ok(())
    }

    pub(crate) fn persist_login_cookies(
        &self,
        member_id: &str,
        pass_hash: &str,
        igneous: Option<&str>,
    ) -> Result<()> {
        let eh_url =
            Url::parse("https://e-hentai.org/").map_err(|e| EhError::internal(e.to_string()))?;
        let ex_url =
            Url::parse("https://exhentai.org/").map_err(|e| EhError::internal(e.to_string()))?;

        {
            let mut store = self.inner.cookie_store.write().unwrap();
            let _ = store.parse(
                &format!(
                    "ipb_member_id={member_id}; Domain=.e-hentai.org; Path=/; Max-Age=31536000"
                ),
                &eh_url,
            );
            let _ = store.parse(
                &format!(
                    "ipb_member_id={member_id}; Domain=.exhentai.org; Path=/; Max-Age=31536000"
                ),
                &ex_url,
            );
            let _ = store.parse(
                &format!(
                    "ipb_pass_hash={pass_hash}; Domain=.e-hentai.org; Path=/; Max-Age=31536000"
                ),
                &eh_url,
            );
            let _ = store.parse(
                &format!(
                    "ipb_pass_hash={pass_hash}; Domain=.exhentai.org; Path=/; Max-Age=31536000"
                ),
                &ex_url,
            );

            if let Some(ign) = igneous
                && !ign.is_empty()
            {
                let _ = store.parse(
                    &format!("igneous={ign}; Domain=.exhentai.org; Path=/; Max-Age=31536000"),
                    &ex_url,
                );
            }
            ensure_default_cookies(&mut store);

            if let Some(ref path) = self.inner.persistent_jar.cookie_path {
                save_cookies(&store, path)?;
            }
        }

        Ok(())
    }

    pub fn import_cookies(&self, cookies: &[RawCookie]) -> Result<()> {
        let eh_url =
            Url::parse("https://e-hentai.org/").map_err(|e| EhError::internal(e.to_string()))?;
        {
            let mut store = self.inner.cookie_store.write().unwrap();
            for c in cookies {
                let path = if c.path.is_empty() { "/" } else { &c.path };
                let cookie_str = format!(
                    "{}={}; Domain={}; Path={}; Max-Age=31536000",
                    c.name, c.value, c.domain, path
                );
                let domain_clean = c.domain.trim_start_matches('.');
                let target_url = Url::parse(&format!("https://{domain_clean}/"))
                    .unwrap_or_else(|_| eh_url.clone());
                let _ = store.parse(&cookie_str, &target_url);
            }
            ensure_default_cookies(&mut store);

            if let Some(ref path) = self.inner.persistent_jar.cookie_path {
                save_cookies(&store, path)?;
            }
        }
        Ok(())
    }

    pub fn logout(&self) -> Result<()> {
        {
            let mut store = self.inner.cookie_store.write().unwrap();
            store.clear();
            ensure_default_cookies(&mut store);

            if let Some(ref path) = self.inner.persistent_jar.cookie_path {
                save_cookies(&store, path)?;
            }
        }
        Ok(())
    }

    // browsing
    pub async fn list(&self, req: &ListRequest) -> Result<GalleryListPage> {
        let url = build_list_url(self.site(), req)?;
        let html = self.request_html(url.as_str(), None).await?;
        let mut list_page = parse_gallery_list(&html)?;

        if list_page.items.is_empty() {
            return Ok(list_page);
        }

        // gdata enrichment in chunks of 25, preserving order
        let keys: Vec<GalleryKey> = list_page
            .items
            .iter()
            .map(|it| GalleryKey {
                gid: it.gid,
                token: it.token.clone(),
            })
            .collect();

        if let Ok(enriched) = self.gdata(&keys).await {
            for item in &mut list_page.items {
                if let Some(g) = enriched.iter().find(|e| e.gid == item.gid) {
                    item.title = g.title.clone();
                    item.title_jpn = g.title_jpn.clone();
                    item.category = g.category;
                    item.thumb = g.thumb.clone();
                    item.uploader = g.uploader.clone();
                    item.posted = g.posted;
                    item.file_count = g.file_count;
                    item.rating = g.rating;
                    item.tags = g.tags.clone();
                    item.expunged = g.expunged;
                }
            }
        }

        Ok(list_page)
    }

    pub async fn gallery_detail(&self, key: &GalleryKey) -> Result<GalleryDetail> {
        let site_root = self.site().gallery_root();
        let url = format!(
            "{}/g/{}/{}/",
            site_root.trim_end_matches('/'),
            key.gid,
            key.token
        );
        let html = self.request_html(&url, None).await?;
        parse_gallery_detail(&html, key)
    }

    pub async fn thumbnails(&self, key: &GalleryKey, page: u32) -> Result<ThumbnailPage> {
        let site_root = self.site().gallery_root();
        let url = format!(
            "{}/g/{}/{}/?p={}",
            site_root.trim_end_matches('/'),
            key.gid,
            key.token,
            page
        );
        let html = self.request_html(&url, None).await?;
        parse_thumbnail_page(&html, page)
    }

    pub async fn page_image(
        &self,
        key: &GalleryKey,
        page: &PageRef,
        reload_key: Option<&str>,
    ) -> Result<PageImage> {
        let site_root = self.site().gallery_root().trim_end_matches('/');
        let mut url = format!(
            "{site_root}/s/{}/{}-{}",
            page.page_token,
            key.gid,
            page.index + 1
        );
        if let Some(nl) = reload_key {
            url = format!("{url}?nl={nl}");
        }
        let gallery_url = format!("{site_root}/g/{}/{}/", key.gid, key.token);
        let html = self.request_html(&url, Some(&gallery_url)).await?;
        parse_image_page(&html, page.index)
    }

    pub async fn fetch_image(&self, url: &str, referer: Option<&str>) -> Result<FetchedImage> {
        let client = self.inner.image_client.read().unwrap().clone();
        let mut req = client.get(url);
        if let Some(ref_url) = referer {
            req = req.header(reqwest::header::REFERER, ref_url);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;
        let status = resp.status();
        let final_url = resp.url().to_string();

        if final_url.ends_with("/509.gif") || final_url.contains("/509.gif") {
            return Err(EhError::image_limit("Viewing limits exceeded (509.gif)"));
        }

        if !status.is_success() {
            return Err(EhError::http(format!("HTTP status {status}")));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let Some(ref ct) = content_type
            && ct.starts_with("text/html")
        {
            return Err(EhError::http("Expected image, received text/html"));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        if bytes.starts_with(b"<!DOCTYPE") || bytes.starts_with(b"<html") {
            return Err(EhError::http("Expected image, received HTML document"));
        }

        Ok(FetchedImage {
            bytes,
            content_type,
        })
    }

    // favorites
    pub async fn favorite_categories(&self) -> Result<Vec<FavoriteCategory>> {
        let url = self.site().favorites_url();
        let html = self.request_html(url, None).await?;
        parse_favorite_categories(&html)
    }

    pub async fn set_favorite(&self, key: &GalleryKey, slot: Option<u8>, note: &str) -> Result<()> {
        let _permit = self.inner.rate_limiter.acquire().await?;
        let client = self.inner.http_client.read().unwrap().clone();
        let popup_url = build_fav_popup_url(self.site().fav_popup_url(), key);
        let form = build_favorite_form(slot, note);
        let form_body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&form)
            .finish();

        let resp = client
            .post(&popup_url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(form_body)
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let final_url = resp.url().to_string();
        let body = resp
            .text()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        detect_special_page(status, &headers, &final_url, &body)?;
        Ok(())
    }

    pub async fn batch_favorites(&self, gids: &[u64], slot: Option<u8>) -> Result<()> {
        let _permit = self.inner.rate_limiter.acquire().await?;
        let client = self.inner.http_client.read().unwrap().clone();
        let fav_url = self.site().favorites_url();
        let form = build_batch_favorites_form(gids, slot);
        let form_body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(form.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .finish();

        let resp = client
            .post(fav_url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(form_body)
            .send()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let final_url = resp.url().to_string();
        let body = resp
            .text()
            .await
            .map_err(|e| EhError::network(e.to_string()))?;

        detect_special_page(status, &headers, &final_url, &body)?;
        Ok(())
    }

    // api.php
    pub async fn gdata(&self, keys: &[GalleryKey]) -> Result<Vec<GallerySummary>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let api_url = self.site().api_url();
        let mut results = Vec::with_capacity(keys.len());

        for chunk in keys.chunks(25) {
            let _permit = self.inner.rate_limiter.acquire().await?;
            let client = self.inner.http_client.read().unwrap().clone();

            let req_body = serde_json::json!({
                "method": "gdata",
                "gidlist": chunk.iter().map(|k| (k.gid, &k.token)).collect::<Vec<_>>(),
                "namespace": 1
            });

            let resp = client
                .post(api_url)
                .json(&req_body)
                .send()
                .await
                .map_err(|e| EhError::network(e.to_string()))?;

            let status = resp.status();
            let headers = resp.headers().clone();
            let final_url = resp.url().to_string();
            let body = resp
                .text()
                .await
                .map_err(|e| EhError::network(e.to_string()))?;

            detect_special_page(status, &headers, &final_url, &body)?;

            let summaries = parse_gdata_response(&body)?;
            results.extend(summaries);
        }

        // Preserve order
        let mut ordered = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(s) = results.iter().find(|s| s.gid == key.gid) {
                ordered.push(s.clone());
            }
        }

        Ok(ordered)
    }
}
