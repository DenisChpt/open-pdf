use std::fs;
use std::path::Path;

use crate::domain::errors::DomainError;
use crate::domain::ports::FileSystemPort;

/// Concrete adapter for local file system operations.
pub struct LocalFileSystem;

impl LocalFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystemPort for LocalFileSystem {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, DomainError> {
        if !path.exists() {
            return Err(DomainError::FileNotFound {
                path: path.display().to_string(),
            });
        }

        fs::read(path).map_err(|e| DomainError::FileReadError {
            reason: format!("{}: {}", path.display(), e),
        })
    }

    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), DomainError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| DomainError::FileWriteError {
                reason: format!("Failed to create directory '{}': {}", parent.display(), e),
            })?;
        }

        fs::write(path, data).map_err(|e| DomainError::FileWriteError {
            reason: format!("{}: {}", path.display(), e),
        })
    }

    fn file_size(&self, path: &Path) -> Result<u64, DomainError> {
        fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| DomainError::FileReadError {
                reason: format!("{}: {}", path.display(), e),
            })
    }
}
