use mgdl::db::Db;
use mgdl::models::Manga;
use tempfile::TempDir;

fn temp_db() -> (Db, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = Db::new(dir.path().join("test.db")).unwrap();
    (db, dir)
}

fn sample_manga(name: &str, status: &str) -> Manga {
    Manga::new("hash1", name, &name.to_lowercase(), "Author A", status)
}

#[test]
fn insert_and_query() {
    let (db, _dir) = temp_db();
    let manga = sample_manga("Test Manga", "Ongoing");
    db.upsert_manga(manga).unwrap();

    let result = db.get_manga_by_normalized_name("test manga").unwrap();
    assert_eq!(result.name, "Test Manga");
    assert_eq!(result.hash, "hash1");
    assert_eq!(result.authors, "Author A");
    assert_eq!(result.status, "Ongoing");
}

#[test]
fn upsert_updates_existing() {
    let (db, _dir) = temp_db();
    db.upsert_manga(sample_manga("Test Manga", "Ongoing"))
        .unwrap();

    let updated = Manga::new("hash2", "Test Manga", "test manga", "Author B", "Complete");
    db.upsert_manga(updated).unwrap();

    let result = db.get_manga_by_normalized_name("test manga").unwrap();
    assert_eq!(result.hash, "hash2");
    assert_eq!(result.authors, "Author B");
    assert_eq!(result.status, "Complete");
}

#[test]
fn get_ongoing_manga() {
    let (db, _dir) = temp_db();
    db.upsert_manga(Manga::new("h1", "Manga A", "manga_a", "A", "Ongoing"))
        .unwrap();
    db.upsert_manga(Manga::new("h2", "Manga B", "manga_b", "B", "Complete"))
        .unwrap();
    db.upsert_manga(Manga::new("h3", "Manga C", "manga_c", "C", "Ongoing"))
        .unwrap();

    let ongoing = db.get_ongoing_manga().unwrap();
    assert_eq!(ongoing.len(), 2);
    let names: Vec<_> = ongoing.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Manga A"));
    assert!(names.contains(&"Manga C"));
}

#[test]
fn delete_manga() {
    let (db, _dir) = temp_db();
    db.upsert_manga(sample_manga("To Delete", "Ongoing"))
        .unwrap();
    db.delete_manga_by_normalized_name("to delete").unwrap();
    assert!(db.get_manga_by_normalized_name("to delete").is_err());
}

#[test]
fn drop_and_recreate() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Db::new(db_path.clone()).unwrap();
    db.upsert_manga(sample_manga("Test", "Ongoing")).unwrap();
    db.drop_table().unwrap();
    drop(db);

    let db = Db::new(db_path).unwrap();
    assert!(db.get_manga_by_normalized_name("test").is_err());
}

#[test]
fn query_nonexistent_returns_error() {
    let (db, _dir) = temp_db();
    assert!(db.get_manga_by_normalized_name("nonexistent").is_err());
}
