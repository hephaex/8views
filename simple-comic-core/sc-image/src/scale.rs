use image::{imageops::FilterType, DynamicImage};
use sc_core::types::ScaleMode;

#[derive(Debug, Clone, Copy)]
pub struct ScaleOptions {
    pub mode: ScaleMode,
    pub window_width: u32,
    pub window_height: u32,
}

pub fn scale_image(img: &DynamicImage, opts: &ScaleOptions) -> DynamicImage {
    match opts.mode {
        ScaleMode::Original => img.clone(),
        ScaleMode::FitWindow => fit_window(img, opts.window_width, opts.window_height),
        ScaleMode::FitWidth => fit_width(img, opts.window_width),
    }
}

fn fit_window(img: &DynamicImage, w: u32, h: u32) -> DynamicImage {
    let (iw, ih) = (img.width(), img.height());
    let scale = (w as f64 / iw as f64).min(h as f64 / ih as f64).min(1.0);
    let nw = (iw as f64 * scale).round() as u32;
    let nh = (ih as f64 * scale).round() as u32;
    img.resize_exact(nw.max(1), nh.max(1), FilterType::Lanczos3)
}

fn fit_width(img: &DynamicImage, w: u32) -> DynamicImage {
    let (iw, ih) = (img.width(), img.height());
    if iw == w {
        return img.clone();
    }
    let scale = w as f64 / iw as f64;
    let nh = (ih as f64 * scale).round() as u32;
    img.resize_exact(w, nh.max(1), FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    fn make_img(w: u32, h: u32) -> DynamicImage {
        DynamicImage::new_rgb8(w, h)
    }

    #[test]
    fn fit_window_portrait() {
        let img = make_img(200, 400);
        let opts = ScaleOptions {
            mode: ScaleMode::FitWindow,
            window_width: 800,
            window_height: 600,
        };
        let out = scale_image(&img, &opts);
        assert!(out.width() <= 800);
        assert!(out.height() <= 600);
    }

    #[test]
    fn fit_width_scales_proportionally() {
        let img = make_img(100, 200);
        let opts = ScaleOptions {
            mode: ScaleMode::FitWidth,
            window_width: 400,
            window_height: 0,
        };
        let out = scale_image(&img, &opts);
        assert_eq!(out.width(), 400);
        assert_eq!(out.height(), 800);
    }
}
