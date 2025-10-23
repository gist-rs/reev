mod app;
mod event;
mod tui;
mod ui;

use anyhow::{Context, Result};
use app::App;
use event::handle_events;
use tracing::{info, subscriber};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use tui::Tui;
use ui::ui;

fn main() -> Result<()> {
    // Load environment variables from a .env file.
    dotenvy::dotenv().ok();

    // Initialize tracing.
    init_tracing()?;
    info!("--- Reev TUI Started ---");

    // Set a panic hook to ensure the terminal is restored even on panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // The Tui struct's Drop implementation will automatically restore the terminal.
        // We just let the original hook print the panic information.
        original_hook(panic_info);
    }));

    // Set the current directory to the workspace root for consistent path resolution.
    let workspace_root = project_root::get_project_root()
        .context("Failed to find workspace root. Please run from within the reev workspace.")?;
    std::env::set_current_dir(&workspace_root)
        .with_context(|| format!("Failed to set current directory to {workspace_root:?}"))?;

    let mut app = App::new();
    let mut tui = Tui::new()?;

    // Main application loop
    while !app.should_quit {
        // Draw the UI
        tui.terminal().draw(|f| ui(f, &mut app))?;

        // Handle events
        handle_events(&mut app)?;
    }

    Ok(())
}

/// Initializes tracing for the TUI.
fn init_tracing() -> Result<()> {
    let subscriber = Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,reev_lib=debug,reev_tui=debug")),
        )
        .with(fmt::layer());

    subscriber::set_global_default(subscriber)
        .context("Failed to set global default tracing subscriber")?;

    Ok(())
}
