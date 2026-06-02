use image::{imageops::FilterType, DynamicImage};
use rayon::prelude::*;

/// Thumbnail size configuration.
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailSpec {
    pub width: u32,
    pub height: u32,
}

impl Default for ThumbnailSpec {
    fn default() -> Self {
        Self {
            width: 128,
            height: 128,
        }
    }
}

/// Generate a thumbnail for a single image using Lanczos3 resampling.
///
/// The image is scaled to fit within `spec.width` × `spec.height` while
/// preserving the original aspect ratio.
///
/// # Examples
///
/// ```
/// use sc_image::thumbnail::{ThumbnailSpec, generate_thumbnail};
/// use image::DynamicImage;
///
/// let img = DynamicImage::new_rgb8(400, 600);
/// let spec = ThumbnailSpec { width: 64, height: 64 };
/// let thumb = generate_thumbnail(&img, spec);
/// assert!(thumb.width() <= 64 && thumb.height() <= 64);
/// ```
pub fn generate_thumbnail(img: &DynamicImage, spec: ThumbnailSpec) -> DynamicImage {
    img.resize(spec.width, spec.height, FilterType::Lanczos3)
}

/// Generate thumbnails for multiple raw image buffers in parallel using rayon.
///
/// Returns `(original_index, thumbnail)` pairs in **arbitrary order**.
/// Entries that fail to decode are silently skipped.
///
/// # Arguments
///
/// * `entries` — slice of `(original_index, raw_bytes)` pairs
/// * `spec`    — target thumbnail dimensions
///
/// # Examples
///
/// ```
/// use sc_image::thumbnail::{ThumbnailSpec, generate_thumbnails_parallel};
///
/// let data: Vec<(usize, Vec<u8>)> = vec![];
/// let results = generate_thumbnails_parallel(&data, ThumbnailSpec::default());
/// assert!(results.is_empty());
/// ```
pub fn generate_thumbnails_parallel(
    entries: &[(usize, Vec<u8>)],
    spec: ThumbnailSpec,
) -> Vec<(usize, DynamicImage)> {
    entries
        .par_iter()
        .filter_map(|(idx, bytes)| {
            let img = image::load_from_memory(bytes).ok()?;
            Some((*idx, generate_thumbnail(&img, spec)))
        })
        .collect()
}

/// Generate thumbnails in parallel and return them sorted by original index.
///
/// Identical to [`generate_thumbnails_parallel`] but the result is sorted by
/// `original_index` in ascending order, which is the stable order required for
/// display.
///
/// Entries that fail to decode are silently skipped; the sort is applied to
/// whatever successfully decoded entries remain.
///
/// # Examples
///
/// ```
/// use sc_image::thumbnail::{ThumbnailSpec, generate_thumbnails_sorted};
///
/// let data: Vec<(usize, Vec<u8>)> = vec![];
/// let results = generate_thumbnails_sorted(&data, ThumbnailSpec::default());
/// assert!(results.is_empty());
/// ```
pub fn generate_thumbnails_sorted(
    entries: &[(usize, Vec<u8>)],
    spec: ThumbnailSpec,
) -> Vec<(usize, DynamicImage)> {
    let mut results = generate_thumbnails_parallel(entries, spec);
    results.sort_by_key(|(idx, _)| *idx);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid 1×1 RGB PNG (69 bytes), generated via Python zlib.
    const MINIMAL_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0xC9, 0xFE, 0x92, 0xEF, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn thumbnail_spec_default() {
        let spec = ThumbnailSpec::default();
        assert_eq!(spec.width, 128);
        assert_eq!(spec.height, 128);
    }

    #[test]
    fn generate_single_thumbnail() {
        let img = DynamicImage::new_rgb8(400, 600);
        let spec = ThumbnailSpec {
            width: 64,
            height: 64,
        };
        let thumb = generate_thumbnail(&img, spec);
        assert!(thumb.width() <= 64, "width {} exceeds 64", thumb.width());
        assert!(thumb.height() <= 64, "height {} exceeds 64", thumb.height());
    }

    #[test]
    fn generate_parallel_5_entries() {
        let entries: Vec<(usize, Vec<u8>)> = (0..5).map(|i| (i, MINIMAL_PNG.to_vec())).collect();

        let spec = ThumbnailSpec {
            width: 32,
            height: 32,
        };
        let results = generate_thumbnails_sorted(&entries, spec);

        assert_eq!(
            results.len(),
            5,
            "expected 5 thumbnails, got {}",
            results.len()
        );
        assert_eq!(results[0].0, 0, "first index should be 0");
        assert_eq!(results[4].0, 4, "last index should be 4");
    }
}
