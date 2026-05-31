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
