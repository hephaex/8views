mod common;

use sc_archive::open_archive;

#[test]
fn cbz_open_and_list_entries() {
    let (tmp, _names) = common::make_cbz(5);
    let reader = open_archive(tmp.path()).expect("open_archive failed");
    assert_eq!(reader.entries().len(), 5);
    assert_eq!(reader.entries()[0].filename, "page001.png");
    assert_eq!(reader.entries()[4].filename, "page005.png");
}

#[test]
fn cbz_read_entry_returns_bytes() {
    let (tmp, _names) = common::make_cbz(3);
    let mut reader = open_archive(tmp.path()).expect("open_archive failed");
    let bytes = reader.read_entry(0).expect("read_entry(0) failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

#[test]
fn cbz_natural_sort_order() {
    let tmp = common::make_cbz_with_names(&["page10.png", "page2.png", "page1.png"]);
    let reader = open_archive(tmp.path()).expect("open_archive failed");
    let entries = reader.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].filename, "page1.png");
    assert_eq!(entries[1].filename, "page2.png");
    assert_eq!(entries[2].filename, "page10.png");
}

#[test]
fn tar_gz_open_and_list() {
    let (tmp, _names) = common::make_tar_gz(3);
    let reader = open_archive(tmp.path()).expect("open_archive failed for tar.gz");
    assert_eq!(reader.entries().len(), 3);
}

// ---------------------------------------------------------------------------
// Folder tests
// ---------------------------------------------------------------------------

#[test]
fn folder_open_and_list_entries() {
    let (dir, _names) = common::make_folder(4);
    let reader = open_archive(dir.path()).expect("open_archive failed for folder");
    assert_eq!(reader.entries().len(), 4);
    assert_eq!(reader.entries()[0].filename, "page001.png");
}

#[test]
fn folder_read_entry_returns_png() {
    let (dir, _names) = common::make_folder(2);
    let mut reader = open_archive(dir.path()).expect("open_archive failed for folder");
    let bytes = reader.read_entry(0).expect("read_entry(0) failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

// ---------------------------------------------------------------------------
// 7z tests
// ---------------------------------------------------------------------------

#[test]
fn sevenz_open_and_list_entries() {
    let (_stage, tmp_7z, _names) = common::make_sevenz(3);
    let reader = open_archive(tmp_7z.path()).expect("open_archive failed for 7z");
    assert_eq!(reader.entries().len(), 3);
    assert_eq!(reader.entries()[0].filename, "page001.png");
}

#[test]
fn sevenz_read_entry_returns_png() {
    let (_stage, tmp_7z, _names) = common::make_sevenz(2);
    let mut reader = open_archive(tmp_7z.path()).expect("open_archive failed for 7z");
    let bytes = reader.read_entry(0).expect("read_entry(0) failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

// ---------------------------------------------------------------------------
// PartialReader tests
// ---------------------------------------------------------------------------

#[test]
fn partial_reader_cbz_first_image() {
    use sc_archive::read_first_image;
    let (tmp, _names) = common::make_cbz(5);
    let bytes = read_first_image(tmp.path()).expect("read_first_image failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

#[test]
fn partial_reader_tar_gz_first_image() {
    use sc_archive::read_first_image;
    let (tmp, _names) = common::make_tar_gz(3);
    let bytes = read_first_image(tmp.path()).expect("read_first_image failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

#[test]
fn partial_reader_folder_first_image() {
    use sc_archive::read_first_image;
    let (dir, _names) = common::make_folder(3);
    let bytes = read_first_image(dir.path()).expect("read_first_image for folder failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

#[test]
fn partial_reader_sevenz_first_image() {
    use sc_archive::read_first_image;
    let (_stage, tmp_7z, _names) = common::make_sevenz(3);
    let bytes = read_first_image(tmp_7z.path()).expect("read_first_image for 7z failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

// ---------------------------------------------------------------------------
// TAR.BZ2 tests
// ---------------------------------------------------------------------------

#[test]
fn tar_bz2_open_and_list() {
    let (tmp, _names) = common::make_tar_bz2(3);
    let reader = open_archive(tmp.path()).expect("open_archive failed for tar.bz2");
    assert_eq!(reader.entries().len(), 3);
    assert_eq!(reader.entries()[0].filename, "page001.png");
    assert_eq!(reader.entries()[2].filename, "page003.png");
}

#[test]
fn tar_bz2_read_entry_returns_png() {
    let (tmp, _names) = common::make_tar_bz2(2);
    let mut reader = open_archive(tmp.path()).expect("open_archive failed for tar.bz2");
    let bytes = reader.read_entry(0).expect("read_entry(0) failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

// ---------------------------------------------------------------------------
// TAR.XZ tests
// ---------------------------------------------------------------------------

#[test]
fn tar_xz_open_and_list() {
    let (tmp, _names) = common::make_tar_xz(3);
    let reader = open_archive(tmp.path()).expect("open_archive failed for tar.xz");
    assert_eq!(reader.entries().len(), 3);
    assert_eq!(reader.entries()[0].filename, "page001.png");
    assert_eq!(reader.entries()[2].filename, "page003.png");
}

#[test]
fn tar_xz_read_entry_returns_png() {
    let (tmp, _names) = common::make_tar_xz(2);
    let mut reader = open_archive(tmp.path()).expect("open_archive failed for tar.xz");
    let bytes = reader.read_entry(0).expect("read_entry(0) failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

// ---------------------------------------------------------------------------
// Edge case tests
// ---------------------------------------------------------------------------

#[test]
fn cbz_empty_archive_has_no_entries() {
    let (tmp, _names) = common::make_cbz(0);
    let reader = open_archive(tmp.path()).expect("open_archive failed");
    assert_eq!(reader.entries().len(), 0);
}

#[test]
fn cbz_non_image_files_filtered_out() {
    let tmp = common::make_cbz_with_names(&["readme.txt", "metadata.xml", "thumbs.db"]);
    let reader = open_archive(tmp.path()).expect("open_archive failed");
    assert_eq!(
        reader.entries().len(),
        0,
        "non-image files should be filtered out"
    );
}

#[test]
fn cbz_mixed_filters_only_images() {
    let tmp = common::make_cbz_with_names(&["readme.txt", "cover.jpg", "page01.png", "thumbs.db"]);
    let reader = open_archive(tmp.path()).expect("open_archive failed");
    let entries = reader.entries();
    assert_eq!(entries.len(), 2, "should have exactly 2 image entries");
    // Natural sort order: jpg then png
    assert_eq!(entries[0].filename, "cover.jpg");
    assert_eq!(entries[1].filename, "page01.png");
}

#[test]
fn open_archive_nonexistent_path_returns_err() {
    use std::path::Path;
    let result = open_archive(Path::new("/nonexistent/path/file.cbz"));
    assert!(result.is_err(), "should return error for nonexistent file");
}

#[test]
fn folder_empty_directory_has_no_entries() {
    let (dir, _names) = common::make_folder(0);
    let reader = open_archive(dir.path()).expect("open_archive failed for empty folder");
    assert_eq!(reader.entries().len(), 0);
}

// ---------------------------------------------------------------------------
// PartialReader TAR.BZ2 and TAR.XZ tests
// ---------------------------------------------------------------------------

#[test]
fn partial_reader_tar_bz2_first_image() {
    use sc_archive::read_first_image;
    let (tmp, _names) = common::make_tar_bz2(3);
    let bytes = read_first_image(tmp.path()).expect("read_first_image failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}

#[test]
fn partial_reader_tar_xz_first_image() {
    use sc_archive::read_first_image;
    let (tmp, _names) = common::make_tar_xz(3);
    let bytes = read_first_image(tmp.path()).expect("read_first_image failed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47], "not PNG magic");
}
