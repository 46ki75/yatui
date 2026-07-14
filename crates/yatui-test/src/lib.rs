//! Deterministic headless application testing for `yatui`.
//!
//! [`TestApp`] drives the real runtime, retained UI tree, renderer, and
//! terminal transaction contract without selecting a concrete terminal backend.

mod app;
mod backend;
mod clock;
mod frame;

pub use app::{SettleOutcome, SettleReport, TestApp, TestError};
pub use backend::TestBackendError;
pub use frame::{TestCell, TestCellContent, TestFrame};

pub use yatui_core::{Point, Size};
pub use yatui_terminal::{
    KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, TerminalEvent,
};
pub use yatui_ui::{Key, NodeId, UiEvent};

#[cfg(test)]
mod tests;
