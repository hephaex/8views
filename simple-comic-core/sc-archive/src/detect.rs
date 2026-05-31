use std::{fs::File, io::Read, path::Path};

/// Archive format identified by magic bytes in the file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP / CBZ — magic `PK\x03\x04`
    Zip,
    /// 7-Zip — magic `7z\xBC\xAF\x27\x1C`
    SevenZ,
    /// RAR — magic `Rar!\x1A\x07`
    Rar,
    /// gzip-compressed tar — magic `\x1F\x8B`
    TarGz,
    /// bzip2-compressed tar — magic `BZh`
    TarBz2,
    /// xz-compressed tar — magic `\xFD7zXZ\x00`
    TarXz,
    /// Plain tar (ustar header at offset 257) — not probed here
    Tar,
    /// Unrecognised or unreadable file
    Unknown,
}

const SIG_ZIP: &[u8] = &[0x50, 0x4B, 0x03, 0x04];
const SIG_7Z: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
const SIG_RAR: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07];
const SIG_GZ: &[u8] = &[0x1F, 0x8B];
const SIG_BZ2: &[u8] = &[0x42, 0x5A, 0x68];
const SIG_XZ: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

/// Detect the archive format of `path` by inspecting its first 8 bytes.
///
/// Returns [`ArchiveFormat::Unknown`] if the file cannot be opened, is shorter
/// than the longest signature being matched, or does not match any known
/// signature.
///
/// # Examples
///
/// ```no_run
/// use sc_archive::detect::{detect_format, ArchiveFormat};
/// use std::path::Path;
///
/// let fmt = detect_format(Path::new("archive.bin"));
/// assert!(matches!(fmt, ArchiveFormat::Zip | ArchiveFormat::Unknown));
/// ```
pub fn detect_format(path: &Path) -> ArchiveFormat {
    let mut buf = [0u8; 8];
    let n = match read_header(path, &mut buf) {
        Some(n) => n,
        None => return ArchiveFormat::Unknown,
    };

    let header = &buf[..n];

    if starts_with(header, SIG_ZIP) {
        ArchiveFormat::Zip
    } else if starts_with(header, SIG_7Z) {
        ArchiveFormat::SevenZ
    } else if starts_with(header, SIG_RAR) {
        ArchiveFormat::Rar
    } else if starts_with(header, SIG_XZ) {
        ArchiveFormat::TarXz
    } else if starts_with(header, SIG_BZ2) {
        ArchiveFormat::TarBz2
    } else if starts_with(header, SIG_GZ) {
        ArchiveFormat::TarGz
    } else {
        ArchiveFormat::Unknown
    }
}

/// Read up to `buf.len()` bytes from the start of `path`.
/// Returns `Some(n)` with bytes read, or `None` on I/O error.
fn read_header(path: &Path, buf: &mut [u8]) -> Option<usize> {
    let mut file = File::open(path).ok()?;
    let n = file.read(buf).ok()?;
    Some(n)
}

#[inline]
fn starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len() && &haystack[..needle.len()] == needle
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(bytes: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile creation failed");
        f.write_all(bytes).expect("tempfile write failed");
        f
    }

    const MINIMAL_ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04, 0, 0, 0, 0];
    const ZERO_BYTES: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0];

    #[test]
    fn detects_zip() {
        let tmp = write_temp(MINIMAL_ZIP_MAGIC);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::Zip);
    }

    #[test]
    fn detects_unknown_for_zeros() {
        let tmp = write_temp(ZERO_BYTES);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::Unknown);
    }

    #[test]
    fn detects_7z() {
        let magic: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0, 0];
        let tmp = write_temp(magic);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::SevenZ);
    }

    #[test]
    fn detects_rar() {
        let magic: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0, 0];
        let tmp = write_temp(magic);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::Rar);
    }

    #[test]
    fn detects_tarbz2() {
        let magic: &[u8] = &[0x42, 0x5A, 0x68, 0, 0, 0, 0, 0];
        let tmp = write_temp(magic);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::TarBz2);
    }

    #[test]
    fn detects_tarxz() {
        let magic: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, 0, 0];
        let tmp = write_temp(magic);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::TarXz);
    }

    #[test]
    fn detects_targz() {
        let magic: &[u8] = &[0x1F, 0x8B, 0, 0, 0, 0, 0, 0];
        let tmp = write_temp(magic);
        assert_eq!(detect_format(tmp.path()), ArchiveFormat::TarGz);
    }

    #[test]
    fn returns_unknown_for_nonexistent_path() {
        let fmt = detect_format(Path::new("/nonexistent/path/that/does/not/exist.bin"));
        assert_eq!(fmt, ArchiveFormat::Unknown);
    }
}
