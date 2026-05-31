pub mod detect;
pub mod encoding;
pub mod folder_reader;
pub mod partial_reader;
pub mod rar_archive;
pub mod reader;
pub mod sevenz_archive;
pub mod sort;
pub mod tar_archive;
pub mod zip_archive;

pub use partial_reader::read_first_image;
pub use reader::{ArchiveEntry, ArchiveReader};

use anyhow::Result;
use std::path::Path;

/// Open an archive at the given path, dispatching to the appropriate reader
/// based on file extension.
pub fn open_archive(path: &Path) -> Result<Box<dyn ArchiveReader>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("zip") | Some("cbz") => Ok(Box::new(zip_archive::ZipArchive::open(path)?)),
        Some("tar") => Ok(Box::new(tar_archive::TarArchive::open(
            path,
            tar_archive::Compression::None,
        )?)),
        Some("gz") | Some("tgz") => Ok(Box::new(tar_archive::TarArchive::open(
            path,
            tar_archive::Compression::Gzip,
        )?)),
        Some("bz2") | Some("tbz2") => Ok(Box::new(tar_archive::TarArchive::open(
            path,
            tar_archive::Compression::Bzip2,
        )?)),
        Some("xz") | Some("txz") => Ok(Box::new(tar_archive::TarArchive::open(
            path,
            tar_archive::Compression::Xz,
        )?)),
        Some("7z") => Ok(Box::new(sevenz_archive::SevenZArchive::open(path)?)),
        Some("rar") | Some("cbr") => Ok(Box::new(rar_archive::RarArchive::open(path)?)),
        None => {
            if path.is_dir() {
                Ok(Box::new(folder_reader::FolderReader::open(path)?))
            } else {
                anyhow::bail!("unsupported format: {:?}", path)
            }
        }
        _ => {
            if path.is_dir() {
                Ok(Box::new(folder_reader::FolderReader::open(path)?))
            } else {
                // Fallback: identify format by magic bytes when the extension is
                // absent or unrecognised.
                match detect::detect_format(path) {
                    detect::ArchiveFormat::Zip => {
                        Ok(Box::new(zip_archive::ZipArchive::open(path)?))
                    }
                    detect::ArchiveFormat::SevenZ => {
                        Ok(Box::new(sevenz_archive::SevenZArchive::open(path)?))
                    }
                    detect::ArchiveFormat::Rar => {
                        Ok(Box::new(rar_archive::RarArchive::open(path)?))
                    }
                    detect::ArchiveFormat::TarGz => Ok(Box::new(tar_archive::TarArchive::open(
                        path,
                        tar_archive::Compression::Gzip,
                    )?)),
                    detect::ArchiveFormat::TarBz2 => Ok(Box::new(tar_archive::TarArchive::open(
                        path,
                        tar_archive::Compression::Bzip2,
                    )?)),
                    detect::ArchiveFormat::TarXz => Ok(Box::new(tar_archive::TarArchive::open(
                        path,
                        tar_archive::Compression::Xz,
                    )?)),
                    _ => anyhow::bail!("unsupported format (unknown magic): {:?}", path),
                }
            }
        }
    }
}
