use anyhow::{anyhow, Result};
use ratatui::{
    text::Text,
    widgets::{ListState, ScrollbarState},
};
use reev_lib::results::{FinalStatus, TestResult};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
use strum::{Display, EnumIter, FromRepr};

#[derive(Clone, PartialEq, Debug)]
pub enum BenchmarkStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Default, Clone, Copy, Debug, Display, FromRepr, EnumIter, PartialEq, Eq)]
pub enum SelectedAgent {
    #[default]
    #[strum(to_string = " Deterministic ")]
    Deterministic,
    #[strum(to_string = " Gemini ")]
    Gemini,
    #[strum(to_string = " Local ")]
    Local,
    #[strum(to_string = " GLM 4.6 ")]
    Glm46,
}

impl SelectedAgent {
    pub fn to_agent_name(self) -> &'static str {
        match self {
            SelectedAgent::Deterministic => "deterministic",
            SelectedAgent::Gemini => "gemini-2.5-pro",
            SelectedAgent::Local => "local",
            SelectedAgent::Glm46 => "glm-4.6",
        }
    }

    pub fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn is_disabled(&self, is_running: bool) -> bool {
        if is_running {
            return true;
        }

        // Check if GLM environment variables are properly configured
        if matches!(self, SelectedAgent::Glm46) {
            let has_glm_key = std::env::var("GLM_API_KEY").is_ok();
            let has_glm_url = std::env::var("GLM_API_URL").is_ok();
            !(has_glm_key && has_glm_url)
        } else {
            false
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ActivePanel {
    BenchmarkNavigator,
    ExecutionTrace,
    AgentLog,
}

pub struct Benchmark<'a> {
    pub path: PathBuf,
    pub status: BenchmarkStatus,
    pub result: Option<TestResult>,
    pub details: Text<'a>,
}

pub enum TuiEvent {
    BenchmarkStarted(usize),
    BenchmarkCompleted(usize, Result<TestResult>),
}

pub struct App<'a> {
    pub should_quit: bool,
    pub is_running_all: bool,
    pub is_running_benchmark: bool,
    pub active_panel: ActivePanel,
    pub show_log_panel: bool,
    pub selected_agent: SelectedAgent,
    pub benchmarks: Vec<Benchmark<'a>>,
    pub benchmark_state: ListState,
    pub event_sender: Sender<TuiEvent>,
    pub event_receiver: Receiver<TuiEvent>,
    pub details_scroll: u16,
    pub details_horizontal_scroll: u16,
    pub details_scroll_state: ScrollbarState,
    pub transaction_log_content: Text<'a>,
    pub log_scroll: u16,
    pub log_horizontal_scroll: u16,
    pub log_scroll_state: ScrollbarState,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> App<'a> {
    pub fn new() -> Self {
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
            show_log_panel: true,
            selected_agent: SelectedAgent::default(),
            benchmarks,
            benchmark_state,
            event_sender,
            event_receiver,
            details_scroll: 0,
            details_horizontal_scroll: 0,
            details_scroll_state: ScrollbarState::default(),
            transaction_log_content: Text::from(""),
            log_scroll: 0,
            log_horizontal_scroll: 0,
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

    pub fn handle_tui_event(&mut self, event: TuiEvent) {
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
                    self.update_log_content_from_selection();
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

    /// Updates the content of the log panel based on the currently selected benchmark's result.
    fn update_log_content_from_selection(&mut self) {
        let new_logs = if let Some(benchmark) = self.get_selected_benchmark() {
            if let Some(result) = &benchmark.result {
                if let Some(step) = result.trace.steps.first() {
                    step.observation.last_transaction_logs.join("\n")
                } else {
                    String::from("No transaction steps found for this result.")
                }
            } else {
                // If benchmark has not been run or failed, there are no logs to show.
                String::new()
            }
        } else {
            String::new()
        };
        self.transaction_log_content = Text::from(new_logs);
    }

    pub fn on_run(&mut self) {
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

    pub fn on_run_all(&mut self) {
        if self.is_running_benchmark || self.benchmarks.is_empty() {
            return;
        }
        self.is_running_all = true;
        self.benchmark_state.select(Some(0));
        self.on_run();
    }

    pub fn reset_benchmarks(&mut self) {
        for benchmark in &mut self.benchmarks {
            benchmark.status = BenchmarkStatus::Pending;
            benchmark.result = None;
            benchmark.details = Text::from("> This benchmark has not been run yet.");
        }
        self.update_log_content_from_selection();
        self.reset_scroll();
    }

    pub fn on_up(&mut self) {
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
        self.update_log_content_from_selection();
    }

    pub fn on_left(&mut self) {
        if !self.is_running_benchmark && !self.selected_agent.is_disabled(self.is_running_benchmark)
        {
            self.selected_agent = self.selected_agent.previous();
            self.reset_benchmarks();
        }
    }

    pub fn on_right(&mut self) {
        if !self.is_running_benchmark && !self.selected_agent.is_disabled(self.is_running_benchmark)
        {
            self.selected_agent = self.selected_agent.next();
            self.reset_benchmarks();
        }
    }

    pub fn select_deterministic(&mut self) {
        if !self.is_running_benchmark {
            self.selected_agent = SelectedAgent::Deterministic;
            self.reset_benchmarks();
        }
    }

    pub fn select_gemini(&mut self) {
        if !self.is_running_benchmark {
            self.selected_agent = SelectedAgent::Gemini;
            self.reset_benchmarks();
        }
    }

    pub fn select_local(&mut self) {
        if !self.is_running_benchmark {
            self.selected_agent = SelectedAgent::Local;
            self.reset_benchmarks();
        }
    }

    pub fn select_glm46(&mut self) {
        if !self.is_running_benchmark && !self.selected_agent.is_disabled(false) {
            self.selected_agent = SelectedAgent::Glm46;
            self.reset_benchmarks();
        }
    }

    pub fn on_down(&mut self) {
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
        self.update_log_content_from_selection();
    }

    pub fn on_tab(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::BenchmarkNavigator => ActivePanel::ExecutionTrace,
            ActivePanel::ExecutionTrace => {
                if self.show_log_panel {
                    ActivePanel::AgentLog
                } else {
                    ActivePanel::BenchmarkNavigator
                }
            }
            ActivePanel::AgentLog => ActivePanel::BenchmarkNavigator,
        };
        self.reset_scroll();
    }

    pub fn get_selected_benchmark(&self) -> Option<&Benchmark> {
        self.benchmark_state
            .selected()
            .and_then(|i| self.benchmarks.get(i))
    }

    fn reset_scroll(&mut self) {
        self.details_scroll = 0;
        self.details_horizontal_scroll = 0;
        self.log_scroll = 0;
        self.log_horizontal_scroll = 0;
    }

    pub fn scroll_down(&mut self) {
        let content_height = self
            .get_selected_benchmark()
            .map_or(0, |b| b.details.height());
        self.details_scroll = self.details_scroll.saturating_add(1);
        if self.details_scroll > content_height as u16 {
            self.details_scroll = content_height as u16;
        }
    }

    pub fn scroll_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(1);
    }

    pub fn scroll_log_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    pub fn scroll_log_down(&mut self) {
        let content_height = self.transaction_log_content.height().saturating_sub(1) as u16;
        self.log_scroll = self.log_scroll.saturating_add(1).min(content_height);
    }

    pub fn on_toggle_log_panel(&mut self) {
        self.show_log_panel = !self.show_log_panel;
        if !self.show_log_panel && self.active_panel == ActivePanel::AgentLog {
            self.active_panel = ActivePanel::ExecutionTrace;
        }
    }

    pub fn scroll_left(&mut self) {
        self.details_horizontal_scroll = self.details_horizontal_scroll.saturating_sub(4);
    }

    pub fn scroll_right(&mut self) {
        self.details_horizontal_scroll = self.details_horizontal_scroll.saturating_add(4);
    }

    pub fn scroll_log_left(&mut self) {
        self.log_horizontal_scroll = self.log_horizontal_scroll.saturating_sub(4);
    }

    pub fn scroll_log_right(&mut self) {
        self.log_horizontal_scroll = self.log_horizontal_scroll.saturating_add(4);
    }
}
