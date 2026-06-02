# 8views
8views is a streamlined comic viewer for macOS. This is a fork from the [original version](https://github.com/arauchfuss/Simple-Comic) since the maintainer of that has gone away.

The basic feature drive is to reduce the number of interactions required to browse a comic.

Quick Comic is a bundled quicklook preview and thumbnail generation plugin for cbr and cbz files.

## Live Text support

8views uses Apple's Optical Character Recognition, on Apple Silicon Macs, to make text in comics selectable for copying and text-to-speech.

* The mouse cursor changes from the Arrow to the I-Beam when over selectable text.
* Click-and-drag to select live text. Recognized words within the dragged out rectangle are selected.
* **Copy**, **Select All**, and **Speak**, on the **Edit** menu work.
* Control-click on selected text for a contextual menu with **Copy** and **Speak**
* ⌘-click-and-drag to add a new selection rectangle to the existing selection.
* Two page spreads, rotations, page ordering, and zooming all work as expected.

## Privacy

We don't collect any user data in the app itself. We know nothing about you and are happy with that.

GitHub collects data when you interact with the project here, but we can't change any of that.

## Rust Core (arc branch)

The `arc` branch is an active refactoring to a **Rust core + Swift/ObjC UI** hybrid architecture.

**Status: v2.0.0 (2026-06-02) — All phases complete**

| Component | Status | Notes |
|-----------|--------|-------|
| Archive engine (ZIP/CBZ/RAR/CBR/7z/TAR) | ✅ Rust | `sc-archive` |
| Image pipeline (scale/rotate/composite/thumbnail) | ✅ Rust | `sc-image` |
| Session persistence (SQLite) | ✅ Rust | `sc-storage`, dual-write with Core Data |
| C FFI bridge (`sc_extras.h`) | ✅ | ObjC ↔ Rust |
| Swift Package (`EightViewsCore`) | ✅ | uniffi bindings |
| UI wiring (AppDelegate, SessionWindowController, PageView) | ✅ Rust | ObjC calls Rust |
| QuickLook thumbnails + previews | ✅ Rust | Both ObjC and Swift extension APIs |
| Performance (PLAN.md targets) | ✅ | All 4 measurable targets met |
| OCR text search index | ✅ Rust | FTS5 via `sc-storage`, `sc_ocr_search` C API + `OCRFind.searchAllCachedPages:` |
| Core Data → SQLite migration tool | ✅ | `migrateCoreDateSessionsToRust` on first launch |
| Memory validation | ✅ | Leaks: 0, RSS: 172 MB Release (target < 200 MB) |

### Building the Rust core

```bash
cd 8views-core
cargo build --release --package sc-ffi --target aarch64-apple-darwin
cargo build --release --package sc-ffi --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/libeightviews.a \
  target/x86_64-apple-darwin/release/libeightviews.a \
  -output ../EightViewsCore/Libraries/libeightviews.a

# Regenerate Swift bindings after UDL changes:
cargo run --bin uniffi-bindgen -- generate \
  sc-ffi/src/eightviews.udl --language swift \
  --out-dir ../EightViewsCore/Sources/EightViewsCore/
```

## Build Instructions

For this to build you need to get the submodules. For that you need to run the following commands.

```
git submodule init
git submodule sync --recursive
git submodule update --init --recursive
```
