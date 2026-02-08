use mgdl::scrape::{
    get_chapter_pages, get_with_retry, manga_from_url, parse_chapters_from_html,
    parse_manga_from_html,
};

fn base_url() -> String {
    std::env::var("MGDL_BASE_URL").expect("MGDL_BASE_URL env var must be set for live tests")
}

fn manga_url() -> String {
    std::env::var("MGDL_TEST_MANGA_URL")
        .expect("MGDL_TEST_MANGA_URL env var must be set for live tests")
}

fn manga_hash() -> String {
    std::env::var("MGDL_TEST_MANGA_HASH")
        .expect("MGDL_TEST_MANGA_HASH env var must be set for live tests")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[tokio::test]
#[ignore]
async fn test_live_manga_scrape() {
    let client = client();
    let manga_url = manga_url();
    let manga_hash = manga_hash();

    let html = get_with_retry(&client, &manga_url, 3)
        .await
        .expect("Failed to fetch manga page");

    let manga = parse_manga_from_html(&html, &manga_url).expect(
        "UPSTREAM FORMAT CHANGE: failed to parse manga page — site likely changed their HTML structure",
    );

    assert!(
        !manga.name.is_empty(),
        "UPSTREAM FORMAT CHANGE: manga name selector returned empty — site likely changed their HTML structure"
    );
    assert!(
        !manga.authors.is_empty(),
        "UPSTREAM FORMAT CHANGE: authors selector returned empty — site likely changed their HTML structure"
    );
    assert!(
        !manga.status.is_empty(),
        "UPSTREAM FORMAT CHANGE: status selector returned empty — site likely changed their HTML structure"
    );
    assert_eq!(manga.hash, manga_hash);
}

#[tokio::test]
#[ignore]
async fn test_live_chapter_list() {
    let client = client();
    let base_url = base_url();
    let manga_hash = manga_hash();

    let url = format!("{}/series/{}/full-chapter-list", base_url, manga_hash);
    let html = get_with_retry(&client, &url, 3)
        .await
        .expect("Failed to fetch chapter list");

    let chapters = parse_chapters_from_html(&html).expect(
        "UPSTREAM FORMAT CHANGE: failed to parse chapter list — site likely changed their HTML structure",
    );

    assert!(
        !chapters.is_empty(),
        "UPSTREAM FORMAT CHANGE: chapter list is empty — site likely changed their HTML structure"
    );

    for ch in &chapters {
        assert!(
            !ch.hash.is_empty(),
            "UPSTREAM FORMAT CHANGE: chapter hash is empty"
        );
        assert!(
            ch.number.contains('-'),
            "UPSTREAM FORMAT CHANGE: chapter number '{}' not in expected XXXX-YY format",
            ch.number
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_live_chapter_pages() {
    let client = client();
    let base_url = base_url();
    let manga_hash = manga_hash();

    let url = format!("{}/series/{}/full-chapter-list", base_url, manga_hash);
    let html = get_with_retry(&client, &url, 3).await.unwrap();
    let chapters = parse_chapters_from_html(&html).unwrap();
    assert!(!chapters.is_empty(), "Need at least one chapter");

    let last_chapter = chapters.last().unwrap();
    let pages = get_chapter_pages(&client, &base_url, &last_chapter.hash, 3)
        .await
        .expect(
            "UPSTREAM FORMAT CHANGE: failed to fetch chapter pages — site likely changed their HTML structure",
        );

    assert!(
        !pages.is_empty(),
        "UPSTREAM FORMAT CHANGE: chapter pages list is empty"
    );

    for page in &pages {
        assert!(
            page.url.starts_with("http"),
            "UPSTREAM FORMAT CHANGE: page URL '{}' is not a valid HTTP URL",
            page.url
        );
        assert!(page.number > 0);
    }
}

#[tokio::test]
#[ignore]
async fn test_live_full_manga_from_url() {
    let client = client();
    let base_url = base_url();
    let manga_url = manga_url();
    let manga_hash = manga_hash();

    let (manga, chapters) = manga_from_url(&client, &base_url, &manga_url, 3)
        .await
        .expect("UPSTREAM FORMAT CHANGE: manga_from_url failed end-to-end");

    assert!(!manga.name.is_empty());
    assert!(!chapters.is_empty());
    assert_eq!(manga.hash, manga_hash);
}

#[tokio::test]
#[ignore]
async fn test_live_page_download() {
    let client = client();
    let base_url = base_url();
    let manga_hash = manga_hash();

    let dir = tempfile::TempDir::new().unwrap();
    let url = format!("{}/series/{}/full-chapter-list", base_url, manga_hash);
    let html = get_with_retry(&client, &url, 3).await.unwrap();
    let chapters = parse_chapters_from_html(&html).unwrap();
    let last = chapters.last().unwrap();

    let pages = get_chapter_pages(&client, &base_url, &last.hash, 3)
        .await
        .unwrap();
    let first_page = &pages[0];

    mgdl::scrape::download_page(
        &client,
        first_page.url.clone(),
        dir.path().to_path_buf(),
        first_page.number,
        3,
    )
    .await
    .expect("Failed to download a page image");

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1, "Expected exactly one downloaded file");

    let file_size = entries[0].metadata().unwrap().len();
    assert!(
        file_size > 0,
        "Downloaded file should have >0 bytes, got {}",
        file_size
    );
}

#[tokio::test]
#[ignore]
async fn test_live_retry_mechanism() {
    let client = client();
    let manga_url = manga_url();

    let result = get_with_retry(&client, &manga_url, 2).await;
    assert!(
        result.is_ok(),
        "get_with_retry should succeed on a valid URL: {:?}",
        result.err()
    );
}
