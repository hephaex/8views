use anyhow::Result;
use std::path::Path;

const XATTR_KEY: &str = "io.github.hephaex.8views.metadata";

/// Store and retrieve JSON metadata as extended file attributes.
/// Mirrors UKXattrMetadataStore behavior.
pub fn write_metadata(path: &Path, value: &str) -> Result<()> {
    xattr::set(path, XATTR_KEY, value.as_bytes())?;
    Ok(())
}

pub fn read_metadata(path: &Path) -> Result<Option<String>> {
    match xattr::get(path, XATTR_KEY)? {
        Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
        None => Ok(None),
    }
}

pub fn remove_metadata(path: &Path) -> Result<()> {
    xattr::remove(path, XATTR_KEY)?;
    Ok(())
}
