//! End-to-end integration tests for the image pipeline.
//!
//! These tests exercise the complete path from CBZ archive through image
//! loading, scaling, rotation, and thumbnail generation using the public APIs
//! exposed by `simplecomic` (the `sc-ffi` library crate, whose lib name is
//! `simplecomic`), `sc_image`, and `sc_archive`.
//!
//! # Fixture
//!
//! `make_cbz_temp` creates a synthetic CBZ (ZIP) archive in a temporary file
//! containing `page_count` identical 1×1 RGB PNG images.  The `NamedTempFile`
//! must be kept alive for the duration of each test; dropping it deletes the
//! underlying file.

use std::io::Write as _;

/// Minimal valid 1×1 RGB PNG (69 bytes) with correct CRCs.
///
/// Generated via Python's `zlib` and `struct` modules.  All chunk CRCs are
/// correct so `image::load_from_memory` accepts the bytes without error and
/// decodes to a 1-pixel-wide, 1-pixel-tall image.
const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // width=1, height=1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB, IHDR CRC (part)
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IHDR CRC + IDAT length + type
    0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // zlib-compressed RGB row
    0x00, 0x03, 0x01, 0x01, 0x00, 0xC9, 0xFE, 0x92, // IDAT data + CRC (part)
    0xEF, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IDAT CRC + IEND length + type
    0x44, 0xAE, 0x42, 0x60, 0x82, // IEND CRC
];

/// Create a temporary CBZ (ZIP) file containing `page_count` 1×1 PNG images.
///
/// Files are named `page001.png` … `pageNNN.png` in ascending order.
///
/// The returned `NamedTempFile` must remain alive for the duration of the
/// test; dropping it deletes the underlying file on disk.
fn make_cbz_temp(page_count: usize) -> tempfile::NamedTempFile {
    let tmp = tempfile::Builder::new()
        .suffix(".cbz")
        .tempfile()
        .expect("failed to create temp cbz");

    let file = tmp.reopen().expect("failed to reopen temp cbz");
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for i in 1..=page_count {
        let name = format!("page{i:03}.png");
        zip.start_file(&name, options).expect("zip start_file");
        zip.write_all(MINIMAL_PNG).expect("zip write_all");
    }
    zip.finish().expect("zip finish");

    tmp
}

// ---------------------------------------------------------------------------
// Test 1: archive → image load
// ---------------------------------------------------------------------------

/// Verify that `archive_list_pages` returns the correct entry count and that
/// `archive_read_page` → `ImageLoader::load_bytes` produces a 1×1 image with
/// correct metadata.
#[test]
fn archive_to_image_load() {
    let tmp = make_cbz_temp(3);
    let path = tmp.path().to_str().expect("path is valid UTF-8");

    // List pages — expect exactly 3 entries.
    let pages = simplecomic::archive_list_pages(path).expect("archive_list_pages failed");
    assert_eq!(pages.len(), 3, "expected 3 pages, got {}", pages.len());

    // Read the first page and decode it.
    let bytes = simplecomic::archive_read_page(path, 0).expect("archive_read_page(0) failed");
    assert!(!bytes.is_empty(), "page bytes must not be empty");

    let (img, meta) =
        sc_image::ImageLoader::load_bytes(&bytes, "page001.png", 0).expect("load_bytes failed");

    assert_eq!(meta.width, 1, "expected width 1, got {}", meta.width);
    assert_eq!(meta.height, 1, "expected height 1, got {}", meta.height);
    assert_eq!(meta.filename, "page001.png");
    assert_eq!(meta.index, 0);
    assert_eq!(img.width(), 1);
    assert_eq!(img.height(), 1);
}

// ---------------------------------------------------------------------------
// Test 2: archive read first + scale
// ---------------------------------------------------------------------------

/// Verify that `archive_read_first_image` feeds correctly into `scale_image`
/// with `FitWindow`.  A 1×1 image scaled into an 800×600 window must not
/// exceed those bounds (and since it is already smaller, it stays at 1×1).
#[test]
fn archive_read_first_and_scale() {
    use sc_core::types::ScaleMode;
    use sc_image::scale::scale_image;

    let tmp = make_cbz_temp(5);
    let path = tmp.path().to_str().expect("path is valid UTF-8");

    let bytes =
        simplecomic::archive_read_first_image(path).expect("archive_read_first_image failed");
    assert!(!bytes.is_empty(), "first image bytes must not be empty");

    let (img, _meta) =
        sc_image::ImageLoader::load_bytes(&bytes, "page001.png", 0).expect("load_bytes failed");

    assert_eq!(img.width(), 1);
    assert_eq!(img.height(), 1);

    let opts = sc_image::ScaleOptions {
        mode: ScaleMode::FitWindow,
        window_width: 800,
        window_height: 600,
    };
    let scaled = scale_image(&img, &opts);

    // 1×1 already fits inside 800×600 — scale factor is min(1.0) → identity.
    assert_eq!(scaled.width(), 1, "width should remain 1");
    assert_eq!(scaled.height(), 1, "height should remain 1");
}

// ---------------------------------------------------------------------------
// Test 3: archive → thumbnails (parallel)
// ---------------------------------------------------------------------------

/// Read all pages from a 5-page CBZ, generate thumbnails in parallel, and
/// verify that the sorted result contains exactly 5 entries in ascending index
/// order.
#[test]
fn archive_to_thumbnails_parallel() {
    let tmp = make_cbz_temp(5);
    let path = tmp.path().to_str().expect("path is valid UTF-8");

    let pages = simplecomic::archive_list_pages(path).expect("archive_list_pages failed");
    assert_eq!(pages.len(), 5);

    // Collect (index, raw_bytes) for each page.
    let entries: Vec<(usize, Vec<u8>)> = pages
        .iter()
        .map(|p| {
            let bytes = simplecomic::archive_read_page(path, p.index)
                .unwrap_or_else(|_| panic!("failed to read page {}", p.index));
            (p.index as usize, bytes)
        })
        .collect();

    let spec = sc_image::ThumbnailSpec::default(); // 128×128
    let thumbnails = sc_image::generate_thumbnails_sorted(&entries, spec);

    assert_eq!(
        thumbnails.len(),
        5,
        "expected 5 thumbnails, got {}",
        thumbnails.len()
    );

    // Verify ascending sort order.
    for (expected_idx, (actual_idx, _)) in thumbnails.iter().enumerate() {
        assert_eq!(
            *actual_idx, expected_idx,
            "thumbnail at position {expected_idx} has index {actual_idx}"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 4: full pipeline — scale then rotate
// ---------------------------------------------------------------------------

/// Exercise the complete pipeline: CBZ → bytes → image → scale → rotate.
/// Asserts that no panic occurs and that the result is a valid image.
#[test]
fn full_pipeline_scale_and_rotate() {
    use sc_core::types::{Rotation, ScaleMode};

    let tmp = make_cbz_temp(1);
    let path = tmp.path().to_str().expect("path is valid UTF-8");

    let bytes =
        simplecomic::archive_read_first_image(path).expect("archive_read_first_image failed");
    let (img, _meta) =
        sc_image::ImageLoader::load_bytes(&bytes, "page001.png", 0).expect("load_bytes failed");

    let opts = sc_image::ScaleOptions {
        mode: ScaleMode::FitWindow,
        window_width: 800,
        window_height: 600,
    };

    // scale_then_rotate scales first (identity for 1×1) then applies R90.
    // R90 on a 1×1 image swaps dimensions, but 1==1 so dimensions stay 1×1.
    let out = sc_image::scale_then_rotate(&img, &opts, Rotation::R90);

    // The result must be a non-zero image — no panic before reaching here.
    assert!(out.width() >= 1, "output width must be at least 1");
    assert!(out.height() >= 1, "output height must be at least 1");
}

// ---------------------------------------------------------------------------
// Test 5: archive → thumbnail via FFI (3 pages)
// ---------------------------------------------------------------------------

/// Read 3 pages through the `sc_ffi` archive API, confirm bytes are non-empty
/// for each, then feed them into `generate_thumbnails_sorted` and verify the
/// count.
#[test]
fn archive_to_thumbnail_via_ffi() {
    let tmp = make_cbz_temp(3);
    let path = tmp.path().to_str().expect("path is valid UTF-8");

    let mut entries: Vec<(usize, Vec<u8>)> = Vec::new();

    for i in 0..3u32 {
        let bytes = simplecomic::archive_read_page(path, i)
            .unwrap_or_else(|_| panic!("failed to read page {i}"));
        assert!(!bytes.is_empty(), "page {i} bytes must not be empty");
        entries.push((i as usize, bytes));
    }

    let spec = sc_image::ThumbnailSpec::default(); // 128×128
    let thumbnails = sc_image::generate_thumbnails_sorted(&entries, spec);

    assert_eq!(
        thumbnails.len(),
        3,
        "expected 3 thumbnails, got {}",
        thumbnails.len()
    );
}
