//! Grapheme-aware cell buffers, composition, and transactional frame diffing.

mod buffer;
mod canvas;
mod cell;
mod frame;
mod grapheme_store;
mod surface;

pub use buffer::{Buffer, BufferError};
pub use canvas::{Canvas, DrawError, TextDraw};
pub use cell::{Cell, CellContent, HyperlinkId};
pub use frame::{
    CellRun, FramePatch, PatchCell, PatchCellContent, PreparedFrame, RenderError, Renderer,
};
pub use grapheme_store::{GraphemeId, GraphemeStore, GraphemeStoreError};
pub use surface::{Compositor, Opacity, Surface};
