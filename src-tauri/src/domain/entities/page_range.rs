use crate::domain::errors::DomainError;

/// Represents a range of pages within a PDF document.
/// Pages are 1-indexed to match user expectations.
#[derive(Debug, Clone)]
pub struct PageRange {
    pub start: u32,
    pub end: u32,
}

impl PageRange {
    pub fn new(start: u32, end: u32, total_pages: u32) -> Result<Self, DomainError> {
        if start == 0 || end == 0 {
            return Err(DomainError::PageOutOfRange {
                page: 0,
                total: total_pages,
            });
        }

        if start > end {
            return Err(DomainError::InvalidPageRange { start, end });
        }

        if end > total_pages {
            return Err(DomainError::PageOutOfRange {
                page: end,
                total: total_pages,
            });
        }

        Ok(Self { start, end })
    }

}
