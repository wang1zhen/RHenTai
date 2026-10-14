use crate::model::GalleryKey;

/// Build query URL for single favorite popup:
/// /gallerypopups.php?gid={gid}&t={token}&act=addfav
pub fn build_fav_popup_url(base_url: &str, key: &GalleryKey) -> String {
    format!("{base_url}?gid={}&t={}&act=addfav", key.gid, key.token)
}

/// Build form body for single gallery favorite update
pub fn build_favorite_form(slot: Option<u8>, note: &str) -> Vec<(&'static str, String)> {
    match slot {
        Some(s) => vec![
            ("favcat", s.to_string()),
            ("favnote", note.to_string()),
            ("apply", "Add to Favorites".to_string()),
            ("update", "1".to_string()),
        ],
        None => vec![
            ("favcat", "favdel".to_string()),
            ("favnote", String::new()),
            ("apply", "Apply Changes".to_string()),
            ("update", "1".to_string()),
        ],
    }
}

/// Build form body for batch favorites
pub fn build_batch_favorites_form(gids: &[u64], slot: Option<u8>) -> Vec<(String, String)> {
    let mut form = Vec::new();
    let ddact = match slot {
        Some(s) => format!("fav{s}"),
        None => "delete".to_string(),
    };
    form.push(("ddact".to_string(), ddact));
    for gid in gids {
        form.push(("modifygids[]".to_string(), gid.to_string()));
    }
    form.push(("apply".to_string(), "Apply".to_string()));
    form
}
