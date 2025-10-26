use crate::app::{ActivePanel, App};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;
use tokio::time::timeout;

pub async fn handle_events(app: &mut App<'_>) -> Result<()> {
    // Try to receive TUI events with timeout to keep UI responsive
    if let Ok(Some(event)) = timeout(Duration::from_millis(10), app.event_receiver.recv()).await {
        app.handle_tui_event(event);
    }

    // Handle keyboard events
    if event::poll(Duration::from_millis(0))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Tab => app.on_tab(),
                    KeyCode::Char('l') => app.on_toggle_log_panel(),
                    KeyCode::Char('r') | KeyCode::Enter if !app.is_running_benchmark => {
                        app.on_run()
                    }
                    KeyCode::Char('a') if !app.is_running_benchmark => app.on_run_all(),
                    KeyCode::Char('s') if !app.is_running_benchmark => {
                        app.on_toggle_shared_surfpool()
                    }
                    _ => match app.active_panel {
                        ActivePanel::BenchmarkNavigator => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                            KeyCode::Left => app.on_left(),
                            KeyCode::Right => app.on_right(),
                            _ => {}
                        },
                        ActivePanel::ExecutionTrace => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            KeyCode::Left => app.scroll_left(),
                            KeyCode::Right => app.scroll_right(),
                            _ => {}
                        },
                        ActivePanel::AgentLog => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_log_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_log_down(),
                            KeyCode::Left => app.scroll_log_left(),
                            KeyCode::Right => app.scroll_log_right(),
                            _ => {}
                        },
                    },
                }
            }
        }
    }
    Ok(())
}
