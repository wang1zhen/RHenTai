use url::Url;

use crate::error::{EhError, Result};
use crate::model::{Category, ListRequest, ListSource, Site, ToplistPeriod};

/// Calculate excluded categories bitmask for f_cats
pub fn calculate_f_cats(excluded: &[Category]) -> u32 {
    let mut mask = 0;
    for cat in excluded {
        mask |= cat.f_cats_bit();
    }
    mask
}

/// Build URL with query parameters for a ListRequest
pub fn build_list_url(site: Site, req: &ListRequest) -> Result<Url> {
    let base_str = match &req.source {
        ListSource::Front => site.gallery_root().to_string(),
        ListSource::Watched => format!("{}watched", site.gallery_root()),
        ListSource::Popular => format!("{}popular", site.gallery_root()),
        ListSource::Toplist { .. } => site.toplist_url().to_string(),
        ListSource::Favorites { .. } => site.favorites_url().to_string(),
    };

    let mut url = Url::parse(&base_str).map_err(|e| EhError::invalid_input(e.to_string()))?;

    // Popular has no parameters
    if let ListSource::Popular = req.source {
        return Ok(url);
    }

    // Toplist
    if let ListSource::Toplist { period } = &req.source {
        let tl_val = match period {
            ToplistPeriod::Day => "15",
            ToplistPeriod::Month => "13",
            ToplistPeriod::Year => "12",
            ToplistPeriod::All => "11",
        };
        url.query_pairs_mut().append_pair("tl", tl_val);
        if let Some(ref c) = req.cursor
            && let Some(p) = c.strip_prefix("page:")
        {
            url.query_pairs_mut().append_pair("p", p);
        }
        return Ok(url);
    }

    // Favorites
    if let ListSource::Favorites { slot } = &req.source
        && let Some(s) = slot
    {
        url.query_pairs_mut().append_pair("favcat", &s.to_string());
    }

    // Search query
    if !req.filter.keyword.is_empty() {
        url.query_pairs_mut()
            .append_pair("f_search", &req.filter.keyword);
    }

    // Excluded categories
    let f_cats = calculate_f_cats(&req.filter.excluded_categories);
    if f_cats > 0 {
        url.query_pairs_mut()
            .append_pair("f_cats", &f_cats.to_string());
    }

    // Advanced search flags
    if req.filter.show_expunged {
        url.query_pairs_mut().append_pair("f_sh", "on");
    }
    if req.filter.require_torrent {
        url.query_pairs_mut().append_pair("f_sto", "on");
    }
    if let Some(min_r) = req.filter.min_rating
        && (2..=5).contains(&min_r)
    {
        url.query_pairs_mut()
            .append_pair("f_srdd", &min_r.to_string());
    }
    if let Some(min_p) = req.filter.min_pages {
        url.query_pairs_mut()
            .append_pair("f_spf", &min_p.to_string());
    }
    if let Some(max_p) = req.filter.max_pages {
        let should_send = match req.filter.min_pages {
            Some(min_p) => max_p >= min_p,
            None => true,
        };
        if should_send {
            url.query_pairs_mut()
                .append_pair("f_spt", &max_p.to_string());
        }
    }

    // Cursors
    if let Some(ref c) = req.cursor {
        if let Some(gid) = c.strip_prefix("next:") {
            url.query_pairs_mut().append_pair("next", gid);
        } else if let Some(gid) = c.strip_prefix("prev:") {
            url.query_pairs_mut().append_pair("prev", gid);
        } else if let Some(p) = c.strip_prefix("page:") {
            url.query_pairs_mut().append_pair("p", p);
        }
    }

    Ok(url)
}
