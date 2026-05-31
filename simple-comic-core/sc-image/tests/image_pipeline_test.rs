use image::DynamicImage;
use sc_core::types::{Rotation, ScaleMode};
use sc_image::{
    apply_rotation,
    cache::ImageCache,
    compositor::Compositor,
    loader::ImageLoader,
    scale::{scale_image, ScaleOptions},
};

/// Verified 1x1 RGB PNG (69 bytes). Generated via Python zlib.compress(level=6), CRC-validated.
const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0xC9, 0xFE, 0x92, 0xEF, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

// ---------------------------------------------------------------------------
// Loader tests
// ---------------------------------------------------------------------------

#[test]
fn load_png_bytes_returns_metadata() {
    let (img, meta) = ImageLoader::load_bytes(MINIMAL_PNG, "test.png", 0)
        .expect("load_bytes should succeed for a valid PNG");

    assert_eq!(meta.width, 1, "metadata width must be 1");
    assert_eq!(meta.height, 1, "metadata height must be 1");
    assert_eq!(meta.filename, "test.png", "metadata filename must match");
    assert_eq!(img.width(), 1, "image width must be 1");
    assert_eq!(img.height(), 1, "image height must be 1");
}

#[test]
fn load_invalid_bytes_returns_err() {
    let result = ImageLoader::load_bytes(b"not an image", "bad.png", 0);
    assert!(result.is_err(), "loading garbage bytes must return Err");
}

#[test]
fn load_from_png_bytes_roundtrip() {
    let (img, meta) = ImageLoader::load_bytes(MINIMAL_PNG, "roundtrip.png", 7)
        .expect("roundtrip load must succeed");

    assert_eq!(img.width(), meta.width, "img.width() must equal meta.width");
    assert_eq!(
        img.height(),
        meta.height,
        "img.height() must equal meta.height"
    );
    assert_eq!(meta.width, 1);
    assert_eq!(meta.height, 1);
    assert_eq!(meta.index, 7, "index must be preserved");
}

// ---------------------------------------------------------------------------
// Scale tests
// ---------------------------------------------------------------------------

#[test]
fn scale_fit_window_downscales() {
    let img = DynamicImage::new_rgb8(2000, 3000);
    let opts = ScaleOptions {
        mode: ScaleMode::FitWindow,
        window_width: 800,
        window_height: 600,
    };
    let out = scale_image(&img, &opts);
    assert!(
        out.width() <= 800,
        "FitWindow width {} must be <= 800",
        out.width()
    );
    assert!(
        out.height() <= 600,
        "FitWindow height {} must be <= 600",
        out.height()
    );
}

#[test]
fn scale_fit_width_preserves_ratio() {
    let img = DynamicImage::new_rgb8(200, 400);
    let opts = ScaleOptions {
        mode: ScaleMode::FitWidth,
        window_width: 400,
        window_height: 0,
    };
    let out = scale_image(&img, &opts);
    assert_eq!(out.width(), 400, "FitWidth must produce exact target width");
    assert_eq!(
        out.height(),
        800,
        "FitWidth must scale height proportionally (400/200 * 400 = 800)"
    );
}

#[test]
fn scale_original_no_change() {
    let img = DynamicImage::new_rgb8(100, 200);
    let opts = ScaleOptions {
        mode: ScaleMode::Original,
        window_width: 9999,
        window_height: 9999,
    };
    let out = scale_image(&img, &opts);
    assert_eq!(out.width(), 100, "Original mode must not change width");
    assert_eq!(out.height(), 200, "Original mode must not change height");
}

// ---------------------------------------------------------------------------
// Compositor tests
// ---------------------------------------------------------------------------

#[test]
fn compositor_two_page_total_width() {
    let left = DynamicImage::new_rgb8(200, 300);
    let right = DynamicImage::new_rgb8(200, 300);
    let out = Compositor::two_page_spread(&left, &right);
    assert_eq!(out.width(), 400, "same-size spread width must be 400");
    assert_eq!(out.height(), 300, "same-size spread height must be 300");
}

#[test]
fn compositor_different_heights_normalizes() {
    let left = DynamicImage::new_rgb8(200, 400);
    let right = DynamicImage::new_rgb8(200, 200);
    let out = Compositor::two_page_spread(&left, &right);
    // The right image (200x200) is scaled to height 400, which doubles its
    // width to 400. Total width = left(200) + right_scaled(400) = 600.
    assert_eq!(
        out.height(),
        400,
        "spread height must equal the taller page"
    );
    assert_eq!(
        out.width(),
        600,
        "spread width must account for right page scaled to match left height"
    );
}

// ---------------------------------------------------------------------------
// Cache tests
// ---------------------------------------------------------------------------

#[test]
fn cache_miss_then_hit() {
    let mut cache = ImageCache::new(50);

    assert!(
        cache.get(0).is_none(),
        "cache miss: key 0 must not exist before insert"
    );

    let img = DynamicImage::new_rgb8(10, 10);
    cache.insert(0, img);

    assert!(
        cache.get(0).is_some(),
        "cache hit: key 0 must exist after insert"
    );
}

#[test]
fn cache_evicts_on_overflow() {
    let mut cache = ImageCache::new(3);

    for i in 0..4usize {
        cache.insert(i, DynamicImage::new_rgb8(1, 1));
    }

    assert_eq!(
        cache.len(),
        3,
        "LRU cache with capacity 3 must hold exactly 3 entries after 4 inserts"
    );
}

// ---------------------------------------------------------------------------
// Rotation tests
// ---------------------------------------------------------------------------

#[test]
fn rotate_90_loads_and_rotates() {
    let (img, _meta) = ImageLoader::load_bytes(MINIMAL_PNG, "rot90.png", 0)
        .expect("load_bytes must succeed for valid PNG");
    // 1×1 is square — rotating 90° leaves dimensions unchanged.
    let out = apply_rotation(img, Rotation::R90);
    assert_eq!(out.width(), 1, "1x1 rotated 90° must remain width 1");
    assert_eq!(out.height(), 1, "1x1 rotated 90° must remain height 1");
}

#[test]
fn rotate_180_preserves_dimensions() {
    let img = DynamicImage::new_rgb8(100, 200);
    let out = apply_rotation(img, Rotation::R180);
    assert_eq!(out.width(), 100, "R180 must preserve width");
    assert_eq!(out.height(), 200, "R180 must preserve height");
}

#[test]
fn rotate_270_swaps_on_non_square() {
    let img = DynamicImage::new_rgb8(100, 200);
    let out = apply_rotation(img, Rotation::R270);
    assert_eq!(
        out.width(),
        200,
        "R270 must swap: width becomes original height"
    );
    assert_eq!(
        out.height(),
        100,
        "R270 must swap: height becomes original width"
    );
}
