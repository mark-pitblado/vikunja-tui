use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::dotenv;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::io;
use tokio::time::Duration;

use html2text::from_read;
use std::borrow::Cow;

// Existing Task struct
#[derive(Deserialize, Debug)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

// New TaskDetail struct
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

struct App {
    tasks: Vec<Task>,
    state: ListState,
    task_detail: Option<TaskDetail>,
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

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    instance_url: &str,
    api_key: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Split the layout into two columns
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);

            // Left panel: Task list
            let tasks_widget = if !app.tasks.is_empty() {
                let tasks: Vec<ListItem> = app
                    .tasks
                    .iter()
                    .map(|task| ListItem::new(task.title.clone()))
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
                let mut lines = Vec::new();

                // due_date
                let due_date = match &detail.due_date {
                    Some(date) if date != "0001-01-01T00:00:00Z" => date.clone(),
                    _ => "No due date".to_string(),
                };
                lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(due_date),
                ]));

                // priority
                let priority_str = match detail.priority {
                    Some(p) => p.to_string(),
                    None => "No priority".to_string(),
                };
                lines.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(priority_str),
                ]));

                // labels
                lines.push(Line::from(vec![Span::styled(
                    "Labels: ",
                    Style::default().add_modifier(Modifier::BOLD),
                )]));

                match &detail.labels {
                    Some(labels) if !labels.is_empty() => {
                        let mut label_spans: Vec<Span> = Vec::new();
                        for (i, label) in labels.iter().enumerate() {
                            if i > 0 {
                                // Add a space between labels
                                label_spans.push(Span::raw(" "));
                            }
                            label_spans.push(Span::styled(
                                format!(" {} ", label.title),
                                Style::default().bg(Color::Yellow).fg(Color::Black),
                            ));
                        }
                        lines.push(Line::from(label_spans));
                    }
                    _ => {
                        lines.push(Line::from(Span::raw("No labels")));
                    }
                }

                // description
                lines.push(Line::from(vec![Span::styled(
                    "Description: ",
                    Style::default().add_modifier(Modifier::BOLD),
                )]));

                if let Some(desc) = &detail.description {
                    if desc.trim() == "<p></p>" {
                        lines.push(Line::from(Span::raw("No description")));
                    } else {
                        // Convert HTML to ANSI-formatted text
                        let width = (chunks[1].width - 2) as usize; // Adjust for padding/borders
                        let ansi_text = html2text::from_read(desc.as_bytes(), width);

                        // Split the ANSI text into lines
                        for line in ansi_text.lines() {
                            // Create a Span with raw text (ANSI escape codes will render correctly)
                            lines.push(Line::from(Span::raw(line.to_string())));
                        }
                    }
                } else {
                    lines.push(Line::from(Span::raw("No description")));
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
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => app.next(),
                    KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => {
                        // Fetch task details
                        if let Err(err) = app.select_task(instance_url, api_key).await {
                            eprintln!("Error fetching task details: {}", err);
                        }
                    }
                    _ => {}
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
        println!("{:?}", err)
    }

    Ok(())
}
