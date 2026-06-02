use anyhow::Result;
use std::path::Path;

use crate::{
    encoding::{decode_filename, is_image_entry},
    reader::{ArchiveEntry, ArchiveReader},
    sort::natural_cmp,
};

pub enum Compression {
    None,
    Gzip,
    Bzip2,
    Xz,
}

pub struct TarArchive {
    path: std::path::PathBuf,
    compression: Compression,
    entries: Vec<ArchiveEntry>,
}

impl TarArchive {
    pub fn open(path: &Path, compression: Compression) -> Result<Self> {
        let mut entries = Self::scan(path, &compression)?;
        entries.sort_by(|(a, _), (b, _)| natural_cmp(a, b));
        let entries = entries
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
            compression,
            entries,
        })
    }

    fn scan(path: &Path, compression: &Compression) -> Result<Vec<(String, u64)>> {
        let file = std::fs::File::open(path)?;
        let reader: Box<dyn std::io::Read> = match compression {
            Compression::None => Box::new(file),
            Compression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
            Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
            Compression::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        };
        let mut archive = tar::Archive::new(reader);
        let mut result = Vec::new();
        for entry in archive.entries()? {
            let entry = entry?;
            let raw = entry.path_bytes();
            let filename = decode_filename(&raw);
            if is_image_entry(&filename) {
                let size = entry.header().size()?;
                result.push((filename, size));
            }
        }
        Ok(result)
    }
}

impl ArchiveReader for TarArchive {
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
        let reader: Box<dyn std::io::Read> = match &self.compression {
            Compression::None => Box::new(file),
            Compression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
            Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
            Compression::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        };
        let mut archive = tar::Archive::new(reader);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let raw = entry.path_bytes().into_owned();
            let name = decode_filename(&raw);
            if name == filename {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut buf)?;
                return Ok(buf);
            }
        }
        anyhow::bail!("entry not found in tar: {}", filename)
    }
}
