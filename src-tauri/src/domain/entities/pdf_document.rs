use serde::Serialize;
use std::path::PathBuf;

/// Represents a PDF document with its metadata.
/// This is a domain entity — it carries no knowledge of how the PDF is stored or processed.
#[derive(Debug, Clone, Serialize)]
pub struct PdfDocument {
    pub path: PathBuf,
    pub page_count: u32,
    pub file_size_bytes: u64,
}

impl PdfDocument {
    pub fn new(path: PathBuf, page_count: u32, file_size_bytes: u64) -> Self {
        Self {
            path,
            page_count,
            file_size_bytes,
        }
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}
