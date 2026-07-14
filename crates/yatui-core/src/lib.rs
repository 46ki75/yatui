//! Foundational value types shared by the `yatui` workspace.
//!
//! This crate deliberately contains no terminal I/O, rendering buffers,
//! layout engine, widgets, or application runtime.

mod color;
mod cursor;
mod geometry;
mod style;

pub use color::Color;
pub use cursor::{CursorShape, CursorState, CursorVisibility};
pub use geometry::{Insets, Point, Rect, Size};
pub use style::{Modifier, Style};
