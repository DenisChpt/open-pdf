use std::path::Path;
use std::sync::Arc;

use crate::domain::entities::PdfDocument;
use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: retrieve metadata about a PDF file.
pub struct GetPdfInfoUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl GetPdfInfoUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    pub fn execute(&self, path: &Path) -> Result<PdfDocument, DomainError> {
        let data = self.file_system.read_file(path)?;
        let page_count = self.pdf_processor.page_count(&data)?;
        let file_size_bytes = self.file_system.file_size(path)?;

        Ok(PdfDocument::new(path.to_path_buf(), page_count, file_size_bytes))
    }
}
