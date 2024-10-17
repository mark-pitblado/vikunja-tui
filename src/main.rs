use reqwest::Client;
use serde::Deserialize;
use std::io;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
};
use tokio::time::{Duration};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Modifier, Style},
    Terminal,
};
use dotenv::dotenv;
use std::env;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;


// Structs to hold the task data.
#[derive(Deserialize, Debug)]
struct Task {
    id: u64,
    title: String,
    done: bool,
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

struct App {
    tasks: Vec<Task>,
    state: ListState,
}

impl App {
    fn new(tasks: Vec<Task>) -> App {
        let mut state = ListState::default();
        if !tasks.is_empty() {
            state.select(Some(0));
        } else {
            state.select(None);
        }
        App { tasks, state }
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
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

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
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => app.next(),
                    KeyCode::Char('k') => app.previous(),
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

    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

