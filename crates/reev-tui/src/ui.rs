use crate::app::{ActivePanel, App, BenchmarkStatus, SelectedAgent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};
use strum::IntoEnumIterator;

fn create_percentage_spans(score_str: String, percentage: u32) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = score_str.chars().collect();

    // Find the first non-zero digit
    let mut first_non_zero_idx = 0;
    for (i, &c) in chars.iter().enumerate() {
        if c != '0' && c != '%' {
            first_non_zero_idx = i;
            break;
        }
    }

    // Add prefix (leading zeros) with black color
    if first_non_zero_idx > 0 {
        let prefix: String = chars.iter().take(first_non_zero_idx).collect();
        spans.push(Span::styled(prefix, Style::default().fg(Color::Black)));
    }

    // Add the number and percent sign with yellow if below 100% but not 0%, grey for 0%, otherwise white
    let suffix: String = chars.iter().skip(first_non_zero_idx).collect();
    let color = if percentage == 0 {
        Color::DarkGray
    } else if percentage < 100 {
        Color::Yellow
    } else {
        Color::White
    };
    spans.push(Span::styled(suffix, Style::default().fg(color)));

    spans
}

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
    let block = Block::default()
        .title(" Reev TUI - Agent Selection ")
        .borders(Borders::ALL);

    let normal_style = Style::default().fg(Color::White);
    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let disabled_style = Style::default().fg(Color::DarkGray);

    // Create custom tabs with individual styling
    let agents: Vec<_> = SelectedAgent::iter().collect();
    let mut tab_spans = Vec::new();

    for (index, agent) in agents.iter().enumerate() {
        let is_selected = *agent == app.selected_agent;
        let is_disabled = agent.is_disabled(app.is_running_benchmark);

        let style = if is_selected {
            highlight_style
        } else if is_disabled {
            disabled_style
        } else {
            normal_style
        };

        let tab_text = agent.to_string();
        tab_spans.push(Span::styled(tab_text, style));

        // Add separator between tabs
        if index < agents.len() - 1 {
            tab_spans.push(Span::styled("|", normal_style));
        }
    }

    let tabs_text = Line::from(tab_spans);
    let paragraph = Paragraph::new(tabs_text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
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
            let (score_prefix, status_symbol) = match b.status {
                BenchmarkStatus::Pending => (
                    Span::styled("000%", Style::default().add_modifier(Modifier::DIM)),
                    Span::styled("[ ]", Style::default()),
                ),
                BenchmarkStatus::Running => (
                    Span::styled("000%", Style::default().add_modifier(Modifier::DIM)),
                    Span::styled("[…]", Style::default().fg(Color::Yellow)),
                ),
                BenchmarkStatus::Succeeded => {
                    let score = b.result.as_ref().map_or(0.0, |r| r.score);
                    let percentage = (score * 100.0).round() as u32;
                    let score_str = format!("{percentage:03}%");
                    (
                        Span::styled(score_str, Style::default().add_modifier(Modifier::DIM)),
                        Span::styled("[✔]", Style::default().fg(Color::Green)),
                    )
                }
                BenchmarkStatus::Failed => {
                    let score = b.result.as_ref().map_or(0.0, |r| r.score);
                    let percentage = (score * 100.0).round() as u32;
                    let score_str = format!("{percentage:03}%");
                    (
                        Span::styled(score_str, Style::default().add_modifier(Modifier::DIM)),
                        Span::styled("[✗]", Style::default().fg(Color::Red)),
                    )
                }
            };
            let file_name = b.path.file_name().unwrap_or_default().to_string_lossy();
            let score_content = score_prefix.content.to_string();
            let score_spans = if score_content
                .chars()
                .all(|c| c.is_ascii_digit() || c == '%')
            {
                if let Some(percent_idx) = score_content.find('%') {
                    let percentage_str = &score_content[..percent_idx];
                    if let Ok(percentage_val) = percentage_str.parse::<u32>() {
                        create_percentage_spans(score_content, percentage_val)
                    } else {
                        vec![score_prefix]
                    }
                } else {
                    vec![score_prefix]
                }
            } else {
                vec![score_prefix]
            };

            let mut line_spans = Vec::new();
            line_spans.extend(score_spans);
            line_spans.push(Span::raw(" "));
            line_spans.push(status_symbol);
            line_spans.push(Span::raw(format!(" {file_name}")));

            ListItem::new(Line::from(line_spans))
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
        Span::styled("1-4", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Agent | "),
        Span::styled("G", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("LM | "),
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
