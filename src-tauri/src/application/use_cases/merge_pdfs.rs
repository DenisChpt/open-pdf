use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: merge multiple PDF files into a single output file.
pub struct MergePdfsUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl MergePdfsUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    /// Merges the PDFs at the given input paths and writes the result to `output_path`.
    pub fn execute(
        &self,
        input_paths: &[PathBuf],
        output_path: &Path,
    ) -> Result<u64, DomainError> {
        if input_paths.len() < 2 {
            return Err(DomainError::InsufficientDocumentsForMerge);
        }

        let pdf_buffers: Vec<Vec<u8>> = input_paths
            .iter()
            .map(|path| self.file_system.read_file(path))
            .collect::<Result<Vec<_>, _>>()?;

        // Validate that each buffer is a readable PDF
        for (i, buf) in pdf_buffers.iter().enumerate() {
            self.pdf_processor.page_count(buf).map_err(|_| {
                DomainError::InvalidPdf {
                    reason: format!(
                        "File '{}' is not a valid PDF",
                        input_paths[i].display()
                    ),
                }
            })?;
        }

        let merged = self.pdf_processor.merge(pdf_buffers)?;
        self.file_system.write_file(output_path, &merged)?;

        let size = self.file_system.file_size(output_path)?;
        Ok(size)
    }
}
