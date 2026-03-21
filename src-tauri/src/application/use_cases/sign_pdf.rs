use std::path::Path;
use std::sync::Arc;

use crate::domain::entities::OverlayElement;
use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: apply visual signature elements (text, images) to a PDF page.
pub struct SignPdfUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl SignPdfUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    pub fn execute(
        &self,
        input_path: &Path,
        output_path: &Path,
        page_number: u32,
        elements: &[OverlayElement],
    ) -> Result<(), DomainError> {
        let pdf_data = self.file_system.read_file(input_path)?;
        let result = self
            .pdf_processor
            .overlay_elements(&pdf_data, page_number, elements)?;
        self.file_system.write_file(output_path, &result)?;
        Ok(())
    }
}
