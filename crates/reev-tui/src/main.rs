use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

/// Application state.
struct App<'a> {
    should_quit: bool,
    benchmarks: Vec<&'a str>,
    benchmark_state: ListState,
}

impl<'a> App<'a> {
    /// Constructs a new instance of `App`.
    fn new() -> Self {
        let mut benchmark_state = ListState::default();
        benchmark_state.select(Some(0)); // Select the first item by default.
        Self {
            should_quit: false,
            // Placeholder data for the benchmark navigator.
            benchmarks: vec![
                "[✔] SPL-TRANSFER-001",
                "[✗] NFT-TRANSFER-001",
                "[ ] TRANSFER-SIMPLE-001",
            ],
            benchmark_state,
        }
    }

    /// Handles the `Up` key press event.
    fn on_up(&mut self) {
        let i = match self.benchmark_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.benchmarks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.benchmark_state.select(Some(i));
    }

    /// Handles the `Down` key press event.
    fn on_down(&mut self) {
        let i = match self.benchmark_state.selected() {
            Some(i) => {
                if i >= self.benchmarks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.benchmark_state.select(Some(i));
    }
}

fn main() -> Result<()> {
    // Set up a custom panic hook to restore the terminal on panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Set up the terminal.
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;

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
fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;
        handle_events(app)?;
    }
    Ok(())
}

/// Handles user input events.
fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Renders the user interface.
fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Content
        ])
        .split(f.area());

    render_header(f, main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(30), // Panel A
            Constraint::Percentage(70), // Panels B & C
        ])
        .split(main_layout[1]);

    render_benchmark_navigator(f, app, content_layout[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(70), // Panel B
            Constraint::Percentage(30), // Panel C
        ])
        .split(content_layout[1]);

    render_trace_view(f, right_layout[0]);
    render_details_pane(f, right_layout[1]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let title = Line::from(vec![" Gemini 2.5 Pro ".into()]);
    f.render_widget(Paragraph::new(title), header_layout[0]);

    let controls = Line::from(vec![
        Span::styled(
            "[R]",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        "UN ".into(),
        Span::styled(
            "[S]",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        "ETTINGS ".into(),
        Span::styled(
            "[Q]",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        "UIT".into(),
    ])
    .right_aligned();
    f.render_widget(Paragraph::new(controls), header_layout[1]);
}

fn render_benchmark_navigator(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.benchmarks.iter().map(|i| ListItem::new(*i)).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Benchmark Navigator")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.benchmark_state);
}

fn render_trace_view(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from("✔ SPL-TRANSFER-SIMPLE-001: SUCCEEDED"),
        Line::from("│"),
        Line::from("└─ Step 1:"),
        Line::from("   ├─ ACTION: spl_transfer(...)"),
        Line::from("   └─ OBSERVATION: Failure"),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("Execution Trace View")
            .borders(Borders::ALL),
    );
    f.render_widget(paragraph, area);
}

fn render_details_pane(f: &mut Frame, area: Rect) {
    let text = "> Error: Transaction failed: RPC response error -32002";
    let paragraph =
        Paragraph::new(text).block(Block::default().title("Details Pane").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
