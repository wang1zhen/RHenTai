use eh_client::favorites::{build_batch_favorites_form, build_fav_popup_url, build_favorite_form};
use eh_client::list::{build_list_url, calculate_f_cats};
use eh_client::model::{
    Category, GalleryKey, ListRequest, ListSource, SearchFilter, Site, ToplistPeriod,
};
use eh_client::parse::rating::parse_ir_rating;

#[test]
fn test_f_cats_bitmask() {
    assert_eq!(calculate_f_cats(&[]), 0);
    assert_eq!(
        calculate_f_cats(&[Category::Doujinshi, Category::Manga]),
        2 | 4
    );
    assert_eq!(
        calculate_f_cats(&[
            Category::Misc,      // 1
            Category::Doujinshi, // 2
            Category::Manga,     // 4
            Category::ArtistCg,  // 8
            Category::GameCg,    // 16
            Category::ImageSet,  // 32
            Category::Cosplay,   // 64
            Category::AsianPorn, // 128
            Category::NonH,      // 256
            Category::Western,   // 512
        ]),
        1023
    );
}

#[test]
fn test_list_url_builder_front_with_filter() {
    let req = ListRequest {
        source: ListSource::Front,
        filter: SearchFilter {
            keyword: "female:glasses$".to_string(),
            excluded_categories: vec![Category::Doujinshi],
            min_rating: Some(4),
            min_pages: Some(20),
            max_pages: Some(50),
            show_expunged: true,
            require_torrent: true,
        },
        cursor: Some("next:12345".to_string()),
    };

    let url = build_list_url(Site::Eh, &req).expect("should build URL");
    let query = url.query().unwrap();

    assert!(query.contains("f_search=female%3Aglasses%24"));
    assert!(query.contains("f_cats=2"));
    assert!(query.contains("f_srdd=4"));
    assert!(query.contains("f_spf=20"));
    assert!(query.contains("f_spt=50"));
    assert!(query.contains("f_sh=on"));
    assert!(query.contains("f_sto=on"));
    assert!(query.contains("next=12345"));
}

#[test]
fn test_list_url_max_pages_less_than_min_pages_omitted() {
    let req = ListRequest {
        source: ListSource::Front,
        filter: SearchFilter {
            min_pages: Some(50),
            max_pages: Some(20), // less than min_pages -> must be omitted per EH_PROTOCOL §1.2
            ..Default::default()
        },
        cursor: None,
    };

    let url = build_list_url(Site::Eh, &req).expect("should build URL");
    let query = url.query().unwrap();
    assert!(query.contains("f_spf=50"));
    assert!(!query.contains("f_spt"));
}

#[test]
fn test_list_url_sources() {
    // Watched
    let req_w = ListRequest {
        source: ListSource::Watched,
        filter: SearchFilter::default(),
        cursor: None,
    };
    let url_w = build_list_url(Site::Eh, &req_w).unwrap();
    assert_eq!(url_w.as_str(), "https://e-hentai.org/watched");

    // Popular
    let req_p = ListRequest {
        source: ListSource::Popular,
        filter: SearchFilter::default(),
        cursor: None,
    };
    let url_p = build_list_url(Site::Eh, &req_p).unwrap();
    assert_eq!(url_p.as_str(), "https://e-hentai.org/popular");

    // Toplist periods
    let req_tl_day = ListRequest {
        source: ListSource::Toplist {
            period: ToplistPeriod::Day,
        },
        filter: SearchFilter::default(),
        cursor: Some("page:2".to_string()),
    };
    let url_tl_day = build_list_url(Site::Eh, &req_tl_day).unwrap();
    assert!(url_tl_day.query().unwrap().contains("tl=15"));
    assert!(url_tl_day.query().unwrap().contains("p=2"));

    // Favorites
    let req_fav = ListRequest {
        source: ListSource::Favorites { slot: Some(3) },
        filter: SearchFilter::default(),
        cursor: Some("prev:99999".to_string()),
    };
    let url_fav = build_list_url(Site::Eh, &req_fav).unwrap();
    assert!(url_fav.path().contains("favorites.php"));
    assert!(url_fav.query().unwrap().contains("favcat=3"));
    assert!(url_fav.query().unwrap().contains("prev=99999"));
}

#[test]
fn test_favorites_forms() {
    let key = GalleryKey {
        gid: 123456,
        token: "abcdef0123".to_string(),
    };
    let popup_url = build_fav_popup_url("https://e-hentai.org/gallerypopups.php", &key);
    assert_eq!(
        popup_url,
        "https://e-hentai.org/gallerypopups.php?gid=123456&t=abcdef0123&act=addfav"
    );

    // Single add / move
    let form_add = build_favorite_form(Some(2), "My note");
    assert_eq!(
        form_add,
        vec![
            ("favcat", "2".to_string()),
            ("favnote", "My note".to_string()),
            ("apply", "Add to Favorites".to_string()),
            ("update", "1".to_string()),
        ]
    );

    // Single remove
    let form_del = build_favorite_form(None, "");
    assert_eq!(
        form_del,
        vec![
            ("favcat", "favdel".to_string()),
            ("favnote", "".to_string()),
            ("apply", "Apply Changes".to_string()),
            ("update", "1".to_string()),
        ]
    );

    // Batch move
    let batch_move = build_batch_favorites_form(&[111, 222], Some(5));
    assert_eq!(
        batch_move,
        vec![
            ("ddact".to_string(), "fav5".to_string()),
            ("modifygids[]".to_string(), "111".to_string()),
            ("modifygids[]".to_string(), "222".to_string()),
            ("apply".to_string(), "Apply".to_string()),
        ]
    );

    // Batch delete
    let batch_del = build_batch_favorites_form(&[333], None);
    assert_eq!(
        batch_del,
        vec![
            ("ddact".to_string(), "delete".to_string()),
            ("modifygids[]".to_string(), "333".to_string()),
            ("apply".to_string(), "Apply".to_string()),
        ]
    );
}

#[test]
fn test_rating_formula() {
    // 5 - X/16 - (Y == 21 ? 0.5 : 0)
    assert!((parse_ir_rating("background-position:0px -1px").unwrap() - 5.0).abs() < 0.001);
    assert!((parse_ir_rating("background-position:0px -21px").unwrap() - 4.5).abs() < 0.001);
    assert!((parse_ir_rating("background-position:-16px -1px").unwrap() - 4.0).abs() < 0.001);
    assert!((parse_ir_rating("background-position:-16px -21px").unwrap() - 3.5).abs() < 0.001);
    assert!((parse_ir_rating("background-position:-32px -1px").unwrap() - 3.0).abs() < 0.001);
    assert!((parse_ir_rating("background-position:-80px -1px").unwrap() - 0.0).abs() < 0.001);
}
