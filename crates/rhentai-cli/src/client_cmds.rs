use std::path::PathBuf;

use eh_client::{
    Category, CookieLogin, GalleryKey, ListRequest, ListSource, PasswordLogin, SearchFilter,
    ToplistPeriod,
};
use rhentai_core::{Core, IpcError};

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
    let res = core
        .client
        .auth_status(false)
        .await
        .map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Logged in: {}", res.logged_in);
        println!("Member ID: {}", res.member_id.as_deref().unwrap_or("None"));
        println!("Username:  {}", res.username.as_deref().unwrap_or("None"));
        println!("EX Access: {}", res.ex_access);
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
    let res = core
        .client
        .login_cookies(&req)
        .await
        .map_err(IpcError::from)?;
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
    let res = core
        .client
        .login_password(&req)
        .await
        .map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Login successful: member_id={:?}", res.member_id);
    }
    Ok(())
}

pub async fn logout(core: &Core, json: bool) -> Result<(), IpcError> {
    core.client.logout().map_err(IpcError::from)?;
    let res = core
        .client
        .auth_status(false)
        .await
        .map_err(IpcError::from)?;
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

    let res = core.client.list(&req).await.map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("{:<10} {:<12} {:<5} TITLE", "GID", "CATEGORY", "RATING");
        println!("{:-<10} {:-<12} {:-<5} {:-<50}", "", "", "", "");
        for it in &res.items {
            println!(
                "{:<10} {:<12?} {:<5.2} {}",
                it.gid, it.category, it.rating, it.title
            );
        }
        if let Some(ref n) = res.next {
            println!("Next cursor: {n}");
        }
        if let Some(ref p) = res.prev {
            println!("Prev cursor: {p}");
        }
    }
    Ok(())
}

pub async fn detail(core: &Core, gallery: String, json: bool) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let res = core
        .client
        .gallery_detail(&key)
        .await
        .map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("GID:         {}", res.summary.gid);
        println!("Token:       {}", res.summary.token);
        println!("Title:       {}", res.summary.title);
        if let Some(ref jpn) = res.summary.title_jpn {
            println!("Title (JPN): {}", jpn);
        }
        println!("Category:    {:?}", res.summary.category);
        println!(
            "Uploader:    {}",
            res.summary.uploader.as_deref().unwrap_or("None")
        );
        println!("Pages:       {}", res.summary.file_count);
        println!("Rating:      {:.2}", res.summary.rating);
        println!("Language:    {}", res.language.as_deref().unwrap_or("None"));
        println!(
            "File Size:   {}",
            res.file_size.as_deref().unwrap_or("None")
        );
        println!("Favorited:   {} times", res.favorited_count.unwrap_or(0));
        println!("Thumb Pages: {}", res.thumbnail_page_count);
    }
    Ok(())
}

pub async fn thumbs(core: &Core, gallery: String, page: u32, json: bool) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let res = core
        .client
        .thumbnails(&key, page)
        .await
        .map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("Thumbnail Page {}: {} items", res.page, res.pages.len());
        println!(
            "{:<6} {:<12} {:<24} THUMB URL",
            "INDEX", "PAGE TOKEN", "FILE NAME"
        );
        println!("{:-<6} {:-<12} {:-<24} {:-<40}", "", "", "", "");
        for p in &res.pages {
            println!(
                "{:<6} {:<12} {:<24} {}",
                p.index,
                p.page_token,
                p.file_name.as_deref().unwrap_or("-"),
                p.thumb.url
            );
        }
    }
    Ok(())
}

pub async fn image(
    core: &Core,
    gallery: String,
    index: u32,
    out: Option<PathBuf>,
    reload: bool,
    json: bool,
) -> Result<(), IpcError> {
    let key = parse_gallery_key(&gallery)?;
    let detail = core
        .client
        .gallery_detail(&key)
        .await
        .map_err(IpcError::from)?;

    let page_ref = if let Some(p) = detail.thumbnails.pages.iter().find(|p| p.index == index) {
        p.clone()
    } else {
        let thumbs_per_page = detail.thumbs_per_page.max(1);
        let page_no = index / thumbs_per_page;
        let thumb_page = core
            .client
            .thumbnails(&key, page_no)
            .await
            .map_err(IpcError::from)?;
        thumb_page
            .pages
            .into_iter()
            .find(|p| p.index == index)
            .ok_or_else(|| {
                IpcError::not_found(format!("Page index {index} not found in gallery"))
            })?
    };

    let mut page_img = core
        .client
        .page_image(&key, &page_ref, None)
        .await
        .map_err(IpcError::from)?;

    if reload {
        page_img = core
            .client
            .page_image(&key, &page_ref, page_img.reload_key.as_deref())
            .await
            .map_err(IpcError::from)?;
    }

    let gallery_url = format!(
        "{}/g/{}/{}/",
        core.client.site().gallery_root().trim_end_matches('/'),
        key.gid,
        key.token
    );

    let fetched = core
        .client
        .fetch_image(&page_img.url, Some(&gallery_url))
        .await
        .map_err(IpcError::from)?;

    let path_str = if let Some(ref out_path) = out {
        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(out_path, &fetched.bytes).map_err(|e| IpcError::io(e.to_string()))?;
        out_path.display().to_string()
    } else {
        "<memory>".to_string()
    };

    let content_type = fetched
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let byte_count = fetched.bytes.len();

    if json {
        let obj = serde_json::json!({
            "path": path_str,
            "contentType": content_type,
            "bytes": byte_count,
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        println!("Path:         {path_str}");
        println!("Content-Type: {content_type}");
        println!("Bytes:        {byte_count}");
    }

    Ok(())
}

pub async fn favcats(core: &Core, json: bool) -> Result<(), IpcError> {
    let res = core
        .client
        .favorite_categories()
        .await
        .map_err(IpcError::from)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
    } else {
        println!("{:<5} {:<24} COUNT", "SLOT", "NAME");
        println!("{:-<5} {:-<24} {:-<6}", "", "", "");
        for cat in &res {
            println!("{:<5} {:<24} {}", cat.slot, cat.name, cat.count);
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
    let target_slot = if remove { None } else { slot };
    core.client
        .set_favorite(&key, target_slot, note.as_deref().unwrap_or(""))
        .await
        .map_err(IpcError::from)?;

    if json {
        println!("{{}}");
    } else {
        println!("Favorite updated successfully.");
    }
    Ok(())
}
