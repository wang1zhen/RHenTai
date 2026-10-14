use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::error::{EhError, Result};
use crate::model::{Category, GalleryListPage, GallerySummary};
use crate::parse::date::parse_eh_date;
use crate::parse::favorites::parse_favorite_slot_from_border;
use crate::parse::rating::parse_ir_rating;

static GALLERY_URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/g/(\d+)/([0-9a-f]{10})/").expect("valid regex"));

static NEXT_CURSOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"next=([\d-]+)").expect("valid regex"));

static PREV_CURSOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"prev=([\d-]+)").expect("valid regex"));

static PAGE_PARAM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[?&]p=(\d+)").expect("valid regex"));

/// Parse a gallery list HTML page (compact, extended, minimal, thumbnail, popular, toplist, favorites)
pub fn parse_gallery_list(html: &str) -> Result<GalleryListPage> {
    if html.contains("No hits found") {
        return Ok(GalleryListPage {
            items: Vec::new(),
            next: None,
            prev: None,
        });
    }

    let document = Html::parse_document(html);

    // 1. Cursors
    let unext_sel = Selector::parse("a#unext").ok();
    let uprev_sel = Selector::parse("a#uprev").ok();

    let mut next_cursor = None;
    if let Some(ref sel) = unext_sel
        && let Some(a) = document.select(sel).next()
        && let Some(href) = a.value().attr("href")
        && let Some(caps) = NEXT_CURSOR_REGEX.captures(href)
    {
        next_cursor = Some(format!("next:{}", &caps[1]));
    }

    let mut prev_cursor = None;
    if let Some(ref sel) = uprev_sel
        && let Some(a) = document.select(sel).next()
        && let Some(href) = a.value().attr("href")
        && let Some(caps) = PREV_CURSOR_REGEX.captures(href)
    {
        prev_cursor = Some(format!("prev:{}", &caps[1]));
    }

    // Toplist pagination (.ptt)
    if next_cursor.is_none()
        && prev_cursor.is_none()
        && let Ok(ptt_sel) = Selector::parse(".ptt tr")
        && let Some(tr) = document.select(&ptt_sel).next()
        && let Ok(td_sel) = Selector::parse("td")
    {
        let tds: Vec<_> = tr.select(&td_sel).collect();
        for (i, td) in tds.iter().enumerate() {
            let is_current = td
                .value()
                .attr("class")
                .map(|c| c.contains("ptds"))
                .unwrap_or(false);
            if is_current {
                // Check previous
                if i > 0
                    && let Ok(a_sel) = Selector::parse("a")
                    && let Some(a) = tds[i - 1].select(&a_sel).next()
                    && let Some(href) = a.value().attr("href")
                {
                    if let Some(caps) = PAGE_PARAM_REGEX.captures(href) {
                        prev_cursor = Some(format!("page:{}", &caps[1]));
                    } else {
                        // Page 0 typically omits ?p=
                        prev_cursor = Some("page:0".to_string());
                    }
                }
                // Check next
                if i + 1 < tds.len()
                    && let Ok(a_sel) = Selector::parse("a")
                    && let Some(a) = tds[i + 1].select(&a_sel).next()
                    && let Some(href) = a.value().attr("href")
                    && let Some(caps) = PAGE_PARAM_REGEX.captures(href)
                {
                    next_cursor = Some(format!("page:{}", &caps[1]));
                }
                break;
            }
        }
    }

    // 2. Locate .itg container
    let itg_sel =
        Selector::parse(".itg").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let itg = match document.select(&itg_sel).next() {
        Some(el) => el,
        None => {
            // If there is no .itg, return empty list
            return Ok(GalleryListPage {
                items: Vec::new(),
                next: next_cursor,
                prev: prev_cursor,
            });
        }
    };

    // 3. Extract items in document order
    // In table modes, row is <tr> or .gl1c; in thumbnail grid mode, row is .gl1t
    let row_sel = Selector::parse("tr, .gl1t, .gl1c")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let anchor_sel =
        Selector::parse("a[href]").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let glink_sel =
        Selector::parse(".glink").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let cat_sel =
        Selector::parse(".cn, .cs").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let img_sel =
        Selector::parse("img").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let ir_sel =
        Selector::parse(".ir").map_err(|e| EhError::parse(format!("Selector error: {e}")))?;
    let posted_sel = Selector::parse("[id^=posted_]")
        .map_err(|e| EhError::parse(format!("Selector error: {e}")))?;

    let mut items = Vec::new();
    let mut seen_gids = HashSet::new();

    for row in itg.select(&row_sel) {
        // Find anchor with /g/{gid}/{token}/ inside this row
        let mut row_key = None;
        for a in row.select(&anchor_sel) {
            if let Some(href) = a.value().attr("href")
                && let Some(caps) = GALLERY_URL_REGEX.captures(href)
                && let (Ok(gid), Some(tok)) = (caps[1].parse::<u64>(), caps.get(2))
            {
                row_key = Some((gid, tok.as_str().to_string()));
                break;
            }
        }

        let (gid, token) = match row_key {
            Some(k) => k,
            None => continue,
        };

        if !seen_gids.insert(gid) {
            continue;
        }

        // Fallback fields:
        let title = row
            .select(&glink_sel)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        let category = row
            .select(&cat_sel)
            .next()
            .map(|el| {
                let text = el.text().collect::<Vec<_>>().join(" ");
                Category::from_site(&text)
            })
            .unwrap_or(Category::Unknown);

        let thumb = row
            .select(&img_sel)
            .next()
            .and_then(|img| {
                img.value()
                    .attr("data-src")
                    .or_else(|| img.value().attr("src"))
            })
            .unwrap_or("")
            .to_string();

        let rating = row
            .select(&ir_sel)
            .next()
            .and_then(|ir| ir.value().attr("style"))
            .and_then(parse_ir_rating)
            .unwrap_or(0.0);

        let (posted, favorite_slot) = if let Some(posted_el) = row.select(&posted_sel).next() {
            let p_text = posted_el.text().collect::<Vec<_>>().join(" ");
            let p_time = parse_eh_date(&p_text).unwrap_or(0);

            let slot = posted_el
                .value()
                .attr("style")
                .and_then(parse_favorite_slot_from_border);

            (p_time, slot)
        } else {
            (0, None)
        };

        items.push(GallerySummary {
            gid,
            token,
            title,
            title_jpn: None,
            category,
            thumb,
            uploader: None,
            posted,
            file_count: 0,
            rating,
            tags: Vec::new(),
            expunged: false,
            favorite_slot,
            favorite_note: None,
        });
    }

    Ok(GalleryListPage {
        items,
        next: next_cursor,
        prev: prev_cursor,
    })
}
