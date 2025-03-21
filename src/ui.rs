use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui::layout::Alignment;
use ratatui::widgets::{Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};
use crate::app::{App, CurrentScreen};
use crate::job::JobStatus;
use crate::task::Task;

// Centering the rectangle helped function

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    // Cut the middle piece into three horizontal pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // return the middle piece

}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
        )
        .split(frame.area());

    // Title block
    let title_block = Block::default().borders(Borders::ALL).style(Style::default());
    let title = Paragraph::new(Text::styled(
        format!("Simulation Screen | Processors: {} | Tasks: {} | Jobs: {}",
                app.get_amount_of_processor_units(),
                app.get_amount_of_tasks(),
                app.get_amount_of_jobs()),
        Style::default().fg(Color::Green),
    )).block(title_block);

    frame.render_widget(title, chunks[0]);

    // Render appropriate screen based on current state
    match app.current_screen {
        CurrentScreen::SimulatorScreen => {
            draw_simulator_screen(frame, app, chunks[1]);
        }
        CurrentScreen::AnalysisScreen => {
            draw_analysis_screen(frame, app, chunks[1]);
        }
        CurrentScreen::PropertiesScreen => {
            draw_properties_screen(frame, app, chunks[1]);
        }
        CurrentScreen::ExitScreen => {
            // Draw exit confirmation dialog
            let popup_area = centered_rect(60, 20, chunks[1]);
            let exit_block = Block::default()
                .borders(Borders::ALL)
                .title("Exit Confirmation")
                .style(Style::default());

            let exit_text = Paragraph::new("Do you want to exit? (y/n)")
                .block(exit_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(Clear, popup_area);
            frame.render_widget(exit_text, popup_area);
        }
    }

    // Footer with key bindings
    let footer = match app.current_screen {
        CurrentScreen::SimulatorScreen => {
            if app.jobs_generated {
                "Enter: Start | r: Reset | c: Edit Cycles | s: Sleep | a: Analysis | p: Properties | q: Quit"
            } else {
                "g: Generate Jobs | r: Reset | c: Edit Cycles | a: Analysis | p: Properties | q: Quit"
            }
        },
        CurrentScreen::AnalysisScreen => "s: Simulate | p: Properties | q: Quit",
        CurrentScreen::PropertiesScreen => "s: Simulate | a: Analysis | q: Quit",
        CurrentScreen::ExitScreen => "y: Yes | n: No",
    };

    let footer_widget = Paragraph::new(Text::styled(
        footer,
        Style::default().fg(Color::White),
    )).block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer_widget, chunks[2]);
}
fn draw_simulator_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split the area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Compact task list
            Constraint::Length(3),  // Sleep time control
            Constraint::Length(3),  // Cycle count input
            Constraint::Min(1),     // Job progress section
        ])
        .split(area);



    // Check if tasks are imported
    if let Some(simulator_vars) = &app.simulator_variables {

        if let Some(tasks) = &simulator_vars.list_of_tasks {
            if !tasks.is_empty() {
                // Draw compact task list at the top
                draw_compact_task_list(
                    frame,
                    tasks,
                    chunks[0],
                    app.task_list_scroll,
                    &mut app.task_list_scrollbar
                );

                // Draw sleep time control
                draw_sleep_control(frame, app, chunks[1]);

                // Draw cycle count input
                draw_cycle_input(frame, app, chunks[2]);

                // Draw job progress section or generation message
                if app.jobs_generated {
                    // Draw job progress bars
                    draw_job_progress(frame, app, chunks[3]);

                } else {
                    // Show message to generate jobs
                    let generate_jobs_text = Paragraph::new(
                        "Set number of cycles above and press 'g' to generate jobs before starting the simulation."
                    )
                        .block(Block::default().borders(Borders::ALL).title("Jobs"))
                        .style(Style::default().fg(Color::Yellow));

                    frame.render_widget(generate_jobs_text, chunks[3]);
                }
                return;
            }
        }
    }

    // No tasks imported yet - show instruction message
    let no_tasks_text = Paragraph::new(
        "No tasks imported yet. Go to Properties screen (p) to import tasks."
    )
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(no_tasks_text, area);
}

fn draw_job_progress(frame: &mut Frame, app: &App, area: Rect) {
    // If there are no tasks or simulator isn't initialized
    if !app.jobs_generated || app.simulator_variables.is_none() {
        let no_jobs_text = Paragraph::new("No jobs available. Press 'g' to generate jobs.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(no_jobs_text, area);
        return;
    }

    // Check for processor with job history
    let job_history_exists = app.simulator_variables.as_ref()
        .and_then(|sim| sim.processor.as_ref())
        .and_then(|proc| proc.job_history.as_ref())
        .is_some();

    if let Some(simulator) = &app.simulator_variables {
        if let Some(processor) = &simulator.processor {
            if let Some(history) = &processor.job_history {
                // Verify job count matches expected count when simulation is complete
                if !app.simulation_running {
                    let expected_job_count = if let Some(setup) = &app.setup_values {
                        setup.number_of_tasks * app.simulator_variables
                            .as_ref()
                            .map(|sim| sim.number_of_tasks_per_cycle)
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    // Panic if job count doesn't match expected count
                    if history.done_jobs.len() != expected_job_count && expected_job_count > 0 {
                        panic!("Job count mismatch: found {} jobs in history but expected {} (tasks × cycles)",
                               history.done_jobs.len(), expected_job_count);
                    }
                }
                Some(&history.done_jobs)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };


    if job_history_exists {
        // Get the job history data
        let history_data = if let Some(simulator) = &app.simulator_variables {
            if let Some(processor) = &simulator.processor {
                if let Some(history) = &processor.job_history {
                    Some(&history.done_jobs)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Display job history if available
        if let Some(jobs) = history_data {
            let mut job_rows = vec![
                Row::new(vec!["ID", "Status", "Execution Time", "Instance"]).style(Style::default().fg(Color::Yellow))
            ];

            for job in jobs {
                let status_str = match job.computation.status {
                    JobStatus::Completed => "Completed",
                    JobStatus::Missed => "Missed",
                    JobStatus::InProgress => "In Progress",
                    JobStatus::NotStarted => "Not Started",
                };

                let status_style = match job.computation.status {
                    JobStatus::Completed => Style::default().fg(Color::Green),
                    JobStatus::Missed => Style::default().fg(Color::Red),
                    JobStatus::InProgress => Style::default().fg(Color::Yellow),
                    JobStatus::NotStarted => Style::default().fg(Color::Gray),
                };

                job_rows.push(
                    Row::new(vec![
                        job.name.to_string(),
                        status_str.to_string(),
                        job.computation.execution_time.to_string(),
                        job.instance.expect("REASON").to_string(),
                    ]).style(status_style)
                );
            }

            let job_table = Table::new(
                job_rows,
                // Define column widths - adjust these based on your data
                [
                    Constraint::Percentage(15), // Job ID
                    Constraint::Percentage(25), // Release Time
                    Constraint::Percentage(25), // Deadline
                    Constraint::Percentage(35), // Status
                ]
            );

            frame.render_widget(job_table, area);
        } else {
            // No job history yet
            let text = if app.simulation_running {
                "Simulation in progress..."
            } else {
                "No job history. Run simulation to see job progress."
            };

            let job_text = Paragraph::new(text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(job_text, area);
        }
    } else {
        // Display all jobs from tasks with their status
        if let Some(simulator) = &app.simulator_variables {
            if let Some(tasks) = &simulator.list_of_tasks {
                let mut all_jobs = Vec::new();

                for task in tasks {
                    if let Some(jobs) = &task.jobs {
                        for job in jobs {
                            all_jobs.push((task.id, job));
                        }
                    }
                }

                if !all_jobs.is_empty() {
                    let mut job_rows = vec![
                        Row::new(vec!["Task", "Job", "Status", "Time", "Release Time"]).style(Style::default().fg(Color::Yellow))
                    ];

                    for (task_id, job) in all_jobs {
                        let status_str = match job.computation.status {
                            JobStatus::Completed => "Completed",
                            JobStatus::Missed => "Missed",
                            JobStatus::InProgress => "In Progress",
                            JobStatus::NotStarted => "Not Started",
                        };

                        let status_style = match job.computation.status {
                            JobStatus::Completed => Style::default().fg(Color::Green),
                            JobStatus::Missed => Style::default().fg(Color::Red),
                            JobStatus::InProgress => Style::default().fg(Color::Yellow),
                            JobStatus::NotStarted => Style::default().fg(Color::Gray),
                        };

                        if job.computation.execution_time > 0 {
                            (job.computation.completed_compuation_time as f64 / job.computation.execution_time as f64 * 100.0).round() as usize
                        } else {
                            0
                        };

                        job_rows.push(
                            Row::new(vec![
                                format!("T{}", task_id),
                                format!("J{}", job.id),
                                status_str.to_string(),
                                format!("{}/{}", job.computation.completed_compuation_time, job.computation.execution_time),
                                format!("{:?}", job.release_time),
                            ]).style(status_style)
                        );
                    }

                    let job_table = Table::new(
                        job_rows,
                        // Define column widths - adjust these based on your data
                        [
                            Constraint::Percentage(15), // Job ID
                            Constraint::Percentage(25), // Release Time
                            Constraint::Percentage(25), // Deadline
                            Constraint::Percentage(35), // Status
                            Constraint::Percentage(35), // Progress
                        ]
                    );

                    frame.render_widget(job_table, area);
                    return;
                }
            }
        }

        // Fallback message
        let no_jobs_text = if app.simulation_running {
            "Simulation in progress..."
        } else {
            "No job history available. Run simulation first."
        };

        frame.render_widget(
            Paragraph::new(no_jobs_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center),
            area
        );
    }
}

// Add function to draw cycle count input
fn draw_cycle_input(frame: &mut Frame, app: &App, area: Rect) {
    let binding = String::from("2"); // Default value
    let input = app.cycle_input.as_ref().unwrap_or(&binding);

    let cycle_block = Block::default()
        .borders(Borders::ALL)
        .title("Number of Jobs per Task")
        .border_style(Style::default().fg(
            if app.editing_cycles { Color::Yellow } else { Color::White }
        ));

    let cycle_text = format!("Cycles: {} (c to edit)", input);
    let cycle_input = Paragraph::new(cycle_text)
        .block(cycle_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(cycle_input, area);

    // Show cursor when editing cycle count
    if app.editing_cycles {
        // Position cursor after the current value
        let value_str = input;
        let text_prefix = "Cycles: ";
        frame.set_cursor_position((
            area.left() + text_prefix.len() as u16 + value_str.len() as u16,
            area.top() + 1
        ));
    }
}

// Display compact task list at the top
pub(crate) fn draw_compact_task_list(
    frame: &mut Frame,
    tasks: &[Task],
    area: Rect,
    scroll_index: usize, // Add parameter for tracking scroll position
    scrollbar_state: &mut ScrollbarState, // Add parameter for scrollbar state
) {
    // Create a block for the task list with a border and title
    let block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL);

    // Calculate available height for tasks
    let inner_area = block.inner(area);
    let available_height = inner_area.height as usize;

    // Update scrollbar state with content length
    *scrollbar_state = scrollbar_state.content_length(tasks.len());

    // Determine visible range based on scroll position
    let start_idx = scroll_index.min(tasks.len().saturating_sub(1));
    let end_idx = (start_idx + available_height).min(tasks.len());
    let visible_tasks = &tasks[start_idx..end_idx];

    // Create text lines for visible tasks
    let task_lines: Vec<Line> = visible_tasks
        .iter()
        .map(|task| {
            Line::from(vec![
                Span::styled(format!("T{}: ", task.id), Style::default().fg(Color::Cyan)),
                Span::raw(format!("WCET={} BCET={} Period={} Deadline={}",
                                  task.worst_case_execution_time,
                                  task.best_case_execution_time,
                                  task.period,
                                  task.relative_deadline,
                )),
            ])
        })
        .collect();

    // Render tasks paragraph
    let tasks_paragraph = Paragraph::new(task_lines)
        .block(block);

    frame.render_widget(tasks_paragraph, area);

    // Render scrollbar if needed
    if tasks.len() > available_height {
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            inner_area,
            scrollbar_state,
        );
    }
}

// Display sleep time control with edit functionality
fn draw_sleep_control(frame: &mut Frame, app: &App, area: Rect) {
    let sleep_block = Block::default()
        .borders(Borders::ALL)
        .title("Simulation Speed")
        .border_style(Style::default().fg(
            if app.editing_sleep { Color::Yellow } else { Color::White }
        ));

    let sleep_text = format!("Sleep time: {}ms (ESC/TAB to toggle edit mode)", app.simulation_sleep);
    let sleep_control = Paragraph::new(sleep_text)
        .block(sleep_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(sleep_control, area);

    // Show cursor when editing sleep time
    if app.editing_sleep {
        // Position cursor after the current value
        let value_str = app.simulation_sleep.to_string();
        let sleep_text_prefix = "Sleep time: ";
        frame.set_cursor_position(
            (area.left() + sleep_text_prefix.len() as u16 + value_str.len() as u16,
             area.top() + 1)
        );
    }
}


// Helper function to show task summary
fn draw_task_summary(frame: &mut Frame, tasks: &[Task], area: Rect) {
    let items: Vec<ListItem> = tasks.iter()
        .map(|task| {
            ListItem::new(format!(
                "Task {}: WCET={}, BCET={}, Period={}, Deadline={}",
                task.id,
                task.worst_case_execution_time,
                task.best_case_execution_time,
                task.period,
                task.relative_deadline
            ))
        })
        .collect();

    let tasks_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Imported Tasks"))
        .style(Style::default())
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(tasks_list, area);
}


fn draw_properties_screen(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Input field
            Constraint::Length(3),  // Status message
            Constraint::Min(1),    // Instructions
        ])
        .split(area);

    // Create input field with highlighted border when editing
    let binding = String::new();
    let input = app.input.as_ref().unwrap_or(&binding);

    // Change border color based on editing state
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Enter CSV URL")
        .border_style(Style::default().fg(
            if app.is_editing { Color::Yellow } else { Color::White }
        ));

    let input_field = Paragraph::new(input.clone())
        .block(input_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(input_field, chunks[0]);

    // Status message (success/error)
    let status_message = match &app.status_message {
        Some(message) => message,
        None => "",
    };

    let status_color = if status_message.starts_with("Error") {
        Color::Red
    } else if status_message.starts_with("Success") {
        Color::Green
    } else {
        Color::White
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .title("Status");

    let status_field = Paragraph::new(status_message)
        .block(status_block)
        .style(Style::default().fg(status_color))
        .wrap(Wrap { trim: true });

    frame.render_widget(status_field, chunks[1]);

    // Instructions - update to include ESC/TAB instructions
    let mode_status = if app.is_editing { "EDIT MODE" } else { "NAVIGATION MODE" };
    let instructions = format!("
Current mode: {}

Enter the URL of a CSV file containing tasks.
Press Enter to import tasks.
Press Ctrl+V (or CMD+V on macOS) to paste from clipboard.
Press ESC or TAB to toggle between edit and navigation modes.

Example: https://wastebin.perekonna.kodalem.com/Jf95Ka.csv

In NAVIGATION MODE:
  Press s to return to the Simulator screen.
  Press a to go to the Analysis screen.
  Press q to quit.
", mode_status);

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .title("Instructions");

    let instructions_field = Paragraph::new(instructions)
        .block(instructions_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(instructions_field, chunks[2]);

    // Show cursor at input position when in edit mode
    if app.is_editing {
        frame.set_cursor_position((
            chunks[0].x + input.len() as u16 + 1,
            chunks[0].y + 1,
        ));
    }
}

fn draw_analysis_screen(frame: &mut Frame, app: &App, area: Rect) {
    // Split the area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Analysis result
            Constraint::Min(1),     // Task details
        ])
        .split(area);

    // Check if we have tasks to analyze
    if let Some(simulator) = &app.simulator_variables {
        if let Some(tasks) = &simulator.list_of_tasks {
            if !tasks.is_empty() {
                // Run schedulability analysis
                let is_schedulable = app.do_analysis();

                // Display analysis result
                let result_text = if is_schedulable {
                    "✓ System is SCHEDULABLE - All tasks meet their deadlines"
                } else {
                    "✗ System is NOT SCHEDULABLE - Some tasks miss deadlines"
                };

                // Create analysis result widget
                let result_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Response Time Analysis Result")
                    .border_style(Style::default().fg(if is_schedulable { Color::Green } else { Color::Red }));

                let result_widget = Paragraph::new(result_text)
                    .block(result_block)
                    .style(Style::default().fg(if is_schedulable { Color::Green } else { Color::Red }))
                    .alignment(ratatui::layout::Alignment::Center);

                frame.render_widget(result_widget, chunks[0]);

                // Display task details in the list
                draw_task_summary(frame, tasks, chunks[1]);

                return;
            }
        }
    }

    // If no tasks, show a message
    let no_tasks_text = Paragraph::new(
        "No tasks available for analysis. Go to Properties screen (p) to import tasks."
    )
        .block(Block::default().borders(Borders::ALL).title("Analysis"))
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(no_tasks_text, area);
}
