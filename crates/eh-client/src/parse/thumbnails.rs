use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

use crate::error::{EhError, Result};
use crate::model::{PageRef, ThumbSprite, ThumbnailPage};

static PAGE_URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/s/([0-9a-f]{10})/(\d+)-(\d+)").expect("valid regex"));

static SPRITE_STYLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"width:\s*(\d+)px;\s*height:\s*(\d+)px;.*?url\(([^)]+)\)\s+(-?\d+)(?:px)?\s+(-?\d+)(?:px)?",
    )
    .expect("valid regex")
});

static FILE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Page \d+:\s*(.+)$").expect("valid regex"));

/// Parse thumbnail PageRefs from a thumbnail container (such as #gdt)
pub fn parse_thumbnails_from_html(html: &str, page_number: u32) -> Result<ThumbnailPage> {
    let document = Html::parse_document(html);

    let gdt_sel =
        Selector::parse("#gdt").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let gdt = match document.select(&gdt_sel).next() {
        Some(el) => el,
        None => {
            return Ok(ThumbnailPage {
                page: page_number,
                pages: Vec::new(),
            });
        }
    };

    let anchor_sel =
        Selector::parse("a[href]").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let div_sel =
        Selector::parse("div").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let img_sel =
        Selector::parse("img").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut pages = Vec::new();

    for a in gdt.select(&anchor_sel) {
        let href = match a.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        let caps = match PAGE_URL_REGEX.captures(href) {
            Some(c) => c,
            None => continue,
        };

        let page_token = caps[1].to_string();
        let page_1_based: u32 = match caps[3].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let index = page_1_based.saturating_sub(1);

        // Thumbnail sprite
        let mut file_name = None;
        let mut thumb_sprite = None;

        if let Some(div) = a.select(&div_sel).next() {
            if let Some(title) = div.value().attr("title")
                && let Some(name_caps) = FILE_NAME_REGEX.captures(title)
            {
                file_name = Some(name_caps[1].trim().to_string());
            }
            if let Some(style) = div.value().attr("style")
                && let Some(s_caps) = SPRITE_STYLE_REGEX.captures(style)
            {
                let width = s_caps[1].parse().unwrap_or(0);
                let height = s_caps[2].parse().unwrap_or(0);
                let url = s_caps[3].trim_matches('\'').trim_matches('"').to_string();
                let offset_x = s_caps[4].parse().unwrap_or(0);
                let offset_y = s_caps[5].parse().unwrap_or(0);

                thumb_sprite = Some(ThumbSprite {
                    url,
                    width,
                    height,
                    offset_x,
                    offset_y,
                });
            }
        }

        // Legacy fallback: img tag
        if thumb_sprite.is_none()
            && let Some(img) = a.select(&img_sel).next()
        {
            let url = img
                .value()
                .attr("src")
                .or_else(|| img.value().attr("data-src"))
                .unwrap_or("")
                .to_string();
            if let Some(title) = img.value().attr("title")
                && let Some(name_caps) = FILE_NAME_REGEX.captures(title)
            {
                file_name = Some(name_caps[1].trim().to_string());
            }
            thumb_sprite = Some(ThumbSprite {
                url,
                width: 100,
                height: 140,
                offset_x: 0,
                offset_y: 0,
            });
        }

        let thumb = thumb_sprite.unwrap_or(ThumbSprite {
            url: String::new(),
            width: 0,
            height: 0,
            offset_x: 0,
            offset_y: 0,
        });

        pages.push(PageRef {
            index,
            page_token,
            thumb,
            file_name,
        });
    }

    Ok(ThumbnailPage {
        page: page_number,
        pages,
    })
}

/// Parse thumbnail page (?p=N)
pub fn parse_thumbnail_page(html: &str, page: u32) -> Result<ThumbnailPage> {
    parse_thumbnails_from_html(html, page)
}
