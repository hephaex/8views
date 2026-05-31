use image::{imageops, DynamicImage};
use sc_core::types::Rotation;

use crate::scale::{scale_image, ScaleOptions};

/// Apply rotation to an image.
///
/// `R90` rotates 90° clockwise; `R270` rotates 90° counter-clockwise
/// (equivalently 270° clockwise).
///
/// # Examples
///
/// ```
/// use image::DynamicImage;
/// use sc_core::types::Rotation;
/// use sc_image::apply_rotation;
///
/// let img = DynamicImage::new_rgb8(100, 200);
/// let rotated = apply_rotation(img, Rotation::R90);
/// assert_eq!(rotated.width(), 200);
/// assert_eq!(rotated.height(), 100);
/// ```
pub fn apply_rotation(img: DynamicImage, rotation: Rotation) -> DynamicImage {
    match rotation {
        Rotation::R0 => img,
        Rotation::R90 => DynamicImage::ImageRgba8(imageops::rotate90(&img.to_rgba8())),
        Rotation::R180 => DynamicImage::ImageRgba8(imageops::rotate180(&img.to_rgba8())),
        Rotation::R270 => DynamicImage::ImageRgba8(imageops::rotate270(&img.to_rgba8())),
    }
}

/// Scale the image first (reducing pixel count), then apply rotation.
///
/// This ordering is preferred because rotating a large image before scaling
/// wastes CPU time on pixels that will be discarded. By scaling first the
/// rotation operates on the smallest pixel buffer needed for display.
///
/// # Examples
///
/// ```
/// use image::DynamicImage;
/// use sc_core::types::{Rotation, ScaleMode};
/// use sc_image::{scale::ScaleOptions, scale_then_rotate};
///
/// let img = DynamicImage::new_rgb8(400, 600);
/// let opts = ScaleOptions {
///     mode: ScaleMode::FitWindow,
///     window_width: 800,
///     window_height: 600,
/// };
/// let out = scale_then_rotate(&img, &opts, Rotation::R90);
/// // 400×600 already fits in 800×600, so scale is identity.
/// // R90 swaps dimensions → 600×400.
/// assert_eq!(out.width(), 600);
/// assert_eq!(out.height(), 400);
/// ```
pub fn scale_then_rotate(
    img: &DynamicImage,
    scale_opts: &ScaleOptions,
    rotation: Rotation,
) -> DynamicImage {
    let scaled = scale_image(img, scale_opts);
    apply_rotation(scaled, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use sc_core::types::{Rotation, ScaleMode};

    fn make_img(w: u32, h: u32) -> DynamicImage {
        DynamicImage::new_rgb8(w, h)
    }

    #[test]
    fn r0_no_change() {
        let img = make_img(200, 300);
        let out = apply_rotation(img, Rotation::R0);
        assert_eq!(out.width(), 200);
        assert_eq!(out.height(), 300);
    }

    #[test]
    fn r90_swaps_dimensions() {
        let img = make_img(200, 300);
        let out = apply_rotation(img, Rotation::R90);
        assert_eq!(out.width(), 300);
        assert_eq!(out.height(), 200);
    }

    #[test]
    fn r180_same_dimensions() {
        let img = make_img(200, 300);
        let out = apply_rotation(img, Rotation::R180);
        assert_eq!(out.width(), 200);
        assert_eq!(out.height(), 300);
    }

    #[test]
    fn r270_swaps_dimensions() {
        let img = make_img(200, 300);
        let out = apply_rotation(img, Rotation::R270);
        assert_eq!(out.width(), 300);
        assert_eq!(out.height(), 200);
    }

    #[test]
    fn scale_then_rotate_correct_order() {
        // 400×600 fits within 800×600 window → scale is identity (400×600).
        // R90 swaps → 600×400.
        let img = make_img(400, 600);
        let opts = ScaleOptions {
            mode: ScaleMode::FitWindow,
            window_width: 800,
            window_height: 600,
        };
        let out = scale_then_rotate(&img, &opts, Rotation::R90);
        assert_eq!(out.width(), 600);
        assert_eq!(out.height(), 400);
    }
}
