use crate::model::Site;

impl Site {
    pub fn host(&self) -> &'static str {
        match self {
            Site::Eh => "e-hentai.org",
            Site::Ex => "exhentai.org",
        }
    }

    pub fn gallery_root(&self) -> &'static str {
        match self {
            Site::Eh => "https://e-hentai.org/",
            Site::Ex => "https://exhentai.org/",
        }
    }

    pub fn api_url(&self) -> &'static str {
        match self {
            Site::Eh => "https://api.e-hentai.org/api.php",
            Site::Ex => "https://s.exhentai.org/api.php",
        }
    }

    pub fn favorites_url(&self) -> &'static str {
        match self {
            Site::Eh => "https://e-hentai.org/favorites.php",
            Site::Ex => "https://exhentai.org/favorites.php",
        }
    }

    pub fn fav_popup_url(&self) -> &'static str {
        match self {
            Site::Eh => "https://e-hentai.org/gallerypopups.php",
            Site::Ex => "https://exhentai.org/gallerypopups.php",
        }
    }

    pub fn toplist_url(&self) -> &'static str {
        "https://e-hentai.org/toplist.php"
    }

    pub fn home_url(&self) -> &'static str {
        match self {
            Site::Eh => "https://e-hentai.org/home.php",
            Site::Ex => "https://exhentai.org/home.php",
        }
    }

    pub fn forums_login_url() -> &'static str {
        "https://forums.e-hentai.org/index.php?act=Login&CODE=01"
    }

    pub fn forums_profile_url(member_id: &str) -> String {
        format!("https://forums.e-hentai.org/index.php?showuser={member_id}")
    }
}
