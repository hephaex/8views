/// Test fixture helpers for building synthetic CBZ and TAR.GZ archives.
///
/// This module is compiled only when `cfg(test)` is active (unit tests,
/// integration tests, doc tests).  Integration tests inside `tests/` are
/// compiled as a separate crate that links against `sc_archive` with the
/// `test` profile, so the `#[cfg(test)]` guard on the parent module declaration
/// in `lib.rs` is sufficient to keep this out of release builds.
///
/// # Usage
///
/// ```rust,ignore
/// use sc_archive::fixtures::make_cbz;
///
/// let (tmp, names) = make_cbz(5);
/// ```
use std::io::Write;

/// A minimal valid 1×1 RGB PNG image (67 bytes).
///
/// Bytes were extracted from a hand-crafted PNG that passes
/// `image::load_from_memory` without error.
pub const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // width=1, height=1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB, IHDR CRC
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT length + type
    0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // zlib-compressed row
    0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, // IDAT CRC
    0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND length + type
    0x44, 0xAE, 0x42, 0x60, 0x82, // IEND CRC
];

/// Format a page filename with zero-padded index, e.g. `page003.png`.
fn page_name(n: usize) -> String {
    format!("page{n:03}.png")
}

/// Create a temporary CBZ (ZIP) file containing `page_count` 1×1 PNG images.
///
/// Files are named `page001.png` … `pageNNN.png` and are written in ascending
/// order so the archive reader's natural-sort logic is exercised.
///
/// # Returns
///
/// `(NamedTempFile, filenames_in_insertion_order)` — the caller must keep
/// `NamedTempFile` alive for the duration of the test; dropping it deletes
/// the underlying file.
pub fn make_cbz(page_count: usize) -> (tempfile::NamedTempFile, Vec<String>) {
    let tmp = tempfile::Builder::new()
        .suffix(".cbz")
        .tempfile()
        .expect("failed to create temp cbz");

    let file = tmp.reopen().expect("failed to reopen temp cbz");
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut names = Vec::with_capacity(page_count);
    for i in 1..=page_count {
        let name = page_name(i);
        zip.start_file(&name, options).expect("zip start_file");
        zip.write_all(MINIMAL_PNG).expect("zip write_all");
        names.push(name);
    }
    zip.finish().expect("zip finish");

    (tmp, names)
}

/// Create a temporary CBZ file with an **arbitrary** set of filenames.
///
/// Useful for testing natural-sort ordering when filenames are not in
/// ascending order inside the archive.
pub fn make_cbz_with_names(names: &[&str]) -> tempfile::NamedTempFile {
    let tmp = tempfile::Builder::new()
        .suffix(".cbz")
        .tempfile()
        .expect("failed to create temp cbz");

    let file = tmp.reopen().expect("failed to reopen temp cbz");
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for name in names {
        zip.start_file(*name, options).expect("zip start_file");
        zip.write_all(MINIMAL_PNG).expect("zip write_all");
    }
    zip.finish().expect("zip finish");

    tmp
}

/// Create a temporary TAR.GZ file containing `page_count` 1×1 PNG images.
///
/// Files are named `page001.png` … `pageNNN.png`.
///
/// # Returns
///
/// `(NamedTempFile, filenames_in_insertion_order)`.
pub fn make_tar_gz(page_count: usize) -> (tempfile::NamedTempFile, Vec<String>) {
    let tmp = tempfile::Builder::new()
        .suffix(".tar.gz")
        .tempfile()
        .expect("failed to create temp tar.gz");

    let file = tmp.reopen().expect("failed to reopen temp tar.gz");
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    let mut names = Vec::with_capacity(page_count);
    for i in 1..=page_count {
        let name = page_name(i);
        let mut header = tar::Header::new_gnu();
        header.set_size(MINIMAL_PNG.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &name, MINIMAL_PNG)
            .expect("tar append_data");
        names.push(name);
    }
    tar.finish().expect("tar finish");

    (tmp, names)
}
