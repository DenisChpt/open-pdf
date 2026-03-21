use std::path::Path;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::domain::errors::DomainError;
use crate::domain::ports::FileSystemPort;

/// Use case: read a PDF file and return its contents as a base64 string.
/// Used by the frontend to feed PDF data to pdf.js for thumbnail rendering.
pub struct GetPdfDataUseCase {
    file_system: Arc<dyn FileSystemPort>,
}

impl GetPdfDataUseCase {
    pub fn new(file_system: Arc<dyn FileSystemPort>) -> Self {
        Self { file_system }
    }

    pub fn execute(&self, path: &Path) -> Result<String, DomainError> {
        let data = self.file_system.read_file(path)?;
        Ok(STANDARD.encode(&data))
    }
}
