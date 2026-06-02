# Changelog

All notable changes to the 8views Rust core refactoring (`arc` branch).

---

## [2.0.0-alpha] — 2026-06-02

### Summary

Hybrid Rust core + Swift/ObjC UI architecture. Archive parsing, image pipeline,
session storage, and QuickLook thumbnail/preview are now handled by the Rust layer
(`libeightviews.a`). The Swift UI layer (AppKit, Touch Bar, Core Data) is retained.
(Previously named Simple Comic (legacy fork).)

### Changed (post-alpha, 2026-06-02)

**`EightViewsAppDelegate.m`** — Core Data → Rust SQLite one-time session migration
- `migrateCoreDateSessionsToRust`: called at app launch (before `sessionRelaunch`),
  guarded by `NSUserDefaults@EVRustSessionMigrationDone` (runs once, idempotent).
- Migrates existing Core Data sessions to Rust SQLite for archives not already
  present in the Rust store. Maps all session state: page position, zoom, rotation,
  two-page spread, page order, scale mode, scroll position.
- `sc_session_exists()` C FFI added (cheaper than `sc_session_load` for existence
  checks; no state deserialisation).

### Known Limitations

- **Core Data architecture retained**: `TSSTManagedGroup`, `TSSTManagedSession`,
  `TSSTPage` still subclass `NSManagedObject`. Full removal requires replacing the
  in-memory object model — deferred to a future major sprint.
- **Session migration is one-way**: once migrated, sessions are maintained in both
  Core Data (runtime) and Rust SQLite (persistence). Core Data is still the
  runtime model; Rust SQLite is the persistence target.
- **OCR text search**: Phase 7 deferred. Vision framework stays in Swift; the Rust
  search index (`aho-corasick`/`tantivy`) is not yet connected.
- **Memory validation**: Instruments leak/allocation profiling not yet completed.
  Benchmarked archive and image operations are within PLAN.md targets.
- **XADMaster dependency retained** for solid RAR archives, nested archives, PDF
  extraction, and cover-name lookup (TSSTManagedGroup, TSSTSessionWindowController,
  EightViewsAppDelegate). Full removal requires a Rust page-cache API.

---

## Phase 7: OCR 텍스트 캐시 (2026-06-02, Sprints 28-29)

### Added

**`sc-storage` — OCR text cache**
- `SCHEMA_V3`: `ocr_cache(archive_path, page_index, text_data, created_at, archive_mtime)`
- `SCHEMA_V4`: `ALTER TABLE ocr_cache ADD COLUMN archive_mtime` (idempotent, safe on existing DBs)
- `OcrCache` struct: `store(path, idx, text, mtime)`, `get`, `has`, `has_valid(mtime)`, `clear`

**`sc-ffi` — OCR C FFI**
- `sc_ocr_store(archive_path, page_index, text, archive_mtime_secs)` — store Vision text
- `sc_ocr_has_valid(archive_path, page_index, current_mtime_secs)` — staleness-aware cache check
- `sc_ocr_has(archive_path, page_index)` — existence check (no mtime validation)
- `sc_ocr_get(archive_path, page_index, out_len) → uint8_t*` — retrieve cached text
- `sc_ocr_clear(archive_path)` — remove all cached pages for an archive

**`OCRFindDelegate` protocol**
- `rustSessionArchivePath` property added — returns archive file path for OCR cache keying

### Changed

**`OCRFind.m`** — Two-phase OCR search with persistent cache
- **Fast path** (cache hit, mtime valid): skip Vision inference; search cached text directly.
  Only runs Vision once more on the matched page for text-selection rendering.
- **Slow path** (cache miss): run Vision as before, then store text in Rust cache with archive mtime.
- Result: Vision OCR runs once per page per archive, ever. Re-opening a comic or
  re-searching uses cached text — no Vision re-inference.

**`TSSTSessionWindowController.m`**
- Removed `sc_ocr_clear` from `prepareToEnd` — cache now persists across sessions.
- `sc_ocr_clear` is available for explicit use when removing an archive from the session list.

### Deferred

The full Rust `aho-corasick` replacement of `OCRFind.m`'s ObjC string search is not
included. The current `ocr_rangeOfString:` ObjC search operates on the cached text
returned by `sc_ocr_get`, which is already fast. `aho-corasick` would be a micro-
optimisation that introduces UTF-8/UTF-16 offset mapping complexity.

---

## Phase 9: 성능 검증 (2026-06-02)

### Performance vs PLAN.md Targets (Sprints 17, 23)

| Metric | Target | Measured | Result |
|--------|--------|---------|--------|
| Archive open+list (200p CBZ) | < 500 ms | 1.13 ms | ✅ 443× |
| Page read (200p CBZ) | < 50 ms | 2.01 ms | ✅ 25× |
| QuickLook first image (200p) | < 1 s | 1.72 ms | ✅ 580× |
| Thumbnail parallel (200 entries) | < 3 s | 25.4 ms | ✅ 118× |
| Memory peak (200p CBZ) | < 200 MB | ⏳ | Pending Instruments |

### Benchmark Infrastructure
- `sc-archive/benches/archive_bench.rs`: 50- and 200-page CBZ benchmarks
- `sc-image/benches/image_bench.rs`: scale, compositor, cache, thumbnail (10/50/200)

---

## Phase 8: QuickLook 플러그인 (2026-06-02, Sprints 20-23)

### Changed

**`QuickComic/GenerateThumbnailForURL.m`** (legacy ObjC QuickLook API)
- No-cover path: `sc_archive_read_first_image()` (partial-read, optimised)
- Eliminates full `XADArchive` open for the common thumbnail case

**`QuickComic/GeneratePreviewForURL.m`** (legacy ObjC QuickLook API)
- `sc_archive_open_pages()` → sorted page list
- `sc_archive_read_page()` per page; time-limit + cancel detection retained

**`QuickComic Thumbnailer/ThumbnailProvider.swift`** (modern Swift QLThumbnailProvider)
- No-cover path: `sc_archive_read_first_image()` C FFI
- Named-cover path: `sc_archive_open_pages()` + name search + `sc_archive_read_page()`
- Removed: `XADArchive`, `PartialArchiveParser` (XADMaster)

**`QuickComic Preview/PreviewProvider.swift`** (modern Swift QLPreviewProvider)
- Pre-fetch pattern: first 25 pages + last page read in `providePreview` async body
- QLPreviewReply closure is I/O-free (no N+1 archive re-opens)
- Fixed: page selection was `index >= 25` (showing last 75%) → `index < 25` (first 25 + last)

**`QuickComicCommonSwift.swift`, `PartialArchiveParser.swift`**
- XADMaster code removed; files retained as empty stubs to avoid Xcode project churn

### Added

**`8views.xcodeproj/project.pbxproj`**
- `HEADER_SEARCH_PATHS`, `LIBRARY_SEARCH_PATHS`, `OTHER_LDFLAGS` added to six build
  configurations (main app + QuickComic Thumbnailer + QuickComic Preview, Debug/Release)
- Points to `EightViewsCore/Sources/eightviewsFFI` (headers) and
  `EightViewsCore/Libraries` (pre-built `libeightviews.a`)

---

## Phase 7: OCR 통합 — Deferred (2026-06-02)

Vision framework OCR extraction remains in Swift (`OCRFind.m`).
The Rust search index (`aho-corasick` / `tantivy`) planned in PLAN.md Phase 7
has not been connected. Text selection works via the existing Swift/Vision path.
This phase is deferred to a future sprint.

---

## Phase 6: UI 배선 완료 (2026-06-01 ~ 2026-06-02)

### Added

**Rust C FFI layer (`EightViewsCore/Sources/eightviewsFFI/sc_extras.h`)**
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

**`EightViewsAppDelegate.m`**
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

- `libeightviews.a` — rebuilt as universal binary (arm64 + x86_64) after each sprint
- `EightViewsCore` Swift Package — `eightviews.swift` bindings regenerated from updated UDL
- 22 new Rust unit tests across `sc-ffi`, `sc-archive`, `sc-storage`

---

## Prior Phases (2026-06-01)

### Phase 1-5 (Sprints 1-10)
- Rust workspace setup, sc-archive (ZIP/CBZ/RAR/7z/TAR), sc-image (scale/rotate/composite/thumbnail), sc-storage (SQLite session), sc-ffi (uniffi 0.29 bindings), Swift XCTest integration

---
