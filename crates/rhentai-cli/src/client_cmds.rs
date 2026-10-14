use std::path::PathBuf;

use eh_client::{
    Category, CookieLogin, GalleryKey, ListRequest, ListSource, PasswordLogin, SearchFilter,
    ToplistPeriod,
};
use rhentai_core::{Core, FavoriteUpdate, IpcError, PageImageKey, ThumbnailRequest};

fn parse_gallery_key(gallery: &str) -> Result<GalleryKey, IpcError> {
    // GALLERY = full URL or "<gid>/<token>"
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

fn parse_list_source(source_str: Option<&str>) -> Result<ListSource, IpcError> {
    match source_str {
        None | Some("front") => Ok(ListSource::Front),
        Some("popular") => Ok(ListSource::Popular),
        Some("watched") => Ok(ListSource::Watched),
        Some("toplist-day") => Ok(ListSource::Toplist {
            period: ToplistPeriod::Day,
        }),
        Some("toplist-month") => Ok(ListSource::Toplist {
            period: ToplistPeriod::Month,
        }),
        Some("toplist-year") => Ok(ListSource::Toplist {
            period: ToplistPeriod::Year,
        }),
        Some("toplist-all") => Ok(ListSource::Toplist {
            period: ToplistPeriod::All,
        }),
        Some(s) if s.starts_with("favorites") => {
            let slot = if let Some(suffix) = s.strip_prefix("favorites:") {
                Some(
                    suffix
                        .parse::<u8>()
                        .map_err(|_| IpcError::invalid_input("Invalid favorites slot number"))?,
                )
            } else {
                None
            };
            Ok(ListSource::Favorites { slot })
        }
        Some(unknown) => Err(IpcError::invalid_input(format!(
            "Unknown list source: {unknown}"
        ))),
    }
}

pub async fn status(core: &Core, json: bool) -> Result<(), IpcError> {
    let res = core.auth_status().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!(
            "Logged in: {}, Member ID: {:?}, EX Access: {}",
            res.logged_in, res.member_id, res.ex_access
        );
    }
    Ok(())
}

pub async fn login_cookies(
    core: &Core,
    member_id: String,
    pass_hash: String,
    igneous: Option<String>,
    json: bool,
) -> Result<(), IpcError> {
    let req = CookieLogin {
        member_id,
        pass_hash,
        igneous,
    };
    let res = core.login_cookies(req).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Login successful: member_id={:?}", res.member_id);
    }
    Ok(())
}

pub async fn login_password(
    core: &Core,
    username: String,
    password: String,
    json: bool,
) -> Result<(), IpcError> {
    let req = PasswordLogin { username, password };
    let res = core.login_password(req).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Login successful: member_id={:?}", res.member_id);
    }
    Ok(())
}

pub async fn logout(core: &Core, json: bool) -> Result<(), IpcError> {
    let res = core.logout().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Logged out successfully.");
    }
    Ok(())
}

pub async fn list(
    core: &Core,
    source: Option<String>,
    query: Option<String>,
    exclude_cats: Option<String>,
    cursor: Option<String>,
    json: bool,
) -> Result<(), IpcError> {
    let list_source = parse_list_source(source.as_deref())?;
    let excluded_categories = if let Some(cats) = exclude_cats {
        cats.split(',')
            .map(|s| Category::from_site(s.trim()))
            .collect()
    } else {
        Vec::new()
    };

    let req = ListRequest {
        source: list_source,
        filter: SearchFilter {
            keyword: query.unwrap_or_default(),
            excluded_categories,
            ..Default::default()
        },
        cursor,
    };

    let res = core.list_galleries(req).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Found {} items", res.items.len());
    }
    Ok(())
}

pub async fn detail(core: &Core, gallery: String, json: bool) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let res = core.gallery_detail(key).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Title: {}", res.summary.title);
    }
    Ok(())
}

pub async fn thumbs(core: &Core, gallery: String, page: u32, json: bool) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let req = ThumbnailRequest {
        gid: key.gid,
        token: key.token,
        page,
    };
    let res = core.gallery_thumbnails(req).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Page {}: {} thumbnails", res.page, res.pages.len());
    }
    Ok(())
}

pub async fn image(
    core: &Core,
    gallery: String,
    index: u32,
    _out: Option<PathBuf>,
    reload: bool,
    _json: bool,
) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    if reload {
        core.reload_page_image(PageImageKey {
            gid: key.gid,
            token: key.token.clone(),
            index,
        })
        .await?;
    }
    // Stub resolution
    Err(IpcError::internal(
        "not implemented: image resolution and download",
    ))
}

pub async fn favcats(core: &Core, json: bool) -> Result<(), IpcError> {
    let res = core.favorite_categories().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        for cat in &res {
            println!("{}: {} ({})", cat.slot, cat.name, cat.count);
        }
    }
    Ok(())
}

pub async fn fav_set(
    core: &Core,
    gallery: String,
    slot: Option<u8>,
    note: Option<String>,
    remove: bool,
    json: bool,
) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let req = FavoriteUpdate {
        gid: key.gid,
        token: key.token,
        slot: if remove { None } else { slot },
        note: note.unwrap_or_default(),
    };
    core.set_favorite(req).await?;
    if json {
        println!("{{}}");
    } else {
        println!("Favorite updated");
    }
    Ok(())
}
