# Changelog

All notable changes to Simple Comic Rust refactoring.

---

## Phase 6: UI 배선 완료 (2026-06-01 ~ 2026-06-02)

### Added

**Rust C FFI layer (`SimpleComicCore/Sources/simplecomicFFI/sc_extras.h`)**
- `sc_archive_is_supported()` — extension + magic bytes archive detection
- `sc_archive_read_page()` + `sc_free_bytes()` — heap-ownership page reading
- `sc_archive_open_pages()` / `sc_page_list_{count,name,size}()` / `sc_archive_pages_free()` — open-once page enumeration (ScPageList opaque handle)
- `sc_session_load()` / `sc_session_save()` / `sc_session_delete_c()` — session CRUD via Rust SQLite
- `sc_thumbnail_from_bytes()` — Rust Lanczos3 thumbnail generation
- `sc_cap_image_bytes()` — conditional Rust pre-scaling for large images (>2048px)
- `SCArchiveError` enum + `SCArchiveErrorDomain` constant

**sc-storage**
- `SessionManager::exists()` — detect saved sessions without returning defaults

### Changed

**`SimpleComicAppDelegate.m`**
- Archive detection: `[TSSTManagedArchive archiveExtensions] containsObject:` → `sc_archive_is_supported()` (3 sites)
- Session end: `endSession:` now calls `sc_session_delete_c()` to sync Rust DB

**`TSSTManagedGroup.m` (TSSTManagedArchive)**
- `requestDataForPageIndex:` — XADMaster replaced with `sc_archive_read_page()` C FFI
- `nestedArchiveContents` — image enumeration replaced with Rust `ScPageList` (fixes XADMaster vs Rust index alignment); nested archive/PDF extraction via XADMaster retained
- `nestedFolderContents` — archive type detection → `sc_archive_is_supported()`

**`TSSTSessionWindowController.m`**
- `restoreSession` — loads Rust SessionManager state if record exists (overrides Core Data)
- `prepareToEnd` — saves session state to Rust on window close (dual-write)
- `applyRustSessionState:` — applies ScSessionState to Core Data + scroll position restore
- `saveSessionToRust` — serialises session state including NSPoint scroll position

**`TSSTPage.m`**
- `prepThumbnail` — Rust Lanczos3 thumbnail generation (AppKit fallback for animated GIF)
- `pageImage` — Rust pre-scaling for images >2048px to reduce memory pressure; CALayer GPU rendering retained

### Fixed

- **Sprint 12 index alignment bug**: `nestedArchiveContents` stored XADMaster raw counters as page indices, but `sc_archive_read_page()` expected Rust sorted-image-only indices. Sprint 13 fixed by using ScPageList iteration index.
- Removed `solidDirectory` disk-cache creation (consumer removed in Sprint 12)
- Removed nested `unsafe {}` blocks inside `unsafe extern "C" fn` (Rust 2024 compatibility)

### Infrastructure

- `libsimplecomic.a` — rebuilt as universal binary (arm64 + x86_64) after each sprint
- `SimpleComicCore` Swift Package — `simplecomic.swift` bindings regenerated from updated UDL
- 22 new Rust unit tests across `sc-ffi`, `sc-archive`, `sc-storage`

---

## Prior Phases (2026-06-01)

### Phase 1-5 (Sprints 1-10)
- Rust workspace setup, sc-archive (ZIP/CBZ/RAR/7z/TAR), sc-image (scale/rotate/composite/thumbnail), sc-storage (SQLite session), sc-ffi (uniffi 0.29 bindings), Swift XCTest integration

---
