use anyhow::Result;
use rusqlite::Connection;

use crate::migration::run_migrations;

/// OCR text cache: stores Vision-recognised text per page so subsequent searches
/// skip Vision inference and query the SQLite store instead.
pub struct OcrCache {
    conn: Connection,
}

impl OcrCache {
    pub fn open(db_path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        run_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        run_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Store recognised text for `(archive_path, page_index)`.
    ///
    /// `archive_mtime_secs` is the Unix timestamp (seconds) of the archive file's
    /// last-modified time.  Used by `has_valid` to detect stale cache entries.
    /// Empty text is stored as an explicit empty string (sentinel: "already OCR'd, no text").
    pub fn store(
        &self,
        archive_path: &str,
        page_index: u32,
        text: &str,
        archive_mtime_secs: i64,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.conn.execute(
            "INSERT OR REPLACE INTO ocr_cache
             (archive_path, page_index, text_data, created_at, archive_mtime)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![archive_path, page_index, text, now, archive_mtime_secs],
        )?;
        Ok(())
    }

    /// Return the cached text for `(archive_path, page_index)`, or `None` if not cached.
    pub fn get(&self, archive_path: &str, page_index: u32) -> Option<String> {
        self.conn
            .query_row(
                "SELECT text_data FROM ocr_cache WHERE archive_path = ?1 AND page_index = ?2",
                rusqlite::params![archive_path, page_index],
                |row| row.get::<_, String>(0),
            )
            .ok()
    }

    /// Return `true` if a cache entry exists for `(archive_path, page_index)` AND
    /// the stored `archive_mtime` matches `current_mtime_secs`.
    ///
    /// Use this instead of `has()` to avoid serving stale OCR text after the
    /// archive file has been replaced or modified.
    pub fn has_valid(
        &self,
        archive_path: &str,
        page_index: u32,
        current_mtime_secs: i64,
    ) -> bool {
        self.conn
            .query_row(
                "SELECT archive_mtime FROM ocr_cache
                 WHERE archive_path = ?1 AND page_index = ?2",
                rusqlite::params![archive_path, page_index],
                |row| row.get::<_, i64>(0),
            )
            .map(|stored_mtime| stored_mtime == current_mtime_secs)
            .unwrap_or(false)
    }

    /// Return `true` if a cache entry exists for `(archive_path, page_index)`.
    /// Does not validate mtime — prefer `has_valid` for correctness.
    pub fn has(&self, archive_path: &str, page_index: u32) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM ocr_cache WHERE archive_path = ?1 AND page_index = ?2",
                rusqlite::params![archive_path, page_index],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0
    }

    /// Remove all cached entries for `archive_path`.
    pub fn clear(&self, archive_path: &str) {
        let _ = self.conn.execute(
            "DELETE FROM ocr_cache WHERE archive_path = ?1",
            rusqlite::params![archive_path],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MTIME: i64 = 1_700_000_000;

    #[test]
    fn store_and_get_round_trip() {
        let cache = OcrCache::open_in_memory().unwrap();
        assert!(!cache.has("/test.cbz", 0));
        cache.store("/test.cbz", 0, "hello world", MTIME).unwrap();
        assert!(cache.has("/test.cbz", 0));
        assert_eq!(cache.get("/test.cbz", 0).unwrap(), "hello world");
    }

    #[test]
    fn has_valid_matches_mtime() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/test.cbz", 0, "text", MTIME).unwrap();
        assert!(cache.has_valid("/test.cbz", 0, MTIME));
        assert!(!cache.has_valid("/test.cbz", 0, MTIME + 1)); // stale
        assert!(!cache.has_valid("/test.cbz", 1, MTIME));     // different page
    }

    #[test]
    fn store_empty_text_as_sentinel() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/test.cbz", 5, "", MTIME).unwrap();
        assert!(cache.has("/test.cbz", 5));
        assert_eq!(cache.get("/test.cbz", 5).unwrap(), "");
    }

    #[test]
    fn clear_removes_archive_entries() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/test.cbz", 0, "page zero", MTIME).unwrap();
        cache.store("/test.cbz", 1, "page one", MTIME).unwrap();
        cache.store("/other.cbz", 0, "other", MTIME).unwrap();
        cache.clear("/test.cbz");
        assert!(!cache.has("/test.cbz", 0));
        assert!(!cache.has("/test.cbz", 1));
        assert!(cache.has("/other.cbz", 0)); // other archive unaffected
    }

    #[test]
    fn upsert_replaces_existing() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/test.cbz", 0, "first", MTIME).unwrap();
        cache.store("/test.cbz", 0, "second", MTIME + 1).unwrap();
        assert_eq!(cache.get("/test.cbz", 0).unwrap(), "second");
        assert!(cache.has_valid("/test.cbz", 0, MTIME + 1));
    }
}
