use anyhow::Result;
use std::path::{Path, PathBuf};
use unrar::Archive as UnrarArchive;

use crate::{
    encoding::is_image_entry,
    reader::{ArchiveEntry, ArchiveReader},
    sort::natural_cmp,
};

pub struct RarArchive {
    path: PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl RarArchive {
    /// Open a RAR/CBR archive and enumerate its image entries.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or the RAR header is corrupt.
    pub fn open(path: &Path) -> Result<Self> {
        let open_archive = UnrarArchive::new(path)
            .open_for_listing()
            .map_err(|e| anyhow::anyhow!("RAR open error: {}", e))?;

        let mut raw: Vec<(String, u64)> = Vec::new();
        for entry in open_archive {
            let header = entry.map_err(|e| anyhow::anyhow!("RAR read header error: {}", e))?;
            if header.is_directory() {
                continue;
            }
            // `FileHeader::filename` is a PathBuf; convert to a UTF-8 string
            // using the platform-native representation (macOS: always UTF-8).
            let filename = header.filename.to_string_lossy().into_owned();
            if !is_image_entry(&filename) {
                continue;
            }
            raw.push((filename, header.unpacked_size));
        }

        raw.sort_by(|(a, _), (b, _)| natural_cmp(a, b));

        let entries = raw
            .into_iter()
            .enumerate()
            .map(|(index, (filename, size))| ArchiveEntry {
                index,
                filename,
                size,
            })
            .collect();

        Ok(Self {
            path: path.to_owned(),
            entries,
        })
    }
}

impl ArchiveReader for RarArchive {
    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    /// Read the raw bytes of the entry at `index`.
    ///
    /// The RAR format must be iterated sequentially from the beginning each
    /// time; we walk through entries until we match the stored filename, then
    /// call `.read()` to decompress into memory.
    ///
    /// # Errors
    ///
    /// Returns an error when `index` is out of range, the archive cannot be
    /// opened, or decompression fails.
    fn read_entry(&mut self, index: usize) -> Result<Vec<u8>> {
        let target = self
            .entries
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("entry index out of range: {}", index))?
            .filename
            .clone();

        // Open in Process mode so we can call `.read()` on matching entries.
        let mut cursor = UnrarArchive::new(&self.path)
            .open_for_processing()
            .map_err(|e| anyhow::anyhow!("RAR open error: {}", e))?;

        loop {
            // Advance to the next header; `None` means end-of-archive.
            let before_file = cursor
                .read_header()
                .map_err(|e| anyhow::anyhow!("RAR read header error: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("entry not found in RAR: {}", target))?;

            let filename = before_file.entry().filename.to_string_lossy().into_owned();

            if filename == target {
                let (data, _next) = before_file
                    .read()
                    .map_err(|e| anyhow::anyhow!("RAR read error for '{}': {}", target, e))?;
                return Ok(data);
            }

            // Not the entry we want — skip the payload and continue.
            cursor = before_file
                .skip()
                .map_err(|e| anyhow::anyhow!("RAR skip error: {}", e))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_nonexistent_returns_err() {
        let result = RarArchive::open(Path::new("/nonexistent/path/to/archive.rar"));
        assert!(
            result.is_err(),
            "opening a non-existent RAR file must return Err, not panic"
        );
    }
}
