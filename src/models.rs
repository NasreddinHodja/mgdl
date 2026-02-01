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
