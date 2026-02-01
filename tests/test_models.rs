use mgdl::models::{Chapter, ChapterRange, Manga};

#[test]
fn chapter_range_full() {
    let r = ChapterRange::parse("1..100").unwrap();
    assert_eq!(r.start, Some(1));
    assert_eq!(r.end, Some(100));
}

#[test]
fn chapter_range_open_start() {
    let r = ChapterRange::parse("..50").unwrap();
    assert_eq!(r.start, None);
    assert_eq!(r.end, Some(50));
    assert!(r.contains(0));
    assert!(r.contains(50));
    assert!(!r.contains(51));
}

#[test]
fn chapter_range_open_end() {
    let r = ChapterRange::parse("10..").unwrap();
    assert_eq!(r.start, Some(10));
    assert_eq!(r.end, None);
    assert!(!r.contains(9));
    assert!(r.contains(10));
    assert!(r.contains(99999));
}

#[test]
fn chapter_range_single_chapter() {
    let r = ChapterRange::parse("42").unwrap();
    assert_eq!(r.start, Some(42));
    assert_eq!(r.end, Some(42));
    assert!(r.contains(42));
    assert!(!r.contains(41));
    assert!(!r.contains(43));
}

#[test]
fn chapter_range_zero() {
    let r = ChapterRange::parse("0").unwrap();
    assert!(r.contains(0));
    assert!(!r.contains(1));
}

#[test]
fn chapter_range_invalid_input() {
    assert!(ChapterRange::parse("abc").is_err());
    assert!(ChapterRange::parse("1..abc").is_err());
    assert!(ChapterRange::parse("abc..5").is_err());
}

#[test]
fn chapter_major_number_standard() {
    let ch = Chapter::new("hash", "0010-01");
    assert_eq!(ch.major_number(), Some(10));
}

#[test]
fn chapter_major_number_zero() {
    let ch = Chapter::new("hash", "0000-01");
    assert_eq!(ch.major_number(), Some(0));
}

#[test]
fn chapter_major_number_large() {
    let ch = Chapter::new("hash", "9999-01");
    assert_eq!(ch.major_number(), Some(9999));
}

#[test]
fn chapter_major_number_decimal() {
    let ch = Chapter::new("hash", "0005-05");
    assert_eq!(ch.major_number(), Some(5));
}

#[test]
fn manga_new_fields() {
    let m = Manga::new("h1", "Name", "name", "Auth", "Ongoing");
    assert_eq!(m.hash, "h1");
    assert_eq!(m.name, "Name");
    assert_eq!(m.normalized_name, "name");
    assert_eq!(m.authors, "Auth");
    assert_eq!(m.status, "Ongoing");
}

#[test]
fn manga_display_contains_all_fields() {
    let m = Manga::new("hash1", "My Manga", "my_manga", "Author X", "Complete");
    let s = format!("{}", m);
    assert!(s.contains("My Manga"));
    assert!(s.contains("hash1"));
    assert!(s.contains("my_manga"));
    assert!(s.contains("Author X"));
    assert!(s.contains("Complete"));
}
