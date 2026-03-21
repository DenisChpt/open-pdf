/// Content to overlay on a PDF page.
pub enum OverlayContent {
    /// Rendered as text using a standard PDF font.
    Text { text: String, font_size: f32 },
    /// Rendered as an image (PNG or JPEG bytes).
    Image { data: Vec<u8> },
}

/// A single element to overlay on a PDF page.
/// Positions and sizes are expressed as ratios (0.0–1.0) of the page dimensions.
pub struct OverlayElement {
    /// Horizontal position ratio from the left edge.
    pub x_ratio: f32,
    /// Vertical position ratio from the top edge.
    pub y_ratio: f32,
    /// Width ratio relative to page width.
    pub width_ratio: f32,
    /// Height ratio relative to page height.
    pub height_ratio: f32,
    /// The content to render.
    pub content: OverlayContent,
}
