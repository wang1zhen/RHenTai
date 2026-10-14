use eh_client::model::Category;
use eh_client::parse::parse_gdata_response;

const GDATA: &str = include_str!("fixtures/gdata.json");

#[test]
fn test_gdata_parser() {
    let summaries = parse_gdata_response(GDATA).expect("should parse gdata response");
    assert_eq!(summaries.len(), 3);

    let item0 = &summaries[0];
    assert_eq!(item0.gid, 4253617);
    assert_eq!(item0.token, "97ec92c515");
    assert_eq!(item0.title, "[北川時] 遠距離恋愛中の二人");
    assert_eq!(item0.category, Category::Manga);
    assert_eq!(item0.uploader, Some("KPs14".to_string()));
    assert_eq!(item0.file_count, 37);
    assert!((item0.rating - 0.50).abs() < 0.01);
    assert_eq!(item0.tags.len(), 1);
    assert_eq!(item0.tags[0].namespace, "language");
    assert_eq!(item0.tags[0].name, "japanese");

    let item1 = &summaries[1];
    assert_eq!(item1.gid, 4253615);
    assert_eq!(item1.category, Category::ArtistCg);
    assert!(item1.title_jpn.is_some());
    assert!(
        item1
            .tags
            .iter()
            .any(|t| t.namespace == "male" && t.name == "yaoi")
    );

    let item2 = &summaries[2];
    assert_eq!(item2.gid, 4253614);
    assert_eq!(item2.category, Category::Misc);
    assert_eq!(item2.file_count, 272);
}
