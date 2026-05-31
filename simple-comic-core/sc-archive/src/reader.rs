use anyhow::Result;
use sc_core::types::PageIndex;

/// Metadata for a single entry (page/image) inside an archive.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub index: PageIndex,
    pub filename: String,
    pub size: u64,
}

/// Core trait for all archive backends.
pub trait ArchiveReader: Send + Sync {
    /// Sorted list of image entries in this archive.
    fn entries(&self) -> &[ArchiveEntry];

    /// Read raw bytes for an entry by index.
    fn read_entry(&mut self, index: PageIndex) -> Result<Vec<u8>>;

    /// Read only the first image entry (for QuickLook thumbnails).
    fn read_first_entry(&mut self) -> Result<Vec<u8>> {
        self.read_entry(0)
    }

    fn len(&self) -> usize {
        self.entries().len()
    }

    fn is_empty(&self) -> bool {
        self.entries().is_empty()
    }
}
