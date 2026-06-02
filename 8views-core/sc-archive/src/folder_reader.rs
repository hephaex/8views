use anyhow::Result;
use std::path::Path;

use crate::{
    encoding::is_image_entry,
    reader::{ArchiveEntry, ArchiveReader},
    sort::natural_cmp,
};

pub struct FolderReader {
    base: std::path::PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl FolderReader {
    pub fn open(path: &Path) -> Result<Self> {
        let mut files: Vec<(String, u64, std::path::PathBuf)> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if !is_image_entry(&name) {
                    return None;
                }
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                Some((name, size, e.path()))
            })
            .collect();

        files.sort_by(|(a, _, _), (b, _, _)| natural_cmp(a, b));

        let entries = files
            .into_iter()
            .enumerate()
            .map(|(index, (filename, size, _))| ArchiveEntry {
                index,
                filename,
                size,
            })
            .collect();

        Ok(Self {
            base: path.to_owned(),
            entries,
        })
    }
}

impl ArchiveReader for FolderReader {
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

        let full_path = self.base.join(&filename);
        Ok(std::fs::read(full_path)?)
    }
}
