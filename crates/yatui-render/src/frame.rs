use std::{fmt, sync::Arc};

use yatui_core::{CursorState, Point, Size, Style};
use yatui_text::WidthPolicy;

use crate::{
    Buffer, BufferError, Canvas, Cell, CellContent, DrawError, GraphemeId, GraphemeStore,
    GraphemeStoreError, HyperlinkId,
};

/// Resolved content in a terminal-independent frame patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatchCellContent {
    /// A visually empty cell.
    Empty,
    /// A complete grapheme at its leading cell.
    Grapheme {
        /// Stable identity used for diagnostics and caching.
        id: GraphemeId,
        /// Shared UTF-8 grapheme text.
        text: Arc<str>,
        /// Number of occupied terminal cells.
        width: u16,
    },
    /// A continuation cell for a wide grapheme.
    Continuation {
        /// Stable grapheme identity.
        id: GraphemeId,
        /// Cell offset from the leading cell.
        offset: u16,
    },
}

/// One resolved cell in a frame patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchCell {
    /// Resolved cell content.
    pub content: PatchCellContent,
    /// Cell style.
    pub style: Style,
    /// Optional hyperlink identity.
    pub hyperlink: Option<HyperlinkId>,
}

/// A contiguous run of changed cells on one row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellRun {
    /// Position of the first cell.
    pub position: Point,
    /// Changed cells in left-to-right order.
    pub cells: Vec<PatchCell>,
}

/// Terminal-independent changes between two logical frames.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FramePatch {
    /// Target frame size.
    pub size: Size,
    /// Contiguous changed cell runs.
    pub runs: Vec<CellRun>,
    /// Desired cursor state after applying the patch.
    pub cursor: CursorState,
    /// Whether the cursor differs from the committed state.
    pub cursor_changed: bool,
    /// Whether this patch describes a complete repaint.
    pub full_repaint: bool,
}

impl FramePatch {
    /// Returns whether applying the patch requires no terminal output.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty() && !self.cursor_changed
    }

    /// Applies this patch to a logical buffer.
    ///
    /// This operation is primarily useful for testing, recording, and remote
    /// transports. It does not apply cursor state.
    pub fn apply_to(&self, buffer: &mut Buffer) -> Result<(), BufferError> {
        if buffer.size() != self.size {
            *buffer = Buffer::new(self.size);
        } else if self.full_repaint {
            buffer.clear(Style::default());
        }

        for run in &self.runs {
            for (offset, cell) in run.cells.iter().enumerate() {
                let point = run.position.translated(offset as i32, 0);
                match cell.content {
                    PatchCellContent::Empty => buffer.set_empty(point, cell.style)?,
                    PatchCellContent::Grapheme { id, width, .. } => {
                        buffer.set_grapheme(point, id, width, cell.style, cell.hyperlink)?;
                    }
                    PatchCellContent::Continuation { .. } => {}
                }
            }
        }
        Ok(())
    }
}

/// A fully painted frame waiting for an output decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedFrame {
    next: Buffer,
    cursor: CursorState,
    patch: FramePatch,
}

impl PreparedFrame {
    /// Returns the terminal-independent patch for this frame.
    #[must_use]
    pub const fn patch(&self) -> &FramePatch {
        &self.patch
    }

    /// Returns the prepared logical buffer.
    #[must_use]
    pub const fn buffer(&self) -> &Buffer {
        &self.next
    }
}

/// Errors produced while preparing a frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderError {
    /// Painting failed.
    Draw(DrawError),
    /// A cell referred to an unknown grapheme.
    GraphemeStore(GraphemeStoreError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draw(error) => error.fmt(formatter),
            Self::GraphemeStore(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RenderError {}

impl From<DrawError> for RenderError {
    fn from(error: DrawError) -> Self {
        Self::Draw(error)
    }
}

impl From<GraphemeStoreError> for RenderError {
    fn from(error: GraphemeStoreError) -> Self {
        Self::GraphemeStore(error)
    }
}

/// Stateful renderer with transactional frame preparation and commit.
#[derive(Clone, Debug)]
pub struct Renderer {
    current: Buffer,
    cursor: CursorState,
    graphemes: GraphemeStore,
    width_policy: WidthPolicy,
    force_full_repaint: bool,
}

impl Renderer {
    /// Creates a renderer with an empty committed frame.
    #[must_use]
    pub fn new(size: Size, width_policy: WidthPolicy) -> Self {
        Self {
            current: Buffer::new(size),
            cursor: CursorState::default(),
            graphemes: GraphemeStore::new(),
            width_policy,
            force_full_repaint: true,
        }
    }

    /// Returns the committed frame.
    #[must_use]
    pub const fn current(&self) -> &Buffer {
        &self.current
    }

    /// Returns the active width policy.
    #[must_use]
    pub const fn width_policy(&self) -> WidthPolicy {
        self.width_policy
    }

    /// Changes the width policy and invalidates the committed physical state.
    pub fn set_width_policy(&mut self, width_policy: WidthPolicy) {
        if self.width_policy != width_policy {
            self.width_policy = width_policy;
            self.invalidate();
        }
    }

    /// Paints and diffs a complete logical frame without committing it.
    pub fn prepare<F>(
        &mut self,
        size: Size,
        cursor: CursorState,
        paint: F,
    ) -> Result<PreparedFrame, RenderError>
    where
        F: FnOnce(&mut Canvas<'_>) -> Result<(), DrawError>,
    {
        let mut next = Buffer::new(size);
        let mut canvas = Canvas::new(&mut next, &mut self.graphemes, self.width_policy);
        paint(&mut canvas)?;

        let full_repaint = self.force_full_repaint || self.current.size() != size;
        let patch = diff(
            &self.current,
            &next,
            self.cursor,
            cursor,
            full_repaint,
            &self.graphemes,
        )?;
        Ok(PreparedFrame {
            next,
            cursor,
            patch,
        })
    }

    /// Commits a prepared frame after its patch has been accepted for output.
    pub fn commit(&mut self, prepared: PreparedFrame) {
        self.current = prepared.next;
        self.cursor = prepared.cursor;
        self.force_full_repaint = false;
    }

    /// Discards a prepared frame without changing committed state.
    pub fn discard(&mut self, _prepared: PreparedFrame) {}

    /// Marks physical terminal state as unknown so the next patch repaints all cells.
    pub fn invalidate(&mut self) {
        self.force_full_repaint = true;
    }
}

fn diff(
    current: &Buffer,
    next: &Buffer,
    current_cursor: CursorState,
    next_cursor: CursorState,
    full_repaint: bool,
    store: &GraphemeStore,
) -> Result<FramePatch, GraphemeStoreError> {
    let size = next.size();
    let mut runs = Vec::new();

    for y in 0..size.height {
        let mut x = 0;
        while x < size.width {
            let point = Point::new(i32::from(x), i32::from(y));
            if !full_repaint && current.get(point) == next.get(point) {
                x += 1;
                continue;
            }

            let start = x;
            let mut cells = Vec::new();
            while x < size.width {
                let point = Point::new(i32::from(x), i32::from(y));
                if !full_repaint && current.get(point) == next.get(point) {
                    break;
                }
                if let Some(cell) = next.get(point) {
                    cells.push(resolve_cell(*cell, store)?);
                }
                x += 1;
            }
            runs.push(CellRun {
                position: Point::new(i32::from(start), i32::from(y)),
                cells,
            });
        }
    }

    Ok(FramePatch {
        size,
        runs,
        cursor: next_cursor,
        cursor_changed: current_cursor != next_cursor,
        full_repaint,
    })
}

fn resolve_cell(cell: Cell, store: &GraphemeStore) -> Result<PatchCell, GraphemeStoreError> {
    let content = match cell.content {
        CellContent::Empty => PatchCellContent::Empty,
        CellContent::Grapheme { id, width } => PatchCellContent::Grapheme {
            id,
            text: Arc::clone(store.get(id)?),
            width,
        },
        CellContent::Continuation { id, offset } => PatchCellContent::Continuation { id, offset },
    };
    Ok(PatchCell {
        content,
        style: cell.style,
        hyperlink: cell.hyperlink,
    })
}

#[cfg(test)]
mod tests {
    use yatui_core::{Color, Point, Style};

    use super::*;

    #[test]
    fn committed_identical_frame_produces_no_patch() -> Result<(), RenderError> {
        let mut renderer = Renderer::new(Size::new(3, 1), WidthPolicy::Unicode);
        let first = renderer.prepare(Size::new(3, 1), CursorState::default(), |canvas| {
            canvas.draw_text(Point::ORIGIN, "abc", Style::default(), None)?;
            Ok(())
        })?;
        assert!(first.patch().full_repaint);
        renderer.commit(first);

        let second = renderer.prepare(Size::new(3, 1), CursorState::default(), |canvas| {
            canvas.draw_text(Point::ORIGIN, "abc", Style::default(), None)?;
            Ok(())
        })?;

        assert!(second.patch().is_empty());
        assert!(!second.patch().full_repaint);
        Ok(())
    }

    #[test]
    fn changed_cells_are_grouped_into_runs() -> Result<(), RenderError> {
        let mut renderer = Renderer::new(Size::new(4, 1), WidthPolicy::Unicode);
        let initial = renderer.prepare(Size::new(4, 1), CursorState::default(), |_| Ok(()))?;
        renderer.commit(initial);

        let changed = renderer.prepare(Size::new(4, 1), CursorState::default(), |canvas| {
            canvas.draw_text(
                Point::new(1, 0),
                "ab",
                Style::new().foreground(Color::Green),
                None,
            )?;
            Ok(())
        })?;

        assert_eq!(changed.patch().runs.len(), 1);
        assert_eq!(changed.patch().runs[0].position, Point::new(1, 0));
        assert_eq!(changed.patch().runs[0].cells.len(), 2);
        let mut replay = renderer.current().clone();
        assert_eq!(changed.patch().apply_to(&mut replay), Ok(()));
        assert_eq!(&replay, changed.buffer());
        Ok(())
    }

    #[test]
    fn discarded_frame_does_not_advance_committed_state() -> Result<(), RenderError> {
        let mut renderer = Renderer::new(Size::new(1, 1), WidthPolicy::Unicode);
        let initial = renderer.prepare(Size::new(1, 1), CursorState::default(), |_| Ok(()))?;
        renderer.commit(initial);
        let changed = renderer.prepare(Size::new(1, 1), CursorState::default(), |canvas| {
            canvas.draw_text(Point::ORIGIN, "x", Style::default(), None)?;
            Ok(())
        })?;
        renderer.discard(changed);

        let retry = renderer.prepare(Size::new(1, 1), CursorState::default(), |canvas| {
            canvas.draw_text(Point::ORIGIN, "x", Style::default(), None)?;
            Ok(())
        })?;
        assert!(!retry.patch().is_empty());
        Ok(())
    }
}
