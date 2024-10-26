use crate::app::{App, InputMode};
use crate::models::{Label, Task};
use ansi_parser::{AnsiParser, Output};
use crossterm::event::{self, Event as CEvent, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

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
            Span::styled(" Enter ", Style::default().fg(Color::Red)),
            Span::raw(": View Details "),
            Span::styled(" a ", Style::default().fg(Color::Red)),
            Span::raw(": Add Task "),
        ])),
        InputMode::Editing => Text::from(Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Color::Red)),
            Span::raw(": Submit "),
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw(": Cancel "),
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
                .constraints(
                    [
                        Constraint::Min(0),    // Body takes up the remaining space
                        Constraint::Length(2), // Footer is 3 rows high
                    ]
                    .as_ref(),
                )
                .split(size);

            let body_chunk = chunks[0];
            let footer_chunk = chunks[1];

            match app.input_mode {
                InputMode::Normal => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(body_chunk);

                    // Left panel: Task list
                    let tasks_widget = if !app.tasks.is_empty() {
                        let tasks: Vec<ListItem> = app
                            .tasks
                            .iter()
                            .map(|task| {
                                // Check if the task is done
                                let content = if task.done {
                                    // Prepend "DONE" for completed tasks
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
                            .block(Block::default().borders(Borders::ALL).title("Tasks"))
                            .highlight_style(
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ")
                    } else {
                        List::new(vec![ListItem::new("No tasks available")])
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
                }
                InputMode::Editing => {
                    let popup_area = centered_rect(60, 10, body_chunk);

                    let popup_block = Block::default()
                        .title("Enter New Task Title (Press Enter to Submit)")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Green));

                    let input = Paragraph::new(Text::from(app.new_task_title.as_str()))
                        .style(Style::default().fg(Color::White))
                        .block(popup_block);

                    f.render_widget(Clear, popup_area);
                    f.render_widget(input, popup_area);
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
