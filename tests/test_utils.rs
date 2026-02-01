use mgdl::utils::{extract_hash, normalize};

#[test]
fn normalize_manga_title() {
    assert_eq!(normalize("Tokyo: Alien Bros!"), "tokyo_alien_bros");
}

#[test]
fn normalize_accented_chars() {
    assert_eq!(normalize("Café Résumé"), "cafe_resume");
}

#[test]
fn normalize_mixed_case_accents() {
    assert_eq!(normalize("ÉLAN Über Ñoño"), "elan_uber_nono");
}

#[test]
fn normalize_leading_trailing_specials() {
    assert_eq!(normalize("!!!hello!!!"), "hello");
}

#[test]
fn normalize_consecutive_underscores() {
    assert_eq!(normalize("a   b"), "a_b");
}

#[test]
fn normalize_numbers() {
    assert_eq!(normalize("One Piece 1999"), "one_piece_1999");
}

#[test]
fn normalize_cedilla() {
    assert_eq!(normalize("Façade Ç"), "facade_c");
}

#[test]
fn normalize_empty() {
    assert_eq!(normalize(""), "");
}

#[test]
fn extract_hash_standard_url() {
    let url = "https://example.com/series/01JK8N8A7W8ZGR7014BM2ZMGBB/tokyo-alien-bros";
    assert_eq!(extract_hash(url).unwrap(), "01JK8N8A7W8ZGR7014BM2ZMGBB");
}

#[test]
fn extract_hash_no_slug() {
    assert_eq!(
        extract_hash("https://example.com/series/HASH123").unwrap(),
        "HASH123"
    );
}

#[test]
fn extract_hash_trailing_slash() {
    assert_eq!(
        extract_hash("https://example.com/series/HASH123/slug/").unwrap(),
        "HASH123"
    );
}

#[test]
fn extract_hash_no_series_segment() {
    assert!(extract_hash("https://example.com/chapters/ABC").is_none());
}

#[test]
fn extract_hash_empty_after_series() {
    assert!(extract_hash("https://example.com/series/").is_none());
}
