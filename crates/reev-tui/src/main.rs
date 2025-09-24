use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

fn main() -> Result<()> {
    // Set up a custom panic hook to restore the terminal on panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Set up the terminal.
    let mut terminal = setup_terminal()?;
    run_app(&mut terminal)?;

    // Restore the terminal.
    restore_terminal()?;
    Ok(())
}

/// Sets up the terminal for TUI rendering.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

/// Restores the terminal to its original state.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// The main application loop.
fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    loop {
        terminal.draw(ui)?;

        // Poll for an event with a timeout to avoid blocking indefinitely.
        if event::poll(std::time::Duration::from_millis(250))? {
            // If an event is available, read it.
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    return Ok(());
                }
            }
        }
    }
}

/// Renders the user interface.
fn ui(f: &mut Frame) {
    // For now, this is a placeholder UI.
    // It will be replaced with the three-panel cockpit layout.
    let main_block = Block::default()
        .title("reev TUI Cockpit (Press 'q' to quit)")
        .borders(Borders::ALL);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.area());

    let placeholder_text =
        Paragraph::new("TUI Application Stub. Layout will be implemented next.").block(main_block);
    f.render_widget(placeholder_text, chunks[0]);
}
