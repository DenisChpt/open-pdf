use std::path::Path;
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::ports::FileSystemPort;

/// Use case: write raw bytes to a file path.
/// Used by the convert feature to save rendered images.
pub struct SaveFileUseCase {
    file_system: Arc<dyn FileSystemPort>,
}

impl SaveFileUseCase {
    pub fn new(file_system: Arc<dyn FileSystemPort>) -> Self {
        Self { file_system }
    }

    pub fn execute(&self, path: &Path, data: &[u8]) -> Result<(), DomainError> {
        self.file_system.write_file(path, data)
    }
}
