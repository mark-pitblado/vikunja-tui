mod api;
mod app;
mod models;
mod ui;

use app::App;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::dotenv;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use ui::run_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Read API key and instance URL from environment variables
    let instance_url = env::var("INSTANCE_URL").expect("INSTANCE_URL not set");
    let api_key = env::var("API_KEY").expect("API_KEY not set");

    // Fetch all tasks from the API
    let all_tasks = api::fetch_tasks(&instance_url, &api_key).await?;

    // Filter out completed tasks
    let incomplete_tasks: Vec<models::Task> =
        all_tasks.into_iter().filter(|task| !task.done).collect();

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
