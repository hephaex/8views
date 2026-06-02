//! Partial archive reader for QuickLook thumbnail generation.
//!
//! Reads only the first image from an archive with the minimum amount of I/O,
//! mirroring the behaviour of the original `DTPartialArchiveParser.m`.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//! use sc_archive::read_first_image;
//!
//! let bytes = read_first_image(Path::new("my_comic.cbz")).unwrap();
//! assert!(!bytes.is_empty());
//! ```

use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    encoding::{decode_filename, is_image_entry},
    sort::natural_cmp,
};

/// Read only the first image from an archive without fully decompressing it.
///
/// The format is determined from the file extension. Entries are naturally
/// sorted and the lexicographically first image entry is returned.
///
/// # Errors
///
/// Returns an error if:
/// - The path does not exist or cannot be opened.
/// - The format is unsupported (e.g. RAR/CBR without the `unrar-ng` feature).
/// - The archive contains no image entries.
/// - Any I/O or decompression failure occurs.
pub fn read_first_image(path: &Path) -> Result<Vec<u8>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("zip") | Some("cbz") => read_first_zip(path),
        Some("tar") => read_first_tar(path, TarCompression::None),
        Some("gz") | Some("tgz") => read_first_tar(path, TarCompression::Gzip),
        Some("bz2") | Some("tbz2") => read_first_tar(path, TarCompression::Bzip2),
        Some("xz") | Some("txz") => read_first_tar(path, TarCompression::Xz),
        Some("7z") => read_first_sevenz(path),
        Some("rar") | Some("cbr") => {
            anyhow::bail!("RAR not supported: install unrar-ng feature");
        }
        _ => {
            if path.is_dir() {
                read_first_folder(path)
            } else {
                anyhow::bail!("unsupported format: {:?}", path)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ZIP / CBZ
// ---------------------------------------------------------------------------

fn read_first_zip(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open zip: {}", path.display()))?;
    let mut zip =
        zip::ZipArchive::new(file).with_context(|| format!("invalid zip: {}", path.display()))?;

    // Collect image entry names from the central directory (no decompression).
    let mut names: Vec<String> = (0..zip.len())
        .filter_map(|i| {
            let entry = zip.by_index_raw(i).ok()?;
            if entry.is_dir() {
                return None;
            }
            let filename = decode_filename(entry.name_raw());
            if is_image_entry(&filename) {
                Some(filename)
            } else {
                None
            }
        })
        .collect();

    if names.is_empty() {
        anyhow::bail!("no image entries in zip: {}", path.display());
    }

    names.sort_by(|a, b| natural_cmp(a, b));
    let target = &names[0];

    // Reopen and decompress only the winning entry.
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to reopen zip: {}", path.display()))?;
    let mut zip = zip::ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let filename = decode_filename(entry.name_raw());
        if &filename == target {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            std::io::Read::read_to_end(&mut entry, &mut buf)?;
            return Ok(buf);
        }
    }

    anyhow::bail!("entry not found in zip after scan: {}", target)
}

// ---------------------------------------------------------------------------
// TAR family  (None / Gzip / Bzip2 / Xz)
// ---------------------------------------------------------------------------

enum TarCompression {
    None,
    Gzip,
    Bzip2,
    Xz,
}

fn read_first_tar(path: &Path, compression: TarCompression) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open tar: {}", path.display()))?;

    let reader: Box<dyn std::io::Read> = match compression {
        TarCompression::None => Box::new(file),
        TarCompression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
        TarCompression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
        TarCompression::Xz => Box::new(xz2::read::XzDecoder::new(file)),
    };

    // Stream through the archive and collect (name, byte_offset) pairs so we
    // can pick the naturally-first image without reading everything.  Since TAR
    // is a sequential format we must stream to build the list, but we stop as
    // soon as we know the answer for uncompressed tars.  For compressed streams
    // we must walk the full header list first, then re-stream to the target.
    //
    // Strategy: collect all image filenames in order, sort, then re-stream
    // from the beginning until we hit the winner.  Two passes, but decompresses
    // only as far as the first match — ideal for QuickLook where archives are
    // typically small.

    let mut archive = tar::Archive::new(reader);

    // First pass — collect image names preserving stream order (index).
    let mut candidates: Vec<(usize, String)> = Vec::new();
    for (stream_idx, entry) in archive.entries()?.enumerate() {
        let entry = entry?;
        let raw = entry.path_bytes();
        let filename = decode_filename(&raw);
        if is_image_entry(&filename) {
            candidates.push((stream_idx, filename));
        }
    }

    if candidates.is_empty() {
        anyhow::bail!("no image entries in tar: {}", path.display());
    }

    // Natural-sort and find which stream index to seek to.
    candidates.sort_by(|(_, a), (_, b)| natural_cmp(a, b));
    let (target_stream_idx, target_name) = candidates.into_iter().next().unwrap();

    // Second pass — stream again until we reach the target entry.
    let file2 = std::fs::File::open(path)
        .with_context(|| format!("failed to reopen tar: {}", path.display()))?;
    let reader2: Box<dyn std::io::Read> = match compression {
        TarCompression::None => Box::new(file2),
        TarCompression::Gzip => Box::new(flate2::read::GzDecoder::new(file2)),
        TarCompression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file2)),
        TarCompression::Xz => Box::new(xz2::read::XzDecoder::new(file2)),
    };
    let mut archive2 = tar::Archive::new(reader2);

    for (stream_idx, entry) in archive2.entries()?.enumerate() {
        let mut entry = entry?;
        if stream_idx == target_stream_idx {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf)?;
            return Ok(buf);
        }
    }

    anyhow::bail!("entry not found in tar on second pass: {}", target_name)
}

// ---------------------------------------------------------------------------
// 7z
// ---------------------------------------------------------------------------

fn read_first_sevenz(path: &Path) -> Result<Vec<u8>> {
    // First, collect image names from the archive metadata (no decompression).
    let archive =
        sevenz_rust::Archive::open(path).map_err(|e| anyhow::anyhow!("7z open error: {}", e))?;

    let mut names: Vec<String> = archive
        .files
        .iter()
        .filter(|e| !e.is_directory && is_image_entry(&e.name))
        .map(|e| e.name.clone())
        .collect();

    if names.is_empty() {
        anyhow::bail!("no image entries in 7z: {}", path.display());
    }

    names.sort_by(|a, b| natural_cmp(a, b));
    let target = names.into_iter().next().unwrap();

    // Decompress only the winning entry; stop immediately after it is read.
    let mut result: Vec<u8> = Vec::new();
    let found = std::cell::Cell::new(false);

    sevenz_rust::decompress_file_with_extract_fn(
        path,
        Path::new(""),
        |entry, reader, _output_path| {
            if entry.name() == target {
                std::io::Read::read_to_end(reader, &mut result)?;
                found.set(true);
                // Return false to abort further extraction immediately.
                return Ok(false);
            }
            Ok(true)
        },
    )
    .map_err(|e| anyhow::anyhow!("7z extraction error: {}", e))?;

    if !found.get() {
        anyhow::bail!("entry not found in 7z: {}", target);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Directory / folder
// ---------------------------------------------------------------------------

fn read_first_folder(path: &Path) -> Result<Vec<u8>> {
    let mut files: Vec<String> = std::fs::read_dir(path)
        .with_context(|| format!("failed to read directory: {}", path.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if is_image_entry(&name) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    if files.is_empty() {
        anyhow::bail!("no image files in directory: {}", path.display());
    }

    files.sort_by(|a, b| natural_cmp(a, b));
    let first = &files[0];
    let full_path = path.join(first);

    std::fs::read(&full_path)
        .with_context(|| format!("failed to read file: {}", full_path.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonexistent_file_returns_err() {
        let result = read_first_image(Path::new("/no/such/file.cbz"));
        assert!(result.is_err(), "expected Err for nonexistent path");
    }

    #[test]
    fn unsupported_extension_returns_err() {
        // Create a real (empty) temp file with a nonsense extension so the
        // extension-matching branch is hit, not the "file does not exist" branch.
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("archive.xyz");
        std::fs::write(&file, b"not an archive").expect("write");
        let result = read_first_image(&file);
        assert!(result.is_err(), "expected Err for unsupported extension");
    }

    #[test]
    fn rar_extension_returns_err() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("archive.rar");
        std::fs::write(&file, b"dummy").expect("write");
        let result = read_first_image(&file);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("RAR not supported"), "unexpected error: {msg}");
    }

    #[test]
    fn cbr_extension_returns_err() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("archive.cbr");
        std::fs::write(&file, b"dummy").expect("write");
        let result = read_first_image(&file);
        assert!(result.is_err());
    }

    #[test]
    fn empty_directory_returns_err() {
        let dir = tempfile::tempdir().expect("tempdir");
        // No files — should return an error about no images.
        let result = read_first_image(dir.path());
        assert!(result.is_err(), "expected Err for empty directory");
    }

    #[test]
    fn directory_with_image_returns_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Write a minimal PNG-ish payload (just needs to be non-empty; we're
        // not validating image correctness here).
        let img_path = dir.path().join("cover.png");
        let fake_png = b"\x89PNG\r\n\x1a\n";
        std::fs::write(&img_path, fake_png).expect("write");

        let result = read_first_image(dir.path());
        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
        assert_eq!(result.unwrap(), fake_png);
    }

    #[test]
    fn directory_natural_sort_picks_first() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("page10.jpg"), b"ten").expect("write");
        std::fs::write(dir.path().join("page2.jpg"), b"two").expect("write");
        std::fs::write(dir.path().join("page1.jpg"), b"one").expect("write");

        let result = read_first_image(dir.path()).expect("ok");
        // Natural sort picks "page1.jpg" before "page2.jpg" and "page10.jpg".
        assert_eq!(result, b"one");
    }

    /// Build an in-memory ZIP, write to disk, and assert we get the right bytes.
    #[test]
    fn zip_returns_first_image_bytes() {
        use std::io::Write;

        let dir = tempfile::tempdir().expect("tempdir");
        let zip_path = dir.path().join("test.cbz");

        {
            let file = std::fs::File::create(&zip_path).expect("create");
            let mut writer = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            // Add images out of natural order.
            writer.start_file("page10.jpg", opts).expect("start");
            writer.write_all(b"page10_data").expect("write");

            writer.start_file("page2.jpg", opts).expect("start");
            writer.write_all(b"page2_data").expect("write");

            writer.start_file("page1.jpg", opts).expect("start");
            writer.write_all(b"page1_data").expect("write");

            writer.finish().expect("finish");
        }

        let result = read_first_image(&zip_path).expect("ok");
        assert_eq!(result, b"page1_data");
    }
}
