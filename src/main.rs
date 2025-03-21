#![feature(let_chains)]
// Include the process.rs from rt-props folder

mod analysis_functions;
mod process;
mod task;
mod job;
mod simulator_variables;

mod task_import;


use std::{error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    },
    Terminal,
};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{DisableMouseCapture, Event, KeyCode, KeyModifiers};
use ratatui::crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

mod app;
mod ui;
use crate::app::{App, CurrentScreen};
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    
    // Download the file from the URL and parse it
    // https://wastebin.perekonna.kodalem.com/Jf95Ka.csv
    
    
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    // Setup the terminal
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    #[allow(unused_variables)]
    let res = run_app(&mut terminal, &mut app);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

        
        
    
    
    /* Old code 
    // Simulate the system of just only two tasks and 2 cycles
    let mut simulator = simulator_variables::SimulatorVariables::new(10, 2, 10000);
    // Set the list of tasks
    simulator.set_list_of_tasks(vec![
        task::Task::new(1, 3, 1, 40, 40),   // T1
        task::Task::new(2, 7, 2, 80, 80),   // T2
        task::Task::new(1, 13, 1, 100, 100), // T3
        task::Task::new(3, 18, 3, 160, 160), // T4
        task::Task::new(1, 22, 1, 200, 200), // T5
        task::Task::new(5, 27, 5, 300, 300), // T6
        task::Task::new(8, 29, 8, 320, 320), // T7
        task::Task::new(9, 34, 10, 400, 400), // T8
        task::Task::new(10, 35, 22, 480, 480), // T9
    ]);
    
    // Generate jobs for the tasks
    simulator.generate_jobs().expect("TODO: panic message");
    
    simulator.setup_processor(1, 2);
    // Run the simulator
    println!("The simulation has started!");
    simulator.run();
    // Print the results
    println!("The simulation has ended!");
    
    // Do some analysis
    let analysis:bool = simulator.check_schedulability();
    if analysis {
        println!("The system is schedulable!");
    }
    else {
        println!("The system is not schedulable!");
    }    
    simulator.debug_print(true, true);
    
        
     */
        
        
    Ok(())
}

// Application logic
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    
    loop {
        // Update simulation tick for throbber animation
        app.update_simulation_tick();
        
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            
            match app.current_screen {
                CurrentScreen::SimulatorScreen => match key.code {
                    KeyCode::Enter => {
                        // First check if jobs are generated
                        if !app.jobs_generated {
                            app.status_message = Some(String::from("Error: Generate jobs first with 'g'"));
                        } else if !app.simulation_running {
                            app.start_simulation();
                        }
                    },
                    KeyCode::Char('g') => {
                        // Generate jobs if tasks are imported but jobs not yet generated
                        if !app.jobs_generated && app.simulator_variables.is_some() {
                            app.generate_jobs();
                        }
                    },
                    KeyCode::Char('r') => {
                        // Reset the simulation
                        app.reset_simulation();
                    },
                    KeyCode::Char('c') => {
                        app.toggle_cycle_edit_mode();
                    },
                    KeyCode::Esc | KeyCode::Tab => {
                        // Toggle edit modes
                        if app.editing_sleep {
                            app.toggle_sleep_edit_mode();
                        } else if app.editing_cycles {
                            app.toggle_cycle_edit_mode();
                        }
                    },
                    KeyCode::Up => {
                        app.scroll_tasks_up();
                    },
                    KeyCode::Down => {
                        app.scroll_tasks_down();
                    },
                    KeyCode::Char(c) if app.editing_sleep && c.is_digit(10) => {
                        app.input_sleep_char(c);
                    },
                    KeyCode::Char(c) if app.editing_cycles && c.is_digit(10) => {
                        app.input_cycle_char(c);
                    },
                    KeyCode::Backspace if app.editing_sleep => {
                        app.delete_sleep_char();
                    },
                    KeyCode::Backspace if app.editing_cycles => {
                        app.delete_cycle_char();
                    },
                    KeyCode::Char('a') => {
                        app.toggle_analysis_screen();
                    },
                    KeyCode::Char('p') => {
                        app.toggle_properties_screen();
                    },
                    KeyCode::Char('q') => {
                        app.toggle_exit_screen();
                    },
                    _ => {},
                },
                CurrentScreen::AnalysisScreen => match key.code {
                    
                    // Go to simulator screen
                    KeyCode::Char('s') => {
                        app.toggle_simulator_screen();
                    }
                    // Go to properties screen
                    KeyCode::Char('p') => {
                        app.toggle_properties_screen();
                    }                    
                    // Go to exit screen
                    KeyCode::Char('q') => {
                        app.toggle_exit_screen();
                    }
                    _ => { todo!() },
                }
                CurrentScreen::PropertiesScreen => match key.code {
                    // Toggle edit mode with ESC or TAB
                    KeyCode::Esc | KeyCode::Tab => {
                        app.toggle_edit_mode();
                    },
                    // Character input when editing
                    KeyCode::Char(c) => {
                        if app.is_editing {
                            app.input_char(c);
                        } else {
                            // Only process navigation keys when not in edit mode
                            match c {
                                'q' => app.toggle_exit_screen(),
                                's' => app.toggle_simulator_screen(),
                                'a' => app.toggle_analysis_screen(),
                                _ => {}
                            }
                        }
                    },
                    KeyCode::Backspace => {
                        if app.is_editing {
                            app.delete_char();
                        }
                    },
                    KeyCode::Enter => {
                        if app.is_editing {
                            app.process_url_import();
                        }
                    },
                    // Check for paste shortcuts: either Ctrl+V (Windows/Linux) or CMD+V (macOS)
                    // Add ignore of unreachable code for now
                    #[allow(unreachable_patterns)]
                    KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) ||
                        key.modifiers.contains(KeyModifiers::SUPER) => {
                        if app.is_editing {
                            app.paste_from_clipboard();
                        }
                    },
                    _ => {},
                }
                
                CurrentScreen::ExitScreen => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') => {
                        return Ok(false);
                    }
                    _ => { todo!() },
                }
            }
            
            
        }
    }
}