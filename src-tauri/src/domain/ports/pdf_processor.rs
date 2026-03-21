use crate::domain::entities::{OverlayElement, PageRange};
use crate::domain::errors::DomainError;

/// Port for PDF manipulation operations.
/// This trait abstracts away the concrete PDF library, allowing it to be swapped
/// without affecting the domain or application layers.
pub trait PdfProcessor: Send + Sync {
    /// Returns the number of pages in the PDF at the given path.
    fn page_count(&self, pdf_data: &[u8]) -> Result<u32, DomainError>;

    /// Merges multiple PDF byte buffers into a single PDF.
    fn merge(&self, pdfs: Vec<Vec<u8>>) -> Result<Vec<u8>, DomainError>;

    /// Extracts pages from a PDF according to the given range.
    fn split(&self, pdf_data: &[u8], range: &PageRange) -> Result<Vec<u8>, DomainError>;

    /// Compresses a PDF by pruning unused objects and compressing streams.
    fn compress(&self, pdf_data: &[u8]) -> Result<Vec<u8>, DomainError>;

    /// Reorders (and optionally removes) pages in a PDF.
    /// `new_order` contains 1-indexed page numbers in the desired output order.
    /// `rotations` maps 1-indexed page numbers to rotation angles (0, 90, 180, 270).
    fn reorder_pages(
        &self,
        pdf_data: &[u8],
        new_order: &[u32],
        rotations: &std::collections::HashMap<u32, u32>,
    ) -> Result<Vec<u8>, DomainError>;

    /// Splits a PDF into chunks where each chunk is at most `max_bytes` in size.
    /// If `compress` is true, each chunk is compressed before size checking.
    fn split_by_size(
        &self,
        pdf_data: &[u8],
        max_bytes: u64,
        apply_compression: bool,
    ) -> Result<Vec<Vec<u8>>, DomainError>;

    /// Overlays text and image elements on a specific page of the PDF.
    fn overlay_elements(
        &self,
        pdf_data: &[u8],
        page_number: u32,
        elements: &[OverlayElement],
    ) -> Result<Vec<u8>, DomainError>;
}
