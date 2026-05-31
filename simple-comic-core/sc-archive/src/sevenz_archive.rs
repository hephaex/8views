use anyhow::Result;
use std::path::Path;

use crate::{
    encoding::is_image_entry,
    reader::{ArchiveEntry, ArchiveReader},
    sort::natural_cmp,
};

pub struct SevenZArchive {
    path: std::path::PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl SevenZArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let archive = sevenz_rust::Archive::open(path)
            .map_err(|e| anyhow::anyhow!("7z open error: {}", e))?;

        let mut raw: Vec<(String, u64)> = archive
            .files
            .iter()
            .filter(|e| !e.is_directory && is_image_entry(&e.name))
            .map(|e| (e.name.clone(), e.size))
            .collect();

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

impl ArchiveReader for SevenZArchive {
    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, index: usize) -> Result<Vec<u8>> {
        let filename = self
            .entries
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("entry index out of range: {}", index))?
            .filename
            .clone();

        let mut result: Vec<u8> = Vec::new();
        let found = std::cell::Cell::new(false);

        sevenz_rust::decompress_file_with_extract_fn(
            &self.path,
            std::path::Path::new(""),
            |entry, reader, _output_path| {
                if entry.name() == filename {
                    std::io::Read::read_to_end(reader, &mut result)?;
                    found.set(true);
                }
                Ok(true)
            },
        )
        .map_err(|e| anyhow::anyhow!("7z extraction error: {}", e))?;

        if !found.get() {
            anyhow::bail!("entry not found in 7z: {}", filename);
        }
        Ok(result)
    }
}
