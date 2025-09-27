use crate::app::{ActivePanel, App, BenchmarkStatus, SelectedAgent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Tabs},
    Frame,
};
use strum::IntoEnumIterator;

pub fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header(f, app, main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_layout[1]);

    render_benchmark_navigator(f, app, content_layout[0]);

    if app.show_log_panel {
        let right_panels_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_layout[1]);

        render_trace_view(f, app, right_panels_layout[0]);
        render_agent_log_view(f, app, right_panels_layout[1]);
    } else {
        render_trace_view(f, app, content_layout[1]);
    }

    render_footer(f, main_layout[2]);
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let titles = SelectedAgent::iter().map(|t| t.to_string());

    let normal_style = Style::default().fg(Color::White);
    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let disabled_style = Style::default().fg(Color::DarkGray);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" Reev TUI - Agent Selection ")
                .borders(Borders::ALL),
        )
        .select(app.selected_agent as usize)
        .style(if app.is_running_benchmark {
            disabled_style
        } else {
            normal_style
        })
        .highlight_style(if app.is_running_benchmark {
            disabled_style
        } else {
            highlight_style
        });

    f.render_widget(tabs, area);
}

fn render_benchmark_navigator(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.active_panel == ActivePanel::BenchmarkNavigator {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title("A: Benchmark Navigator")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = app
        .benchmarks
        .iter()
        .map(|b| {
            let status_symbol = match b.status {
                BenchmarkStatus::Pending => Span::styled("[ ]", Style::default()),
                BenchmarkStatus::Running => Span::styled("[…]", Style::default().fg(Color::Yellow)),
                BenchmarkStatus::Succeeded => {
                    Span::styled("[✔]", Style::default().fg(Color::Green))
                }
                BenchmarkStatus::Failed => Span::styled("[✗]", Style::default().fg(Color::Red)),
            };
            let file_name = b.path.file_name().unwrap_or_default().to_string_lossy();
            ListItem::new(Line::from(vec![
                status_symbol,
                Span::raw(format!(" {file_name}")),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.benchmark_state);
}

fn render_scrollable_text_panel(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    title: &str,
    is_active: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let text = app.get_selected_benchmark().map_or_else(
        || Text::from("No benchmark selected"),
        |b| b.details.clone(),
    );

    let content_height = text.height();

    let paragraph = Paragraph::new(text)
        .block(block)
        .scroll((app.details_scroll, app.details_horizontal_scroll));

    f.render_widget(paragraph, area);

    app.details_scroll_state = app
        .details_scroll_state
        .content_length(content_height)
        .position(app.details_scroll as usize);

    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.details_scroll_state,
    );
}

fn render_trace_view(f: &mut Frame, app: &mut App, area: Rect) {
    render_scrollable_text_panel(
        f,
        app,
        area,
        "B: Execution Trace View",
        app.active_panel == ActivePanel::ExecutionTrace,
    );
}

fn render_agent_log_view(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.active_panel == ActivePanel::AgentLog {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title("C: Transaction Logs")
        .borders(Borders::ALL)
        .border_style(border_style);

    let text = app.transaction_log_content.clone();
    let content_height = text.height();

    let paragraph = Paragraph::new(text)
        .block(block)
        .scroll((app.log_scroll, app.log_horizontal_scroll));

    f.render_widget(paragraph, area);

    app.log_scroll_state = app
        .log_scroll_state
        .content_length(content_height)
        .position(app.log_scroll as usize);

    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.log_scroll_state,
    );
}

fn render_footer(f: &mut Frame, area: Rect) {
    let controls = Line::from(vec![
        Span::raw("◄ ► Agent | "),
        Span::styled("h l", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Scroll | "),
        Span::styled("[L]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("og | "),
        Span::styled("[Enter/R]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("un | "),
        Span::styled("[A]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("ll | "),
        Span::styled("[Q]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("uit"),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Paragraph::new(controls), area);
}
