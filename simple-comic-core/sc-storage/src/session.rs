use anyhow::Result;
use rusqlite::{params, Connection};
use sc_core::types::{PageOrder, Rotation, ScaleMode};
use serde::{Deserialize, Serialize};

use crate::migration::run_migrations;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    pub page_index: usize,
    pub zoom_level: f64,
    pub rotation: Rotation,
    pub two_page_spread: bool,
    pub page_order: PageOrder,
    pub scale_mode: ScaleMode,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

pub struct SessionManager {
    conn: Connection,
}

impl SessionManager {
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

    /// Load session state for an archive path. Returns default state if not found.
    pub fn load(&self, archive_path: &str) -> Result<SessionState> {
        let result = self.conn.query_row(
            "SELECT page_index, zoom_level, rotation, two_page_spread, page_order, scale_mode,
                    scroll_x, scroll_y
             FROM sessions s
             JOIN session_state ss ON ss.session_id = s.id
             WHERE s.path = ?1",
            params![archive_path],
            |row| {
                Ok(SessionState {
                    page_index: row.get::<_, i64>(0)? as usize,
                    zoom_level: row.get(1)?,
                    rotation: match row.get::<_, i64>(2)? {
                        90 => Rotation::R90,
                        180 => Rotation::R180,
                        270 => Rotation::R270,
                        _ => Rotation::R0,
                    },
                    two_page_spread: row.get::<_, bool>(3)?,
                    page_order: if row.get::<_, bool>(4)? {
                        PageOrder::RightToLeft
                    } else {
                        PageOrder::LeftToRight
                    },
                    scale_mode: match row.get::<_, i64>(5)? {
                        0 => ScaleMode::Original,
                        2 => ScaleMode::FitWidth,
                        _ => ScaleMode::FitWindow,
                    },
                    scroll_x: row.get(6)?,
                    scroll_y: row.get(7)?,
                })
            },
        );

        match result {
            Ok(state) => Ok(state),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(SessionState::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Persist session state for an archive path.
    pub fn save(&self, archive_path: &str, state: &SessionState) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO sessions (path, created_at, updated_at) VALUES (?1, ?2, ?2)
             ON CONFLICT(path) DO UPDATE SET updated_at = ?2",
            params![archive_path, now],
        )?;

        let session_id: i64 = self.conn.query_row(
            "SELECT id FROM sessions WHERE path = ?1",
            params![archive_path],
            |row| row.get(0),
        )?;

        let rotation_deg = match state.rotation {
            Rotation::R0 => 0i64,
            Rotation::R90 => 90,
            Rotation::R180 => 180,
            Rotation::R270 => 270,
        };
        let scale_int = match state.scale_mode {
            ScaleMode::Original => 0i64,
            ScaleMode::FitWindow => 1,
            ScaleMode::FitWidth => 2,
        };

        self.conn.execute(
            "INSERT INTO session_state
             (session_id, page_index, zoom_level, rotation, two_page_spread, page_order,
              scale_mode, scroll_x, scroll_y)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(session_id) DO UPDATE SET
               page_index = ?2, zoom_level = ?3, rotation = ?4,
               two_page_spread = ?5, page_order = ?6, scale_mode = ?7,
               scroll_x = ?8, scroll_y = ?9",
            params![
                session_id,
                state.page_index as i64,
                state.zoom_level,
                rotation_deg,
                state.two_page_spread,
                matches!(state.page_order, PageOrder::RightToLeft),
                scale_int,
                state.scroll_x,
                state.scroll_y,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, archive_path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM sessions WHERE path = ?1",
            params![archive_path],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let mgr = SessionManager::open_in_memory().unwrap();
        let state = SessionState {
            page_index: 42,
            zoom_level: 1.5,
            rotation: Rotation::R90,
            two_page_spread: true,
            page_order: PageOrder::RightToLeft,
            scale_mode: ScaleMode::FitWidth,
            scroll_x: 10.0,
            scroll_y: 20.0,
        };
        mgr.save("/path/to/comic.cbz", &state).unwrap();
        let loaded = mgr.load("/path/to/comic.cbz").unwrap();
        assert_eq!(loaded.page_index, 42);
        assert_eq!(loaded.zoom_level, 1.5);
        assert!(matches!(loaded.rotation, Rotation::R90));
        assert!(loaded.two_page_spread);
        assert!(matches!(loaded.page_order, PageOrder::RightToLeft));
        assert!(matches!(loaded.scale_mode, ScaleMode::FitWidth));
        assert_eq!(loaded.scroll_x, 10.0);
        assert_eq!(loaded.scroll_y, 20.0);
    }

    #[test]
    fn load_missing_returns_default() {
        let mgr = SessionManager::open_in_memory().unwrap();
        let state = mgr.load("/nonexistent.cbz").unwrap();
        assert_eq!(state.page_index, 0);
        assert!(!state.two_page_spread);
    }

    #[test]
    fn delete_removes_session() {
        let mgr = SessionManager::open_in_memory().unwrap();
        let state = SessionState {
            page_index: 10,
            ..Default::default()
        };
        mgr.save("/test.cbz", &state).unwrap();
        mgr.delete("/test.cbz").unwrap();
        let loaded = mgr.load("/test.cbz").unwrap();
        assert_eq!(loaded.page_index, 0);
    }
}
