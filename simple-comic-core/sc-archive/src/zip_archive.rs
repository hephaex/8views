use anyhow::Result;
use std::path::Path;
use zip::ZipArchive as ZipFile;

use crate::{
    encoding::{decode_filename, is_image_entry},
    reader::{ArchiveEntry, ArchiveReader},
    sort::natural_cmp,
};

pub struct ZipArchive {
    path: std::path::PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl ZipArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut zip = ZipFile::new(file)?;

        let mut raw_entries: Vec<(String, u64)> = (0..zip.len())
            .filter_map(|i| {
                let entry = zip.by_index_raw(i).ok()?;
                let raw_name = entry.name_raw().to_vec();
                let filename = decode_filename(&raw_name);
                if !is_image_entry(&filename) || entry.is_dir() {
                    return None;
                }
                Some((filename, entry.size()))
            })
            .collect();

        raw_entries.sort_by(|(a, _), (b, _)| natural_cmp(a, b));

        let entries = raw_entries
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

impl ArchiveReader for ZipArchive {
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

        let file = std::fs::File::open(&self.path)?;
        let mut zip = ZipFile::new(file)?;

        // Find by filename (order within zip may differ from our sorted order)
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let raw_name = entry.name_raw().to_vec();
            let name = super::encoding::decode_filename(&raw_name);
            if name == filename {
                let mut buf = Vec::with_capacity(entry.size() as usize);
                std::io::Read::read_to_end(&mut entry, &mut buf)?;
                return Ok(buf);
            }
        }

        anyhow::bail!("entry not found in zip: {}", filename)
    }
}
