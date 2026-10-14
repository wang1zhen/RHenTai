use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::error::{EhError, ErrorKind, Result};
use crate::model::{
    AuthStatus, CookieLogin, FavoriteCategory, FetchedImage, GalleryDetail, GalleryKey,
    GalleryListPage, GallerySummary, ListRequest, PageImage, PageRef, PasswordLogin, RawCookie,
    Site, ThumbnailPage,
};

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

struct Inner {
    site: RwLock<Site>,
    #[allow(dead_code)]
    config: ClientConfig,
}

#[derive(Clone)]
pub struct EhClient {
    inner: Arc<Inner>,
}

impl EhClient {
    pub fn new(cfg: ClientConfig) -> Result<Self> {
        let site = cfg.site;
        Ok(Self {
            inner: Arc::new(Inner {
                site: RwLock::new(site),
                config: cfg,
            }),
        })
    }

    pub fn site(&self) -> Site {
        *self.inner.site.read().unwrap()
    }

    pub fn set_site(&self, site: Site) {
        *self.inner.site.write().unwrap() = site;
    }

    // auth
    pub async fn auth_status(&self, _verify_online: bool) -> Result<AuthStatus> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: auth_status",
        ))
    }

    pub async fn login_password(&self, _req: &PasswordLogin) -> Result<AuthStatus> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: login_password",
        ))
    }

    pub async fn login_cookies(&self, _req: &CookieLogin) -> Result<AuthStatus> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: login_cookies",
        ))
    }

    pub fn import_cookies(&self, _cookies: &[RawCookie]) -> Result<()> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: import_cookies",
        ))
    }

    pub fn logout(&self) -> Result<()> {
        Err(EhError::new(ErrorKind::Internal, "not implemented: logout"))
    }

    // browsing
    pub async fn list(&self, _req: &ListRequest) -> Result<GalleryListPage> {
        Err(EhError::new(ErrorKind::Internal, "not implemented: list"))
    }

    pub async fn gallery_detail(&self, _key: &GalleryKey) -> Result<GalleryDetail> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: gallery_detail",
        ))
    }

    pub async fn thumbnails(&self, _key: &GalleryKey, _page: u32) -> Result<ThumbnailPage> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: thumbnails",
        ))
    }

    pub async fn page_image(
        &self,
        _key: &GalleryKey,
        _page: &PageRef,
        _reload_key: Option<&str>,
    ) -> Result<PageImage> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: page_image",
        ))
    }

    pub async fn fetch_image(&self, _url: &str, _referer: Option<&str>) -> Result<FetchedImage> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: fetch_image",
        ))
    }

    // favorites
    pub async fn favorite_categories(&self) -> Result<Vec<FavoriteCategory>> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: favorite_categories",
        ))
    }

    pub async fn set_favorite(
        &self,
        _key: &GalleryKey,
        _slot: Option<u8>,
        _note: &str,
    ) -> Result<()> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: set_favorite",
        ))
    }

    pub async fn batch_favorites(&self, _gids: &[u64], _slot: Option<u8>) -> Result<()> {
        Err(EhError::new(
            ErrorKind::Internal,
            "not implemented: batch_favorites",
        ))
    }

    // api.php
    pub async fn gdata(&self, _keys: &[GalleryKey]) -> Result<Vec<GallerySummary>> {
        Err(EhError::new(ErrorKind::Internal, "not implemented: gdata"))
    }
}
