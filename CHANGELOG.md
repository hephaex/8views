# Changelog

All notable changes to the Simple Comic Rust core refactoring (`arc` branch).

---

## [2.0.0-alpha] — 2026-06-02

### Summary

Hybrid Rust core + Swift/ObjC UI architecture. Archive parsing, image pipeline,
session storage, and QuickLook thumbnail/preview are now handled by the Rust layer
(`libsimplecomic.a`). The Swift UI layer (AppKit, Touch Bar, Core Data) is retained.

### Known Limitations

- **Core Data sessions**: New sessions are dual-written (Core Data + Rust SQLite).
  Existing Core Data sessions are not migrated to Rust automatically. A migration
  tool (Phase 4) is planned but not included in this alpha.
- **OCR text search**: Phase 7 deferred. Vision framework stays in Swift; the Rust
  search index (`aho-corasick`/`tantivy`) is not yet connected.
- **Memory validation**: Instruments leak/allocation profiling not yet completed.
  Benchmarked archive and image operations are within PLAN.md targets.
- **XADMaster dependency retained** for solid RAR archives, nested archives, PDF
  extraction, and cover-name lookup (TSSTManagedGroup, TSSTSessionWindowController,
  SimpleComicAppDelegate). Full removal requires a Rust page-cache API.

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

**`SimpleComic.xcodeproj/project.pbxproj`**
- `HEADER_SEARCH_PATHS`, `LIBRARY_SEARCH_PATHS`, `OTHER_LDFLAGS` added to six build
  configurations (main app + QuickComic Thumbnailer + QuickComic Preview, Debug/Release)
- Points to `SimpleComicCore/Sources/simplecomicFFI` (headers) and
  `SimpleComicCore/Libraries` (pre-built `libsimplecomic.a`)

---

## Phase 7: OCR 통합 — Deferred (2026-06-02)

Vision framework OCR extraction remains in Swift (`OCRFind.m`).
The Rust search index (`aho-corasick` / `tantivy`) planned in PLAN.md Phase 7
has not been connected. Text selection works via the existing Swift/Vision path.
This phase is deferred to a future sprint.

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
