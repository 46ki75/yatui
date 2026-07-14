//! Launches the focus queue pilot with the default Crossterm backend.

use std::{error::Error, io, time::Duration};

use arborui::{CrosstermBackend, TerminalState, run, terminal::MouseMode};
use arborui_example_focus_queue::FocusQueue;

fn main() -> Result<(), Box<dyn Error>> {
    let backend = CrosstermBackend::new(io::stdout())?;
    let mut terminal_state = TerminalState::fullscreen();
    terminal_state.mouse = MouseMode::Capture;
    terminal_state.title = Some("ArborUI Focus Queue".to_owned());

    run(
        FocusQueue::default(),
        backend,
        terminal_state,
        Duration::from_millis(16),
    )?;
    Ok(())
}
