use anyhow::Result;
use image::DynamicImage;
use sc_core::types::ImageMetadata;

/// Load a raw image buffer into a DynamicImage.
pub struct ImageLoader;

impl ImageLoader {
    pub fn load_bytes(
        data: &[u8],
        filename: &str,
        index: usize,
    ) -> Result<(DynamicImage, ImageMetadata)> {
        let img = image::load_from_memory(data)?;
        let meta = ImageMetadata {
            width: img.width(),
            height: img.height(),
            filename: filename.to_owned(),
            index,
        };
        Ok((img, meta))
    }

    pub fn load_file(
        path: &std::path::Path,
        index: usize,
    ) -> Result<(DynamicImage, ImageMetadata)> {
        let data = std::fs::read(path)?;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Self::load_bytes(&data, &filename, index)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests added in Sprint 7 with fixture images.
}
