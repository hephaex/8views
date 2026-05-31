use image::{DynamicImage, GenericImage, RgbaImage};

/// Composite two page images side-by-side for two-page spread view.
/// Mirrors TSSTPageView compositing logic.
pub struct Compositor;

impl Compositor {
    /// Place left and right images side by side, normalizing heights.
    /// Returns a single RGBA image with both pages at equal height.
    pub fn two_page_spread(left: &DynamicImage, right: &DynamicImage) -> DynamicImage {
        let target_h = left.height().max(right.height());

        let scaled_left = scale_to_height(left, target_h).to_rgba8();
        let scaled_right = scale_to_height(right, target_h).to_rgba8();

        let total_w = scaled_left.width() + scaled_right.width();
        let mut canvas = DynamicImage::ImageRgba8(RgbaImage::new(total_w, target_h));

        canvas.copy_from(&scaled_left, 0, 0).unwrap_or(());
        canvas
            .copy_from(&scaled_right, scaled_left.width(), 0)
            .unwrap_or(());

        canvas
    }
}

fn scale_to_height(img: &DynamicImage, height: u32) -> DynamicImage {
    if img.height() == height {
        return img.clone();
    }
    let scale = height as f64 / img.height() as f64;
    let new_w = (img.width() as f64 * scale).round() as u32;
    img.resize_exact(new_w.max(1), height, image::imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_page_same_size() {
        let left = DynamicImage::new_rgb8(200, 300);
        let right = DynamicImage::new_rgb8(200, 300);
        let out = Compositor::two_page_spread(&left, &right);
        assert_eq!(out.width(), 400);
        assert_eq!(out.height(), 300);
    }

    #[test]
    fn two_page_different_heights() {
        let left = DynamicImage::new_rgb8(200, 400);
        let right = DynamicImage::new_rgb8(200, 200);
        let out = Compositor::two_page_spread(&left, &right);
        assert_eq!(out.height(), 400);
        // right is scaled to 400h so width doubles: 200 * (400/200) = 400
        assert_eq!(out.width(), 200 + 400);
    }
}
