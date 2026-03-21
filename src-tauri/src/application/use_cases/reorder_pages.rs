use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: reorder (and optionally remove/rotate) pages within a PDF.
pub struct ReorderPagesUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl ReorderPagesUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    /// Reorders pages of the PDF at `input_path` according to `new_order`,
    /// applies `rotations` (page_number → angle delta), and writes to `output_path`.
    pub fn execute(
        &self,
        input_path: &Path,
        new_order: &[u32],
        rotations: &HashMap<u32, u32>,
        output_path: &Path,
    ) -> Result<(), DomainError> {
        if new_order.is_empty() {
            return Err(DomainError::ProcessingError {
                reason: "New page order cannot be empty".to_string(),
            });
        }

        let pdf_data = self.file_system.read_file(input_path)?;
        let result = self
            .pdf_processor
            .reorder_pages(&pdf_data, new_order, rotations)?;
        self.file_system.write_file(output_path, &result)?;

        Ok(())
    }
}
