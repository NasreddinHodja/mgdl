use std::fmt;

#[derive(Debug)]
pub struct Chapter {
    pub hash: String,
    pub number: String,
}

impl Chapter {
    pub fn new(hash: &str, number: &str) -> Self {
        Self {
            hash: hash.to_string(),
            number: number.to_string(),
        }
    }

    pub fn major_number(&self) -> Option<usize> {
        self.number.split('-').next()?.parse().ok()
    }
}

#[derive(Debug, Clone)]
pub struct ChapterRange {
    pub start: Option<usize>,
    pub end: Option<usize>,
}

impl ChapterRange {
    pub fn parse(s: &str) -> Result<Self, String> {
        if let Some((start, end)) = s.split_once("..") {
            let start = if start.is_empty() {
                None
            } else {
                Some(start.parse::<usize>().map_err(|e| e.to_string())?)
            };
            let end = if end.is_empty() {
                None
            } else {
                Some(end.parse::<usize>().map_err(|e| e.to_string())?)
            };
            Ok(Self { start, end })
        } else {
            let n = s.parse::<usize>().map_err(|e| e.to_string())?;
            Ok(Self {
                start: Some(n),
                end: Some(n),
            })
        }
    }

    pub fn contains(&self, chapter: usize) -> bool {
        self.start.is_none_or(|s| chapter >= s) && self.end.is_none_or(|e| chapter <= e)
    }
}

#[derive(Debug)]
pub struct Manga {
    pub hash: String,
    pub name: String,
    pub normalized_name: String,
    pub authors: String,
    pub status: String,
}

impl Manga {
    pub fn new(hash: &str, name: &str, normalized_name: &str, authors: &str, status: &str) -> Self {
        Self {
            hash: hash.to_string(),
            name: name.to_string(),
            normalized_name: normalized_name.to_string(),
            authors: authors.to_string(),
            status: status.to_string(),
        }
    }
}

impl fmt::Display for Manga {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:\n\thash: {}\n\tnormalized_name: {}\n\tauthors: {}\n\tstatus: {}",
            self.name, self.hash, self.normalized_name, self.authors, self.status
        )
    }
}

#[derive(Debug)]
pub struct Page {
    pub url: String,
    pub number: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_range_full_range() {
        let r = ChapterRange::parse("5..10").unwrap();
        assert_eq!(r.start, Some(5));
        assert_eq!(r.end, Some(10));
    }

    #[test]
    fn chapter_range_open_end() {
        let r = ChapterRange::parse("5..").unwrap();
        assert_eq!(r.start, Some(5));
        assert_eq!(r.end, None);
    }

    #[test]
    fn chapter_range_open_start() {
        let r = ChapterRange::parse("..10").unwrap();
        assert_eq!(r.start, None);
        assert_eq!(r.end, Some(10));
    }

    #[test]
    fn chapter_range_single() {
        let r = ChapterRange::parse("5").unwrap();
        assert_eq!(r.start, Some(5));
        assert_eq!(r.end, Some(5));
    }

    #[test]
    fn chapter_range_contains_in_range() {
        let r = ChapterRange::parse("5..10").unwrap();
        assert!(r.contains(5));
        assert!(r.contains(7));
        assert!(r.contains(10));
        assert!(!r.contains(4));
        assert!(!r.contains(11));
    }

    #[test]
    fn chapter_range_contains_open_end() {
        let r = ChapterRange::parse("5..").unwrap();
        assert!(r.contains(5));
        assert!(r.contains(9999));
        assert!(!r.contains(4));
    }

    #[test]
    fn chapter_range_contains_open_start() {
        let r = ChapterRange::parse("..10").unwrap();
        assert!(r.contains(0));
        assert!(r.contains(10));
        assert!(!r.contains(11));
    }

    #[test]
    fn chapter_range_parse_invalid() {
        assert!(ChapterRange::parse("abc").is_err());
        assert!(ChapterRange::parse("abc..10").is_err());
    }

    #[test]
    fn chapter_major_number() {
        let ch = Chapter::new("hash", "0010-01");
        assert_eq!(ch.major_number(), Some(10));
    }

    #[test]
    fn chapter_major_number_decimal() {
        let ch = Chapter::new("hash", "0005-05");
        assert_eq!(ch.major_number(), Some(5));
    }

    #[test]
    fn chapter_new_stores_fields() {
        let ch = Chapter::new("abc123", "0001-01");
        assert_eq!(ch.hash, "abc123");
        assert_eq!(ch.number, "0001-01");
    }

    #[test]
    fn manga_display() {
        let m = Manga::new("h", "Test Manga", "test_manga", "Author A", "Ongoing");
        let display = format!("{}", m);
        assert!(display.contains("Test Manga"));
        assert!(display.contains("test_manga"));
        assert!(display.contains("Author A"));
        assert!(display.contains("Ongoing"));
    }
}
