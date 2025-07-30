use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Sparkline,
        Tabs, Wrap,
    },
    Frame, Terminal,
};

use ellex_core::runtime::EllexRuntime;
use ellex_parser::parse;
use ellex_repl::ReplSession;

#[derive(Debug, Clone)]
pub struct ParseMetrics {
    pub timestamp: DateTime<Local>,
    pub parse_time_ms: f64,
    pub tokens_count: usize,
    pub ast_depth: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub timestamp: DateTime<Local>,
    pub memory_usage_mb: f64,
    pub execution_time_ms: f64,
    pub loop_iterations: usize,
    pub recursion_depth: usize,
    pub variables_count: usize,
}

#[derive(Default)]
struct MetricsHistory {
    parse_metrics: VecDeque<ParseMetrics>,
    runtime_metrics: VecDeque<RuntimeMetrics>,
    parse_times: VecDeque<u64>,
    memory_usage: VecDeque<u64>,
    max_history: usize,
}

impl MetricsHistory {
    fn new() -> Self {
        Self {
            parse_metrics: VecDeque::with_capacity(100),
            runtime_metrics: VecDeque::with_capacity(100),
            parse_times: VecDeque::with_capacity(60),
            memory_usage: VecDeque::with_capacity(60),
            max_history: 100,
        }
    }

    fn add_parse_metric(&mut self, metric: ParseMetrics) {
        if self.parse_metrics.len() >= self.max_history {
            self.parse_metrics.pop_front();
        }
        self.parse_times.push_back(metric.parse_time_ms as u64);
        if self.parse_times.len() > 60 {
            self.parse_times.pop_front();
        }
        self.parse_metrics.push_back(metric);
    }

    fn add_runtime_metric(&mut self, metric: RuntimeMetrics) {
        if self.runtime_metrics.len() >= self.max_history {
            self.runtime_metrics.pop_front();
        }
        self.memory_usage.push_back(metric.memory_usage_mb as u64);
        if self.memory_usage.len() > 60 {
            self.memory_usage.pop_front();
        }
        self.runtime_metrics.push_back(metric);
    }
}

#[derive(Default)]
struct CodeEditor {
    content: String,
    cursor_position: usize,
    scroll_offset: usize,
}

impl CodeEditor {
    fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    fn get_cursor_xy(&self, width: u16) -> (u16, u16) {
        let mut x = 0u16;
        let mut y = 0u16;
        
        for (i, ch) in self.content.chars().enumerate() {
            if i == self.cursor_position {
                break;
            }
            if ch == '\n' || x >= width - 2 {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        
        (x, y.saturating_sub(self.scroll_offset as u16))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveTab {
    Editor,
    Metrics,
    ParseTree,
    Output,
}

pub struct App {
    editor: CodeEditor,
    active_tab: ActiveTab,
    metrics: Arc<Mutex<MetricsHistory>>,
    parse_tree: String,
    output_log: Vec<String>,
    runtime: EllexRuntime,
    repl_session: ReplSession,  // Integrated REPL session
    is_running: bool,
    show_help: bool,
    ast_visualization: String,
    current_parse_depth: usize,
    parsing_animation_frame: usize,
    ir_buffer: String,      // For LLVM-IR text
    ir_graph: String,       // Rendered graph (ASCII)
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: CodeEditor::default(),
            active_tab: ActiveTab::Editor,
            metrics: Arc::new(Mutex::new(MetricsHistory::new())),
            parse_tree: String::new(),
            output_log: Vec::new(),
            runtime: EllexRuntime::new(),
            repl_session: ReplSession::new(),
            is_running: false,
            show_help: false,
            ast_visualization: String::new(),
            current_parse_depth: 0,
            parsing_animation_frame: 0,
            ir_buffer: String::new(),
            ir_graph: String::new(),
        }
    }

    fn parse_code(&mut self) {
        let start = Instant::now();
        let code = self.editor.content.clone();
        
        self.parsing_animation_frame = 0;
        
        // Use REPL session for execution
        match self.repl_session.execute_line(&code) {
            Ok(output) => {
                let parse_time = start.elapsed().as_secs_f64() * 1000.0;
                
                // Add output to log
                for line in output {
                    self.output_log.push(line);
                }
                
                // Also try to parse for visualization
                if let Ok(statements) = parse(&code) {
                    self.parse_tree = format!("{:#?}", statements);
                    self.ast_visualization = self.generate_ast_visualization(&statements);
                } else {
                    self.parse_tree = "Parse error".to_string();
                    self.ast_visualization = "Unable to visualize".to_string();
                }
                
                let metric = ParseMetrics {
                    timestamp: Local::now(),
                    parse_time_ms: parse_time,
                    tokens_count: self.repl_session.execution_count,
                    ast_depth: 1, // Simplified for REPL usage
                    success: true,
                    error_message: None,
                };
                
                self.metrics.lock().unwrap().add_parse_metric(metric);
                self.output_log.push(format!(
                    "[{}] Parse successful in {:.2}ms",
                    Local::now().format("%H:%M:%S"),
                    parse_time
                ));
            }
            Err(e) => {
                let parse_time = start.elapsed().as_secs_f64() * 1000.0;
                
                let metric = ParseMetrics {
                    timestamp: Local::now(),
                    parse_time_ms: parse_time,
                    tokens_count: 0,
                    ast_depth: 0,
                    success: false,
                    error_message: Some(e.to_string()),
                };
                
                self.metrics.lock().unwrap().add_parse_metric(metric);
                self.output_log.push(format!(
                    "[{}] Parse error: {}",
                    Local::now().format("%H:%M:%S"),
                    e
                ));
                
                self.parse_tree = format!("Parse Error:\n{}", e);
                self.ast_visualization = String::new();
            }
        }
    }

    fn run_code(&mut self) {
        self.is_running = true;
        let start = Instant::now();
        
        // Use REPL session for execution
        match self.repl_session.execute_line(&self.editor.content) {
            Ok(output) => {
                let exec_time = start.elapsed().as_secs_f64() * 1000.0;
                
                // Add all output to the log
                for line in output {
                    self.output_log.push(line);
                }
                
                let metric = RuntimeMetrics {
                    timestamp: Local::now(),
                    memory_usage_mb: 0.5, // Placeholder
                    execution_time_ms: exec_time,
                    loop_iterations: 0,
                    recursion_depth: 0,
                    variables_count: self.repl_session.variables.len(),
                };
                
                self.metrics.lock().unwrap().add_runtime_metric(metric);
                self.output_log.push(format!(
                    "[{}] Execution completed in {:.2}ms",
                    Local::now().format("%H:%M:%S"),
                    exec_time
                ));
            }
            Err(e) => {
                self.output_log.push(format!(
                    "[{}] Runtime error: {}",
                    Local::now().format("%H:%M:%S"),
                    e
                ));
            }
        }
        
        self.is_running = false;
    }

    fn generate_ast_visualization(&self, _statements: &[ellex_core::values::Statement]) -> String {
        let mut viz = String::new();
        viz.push_str("Program\n");
        viz.push_str("├─ Statements\n");
        
        for (i, stmt) in _statements.iter().enumerate() {
            let is_last = i == _statements.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            viz.push_str(&format!("{}  {}: {:?}\n", prefix, i + 1, stmt));
        }
        
        viz
    }

    fn calculate_ast_depth(&self, _statements: &[ellex_core::values::Statement]) -> usize {
        3 // Placeholder
    }

    fn update_parsing_animation(&mut self) {
        self.parsing_animation_frame = (self.parsing_animation_frame + 1) % 8;
    }
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Code Editor")
        .borders(Borders::ALL)
        .border_style(if app.active_tab == ActiveTab::Editor {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let inner_area = block.inner(area);
    let text = Text::from(app.editor.content.as_str());
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.editor.scroll_offset as u16, 0));

    f.render_widget(paragraph, area);

    if app.active_tab == ActiveTab::Editor {
        let (cursor_x, cursor_y) = app.editor.get_cursor_xy(inner_area.width);
        f.set_cursor_position((
            inner_area.x + cursor_x + 1,
            inner_area.y + cursor_y + 1,
        ));
    }
}

fn draw_metrics(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    let metrics = app.metrics.lock().unwrap();

    // Current stats
    let latest_parse = metrics.parse_metrics.back();
    let _latest_runtime = metrics.runtime_metrics.back();

    let stats_text = if let Some(parse) = latest_parse {
        vec![
            Line::from(vec![
                Span::raw("Parse Time: "),
                Span::styled(
                    format!("{:.2}ms", parse.parse_time_ms),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Tokens: "),
                Span::styled(
                    parse.tokens_count.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" | AST Depth: "),
                Span::styled(
                    parse.ast_depth.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Status: "),
                if parse.success {
                    Span::styled("Success", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Failed", Style::default().fg(Color::Red))
                },
            ]),
        ]
    } else {
        vec![Line::from("No parse metrics available")]
    };

    let stats = Paragraph::new(stats_text)
        .block(Block::default().title("Current Stats").borders(Borders::ALL));
    f.render_widget(stats, chunks[0]);

    // Parse time sparkline
    let parse_times: Vec<u64> = metrics.parse_times.iter().cloned().collect();
    let sparkline = Sparkline::default()
        .block(Block::default().title("Parse Times (ms)").borders(Borders::ALL))
        .data(&parse_times)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(sparkline, chunks[1]);

    // Memory usage sparkline
    let memory_data: Vec<u64> = metrics.memory_usage.iter().cloned().collect();
    let memory_sparkline = Sparkline::default()
        .block(Block::default().title("Memory Usage (MB)").borders(Borders::ALL))
        .data(&memory_data)
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(memory_sparkline, chunks[2]);

    // Metrics history
    let history_items: Vec<ListItem> = metrics
        .parse_metrics
        .iter()
        .rev()
        .take(10)
        .map(|m| {
            let style = if m.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            ListItem::new(format!(
                "[{}] {:.2}ms - {} tokens",
                m.timestamp.format("%H:%M:%S"),
                m.parse_time_ms,
                m.tokens_count
            ))
            .style(style)
        })
        .collect();

    let history_list = List::new(history_items)
        .block(Block::default().title("Parse History").borders(Borders::ALL));
    f.render_widget(history_list, chunks[3]);
}

fn draw_parse_tree(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // AST Text
    let parse_tree = Paragraph::new(app.parse_tree.as_str())
        .block(
            Block::default()
                .title("Parse Tree (Debug)")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .scroll((0, 0));
    f.render_widget(parse_tree, chunks[0]);

    // Visual AST
    let visual_title = if app.is_running {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        format!(
            "AST Visualization {}",
            spinner[app.parsing_animation_frame % spinner.len()]
        )
    } else {
        "AST Visualization".to_string()
    };

    let visual_ast = Paragraph::new(app.ast_visualization.as_str())
        .block(
            Block::default()
                .title(visual_title)
                .borders(Borders::ALL)
                .border_style(if app.is_running {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(visual_ast, chunks[1]);
}

fn draw_output(f: &mut Frame, app: &App, area: Rect) {
    let output_items: Vec<ListItem> = app
        .output_log
        .iter()
        .rev()
        .map(|log| ListItem::new(log.as_str()))
        .collect();

    let output = List::new(output_items)
        .block(
            Block::default()
                .title("Output Log")
                .borders(Borders::ALL),
        );
    f.render_widget(output, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Keyboard Shortcuts", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw("         - Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+P", Style::default().fg(Color::Cyan)),
            Span::raw("      - Parse code"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+R", Style::default().fg(Color::Cyan)),
            Span::raw("      - Run code"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+L", Style::default().fg(Color::Cyan)),
            Span::raw("      - Clear output"),
        ]),
        Line::from(vec![
            Span::styled("F1", Style::default().fg(Color::Cyan)),
            Span::raw("          - Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("Esc/q", Style::default().fg(Color::Cyan)),
            Span::raw("       - Exit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Editor Controls", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Arrow keys  - Move cursor"),
        Line::from("Backspace   - Delete character"),
        Line::from("Enter       - New line"),
        Line::from(""),
        Line::from("Press F1 to close help"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    f.render_widget(help_paragraph, area);
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    // Tab bar
    let tab_titles = vec!["Editor", "Metrics", "Parse Tree", "Output"];
    let selected_tab = match app.active_tab {
        ActiveTab::Editor => 0,
        ActiveTab::Metrics => 1,
        ActiveTab::ParseTree => 2,
        ActiveTab::Output => 3,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .title("Ellex Language TUI")
                .borders(Borders::ALL),
        )
        .select(selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Main content area
    if app.show_help {
        draw_help(f, chunks[1]);
    } else {
        match app.active_tab {
            ActiveTab::Editor => draw_editor(f, app, chunks[1]),
            ActiveTab::Metrics => draw_metrics(f, app, chunks[1]),
            ActiveTab::ParseTree => draw_parse_tree(f, app, chunks[1]),
            ActiveTab::Output => draw_output(f, app, chunks[1]),
        }
    }
}

pub async fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_help {
                        match key.code {
                            KeyCode::F(1) | KeyCode::Esc => app.show_help = false,
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => break,
                            KeyCode::Tab => {
                                app.active_tab = match app.active_tab {
                                    ActiveTab::Editor => ActiveTab::Metrics,
                                    ActiveTab::Metrics => ActiveTab::ParseTree,
                                    ActiveTab::ParseTree => ActiveTab::Output,
                                    ActiveTab::Output => ActiveTab::Editor,
                                };
                            }
                            KeyCode::F(1) => app.show_help = true,
                            KeyCode::Char('p') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                app.parse_code();
                            }
                            KeyCode::Char('r') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                app.run_code();
                            }
                            KeyCode::Char('l') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                app.output_log.clear();
                            }
                            _ => {
                                if app.active_tab == ActiveTab::Editor {
                                    match key.code {
                                        KeyCode::Char(c) => app.editor.insert_char(c),
                                        KeyCode::Backspace => app.editor.delete_char(),
                                        KeyCode::Enter => app.editor.insert_char('\n'),
                                        KeyCode::Left => app.editor.move_cursor_left(),
                                        KeyCode::Right => app.editor.move_cursor_right(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update_parsing_animation();
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}
