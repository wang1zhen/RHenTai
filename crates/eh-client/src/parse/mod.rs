//! HTML and JSON parsers for gallery lists, details, thumbnails, images, favorites, and auth.

pub mod api;
pub mod auth;
pub mod date;
pub mod detail;
pub mod favorites;
pub mod image;
pub mod list;
pub mod rating;
pub mod special;
pub mod thumbnails;

pub use api::parse_gdata_response;
pub use auth::{LoginSuccess, parse_login_response};
pub use date::parse_eh_date;
pub use detail::parse_gallery_detail;
pub use favorites::{
    parse_favorite_categories, parse_favorite_slot_from_border, parse_favorite_slot_from_position,
};
pub use image::parse_image_page;
pub use list::parse_gallery_list;
pub use rating::{parse_ir_rating, parse_rating_label};
pub use special::{detect_special_page, is_content_warning};
pub use thumbnails::{parse_thumbnail_page, parse_thumbnails_from_html};
