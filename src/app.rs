use crate::api::{create_new_task, fetch_task_detail, fetch_tasks};
use crate::models::{Task, TaskDetail};
use crate::parser::parse_task_input;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use std::io;

pub struct App {
    pub tasks: Vec<Task>,
    pub state: ListState,
    pub task_detail: Option<TaskDetail>,
    pub input_mode: InputMode,
    pub new_task_title: String,
    pub page: usize,
    pub show_done_tasks: bool,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl App {
    pub fn new(tasks: Vec<Task>) -> App {
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
            page: 1,
            show_done_tasks: false,
        }
    }

    pub async fn refresh_tasks(
        &mut self,
        instance_url: &str,
        api_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let new_tasks = fetch_tasks(instance_url, api_key, self.page).await?;
        if self.show_done_tasks {
            self.tasks = new_tasks;
        } else {
            self.tasks = new_tasks.into_iter().filter(|task| !task.done).collect();
        }
        self.state.select(Some(0));
        Ok(())
    }

    pub fn next_page(&mut self) {
        self.page += 1;
    }

    pub fn previous_page(&mut self) {
        if self.page > 1 {
            self.page -= 1;
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub async fn select_task(
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

    pub async fn handle_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        instance_url: &str,
        api_key: &str,
    ) -> io::Result<bool> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('j') => self.next(),
                KeyCode::Char('k') => self.previous(),
                KeyCode::Char('n') => {
                    // Next page
                    self.next_page();
                    if let Err(err) = self.refresh_tasks(instance_url, api_key).await {
                        eprintln!("Error fetching tasks: {}", err);
                    }
                }
                KeyCode::Char('p') => {
                    // Previous page
                    self.previous_page();
                    if let Err(err) = self.refresh_tasks(instance_url, api_key).await {
                        eprintln!("Error fetching tasks: {}", err);
                    }
                }
                KeyCode::Char('t') => {
                    self.show_done_tasks = !self.show_done_tasks;
                    if let Err(err) = self.refresh_tasks(instance_url, api_key).await {
                        eprintln!("Error fetching tasks: {}", err);
                    }
                }
                KeyCode::Char('a') => {
                    self.input_mode = InputMode::Editing;
                    self.new_task_title.clear();
                }
                KeyCode::Enter => {
                    if let Err(err) = self.select_task(instance_url, api_key).await {
                        eprintln!("Error fetching task details: {}", err);
                    }
                }
                _ => {}
            },
            InputMode::Editing => {
                match key.code {
                    KeyCode::Enter => {
                        if !self.new_task_title.trim().is_empty() {
                            // Use the parser to extract the task title, priority, and label titles
                            let parsed_task = parse_task_input(&self.new_task_title);

                            // Create the new task with the parsed title, priority, and labels
                            if let Err(err) = create_new_task(
                                instance_url,
                                api_key,
                                &parsed_task.title,
                                parsed_task.priority,
                            )
                            .await
                            {
                                eprintln!("Error creating task: {}", err);
                            } else {
                                // Refresh the task list
                                if let Ok(all_tasks) =
                                    fetch_tasks(instance_url, api_key, self.page).await
                                {
                                    self.tasks =
                                        all_tasks.into_iter().filter(|task| !task.done).collect();
                                    self.state.select(Some(0));
                                }
                            }
                        }
                        self.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        self.new_task_title.push(c); // Handle character input
                    }
                    KeyCode::Backspace => {
                        self.new_task_title.pop(); // Handle backspace
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal; // Exit editing mode
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }
}
