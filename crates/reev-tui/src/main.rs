mod app;
mod event;
mod tui;
mod ui;

use anyhow::Result;
use app::App;
use event::handle_events;
use tui::Tui;
use ui::ui;

fn main() -> Result<()> {
    // Set a panic hook to ensure the terminal is restored even on panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // The Tui struct's Drop implementation will automatically restore the terminal.
        // We just let the original hook print the panic information.
        original_hook(panic_info);
    }));

    let mut app = App::new();
    let mut tui = Tui::new()?;

    // Main application loop
    while !app.should_quit {
        // Update app state before drawing
        app.update_logs()?;

        // Draw the UI
        tui.terminal().draw(|f| ui(f, &mut app))?;

        // Handle events
        handle_events(&mut app)?;
    }

    Ok(())
}
