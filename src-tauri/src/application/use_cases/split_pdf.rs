use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::entities::PageRange;
use crate::domain::errors::DomainError;
use crate::domain::ports::{FileSystemPort, PdfProcessor};

/// Use case: split a PDF file into one or more output files based on page ranges.
pub struct SplitPdfUseCase {
    pdf_processor: Arc<dyn PdfProcessor>,
    file_system: Arc<dyn FileSystemPort>,
}

impl SplitPdfUseCase {
    pub fn new(
        pdf_processor: Arc<dyn PdfProcessor>,
        file_system: Arc<dyn FileSystemPort>,
    ) -> Self {
        Self {
            pdf_processor,
            file_system,
        }
    }

    /// Splits the PDF at `input_path` into multiple files based on the provided page ranges.
    /// Each range produces one output file in `output_dir`, named `{original_name}_part{n}.pdf`.
    pub fn execute(
        &self,
        input_path: &Path,
        ranges: &[(u32, u32)],
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, DomainError> {
        let pdf_data = self.file_system.read_file(input_path)?;
        let total_pages = self.pdf_processor.page_count(&pdf_data)?;

        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let validated_ranges: Vec<PageRange> = ranges
            .iter()
            .map(|(start, end)| PageRange::new(*start, *end, total_pages))
            .collect::<Result<Vec<_>, _>>()?;

        let mut output_paths = Vec::with_capacity(validated_ranges.len());

        for (i, range) in validated_ranges.iter().enumerate() {
            let split_data = self.pdf_processor.split(&pdf_data, range)?;
            let output_path = output_dir.join(format!("{}_part{}.pdf", file_stem, i + 1));
            self.file_system.write_file(&output_path, &split_data)?;
            output_paths.push(output_path);
        }

        Ok(output_paths)
    }
}
