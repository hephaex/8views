use anyhow::Result;
use rusqlite::Connection;

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS sessions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS session_state (
    session_id      INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    page_index      INTEGER NOT NULL DEFAULT 0,
    zoom_level      REAL NOT NULL DEFAULT 1.0,
    rotation        INTEGER NOT NULL DEFAULT 0,
    two_page_spread INTEGER NOT NULL DEFAULT 0,
    page_order      INTEGER NOT NULL DEFAULT 0,
    scale_mode      INTEGER NOT NULL DEFAULT 1,
    scroll_x        REAL NOT NULL DEFAULT 0.0,
    scroll_y        REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (session_id)
);

CREATE TABLE IF NOT EXISTS groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    parent_id   INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL,
    modified_at INTEGER
);

CREATE TABLE IF NOT EXISTS pages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    group_id    INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    page_index  INTEGER NOT NULL,
    filename    TEXT NOT NULL,
    width       INTEGER,
    height      INTEGER
);

CREATE INDEX IF NOT EXISTS idx_pages_session ON pages(session_id);
CREATE INDEX IF NOT EXISTS idx_pages_group ON pages(group_id);
";

const SCHEMA_V2: &str = "
CREATE TABLE IF NOT EXISTS page_metadata (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    page_index  INTEGER NOT NULL,
    filename    TEXT NOT NULL,
    width       INTEGER,
    height      INTEGER,
    UNIQUE(session_id, page_index)
);
CREATE INDEX IF NOT EXISTS idx_page_metadata_session ON page_metadata(session_id);
";

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(SCHEMA_V1)?;
    conn.execute_batch(SCHEMA_V2)?;
    Ok(())
}
