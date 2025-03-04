use std::fmt;

#[derive(Debug)]
pub struct Chapter {
    pub hash: String,
    pub number: String,
    pub manga: String,
}

impl Chapter {
    pub fn new(hash: &str, number: &str, manga: &str) -> Self {
        Self {
            hash: hash.to_string(),
            number: number.to_string(),
            manga: manga.to_string(),
        }
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
            "{}:\n\thash: {}\n\tnormalize_name: {}\n\tauthors: {}\n\tstatus: {}",
            self.name, self.hash, self.normalized_name, self.authors, self.status
        )
    }
}
