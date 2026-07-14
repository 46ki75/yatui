//! Launches the public counter example with the default Crossterm backend.

use std::{error::Error, io, time::Duration};

use yatui::{CrosstermBackend, TerminalState, run};
use yatui_example_counter::Counter;

fn main() -> Result<(), Box<dyn Error>> {
    let backend = CrosstermBackend::new(io::stdout())?;
    run(
        Counter::default(),
        backend,
        TerminalState::fullscreen(),
        Duration::from_millis(16),
    )?;
    Ok(())
}
