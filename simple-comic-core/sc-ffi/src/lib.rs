// FFI entry point for Simple Comic Core.
//
// This crate exposes sc-archive and sc-storage functionality to Swift via
// uniffi-generated bindings.  The UDL interface is defined in
// `src/simplecomic.udl`.
//
// Sprint 6: activate uniffi scaffolding by:
//   1. Uncommenting `uniffi::generate_scaffolding(...)` in build.rs.
//   2. Replacing the hand-written stubs below with:
//        uniffi::include_scaffolding!("simplecomic");
//      and implementing the UDL function signatures exactly.
//
// Until then, the stubs compile against sc-archive / sc-storage directly so
// that the type boundaries are validated by `cargo check` every sprint.

use thiserror::Error;

// ── Error type ───────────────────────────────────────────────────────────────
//
// Variants match the [Error] enum declared in simplecomic.udl.  The `Display`
// impl produced by thiserror is what uniffi will surface as the Swift error
// message.

#[derive(Debug, Error)]
pub enum ScError {
    #[error("archive error: {0}")]
    Archive(String),

    #[error("image error: {0}")]
    Image(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("entry not found at index {0}")]
    EntryNotFound(u32),
}

impl From<anyhow::Error> for ScError {
    fn from(e: anyhow::Error) -> Self {
        // Inspect the error chain for well-known categories; fall back to Archive.
        let msg = e.to_string();
        if msg.contains("unsupported format") {
            ScError::UnsupportedFormat(msg)
        } else if msg.contains("not found") || msg.contains("no image") {
            ScError::EntryNotFound(0)
        } else {
            ScError::Archive(msg)
        }
    }
}

// ── Smoke-test export (legacy, kept for Swift link-check until Sprint 6) ────

/// Returns the library version as a C string.
///
/// # Safety
///
/// The returned pointer is valid for the lifetime of the process; callers must
/// not free it.
#[no_mangle]
pub extern "C" fn sc_version() -> *const std::ffi::c_char {
    c"0.1.0".as_ptr()
}

// ── Archive stubs ────────────────────────────────────────────────────────────
//
// These functions mirror the UDL namespace functions in simplecomic.udl.
// Sprint 6: replaced by uniffi-generated wrappers that call these (or
// equivalent) implementations.

/// List all image pages in an archive, sorted in natural order.
///
/// # Errors
///
/// Returns [`ScError::Archive`] if the archive cannot be opened or parsed,
/// or [`ScError::UnsupportedFormat`] if the file extension is unrecognised.
pub fn archive_list_pages(archive_path: &str) -> Result<Vec<sc_archive::ArchiveEntry>, ScError> {
    let archive =
        sc_archive::open_archive(std::path::Path::new(archive_path)).map_err(ScError::from)?;
    Ok(archive.entries().to_vec())
}

/// Read raw image bytes for a single page by zero-based index.
///
/// # Errors
///
/// Returns [`ScError::EntryNotFound`] if `index` is out of range, or
/// [`ScError::Archive`] on decompression failure.
pub fn archive_read_page(archive_path: &str, index: u32) -> Result<Vec<u8>, ScError> {
    let mut archive =
        sc_archive::open_archive(std::path::Path::new(archive_path)).map_err(ScError::from)?;
    let page_count = archive.entries().len();
    let idx = index as usize;
    if idx >= page_count {
        return Err(ScError::EntryNotFound(index));
    }
    archive.read_entry(idx).map_err(ScError::from)
}

/// Read raw bytes for the first image in an archive (QuickLook thumbnail path).
///
/// Uses the optimised partial-read path that avoids decompressing the whole
/// archive.
///
/// # Errors
///
/// Returns [`ScError::Archive`] on failure, or [`ScError::UnsupportedFormat`]
/// if the format is not supported.
pub fn archive_read_first_image(archive_path: &str) -> Result<Vec<u8>, ScError> {
    sc_archive::read_first_image(std::path::Path::new(archive_path)).map_err(ScError::from)
}

// ── Session stubs ────────────────────────────────────────────────────────────
//
// Session state is stored per archive path via sc-storage::SessionManager.
// The database path is hard-coded to an in-memory DB in the stubs; the real
// Sprint 6 implementation resolves a platform app-support path at runtime.

/// Load persisted session state for an archive.
///
/// Returns a default `SessionState` when no record exists.
///
/// # Errors
///
/// Returns [`ScError::Storage`] if the underlying SQLite operation fails.
pub fn session_load(archive_path: &str) -> Result<sc_storage::SessionState, ScError> {
    // Sprint 6: resolve a persistent DB path via platform dirs.
    let mgr = sc_storage::SessionManager::open_in_memory()
        .map_err(|e| ScError::Storage(e.to_string()))?;
    mgr.load(archive_path)
        .map_err(|e| ScError::Storage(e.to_string()))
}

/// Persist session state for an archive.
///
/// # Errors
///
/// Returns [`ScError::Storage`] if the underlying SQLite operation fails.
pub fn session_save(archive_path: &str, state: &sc_storage::SessionState) -> Result<(), ScError> {
    // Sprint 6: resolve a persistent DB path via platform dirs.
    let mgr = sc_storage::SessionManager::open_in_memory()
        .map_err(|e| ScError::Storage(e.to_string()))?;
    mgr.save(archive_path, state)
        .map_err(|e| ScError::Storage(e.to_string()))
}

/// Delete the session record for an archive.
///
/// No-op if no record exists.
///
/// # Errors
///
/// Returns [`ScError::Storage`] if the underlying SQLite operation fails.
pub fn session_delete(archive_path: &str) -> Result<(), ScError> {
    let mgr = sc_storage::SessionManager::open_in_memory()
        .map_err(|e| ScError::Storage(e.to_string()))?;
    mgr.delete(archive_path)
        .map_err(|e| ScError::Storage(e.to_string()))
}

/// Returns the library version string.
pub fn sc_library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sc_version_ptr_is_nonnull() {
        let ptr = sc_version();
        assert!(!ptr.is_null());
    }

    #[test]
    fn sc_library_version_matches_cargo() {
        let v = sc_library_version();
        assert!(!v.is_empty());
        // SemVer format: MAJOR.MINOR.PATCH
        assert_eq!(v.split('.').count(), 3, "expected SemVer, got {v}");
    }

    #[test]
    fn archive_list_pages_nonexistent_returns_err() {
        let result = archive_list_pages("/no/such/archive.cbz");
        assert!(result.is_err());
    }

    #[test]
    fn archive_read_first_image_nonexistent_returns_err() {
        let result = archive_read_first_image("/no/such/archive.cbz");
        assert!(result.is_err());
    }

    #[test]
    fn archive_read_page_nonexistent_returns_err() {
        let result = archive_read_page("/no/such/archive.cbz", 0);
        assert!(result.is_err());
    }

    #[test]
    fn session_load_missing_returns_default() {
        let state = session_load("/nonexistent.cbz").expect("should return default state");
        assert_eq!(state.page_index, 0);
        assert!(!state.two_page_spread);
    }

    #[test]
    fn session_delete_missing_is_noop() {
        let result = session_delete("/nonexistent.cbz");
        assert!(result.is_ok());
    }
}
