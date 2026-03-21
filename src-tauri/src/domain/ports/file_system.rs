use crate::domain::errors::DomainError;
use std::path::Path;

/// Port for file system operations.
/// Abstracts reading and writing files so the domain remains decoupled from I/O.
pub trait FileSystemPort: Send + Sync {
    /// Reads the entire contents of a file into a byte buffer.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, DomainError>;

    /// Writes a byte buffer to the specified path, creating parent directories if needed.
    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), DomainError>;

    /// Returns the file size in bytes.
    fn file_size(&self, path: &Path) -> Result<u64, DomainError>;
}
