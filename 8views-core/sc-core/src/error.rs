use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScError {
    #[error("archive error: {0}")]
    Archive(String),

    #[error("image error: {0}")]
    Image(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("entry not found: index {0}")]
    EntryNotFound(usize),
}
