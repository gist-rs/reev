use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Tabs, Wrap,
    },
    Frame, Terminal,
};
use reev_lib::results::{FinalStatus, TestResult};
use std::{
    fs,
    io::{self, Stdout},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Clone, PartialEq, Debug)]
enum BenchmarkStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq, Eq)]
enum SelectedAgent {
    #[default]
    #[strum(to_string = " Deterministic ")]
    Deterministic,
    #[strum(to_string = " Gemini ")]
    Gemini,
    #[strum(to_string = " Local ")]
    Local,
}

impl SelectedAgent {
    fn to_agent_name(self) -> &'static str {
        match self {
            SelectedAgent::Deterministic => "deterministic",
            SelectedAgent::Gemini => "gemini-pro",
            SelectedAgent::Local => "local-model",
        }
    }

    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ActivePanel {
    BenchmarkNavigator,
    ExecutionTrace,
    AgentLog,
}

struct Benchmark<'a> {
    path: PathBuf,
    status: BenchmarkStatus,
    result: Option<TestResult>,
    details: Text<'a>,
}

enum TuiEvent {
    BenchmarkStarted(usize),
    BenchmarkCompleted(usize, Result<TestResult>),
}

struct App<'a> {
    should_quit: bool,
    is_running_all: bool,
    is_running_benchmark: bool,
    active_panel: ActivePanel,
    selected_agent: SelectedAgent,
    benchmarks: Vec<Benchmark<'a>>,
    benchmark_state: ListState,
    event_sender: Sender<TuiEvent>,
    event_receiver: Receiver<TuiEvent>,
    // Scroll state for the details/trace panels
    details_scroll: u16,
    details_scroll_state: ScrollbarState,
    // Agent log viewer state
    agent_log_content: Text<'a>,
    log_scroll: u16,
    log_scroll_state: ScrollbarState,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        let mut benchmark_state = ListState::default();
        let benchmarks = Self::discover_benchmarks().unwrap_or_else(|_| vec![]);
        if !benchmarks.is_empty() {
            benchmark_state.select(Some(0));
        }

        Self {
            should_quit: false,
            is_running_all: false,
            is_running_benchmark: false,
            active_panel: ActivePanel::BenchmarkNavigator,
            selected_agent: SelectedAgent::default(),
            benchmarks,
            benchmark_state,
            event_sender,
            event_receiver,
            details_scroll: 0,
            details_scroll_state: ScrollbarState::default(),
            agent_log_content: Text::from(""),
            log_scroll: 0,
            log_scroll_state: ScrollbarState::default(),
        }
    }

    fn discover_benchmarks() -> Result<Vec<Benchmark<'a>>> {
        let mut benchmarks = vec![];
        let root = project_root::get_project_root()?;
        let benchmarks_dir = root.join("benchmarks");

        if benchmarks_dir.is_dir() {
            for entry in fs::read_dir(benchmarks_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file()
                    && (path.extension() == Some("yml".as_ref())
                        || path.extension() == Some("yaml".as_ref()))
                {
                    benchmarks.push(Benchmark {
                        path,
                        status: BenchmarkStatus::Pending,
                        result: None,
                        details: Text::from("> This benchmark has not been run yet."),
                    });
                }
            }
        }
        benchmarks.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(benchmarks)
    }

    fn handle_tui_event(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::BenchmarkStarted(index) => {
                self.is_running_benchmark = true;
                if let Some(benchmark) = self.benchmarks.get_mut(index) {
                    benchmark.status = BenchmarkStatus::Running;
                    benchmark.details = Text::from("Benchmark is running...");
                    benchmark.result = None;
                }
            }
            TuiEvent::BenchmarkCompleted(index, result) => {
                self.is_running_benchmark = false;
                if let Some(benchmark) = self.benchmarks.get_mut(index) {
                    match result {
                        Ok(test_result) => {
                            benchmark.status = match test_result.final_status {
                                FinalStatus::Succeeded => BenchmarkStatus::Succeeded,
                                FinalStatus::Failed => BenchmarkStatus::Failed,
                            };
                            let rendered_tree =
                                reev_runner::renderer::render_result_as_tree(&test_result);
                            benchmark.details = Text::from(rendered_tree);
                            benchmark.result = Some(test_result);
                        }
                        Err(e) => {
                            benchmark.status = BenchmarkStatus::Failed;
                            benchmark.details = Text::from(format!("Error: {e}"));
                            benchmark.result = None;
                        }
                    }
                }

                if self.is_running_all {
                    let next_index = index + 1;
                    if next_index < self.benchmarks.len() {
                        self.benchmark_state.select(Some(next_index));
                        self.on_run();
                    } else {
                        self.is_running_all = false;
                    }
                }
            }
        }
    }

    fn update_logs(&mut self) -> Result<()> {
        let root = project_root::get_project_root()?;
        let log_path = root.join("logs").join("reev-agent.log");
        if log_path.exists() {
            let content = fs::read_to_string(log_path)?;
            self.agent_log_content = Text::from(content);
        } else {
            self.agent_log_content = Text::from("Log file not found at logs/reev-agent.log");
        }
        Ok(())
    }

    fn on_run(&mut self) {
        if self.is_running_benchmark {
            return;
        }

        if let Some(selected_index) = self.benchmark_state.selected() {
            let path = self.benchmarks[selected_index].path.clone();
            let sender = self.event_sender.clone();
            let agent_name = self.selected_agent.to_agent_name();

            thread::spawn(move || {
                sender
                    .send(TuiEvent::BenchmarkStarted(selected_index))
                    .unwrap();

                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(reev_runner::run_benchmarks(path, agent_name));

                let final_result = match result {
                    Ok(mut results) => results
                        .pop()
                        .ok_or_else(|| anyhow!("Benchmark runner returned no results.")),
                    Err(e) => Err(e),
                };

                sender
                    .send(TuiEvent::BenchmarkCompleted(selected_index, final_result))
                    .unwrap();
            });
        }
    }

    fn on_run_all(&mut self) {
        if self.is_running_benchmark || self.benchmarks.is_empty() {
            return;
        }
        self.is_running_all = true;
        self.benchmark_state.select(Some(0));
        self.on_run();
    }

    fn reset_benchmarks(&mut self) {
        for benchmark in &mut self.benchmarks {
            benchmark.status = BenchmarkStatus::Pending;
            benchmark.result = None;
            benchmark.details = Text::from("> This benchmark has not been run yet.");
        }
        self.reset_scroll();
    }

    fn on_up(&mut self) {
        if self.benchmarks.is_empty() {
            return;
        }
        let i = self.benchmark_state.selected().map_or(0, |i| {
            if i == 0 {
                self.benchmarks.len() - 1
            } else {
                i - 1
            }
        });
        self.benchmark_state.select(Some(i));
        self.reset_scroll();
    }

    fn on_left(&mut self) {
        if !self.is_running_benchmark {
            self.selected_agent = self.selected_agent.previous();
            self.reset_benchmarks();
        }
    }

    fn on_right(&mut self) {
        if !self.is_running_benchmark {
            self.selected_agent = self.selected_agent.next();
            self.reset_benchmarks();
        }
    }

    fn on_down(&mut self) {
        if self.benchmarks.is_empty() {
            return;
        }
        let i = self.benchmark_state.selected().map_or(0, |i| {
            if i >= self.benchmarks.len() - 1 {
                0
            } else {
                i + 1
            }
        });
        self.benchmark_state.select(Some(i));
        self.reset_scroll();
    }

    fn on_tab(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::BenchmarkNavigator => ActivePanel::ExecutionTrace,
            ActivePanel::ExecutionTrace => ActivePanel::AgentLog,
            ActivePanel::AgentLog => ActivePanel::BenchmarkNavigator,
        };
        self.reset_scroll();
    }

    fn get_selected_benchmark(&self) -> Option<&Benchmark> {
        self.benchmark_state
            .selected()
            .and_then(|i| self.benchmarks.get(i))
    }

    fn reset_scroll(&mut self) {
        self.details_scroll = 0;
        self.log_scroll = 0;
    }

    fn scroll_down(&mut self) {
        let content_height = self
            .get_selected_benchmark()
            .map_or(0, |b| b.details.height());
        self.details_scroll = self.details_scroll.saturating_add(1);
        if self.details_scroll > content_height as u16 {
            self.details_scroll = content_height as u16;
        }
    }

    fn scroll_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
    }

    fn scroll_log_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    fn scroll_log_down(&mut self) {
        let content_height = self.agent_log_content.height().saturating_sub(1) as u16;
        self.log_scroll = self.log_scroll.saturating_add(1).min(content_height);
    }
}

fn main() -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;

    restore_terminal()?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(Into::into)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        app.update_logs()?;
        terminal.draw(|f| ui(f, app))?;
        handle_events(app)?;
    }
    Ok(())
}

fn handle_events(app: &mut App) -> Result<()> {
    if let Ok(event) = app.event_receiver.try_recv() {
        app.handle_tui_event(event);
    }

    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Tab => app.on_tab(),
                    KeyCode::Char('h') | KeyCode::Left => app.on_left(),
                    KeyCode::Char('l') | KeyCode::Right => app.on_right(),
                    KeyCode::Char('r') => app.on_run(),
                    KeyCode::Char('a') => app.on_run_all(),
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

fn ui(f: &mut Frame, app: &mut App) {
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

    let right_panels_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_layout[1]);

    render_trace_view(f, app, right_panels_layout[0]);
    render_agent_log_view(f, app, right_panels_layout[1]);

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
        .wrap(Wrap { trim: true })
        .scroll((app.details_scroll, 0));

    f.render_widget(paragraph, area);

    // Update the scroll state after the paragraph is rendered and its borrow on `app` is released.
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
        .title("C: Agent Log")
        .borders(Borders::ALL)
        .border_style(border_style);

    let text = app.agent_log_content.clone();
    let content_height = text.height();

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.log_scroll, 0));

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
        Span::styled("◄ ► / (h/l)", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Select Agent | "),
        Span::styled("[R]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("un | "),
        Span::styled("[A]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("ll | "),
        Span::styled("[Q]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("uit"),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(Paragraph::new(controls), area);
}
