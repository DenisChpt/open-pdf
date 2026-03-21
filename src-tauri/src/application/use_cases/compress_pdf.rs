use std::path::Path;
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: compress a PDF file to reduce its size.
pub struct CompressPdfUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl CompressPdfUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    /// Compresses the PDF at `input_path` and writes the result to `output_path`.
    /// Returns a tuple of (original_size, compressed_size) in bytes.
    pub fn execute(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(u64, u64), DomainError> {
        let original_size = self.file_system.file_size(input_path)?;
        let pdf_data = self.file_system.read_file(input_path)?;
        let compressed = self.pdf_processor.compress(&pdf_data)?;
        self.file_system.write_file(output_path, &compressed)?;
        let compressed_size = self.file_system.file_size(output_path)?;

        Ok((original_size, compressed_size))
    }
}
