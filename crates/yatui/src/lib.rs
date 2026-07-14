//! Rust-native terminal user interface primitives.
//!
//! The facade currently exposes foundational [`core`], Unicode [`text`], and
//! cell-based [`render`] APIs. More subsystems will be added as their
//! individual crates are implemented.

/// Foundational geometry, color, style, and cursor types.
pub use yatui_core as core;
/// Grapheme-aware cell buffers, composition, and transactional frame diffing.
pub use yatui_render as render;
/// Unicode grapheme segmentation and terminal width measurement.
pub use yatui_text as text;

pub use yatui_core::{
    Color, CursorShape, CursorState, CursorVisibility, Insets, Modifier, Point, Rect, Size, Style,
};
pub use yatui_render::{Buffer, Canvas, FramePatch, PreparedFrame, Renderer};
pub use yatui_text::{TextMetrics, WidthPolicy, grapheme_width, graphemes, measure};
