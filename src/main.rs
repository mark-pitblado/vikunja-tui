mod api;
mod app;
mod models;
mod ui;

use crate::api::fetch_tasks;

use app::App;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::config_dir;
use ratatui::{backend::CrosstermBackend, Terminal};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::PathBuf;
use ui::run_app;

#[derive(Deserialize)]
struct VikunjaConfig {
    instance_url: String,
    api_key: String,
}

#[derive(Deserialize)]
struct Config {
    vikunja: VikunjaConfig,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config_path: PathBuf = config_dir().expect("Could not determine config directory");
    config_path.push("vikunja-tui/config.toml");

    // Read the config file
    let config_content = fs::read_to_string(config_path)?;

    // Parse the TOML content
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config().expect("Failed to load config file");

    let instance_url = config.vikunja.instance_url;
    let api_key = config.vikunja.api_key;

    let show_done_tasks = false;

    let tasks = fetch_tasks(&instance_url, &api_key, 1).await?;
    let tasks = if show_done_tasks {
        tasks
    } else {
        tasks.into_iter().filter(|task| !task.done).collect()
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let app = App::new(tasks);

    let res = run_app(&mut terminal, app, &instance_url, &api_key).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
