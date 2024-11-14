use crate::app::{ActiveInput, App, InputMode};
use ansi_parser::{AnsiParser, Output};
use crossterm::event::{self, Event as CEvent};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

fn get_cursor_position(input: &str, area: Rect) -> (u16, u16) {
    let lines: Vec<&str> = input.lines().collect();
    let last_line = lines.last().unwrap_or(&"");
    let x = area.x + last_line.chars().count() as u16 + 1;
    let y = area.y + lines.len() as u16 - 1 + 1;
    (x, y)
}

fn centered_rect_absolute(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2u16),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height) + 1u16) / 2u16),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2u16),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width) + 1u16) / 2u16),
        ])
        .split(popup_layout[1])[1]
}

pub fn ansi_to_text(ansi_str: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for ansi_line in ansi_str.lines() {
        let mut spans = Vec::new();
        let parsed = ansi_line.ansi_parse();
        for item in parsed {
            match item {
                Output::TextBlock(text) => {
                    spans.push(Span::raw(text.to_string()));
                }
                Output::Escape(_escape) => {}
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn get_legend(input_mode: &InputMode) -> Text<'static> {
    match input_mode {
        InputMode::Normal => Text::from(Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Red)),
            Span::raw(": Quit "),
            Span::styled(" j ", Style::default().fg(Color::Red)),
            Span::raw(": Down "),
            Span::styled(" k ", Style::default().fg(Color::Red)),
            Span::raw(": Up "),
            Span::styled(" n ", Style::default().fg(Color::Red)),
            Span::raw(": Next Page "),
            Span::styled(" p ", Style::default().fg(Color::Red)),
            Span::raw(": Previous Page "),
            Span::styled(" t ", Style::default().fg(Color::Red)),
            Span::raw(": Toggle Done "),
            Span::styled(" Enter ", Style::default().fg(Color::Red)),
            Span::raw(": View Details "),
            Span::styled(" a ", Style::default().fg(Color::Red)),
            Span::raw(": Add Task "),
        ])),
        InputMode::Editing => Text::from(Line::from(vec![
            Span::styled(" i ", Style::default().fg(Color::Red)),
            Span::raw(": Insert "),
            Span::styled(" Tab ", Style::default().fg(Color::Red)),
            Span::raw(": Switch Input "),
            Span::styled(" Enter ", Style::default().fg(Color::Red)),
            Span::raw(": Submit "),
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw(": Cancel "),
        ])),
        InputMode::Insert => Text::from(Line::from(vec![
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw(": Exit Insert Mode "),
        ])),
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    instance_url: &str,
    api_key: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Split the main layout into body and footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Min(0), Constraint::Length(2u16)])
                .split(size);

            let body_chunk = chunks[0];
            let footer_chunk = chunks[1];

            match app.input_mode {
                InputMode::Normal => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                        .split(body_chunk);

                    let task_title = if app.show_done_tasks {
                        "Tasks (All)"
                    } else {
                        "Tasks (Undone)"
                    };

                    // Left panel: Task list
                    let tasks_widget = if !app.tasks.is_empty() {
                        let tasks: Vec<ListItem> = app
                            .tasks
                            .iter()
                            .map(|task| {
                                let content = if task.done {
                                    vec![
                                        Span::styled("DONE ", Style::default().fg(Color::Green)),
                                        Span::raw(&task.title),
                                    ]
                                } else {
                                    vec![Span::raw(&task.title)]
                                };
                                ListItem::new(Line::from(content))
                            })
                            .collect();

                        List::new(tasks)
                            .block(Block::default().borders(Borders::ALL).title(task_title))
                            .highlight_style(
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ")
                    } else {
                        List::new(vec![ListItem::new("No tasks available")])
                            .block(Block::default().borders(Borders::ALL).title(task_title))
                    };

                    f.render_stateful_widget(tasks_widget, chunks[0], &mut app.state);

                    // Right panel: Task details
                    let detail_block = Block::default().borders(Borders::ALL).title("Task Details");

                    if let Some(ref detail) = app.task_detail {
                        let mut lines: Vec<Line<'static>> = Vec::new();

                        // Due date
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

                        // Priority
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

                        // Labels
                        lines.push(Line::from(vec![Span::styled(
                            "Labels: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));

                        match &detail.labels {
                            Some(labels) if !labels.is_empty() => {
                                let mut label_spans: Vec<Span<'static>> = Vec::new();
                                for (i, label) in labels.iter().enumerate() {
                                    if i > 0 {
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
                                lines.push(Line::from(Span::raw("No labels".to_string())));
                            }
                        }

                        // Description
                        lines.push(Line::from(vec![Span::styled(
                            "Description: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        )]));

                        if let Some(desc) = &detail.description {
                            if desc.trim() == "<p></p>" {
                                lines.push(Line::from(Span::raw("No description".to_string())));
                            } else {
                                let width = (chunks[1].width - 2) as usize; // Adjust for borders
                                let ansi_text = html2text::from_read(desc.as_bytes(), width);

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
                    // Render the error message if there is one
                    if let Some(ref msg) = app.error_message {
                        // Create a centered rectangle for the popup
                        let error_area = centered_rect_absolute(60, 5, size);
                        let error_block = Block::default()
                            .borders(Borders::ALL)
                            .title("Error")
                            .style(Style::default().fg(Color::Red));

                        let error_paragraph = Paragraph::new(msg.as_str())
                            .style(Style::default().fg(Color::White))
                            .block(error_block)
                            .alignment(Alignment::Center)
                            .wrap(Wrap { trim: true });

                        // Clear the area and render the popup
                        f.render_widget(Clear, error_area); // Clear the area first
                        f.render_widget(error_paragraph, error_area);
                    }
                }
                InputMode::Editing | InputMode::Insert => {
                    let popup_width_percentage = 60u16;
                    let popup_width =
                        (size.width * popup_width_percentage / 100u16).saturating_sub(2u16);

                    // Calculate the required heights for the input boxes
                    let title_lines_required =
                        calculate_wrapped_lines(&app.new_task_title, popup_width);
                    let description_lines_required =
                        calculate_wrapped_lines(&app.new_task_description, popup_width);

                    let title_height = std::cmp::max(title_lines_required as u16, 1u16);
                    let description_height = std::cmp::max(description_lines_required as u16, 2u16); // At least 2 lines tall

                    let total_height = title_height + description_height + 6u16; // +6 for borders and titles

                    let max_popup_height = size.height - 2u16;
                    let popup_height = std::cmp::min(total_height, max_popup_height);

                    let popup_area =
                        centered_rect_absolute(popup_width + 2u16, popup_height, body_chunk);

                    let popup_block = Block::default()
                        .title("Enter New Task (Press Enter to Submit, Tab to Switch)")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Green));

                    // Split the popup area vertically for the two input boxes
                    let input_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(title_height + 2u16), // +2 for borders
                            Constraint::Length(description_height + 2u16), // +2 for borders
                        ])
                        .split(popup_area.inner(Margin {
                            vertical: 1u16,
                            horizontal: 1u16,
                        })); // Adjust for popup_block borders

                    // Title input box
                    let title_block = Block::default().borders(Borders::ALL).title("Title").style(
                        if app.active_input == ActiveInput::Title {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        },
                    );

                    let title_paragraph = Paragraph::new(app.new_task_title.as_str())
                        .style(Style::default().fg(Color::White))
                        .block(title_block)
                        .wrap(Wrap { trim: false });

                    // Description input box
                    let description_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Description")
                        .style(if app.active_input == ActiveInput::Description {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        });

                    let description_paragraph = Paragraph::new(app.new_task_description.as_str())
                        .style(Style::default().fg(Color::White))
                        .block(description_block)
                        .wrap(Wrap { trim: false });

                    f.render_widget(Clear, popup_area);
                    f.render_widget(popup_block, popup_area);

                    f.render_widget(title_paragraph, input_chunks[0]);
                    f.render_widget(description_paragraph, input_chunks[1]);

                    // Set cursor position
                    match app.active_input {
                        ActiveInput::Title => {
                            // Calculate cursor position in title input
                            let cursor_position =
                                get_cursor_position(&app.new_task_title, input_chunks[0]);
                            f.set_cursor(cursor_position.0, cursor_position.1);
                        }
                        ActiveInput::Description => {
                            // Calculate cursor position in description input
                            let cursor_position =
                                get_cursor_position(&app.new_task_description, input_chunks[1]);
                            f.set_cursor(cursor_position.0, cursor_position.1);
                        }
                    }
                }
            }

            // Render the legend in the footer
            let legend = Paragraph::new(get_legend(&app.input_mode))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            f.render_widget(legend, footer_chunk);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                let should_quit = app.handle_input(key, instance_url, api_key).await?;
                if should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn calculate_wrapped_lines(text: &str, max_width: u16) -> usize {
    let mut line_count = 0;
    for line in text.lines() {
        let line_width = line.chars().count() as u16;
        let total_width = line_width + max_width - 1u16;
        line_count += (total_width / max_width) as usize;
    }
    line_count
}
