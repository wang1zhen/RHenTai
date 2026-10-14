use futures::future::BoxFuture;

use eh_client::{
    EhClient, FetchedImage, GalleryDetail, GalleryKey, PageImage, PageRef, Site, ThumbnailPage,
};

use crate::error::{IpcError, Result};

pub trait PageSource: Send + Sync {
    fn site(&self) -> Site;
    fn gallery_detail<'a>(&'a self, key: &'a GalleryKey) -> BoxFuture<'a, Result<GalleryDetail>>;
    fn thumbnails<'a>(
        &'a self,
        key: &'a GalleryKey,
        page: u32,
    ) -> BoxFuture<'a, Result<ThumbnailPage>>;
    fn page_image<'a>(
        &'a self,
        key: &'a GalleryKey,
        page: &'a PageRef,
        reload_key: Option<&'a str>,
    ) -> BoxFuture<'a, Result<PageImage>>;
    fn fetch_image<'a>(
        &'a self,
        url: &'a str,
        referer: Option<&'a str>,
    ) -> BoxFuture<'a, Result<FetchedImage>>;
}

impl PageSource for EhClient {
    fn site(&self) -> Site {
        self.site()
    }

    fn gallery_detail<'a>(&'a self, key: &'a GalleryKey) -> BoxFuture<'a, Result<GalleryDetail>> {
        Box::pin(async move { self.gallery_detail(key).await.map_err(IpcError::from) })
    }

    fn thumbnails<'a>(
        &'a self,
        key: &'a GalleryKey,
        page: u32,
    ) -> BoxFuture<'a, Result<ThumbnailPage>> {
        Box::pin(async move { self.thumbnails(key, page).await.map_err(IpcError::from) })
    }

    fn page_image<'a>(
        &'a self,
        key: &'a GalleryKey,
        page: &'a PageRef,
        reload_key: Option<&'a str>,
    ) -> BoxFuture<'a, Result<PageImage>> {
        Box::pin(async move {
            self.page_image(key, page, reload_key)
                .await
                .map_err(IpcError::from)
        })
    }

    fn fetch_image<'a>(
        &'a self,
        url: &'a str,
        referer: Option<&'a str>,
    ) -> BoxFuture<'a, Result<FetchedImage>> {
        Box::pin(async move { self.fetch_image(url, referer).await.map_err(IpcError::from) })
    }
}
