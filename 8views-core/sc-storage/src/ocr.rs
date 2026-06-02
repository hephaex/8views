use anyhow::Result;
use rusqlite::Connection;

use crate::migration::run_migrations;

/// A single result from an FTS5 full-text search of the OCR cache.
#[derive(Debug, Clone)]
pub struct OcrSearchResult {
    /// Path to the archive that contains the matching page.
    pub archive_path: String,
    /// Zero-based index of the page within the archive.
    pub page_index: u32,
    /// Short excerpt of the matched text with surrounding context.
    pub snippet: String,
}

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

    /// Full-text search within a single archive using the FTS5 index.
    ///
    /// Returns results ordered by page index.  Empty query returns empty vec.
    /// Only pages that have been OCR'd and cached are searched.
    pub fn search(&self, archive_path: &str, query: &str) -> Result<Vec<OcrSearchResult>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let mut stmt = self.conn.prepare(
            "SELECT c.archive_path, c.page_index,
                    snippet(ocr_fts, 0, '<b>', '</b>', '…', 20)
             FROM ocr_fts
             JOIN ocr_cache c ON c.id = ocr_fts.rowid
             WHERE ocr_fts MATCH ?1
               AND c.archive_path = ?2
               AND c.text_data != ''
             ORDER BY c.page_index",
        )?;
        let results = stmt.query_map(rusqlite::params![query, archive_path], |row| {
            Ok(OcrSearchResult {
                archive_path: row.get(0)?,
                page_index: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(results)
    }

    /// Full-text search across all cached archives using the FTS5 index.
    ///
    /// Returns results ordered by archive path then page index.
    pub fn search_all(&self, query: &str) -> Result<Vec<OcrSearchResult>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let mut stmt = self.conn.prepare(
            "SELECT c.archive_path, c.page_index,
                    snippet(ocr_fts, 0, '<b>', '</b>', '…', 20)
             FROM ocr_fts
             JOIN ocr_cache c ON c.id = ocr_fts.rowid
             WHERE ocr_fts MATCH ?1
               AND c.text_data != ''
             ORDER BY c.archive_path, c.page_index",
        )?;
        let results = stmt.query_map(rusqlite::params![query], |row| {
            Ok(OcrSearchResult {
                archive_path: row.get(0)?,
                page_index: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(results)
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

    #[test]
    fn search_finds_word_in_archive() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/comic.cbz", 0, "The hero defeated the dragon", MTIME).unwrap();
        cache.store("/comic.cbz", 1, "The princess was rescued", MTIME).unwrap();
        cache.store("/comic.cbz", 2, "They lived happily", MTIME).unwrap();
        let results = cache.search("/comic.cbz", "princess").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_index, 1);
    }

    #[test]
    fn search_is_case_insensitive() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/comic.cbz", 0, "The Hero defeated the DRAGON", MTIME).unwrap();
        let results = cache.search("/comic.cbz", "hero").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/comic.cbz", 0, "text here", MTIME).unwrap();
        assert!(cache.search("/comic.cbz", "").unwrap().is_empty());
        assert!(cache.search("/comic.cbz", "   ").unwrap().is_empty());
    }

    #[test]
    fn search_skips_sentinel_empty_pages() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/comic.cbz", 0, "", MTIME).unwrap(); // sentinel — no text
        cache.store("/comic.cbz", 1, "visible text", MTIME).unwrap();
        let results = cache.search_all("text").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_index, 1);
    }

    #[test]
    fn search_all_finds_across_archives() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/vol1.cbz", 0, "A dragon appears", MTIME).unwrap();
        cache.store("/vol2.cbz", 3, "The dragon returns", MTIME).unwrap();
        let results = cache.search_all("dragon").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_scoped_to_archive() {
        let cache = OcrCache::open_in_memory().unwrap();
        cache.store("/vol1.cbz", 0, "sword fight", MTIME).unwrap();
        cache.store("/vol2.cbz", 0, "sword fight", MTIME).unwrap();
        let results = cache.search("/vol1.cbz", "sword").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].archive_path, "/vol1.cbz");
    }
}
