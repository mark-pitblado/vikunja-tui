// main.rs

use ansi_parser::{AnsiParser, Output};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::dotenv;
use html2text::from_read;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use reqwest::Client;
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::io;
use tokio::time::Duration; // Correct crate import

// Task struct
#[derive(Deserialize, Debug)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

// TaskDetail struct with description
#[derive(Deserialize, Debug)]
struct TaskDetail {
    id: u64,
    title: String,
    done: bool,
    due_date: Option<String>,
    labels: Option<Vec<Label>>,
    priority: Option<i32>,
    description: Option<String>,
}

// Label struct
#[derive(Deserialize, Debug)]
struct Label {
    id: u64,
    title: String,
}

// Fetch tasks from the API
async fn fetch_tasks(instance_url: &str, api_key: &str) -> Result<Vec<Task>, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}/api/v1/tasks/all", instance_url);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<Vec<Task>>()
        .await?;

    Ok(res)
}

// Fetch task details from the API
async fn fetch_task_detail(
    instance_url: &str,
    api_key: &str,
    task_id: u64,
) -> Result<TaskDetail, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/api/v1/tasks/{}", instance_url, task_id);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if res.status().is_success() {
        let task_detail = res.json::<TaskDetail>().await?;
        Ok(task_detail)
    } else {
        let error_text = res.text().await?;
        Err(format!("Error fetching task detail: {}", error_text).into())
    }
}

// Create a new task via the API
async fn create_new_task(
    instance_url: &str,
    api_key: &str,
    task_title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/api/v1/projects/1/tasks", instance_url);

    let mut map = std::collections::HashMap::new();
    map.insert("title", task_title);

    let res = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&map)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        let error_text = res.text().await?;
        Err(format!("Error creating task: {}", error_text).into())
    }
}

struct App {
    tasks: Vec<Task>,
    state: ListState,
    task_detail: Option<TaskDetail>,
    input_mode: InputMode,
    new_task_title: String,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    fn new(tasks: Vec<Task>) -> App {
        let mut state = ListState::default();
        if !tasks.is_empty() {
            state.select(Some(0));
        } else {
            state.select(None);
        }
        App {
            tasks,
            state,
            task_detail: None,
            input_mode: InputMode::Normal,
            new_task_title: String::new(),
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    async fn select_task(
        &mut self,
        instance_url: &str,
        api_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(selected) = self.state.selected() {
            let task = &self.tasks[selected];
            let task_detail = fetch_task_detail(instance_url, api_key, task.id).await?;
            self.task_detail = Some(task_detail);
        }
        Ok(())
    }
}

// Convert ANSI-formatted text into Lines for rendering
fn ansi_to_text(ansi_str: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for ansi_line in ansi_str.lines() {
        let mut spans = Vec::new();
        let parsed = ansi_line.ansi_parse();
        for item in parsed {
            match item {
                Output::TextBlock(text) => {
                    spans.push(Span::raw(text.to_string())); // Own the data
                }
                Output::Escape(_escape) => {
                    // Handle styling here if needed
                }
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// Main event loop and UI rendering
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    instance_url: &str,
    api_key: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            match app.input_mode {
                InputMode::Normal => {
                    // Render the main UI
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(size);

                    // Left panel: Task list
                    let tasks_widget = if !app.tasks.is_empty() {
                        let tasks: Vec<ListItem> = app
                            .tasks
                            .iter()
                            .map(|task| ListItem::new(Line::from(task.title.clone())))
                            .collect();

                        List::new(tasks)
                            .block(Block::default().borders(Borders::ALL).title("Tasks"))
                            .highlight_style(
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ")
                    } else {
                        List::new(vec![ListItem::new("No incomplete tasks available")])
                            .block(Block::default().borders(Borders::ALL).title("Tasks"))
                    };

                    f.render_stateful_widget(tasks_widget, chunks[0], &mut app.state);

                    // Right panel: Task details
                    let detail_block = Block::default().borders(Borders::ALL).title("Task Details");

                    if let Some(ref detail) = app.task_detail {
                        // Initialize lines with 'static lifetime
                        let mut lines: Vec<Line<'static>> = Vec::new();

                        // due_date
                        let due_date = match &detail.due_date {
                            Some(date) if date != "0001-01-01T00:00:00Z" => date.clone(),
                            _ => "No due date".to_string(),
                        };
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Due Date: ",
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(due_date),
                        ]));

                        // priority
                        let priority_str = match detail.priority {
                            Some(p) => p.to_string(),
                            None => "No priority".to_string(),
                        };
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Priority: ",
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(priority_str),
                        ]));

                        // labels
                        lines.push(Line::from(vec![Span::styled(
                            "Labels: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));

                        match &detail.labels {
                            Some(labels) if !labels.is_empty() => {
                                let mut label_spans: Vec<Span<'static>> = Vec::new();
                                for (i, label) in labels.iter().enumerate() {
                                    if i > 0 {
                                        // Add a space between labels
                                        label_spans.push(Span::raw(" ".to_string()));
                                    }
                                    label_spans.push(Span::styled(
                                        format!(" {} ", label.title),
                                        Style::default().bg(Color::Yellow).fg(Color::Black),
                                    ));
                                }
                                lines.push(Line::from(label_spans));
                            }
                            _ => {
                                lines.push(Line::from(Span::raw("No labels".to_string())));
                            }
                        }

                        // description
                        lines.push(Line::from(vec![Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));

                        if let Some(desc) = &detail.description {
                            if desc.trim() == "<p></p>" {
                                lines.push(Line::from(Span::raw("No description".to_string())));
                            } else {
                                // Convert HTML to ANSI-formatted text
                                let width = (chunks[1].width - 2) as usize; // Adjust for borders
                                let ansi_text = html2text::from_read(desc.as_bytes(), width);

                                // Convert ANSI text to Lines with owned data
                                let mut desc_lines = ansi_to_text(&ansi_text);
                                lines.append(&mut desc_lines);
                            }
                        } else {
                            lines.push(Line::from(Span::raw("No description".to_string())));
                        }

                        let paragraph = Paragraph::new(lines)
                            .block(detail_block)
                            .wrap(Wrap { trim: true });
                        f.render_widget(paragraph, chunks[1]);
                    } else {
                        let paragraph = Paragraph::new("Press Enter to view task details")
                            .block(detail_block)
                            .wrap(Wrap { trim: true });
                        f.render_widget(paragraph, chunks[1]);
                    }
                }
                InputMode::Editing => {
                    // Render the popover input box
                    let popup_block = Block::default()
                        .title("Enter New Task Title (Press Enter to Submit)")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Reset).fg(Color::Green));

                    let popup_area = centered_rect(60, 10, size);

                    let input = Paragraph::new(Text::from(app.new_task_title.as_str()))
                        .style(Style::default().fg(Color::White))
                        .block(popup_block);

                    f.render_widget(Clear, popup_area); // Clear the area first
                    f.render_widget(input, popup_area);
                }
            }
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => {
                        // Key handling in normal mode
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('j') => app.next(),
                            KeyCode::Char('k') => app.previous(),
                            KeyCode::Char('a') => {
                                app.input_mode = InputMode::Editing;
                                app.new_task_title.clear();
                            }
                            KeyCode::Enter => {
                                // Fetch task details
                                if let Err(err) = app.select_task(instance_url, api_key).await {
                                    eprintln!("Error fetching task details: {}", err);
                                }
                            }
                            _ => {}
                        }
                    }
                    InputMode::Editing => {
                        // Key handling in editing mode
                        match key.code {
                            KeyCode::Enter => {
                                // Send the PUT request to create the new task
                                if !app.new_task_title.trim().is_empty() {
                                    if let Err(err) = create_new_task(
                                        &instance_url,
                                        &api_key,
                                        &app.new_task_title,
                                    )
                                    .await
                                    {
                                        eprintln!("Error creating task: {}", err);
                                    } else {
                                        // Refresh the task list
                                        if let Ok(all_tasks) =
                                            fetch_tasks(&instance_url, &api_key).await
                                        {
                                            app.tasks = all_tasks
                                                .into_iter()
                                                .filter(|task| !task.done)
                                                .collect();
                                            app.state.select(Some(0));
                                        }
                                    }
                                }
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                app.new_task_title.push(c);
                            }
                            KeyCode::Backspace => {
                                app.new_task_title.pop();
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Read API key and instance URL from environment variables
    let instance_url = env::var("INSTANCE_URL").expect("INSTANCE_URL not set");
    let api_key = env::var("API_KEY").expect("API_KEY not set");

    // Fetch all tasks from the API
    let all_tasks = fetch_tasks(&instance_url, &api_key).await?;

    // Filter out completed tasks
    let incomplete_tasks: Vec<Task> = all_tasks.into_iter().filter(|task| !task.done).collect();

    // Setup terminal UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?; // Hide the cursor

    let app = App::new(incomplete_tasks); // Initialize app with incomplete tasks

    let res = run_app(&mut terminal, app, &instance_url, &api_key).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
