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
    pub active_input: ActiveInput,
    pub new_task_title: String,
    pub new_task_description: String,
    pub page: usize,
    pub show_done_tasks: bool,
    pub error_message: Option<String>,
}

pub enum InputMode {
    Normal,
    Editing,
    Insert,
}
#[derive(PartialEq)]
pub enum ActiveInput {
    Title,
    Description,
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
            active_input: ActiveInput::Title,
            new_task_title: String::new(),
            new_task_description: String::new(),
            page: 1,
            show_done_tasks: false,
            error_message: None,
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
                    self.new_task_description.clear();
                    self.active_input = ActiveInput::Title;
                }
                KeyCode::Enter => {
                    if let Err(err) = self.select_task(instance_url, api_key).await {
                        eprintln!("Error fetching task details: {}", err);
                    }
                }
                _ => {}
            },

            InputMode::Editing => match key.code {
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Insert;
                }
                KeyCode::Tab => {
                    self.active_input = match self.active_input {
                        ActiveInput::Title => ActiveInput::Description,
                        ActiveInput::Description => ActiveInput::Title,
                    };
                }
                KeyCode::Enter => {
                    if self.new_task_title.trim().is_empty() {
                        self.error_message = Some("Task title cannot be empty.".to_string());
                    } else {
                        match parse_task_input(&self.new_task_title) {
                            Ok(parsed_task) => {
                                let description = if self.new_task_description.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.new_task_description.as_str())
                                };

                                if let Err(err) = create_new_task(
                                    instance_url,
                                    api_key,
                                    &parsed_task.title,
                                    description,
                                    parsed_task.priority,
                                    parsed_task.due_date,
                                )
                                .await
                                {
                                    self.error_message =
                                        Some(format!("Error creating new task: {}", err));
                                } else if let Err(err) =
                                    self.refresh_tasks(instance_url, api_key).await
                                {
                                    self.error_message =
                                        Some(format!("Error fetching tasks: {}", err));
                                } else {
                                    // Clear input and return to normal mode
                                    self.new_task_title.clear();
                                    self.new_task_description.clear();
                                    self.input_mode = InputMode::Normal;
                                }
                            }
                            Err(parse_error) => {
                                self.error_message = Some(format!("Input Error: {}", parse_error));
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    self.new_task_title.clear();
                    self.new_task_description.clear();
                    self.input_mode = InputMode::Normal;
                }
                _ => {}
            },
            InputMode::Insert => match key.code {
                KeyCode::Char(c) => match self.active_input {
                    ActiveInput::Title => self.new_task_title.push(c),
                    ActiveInput::Description => self.new_task_description.push(c),
                },
                KeyCode::Backspace => match self.active_input {
                    ActiveInput::Title => {
                        self.new_task_title.pop();
                    }
                    ActiveInput::Description => {
                        self.new_task_description.pop();
                    }
                },
                KeyCode::Esc => {
                    self.input_mode = InputMode::Editing;
                }
                _ => {}
            },
        }
        Ok(false)
    }
}
