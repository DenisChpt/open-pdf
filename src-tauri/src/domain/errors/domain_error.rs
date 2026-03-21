use serde::Serialize;
use thiserror::Error;

/// Domain-level errors that represent business rule violations or expected failure cases.
/// These errors are independent of any infrastructure concern.
#[derive(Debug, Error, Serialize)]
pub enum DomainError {
    #[error("Invalid PDF file: {reason}")]
    InvalidPdf { reason: String },

    #[error("Page {page} is out of range (document has {total} pages)")]
    PageOutOfRange { page: u32, total: u32 },

    #[error("Invalid page range: {start} to {end}")]
    InvalidPageRange { start: u32, end: u32 },

    #[error("Merge requires at least 2 documents")]
    InsufficientDocumentsForMerge,

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File read error: {reason}")]
    FileReadError { reason: String },

    #[error("File write error: {reason}")]
    FileWriteError { reason: String },

    #[error("PDF processing error: {reason}")]
    ProcessingError { reason: String },
}
