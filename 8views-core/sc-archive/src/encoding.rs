use chardetng::EncodingDetector;

/// Detect encoding of raw bytes and convert to UTF-8.
/// Fallback to lossy UTF-8 conversion if detection fails.
pub fn decode_filename(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_owned();
    }

    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let (decoded, _, _) = encoding.decode(bytes);
    decoded.into_owned()
}

/// Check if a filename looks like an image file we should include.
pub fn is_image_entry(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    matches!(
        std::path::Path::new(&lower)
            .extension()
            .and_then(|e| e.to_str()),
        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_entry() {
        assert!(is_image_entry("page01.jpg"));
        assert!(is_image_entry("COVER.PNG"));
        assert!(is_image_entry("image.webp"));
        assert!(!is_image_entry("readme.txt"));
        assert!(!is_image_entry("metadata.xml"));
    }

    #[test]
    fn test_decode_utf8() {
        let bytes = b"page01.jpg";
        assert_eq!(decode_filename(bytes), "page01.jpg");
    }
}
