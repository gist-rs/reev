use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// A wrapper around the ratatui Terminal that handles setup and restoration.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Creates a new `Tui` instance, setting up the terminal.
    pub fn new() -> Result<Self> {
        let terminal = setup_terminal()?;
        Ok(Self { terminal })
    }

    /// Provides mutable access to the underlying `Terminal`.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

/// The `Drop` implementation ensures that the terminal state is restored
/// when the `Tui` instance goes out of scope, even in case of a panic.
impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = restore_terminal() {
            // It's good practice to at least log this error.
            // In a simple TUI, printing might mess up the screen further,
            // but it's better than silently failing.
            eprintln!("Failed to restore terminal: {e}");
        }
    }
}

/// Sets up the terminal for TUI display.
///
/// This function enables raw mode, enters the alternate screen, and enables mouse capture.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(Into::into)
}

/// Restores the terminal to its original state.
///
/// This function disables raw mode, leaves the alternate screen, and disables mouse capture.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
