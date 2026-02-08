mod common;

use mgdl::scrape::{parse_chapters_from_html, parse_manga_from_html, parse_pages_from_html};

// URL only used for extract_hash parsing — domain is irrelevant
const FIXTURE_MANGA_URL: &str =
    "https://example.com/series/01JK8N8A7W8ZGR7014BM2ZMGBB/tokyo-alien-bros";

#[test]
fn parse_manga_name_and_hash() {
    let html = common::load_fixture("manga_page.html");
    let manga = parse_manga_from_html(&html, FIXTURE_MANGA_URL).unwrap();

    assert!(
        !manga.name.is_empty(),
        "FIXTURE PARSE FAILURE: manga name is empty — HTML structure may have changed"
    );
    assert!(
        manga.name.contains("Tokyo Alien Bros"),
        "Expected manga name to contain 'Tokyo Alien Bros', got: '{}'",
        manga.name
    );
    assert_eq!(manga.hash, "01JK8N8A7W8ZGR7014BM2ZMGBB");
    assert!(!manga.normalized_name.is_empty());
}

#[test]
fn parse_manga_authors_and_status() {
    let html = common::load_fixture("manga_page.html");
    let manga = parse_manga_from_html(&html, FIXTURE_MANGA_URL).unwrap();

    assert!(
        !manga.authors.is_empty(),
        "FIXTURE PARSE FAILURE: authors is empty — HTML structure may have changed"
    );
    assert!(
        !manga.status.is_empty(),
        "FIXTURE PARSE FAILURE: status is empty — HTML structure may have changed"
    );
}

#[test]
fn parse_manga_bad_url_fails() {
    let html = common::load_fixture("manga_page.html");
    let result = parse_manga_from_html(&html, "https://example.com/no-series");
    assert!(result.is_err());
}

#[test]
fn parse_chapters_count_and_hashes() {
    let html = common::load_fixture("chapter_list.html");
    let chapters = parse_chapters_from_html(&html).unwrap();

    assert!(
        !chapters.is_empty(),
        "FIXTURE PARSE FAILURE: no chapters parsed — HTML structure may have changed"
    );

    for ch in &chapters {
        assert!(!ch.hash.is_empty(), "Chapter hash should not be empty");
        assert!(
            ch.number.contains('-'),
            "Chapter number '{}' should be in XXXX-YY format",
            ch.number
        );
    }
}

#[test]
fn parse_chapters_number_format() {
    let html = common::load_fixture("chapter_list.html");
    let chapters = parse_chapters_from_html(&html).unwrap();

    for ch in &chapters {
        let parts: Vec<&str> = ch.number.split('-').collect();
        assert_eq!(
            parts.len(),
            2,
            "Expected XXXX-YY format, got: {}",
            ch.number
        );
        assert_eq!(
            parts[0].len(),
            4,
            "Major part should be 4 digits: {}",
            ch.number
        );
        assert_eq!(
            parts[1].len(),
            2,
            "Minor part should be 2 digits: {}",
            ch.number
        );
    }
}

#[test]
fn parse_pages_count_and_urls() {
    let html = common::load_fixture("chapter_pages.html");
    let pages = parse_pages_from_html(&html).unwrap();

    assert!(
        !pages.is_empty(),
        "FIXTURE PARSE FAILURE: no pages parsed — HTML structure may have changed"
    );

    for page in &pages {
        assert!(
            page.url.starts_with("http"),
            "Page URL should start with http: {}",
            page.url
        );
        assert!(page.number > 0, "Page number should be > 0");
    }
}

#[test]
fn parse_pages_sequential_numbers() {
    let html = common::load_fixture("chapter_pages.html");
    let pages = parse_pages_from_html(&html).unwrap();

    let mut numbers: Vec<usize> = pages.iter().map(|p| p.number).collect();
    numbers.sort();
    let expected: Vec<usize> = (1..=numbers.len()).collect();
    assert_eq!(
        numbers, expected,
        "Page numbers should be sequential starting at 1"
    );
}

#[test]
fn parse_pages_empty_html_fails() {
    let result = parse_pages_from_html("<html><body></body></html>");
    assert!(
        result.is_err(),
        "Empty HTML should fail with no pages error"
    );
}

#[test]
fn parse_chapters_empty_html() {
    let chapters = parse_chapters_from_html("<html><body></body></html>").unwrap();
    assert!(chapters.is_empty());
}
