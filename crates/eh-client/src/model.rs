use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Site {
    Eh,
    Ex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Category {
    Doujinshi,
    Manga,
    ArtistCg,
    GameCg,
    Western,
    NonH,
    ImageSet,
    Cosplay,
    AsianPorn,
    Misc,
    Private,
    Unknown,
}

impl Category {
    pub fn from_site(s: &str) -> Self {
        let trimmed = s.trim();
        let normalized: String = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        match normalized.as_str() {
            "doujinshi" => Category::Doujinshi,
            "manga" => Category::Manga,
            "artistcg" => Category::ArtistCg,
            "gamecg" => Category::GameCg,
            "western" => Category::Western,
            "nonh" => Category::NonH,
            "imageset" => Category::ImageSet,
            "cosplay" => Category::Cosplay,
            "asianporn" => Category::AsianPorn,
            "misc" => Category::Misc,
            "private" => Category::Private,
            _ => Category::Unknown,
        }
    }

    pub fn f_cats_bit(self) -> u32 {
        match self {
            Category::Misc => 1,
            Category::Doujinshi => 2,
            Category::Manga => 4,
            Category::ArtistCg => 8,
            Category::GameCg => 16,
            Category::ImageSet => 32,
            Category::Cosplay => 64,
            Category::AsianPorn => 128,
            Category::NonH => 256,
            Category::Western => 512,
            Category::Private | Category::Unknown => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GalleryKey {
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Tag {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TagGroup {
    pub namespace: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GallerySummary {
    #[ts(type = "number")]
    pub gid: u64,
    pub token: String,
    pub title: String,
    pub title_jpn: Option<String>,
    pub category: Category,
    pub thumb: String,
    pub uploader: Option<String>,
    #[ts(type = "number")]
    pub posted: i64,
    pub file_count: u32,
    pub rating: f32,
    pub tags: Vec<Tag>,
    pub expunged: bool,
    pub favorite_slot: Option<u8>,
    pub favorite_note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ToplistPeriod {
    Day,
    Month,
    Year,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ListSource {
    Front,
    Watched,
    Popular,
    Toplist { period: ToplistPeriod },
    Favorites { slot: Option<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchFilter {
    pub keyword: String,
    pub excluded_categories: Vec<Category>,
    pub min_rating: Option<u8>,
    pub min_pages: Option<u32>,
    pub max_pages: Option<u32>,
    pub show_expunged: bool,
    pub require_torrent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListRequest {
    pub source: ListSource,
    pub filter: SearchFilter,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GalleryListPage {
    pub items: Vec<GallerySummary>,
    pub next: Option<String>,
    pub prev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ThumbSprite {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PageRef {
    pub index: u32,
    pub page_token: String,
    pub thumb: ThumbSprite,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ThumbnailPage {
    pub page: u32,
    pub pages: Vec<PageRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Comment {
    pub author: String,
    pub posted: String,
    pub score: Option<String>,
    pub body: String,
    pub is_uploader: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GalleryDetail {
    pub summary: GallerySummary,
    pub language: Option<String>,
    pub file_size: Option<String>,
    pub favorited_count: Option<u32>,
    pub rating_count: Option<u32>,
    pub parent: Option<GalleryKey>,
    pub tag_groups: Vec<TagGroup>,
    pub thumbnails: ThumbnailPage,
    pub thumbnail_page_count: u32,
    pub thumbs_per_page: u32,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PageImage {
    pub index: u32,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub reload_key: Option<String>,
    pub original_url: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FavoriteCategory {
    pub slot: u8,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub member_id: Option<String>,
    pub username: Option<String>,
    pub ex_access: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CookieLogin {
    pub member_id: String,
    pub pass_hash: String,
    pub igneous: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PasswordLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FetchedImage {
    pub bytes: bytes::Bytes,
    pub content_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_site() {
        assert_eq!(Category::from_site("Doujinshi"), Category::Doujinshi);
        assert_eq!(Category::from_site("doujinshi"), Category::Doujinshi);
        assert_eq!(Category::from_site("Manga"), Category::Manga);
        assert_eq!(Category::from_site("Artist CG"), Category::ArtistCg);
        assert_eq!(Category::from_site("artistcg"), Category::ArtistCg);
        assert_eq!(Category::from_site("Game CG"), Category::GameCg);
        assert_eq!(Category::from_site("Western"), Category::Western);
        assert_eq!(Category::from_site("Non-H"), Category::NonH);
        assert_eq!(Category::from_site("Image Set"), Category::ImageSet);
        assert_eq!(Category::from_site("Cosplay"), Category::Cosplay);
        assert_eq!(Category::from_site("Asian Porn"), Category::AsianPorn);
        assert_eq!(Category::from_site("Misc"), Category::Misc);
        assert_eq!(Category::from_site("Private"), Category::Private);
        assert_eq!(Category::from_site("something_else"), Category::Unknown);
    }

    #[test]
    fn test_category_f_cats_bit() {
        assert_eq!(Category::Misc.f_cats_bit(), 1);
        assert_eq!(Category::Doujinshi.f_cats_bit(), 2);
        assert_eq!(Category::Manga.f_cats_bit(), 4);
        assert_eq!(Category::ArtistCg.f_cats_bit(), 8);
        assert_eq!(Category::GameCg.f_cats_bit(), 16);
        assert_eq!(Category::ImageSet.f_cats_bit(), 32);
        assert_eq!(Category::Cosplay.f_cats_bit(), 64);
        assert_eq!(Category::AsianPorn.f_cats_bit(), 128);
        assert_eq!(Category::NonH.f_cats_bit(), 256);
        assert_eq!(Category::Western.f_cats_bit(), 512);
        assert_eq!(Category::Private.f_cats_bit(), 0);
        assert_eq!(Category::Unknown.f_cats_bit(), 0);
    }
}
