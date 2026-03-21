use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: split a PDF into multiple files based on a maximum file size.
pub struct SplitBySizeUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl SplitBySizeUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    /// Splits the PDF at `input_path` into chunks of at most `max_bytes` each.
    /// Writes each chunk to `output_dir` and returns the output paths.
    pub fn execute(
        &self,
        input_path: &Path,
        max_bytes: u64,
        compress: bool,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, DomainError> {
        let pdf_data = self.file_system.read_file(input_path)?;
        let chunks = self
            .pdf_processor
            .split_by_size(&pdf_data, max_bytes, compress)?;

        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let mut output_paths = Vec::with_capacity(chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            let output_path = output_dir.join(format!("{}_part{}.pdf", file_stem, i + 1));
            self.file_system.write_file(&output_path, chunk)?;
            output_paths.push(output_path);
        }

        Ok(output_paths)
    }
}
