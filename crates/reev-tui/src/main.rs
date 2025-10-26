mod app;
mod event;
mod tui;
mod ui;

use anyhow::{Context, Result};
use app::App;
use event::handle_events;

use tui::Tui;
use ui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file.
    dotenvy::dotenv().ok();

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

    let mut app = App::new().await;
    let mut tui = Tui::new()?;

    // Main application loop
    while !app.should_quit {
        // Draw the UI
        tui.terminal().draw(|f| ui(f, &mut app))?;

        // Handle events
        handle_events(&mut app).await?;
    }

    Ok(())
}
