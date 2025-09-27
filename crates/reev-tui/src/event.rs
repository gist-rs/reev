use crate::app::{ActivePanel, App};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> Result<()> {
    if let Ok(event) = app.event_receiver.try_recv() {
        app.handle_tui_event(event);
    }

    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Tab => app.on_tab(),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Char('l') => app.on_toggle_log_panel(),
                    KeyCode::Char('r') | KeyCode::Enter if !app.is_running_benchmark => {
                        app.on_run()
                    }
                    KeyCode::Char('a') if !app.is_running_benchmark => app.on_run_all(),
                    _ => match app.active_panel {
                        ActivePanel::BenchmarkNavigator => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                            _ => {}
                        },
                        ActivePanel::ExecutionTrace => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                            _ => {}
                        },
                        ActivePanel::AgentLog => match key.code {
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_log_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_log_down(),
                            _ => {}
                        },
                    },
                }
            }
        }
    }
    Ok(())
}
