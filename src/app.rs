// Include the process.rs from rt-props folder
use crate::simulator_variables::SimulatorVariables;
use crate::task_import::download_and_parse_tasks;
use clipboard::{ClipboardContext, ClipboardProvider};
use ratatui::widgets::ScrollbarState;
use crate::job::JobStatus;
use crate::process::JobHistory;

// Screens for the simulator
pub enum CurrentScreen {
    SimulatorScreen, // The screen when simulation is running
    AnalysisScreen, // The screen when analysis is being done and shown to the user
    PropertiesScreen, // The screen when the properties of the system are being shown to the user
    ExitScreen, // The screen when the user wants to exit the application
}

pub(crate) struct SetupValues {
    pub number_of_tasks: usize,
    pub number_of_tasks_per_cycle: usize,
    pub maximum_time_to_run: usize,
    // Todo: Add processor variables
}
impl SetupValues {
    fn new(number_of_tasks: usize, number_of_tasks_per_cycle: usize, maximum_time_to_run: usize) -> Self {
        Self {
            number_of_tasks,
            number_of_tasks_per_cycle,
            maximum_time_to_run,
        }
    }
}

pub struct App {
    pub current_screen: CurrentScreen, // The current screen of the app
    pub simulator_variables: Option<SimulatorVariables>, // The simulator variables for the system
    pub setup_values: Option<SetupValues>, // The setup values for the simulator
    pub input: Option<String>,         // For URL input
    pub is_editing: bool,              // Flag to indicate edit mode
    pub status_message: Option<String>, // Status messages for operations
    pub simulation_running: bool, // Flag to indicate if simulation is running
    pub simulation_sleep: u64, // Sleep time in milliseconds for each step
    pub editing_sleep: bool,   // Flag indicating if we're editing sleep time
    pub jobs_generated: bool, // Flag indicating if jobs have been generated
    pub cycle_input: Option<String>,   // For cycle count input
    pub editing_cycles: bool,         // Flag for editing cycle count
    pub simulation_tick: u64, // The tick of the simulation
    pub(crate) task_list_scroll: usize,
    pub(crate) task_list_scrollbar: ScrollbarState,
}

impl App {
    
    // Get amount of processor units
    pub fn get_amount_of_processor_units(&self) -> usize {
        if let Some(simulator_variables) = &self.simulator_variables {
            return simulator_variables.get_number_of_processor_units();
        }
        0
    }
    
    pub fn get_amount_of_tasks(&self) -> usize {
        if let Some(setup_values) = &self.setup_values {
            return setup_values.number_of_tasks;
        }
        0
    }
    

    pub fn toggle_cycle_edit_mode(&mut self) {
        self.editing_cycles = !self.editing_cycles;
        // Ensure only one edit mode at a time
        if self.editing_cycles {
            self.is_editing = false;
            self.editing_sleep = false;
        }
    }


    pub fn input_cycle_char(&mut self, c: char) {
        if self.editing_cycles && c.is_digit(10) {
            if let Some(input) = &mut self.cycle_input {
                input.push(c);
            } else {
                self.cycle_input = Some(c.to_string());
            }
        }
    }

    pub fn delete_cycle_char(&mut self) {
        if self.editing_cycles {
            if let Some(input) = &mut self.cycle_input {
                input.pop();
            }
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            current_screen: CurrentScreen::SimulatorScreen,
            simulator_variables: Some(SimulatorVariables::new(0, 0, 10000)),
            setup_values: Some(SetupValues::new(2, 2, 10000)),
            input: Some(String::new()),
            is_editing: false,
            status_message: None,
            simulation_running: false,
            simulation_sleep: 100,
            editing_sleep: false,
            jobs_generated: false,
            cycle_input: Some(String::new()),
            editing_cycles: false,
            simulation_tick: 0,
            task_list_scroll: 0,
            task_list_scrollbar: ScrollbarState::default(),
        }
    }

    // Add methods to control scrolling
    pub(crate) fn scroll_tasks_up(&mut self) {
        if self.task_list_scroll > 0 {
            self.task_list_scroll -= 1;
            self.task_list_scrollbar = self.task_list_scrollbar.position(self.task_list_scroll);
        }
    }

    pub fn reset_simulation(&mut self) {
        // Reset simulation flags
        self.simulation_running = false;
        self.jobs_generated = false;
        self.simulation_tick = 0;

        // Clear status message
        self.status_message = None;

        // Re-initialize simulator variables
        if let Some(setup_values) = &self.setup_values {
            self.simulator_variables = Some(SimulatorVariables::new(
                setup_values.number_of_tasks,
                setup_values.number_of_tasks_per_cycle,
                setup_values.maximum_time_to_run,
            ));
        } else {
            self.simulator_variables = None;
        }
    }

    pub(crate) fn scroll_tasks_down(&mut self) {
        if let Some(simulator) = &self.simulator_variables {
            if let Some(tasks) = &simulator.list_of_tasks {
                self.task_list_scroll = self.task_list_scroll.saturating_add(1);
                if self.task_list_scroll >= tasks.len() {
                    self.task_list_scroll = tasks.len().saturating_sub(1);
                }
                self.task_list_scrollbar = self.task_list_scrollbar.position(self.task_list_scroll);
            }
        }
    }
    
    

    pub fn update_simulation_tick(&mut self) {
        if self.simulation_running {
            self.simulation_tick = self.simulation_tick.wrapping_add(1);
        }
    }

    // Add methods to manage sleep time
    pub fn toggle_sleep_edit_mode(&mut self) {
        self.editing_sleep = !self.editing_sleep;
        // Ensure only one edit mode at a time
        if self.editing_sleep {
            self.is_editing = false;
        }
    }
    

    pub fn input_sleep_char(&mut self, c: char) {
        if self.editing_sleep && c.is_digit(10) {
            let digit = c.to_digit(10).unwrap() as u64;
            // Keep within reasonable values (max 9999ms)
            if self.simulation_sleep < 1000 {
                self.simulation_sleep = self.simulation_sleep * 10 + digit;
            }
        }
    }

    pub fn delete_sleep_char(&mut self) {
        if self.editing_sleep {
            self.simulation_sleep /= 10;
        }
    }
    

    // Add typing functionality
    pub(crate) fn input_char(&mut self, c: char) {
        if self.is_editing {
            if let Some(input) = &mut self.input {
                input.push(c);
            } else {
                self.input = Some(c.to_string());
            }
        }
    }

    pub(crate) fn delete_char(&mut self) {
        if self.is_editing {
            if let Some(input) = &mut self.input {
                input.pop();
            }
        }
    }
    
    pub(crate) fn process_url_import(&mut self) {
        // Clone the URL to avoid borrowing issues
        let url = match &self.input {
            Some(input) => {
                if input.is_empty() {
                    self.status_message = Some(String::from("Error: URL cannot be empty"));
                    return;
                }
                input.clone()
            },
            None => {
                self.status_message = Some(String::from("Error: URL is not set"));
                return;
            }
        };

        // Initialize simulator if needed
        if self.simulator_variables.is_none() {
            if let Some(setup) = &self.setup_values {
                self.simulator_variables = Some(
                    SimulatorVariables::new(
                        setup.number_of_tasks,
                        setup.number_of_tasks_per_cycle,
                        setup.maximum_time_to_run
                    )
                );
            } else {
                self.status_message = Some(String::from("Error: Setup values not initialized"));
                return;
            }
        }

        // Import tasks from URL
        match self.get_tasks_from_url(&url) {
            Ok(_) => {
                self.status_message = Some(format!("Success: Imported tasks from {}", url));

                // Update the setup values with the new task count
                if let Some(simulator) = &self.simulator_variables {
                    if let Some(tasks) = &simulator.list_of_tasks {
                        if let Some(setup) = &mut self.setup_values {
                            setup.number_of_tasks = tasks.len();
                        }
                    }
                }

                // Generate jobs for the tasks
                if let Some(simulator) = &mut self.simulator_variables {
                    let _ = simulator.generate_jobs();

                    // Setup processor if it doesn't exist
                    if simulator.processor.is_none() {
                        simulator.setup_processor(1, 2);
                    }
                }
            },
            Err(e) => {
                self.status_message = Some(format!("Error: Failed to import tasks: {}", e));
            }
        }
    }

    pub(crate) fn paste_from_clipboard(&mut self) {
        if self.is_editing {
            // Try to get clipboard context
            if let Ok(mut ctx) = ClipboardContext::new() {
                if let Ok(contents) = ctx.get_contents() {
                    // If successful, append clipboard contents to input
                    if let Some(input) = &mut self.input {
                        input.push_str(&contents);
                    } else {
                        self.input = Some(contents);
                    }
                } else {
                    self.status_message = Some(String::from("Error: Failed to get clipboard contents"));
                }
            } else {
                self.status_message = Some(String::from("Error: Could not access clipboard"));
            }
        }
    }

    pub(crate) fn toggle_edit_mode(&mut self) {
        self.is_editing = !self.is_editing;
    }

    // Keep the existing toggle_properties_screen method
    pub(crate) fn toggle_properties_screen(&mut self) {
        self.current_screen = CurrentScreen::PropertiesScreen;
        self.is_editing = true; // Enable editing mode when entering properties screen
    }

    // Other toggle methods should disable editing mode
    pub(crate) fn toggle_simulator_screen(&mut self) {
        self.current_screen = CurrentScreen::SimulatorScreen;
        self.is_editing = false;
    }

    pub(crate) fn toggle_analysis_screen(&mut self) {
        self.current_screen = CurrentScreen::AnalysisScreen;
        self.is_editing = false;
    }

    pub(crate) fn toggle_exit_screen(&mut self) {
        self.current_screen = CurrentScreen::ExitScreen;
        self.is_editing = false;
    }
    
    
    ///
    // In src/app.rs
    // Update get_amount_of_jobs method to correctly count all jobs

    pub fn get_amount_of_jobs(&self) -> usize {
        if !self.jobs_generated {
            return 0; // Return 0 if jobs haven't been generated yet
        }

        // Existing logic for counting jobs
        if let Some(simulator_vars) = &self.simulator_variables {
            if let Some(tasks) = &simulator_vars.list_of_tasks {
                let mut job_count = 0;
                for task in tasks {
                    if let Some(jobs) = &task.jobs {
                        job_count += jobs.len();
                    }
                }
                return job_count;
            }
        }
        0
    }

    // Update generate_jobs method to set up processor unit if not already done
    pub fn generate_jobs(&mut self) {
        if let Some(cycle_str) = &self.cycle_input {
            if let Ok(cycles) = cycle_str.parse::<usize>() {
                if cycles > 0 {
                    if let Some(simulator) = &mut self.simulator_variables {
                        // Set the number of tasks per cycle
                        simulator.set_number_of_tasks_per_cycle(cycles);

                        // Generate jobs
                        match simulator.generate_jobs() {
                            Ok(_) => {
                                // Set up processor if not already done
                                if simulator.processor.is_none() {
                                    simulator.setup_processor(1, 1); // id=1, clock_period=1
                                }
                                self.jobs_generated = true;
                                self.status_message = Some(String::from("Success: Jobs generated"));
                            },
                            Err(e) => {
                                self.status_message = Some(format!("Error: {}", e));
                            }
                        }
                    } else {
                        self.status_message = Some(String::from("Error: Simulator not initialized"));
                    }
                } else {
                    self.status_message = Some(String::from("Error: Number of cycles must be greater than 0"));
                }
            } else {
                self.status_message = Some(String::from("Error: Invalid number of cycles"));
            }
        } else {
            self.status_message = Some(String::from("Error: Please enter number of cycles"));
        }
    }

    // Update start_simulation method to properly handle simulation state
    pub fn start_simulation(&mut self) {
        self.simulation_running = true;
        self.status_message = Some(String::from("Simulation in progress..."));

        if let Some(mut simulator) = self.simulator_variables.take() {
            // Configure the sleep time
            simulator.set_simulation_speed(self.simulation_sleep as u128);

            if simulator.processor.is_none() {
                simulator.setup_processor(1, 1);
            }

            // Run the simulator
            let result = simulator.run();

            // Initialize job history if needed
            if simulator.processor.is_some() && simulator.processor.as_ref().unwrap().job_history.is_none() {
                simulator.processor.as_mut().unwrap().job_history = Some(JobHistory { done_jobs: Vec::new() });
            }

            // Create a collection of all jobs that should be tracked
            let mut all_jobs = Vec::new();
            if let Some(tasks) = &simulator.list_of_tasks {
                for task in tasks {
                    if let Some(jobs) = &task.jobs {
                        for job in jobs {
                            all_jobs.push(job.clone());
                        }
                    }
                }
            }

            // Check which jobs are in history
            let processor_jobs_map = if let Some(processor) = &simulator.processor {
                if let Some(history) = &processor.job_history {
                    history.done_jobs.iter()
                        .map(|job| ((job.id, job.release_time.unwrap_or(0)), true))
                        .collect::<std::collections::HashMap<_, _>>()
                } else {
                    std::collections::HashMap::new()
                }
            } else {
                std::collections::HashMap::new()
            };

            // Add missing jobs to history
            if let Some(processor) = &mut simulator.processor {
                if let Some(history) = &mut processor.job_history {
                    for job in all_jobs {
                        let job_key = (job.id, job.release_time.unwrap_or(0));
                        if !processor_jobs_map.contains_key(&job_key) {
                            // Add job to history with completed status if simulation succeeded
                            let mut job_copy = job.clone();
                            job_copy.computation.status = if result {
                                JobStatus::Completed
                            } else {
                                JobStatus::Missed
                            };
                            history.done_jobs.push(job_copy);
                        }
                    }
                }
            }

            // Update all job statuses in the task list
            if let Some(tasks) = &mut simulator.list_of_tasks {
                for task in tasks {
                    if let Some(jobs) = &mut task.jobs {
                        for job in jobs {
                            // Make sure ALL job statuses are properly reflected
                            if result && job.computation.status != JobStatus::Missed {
                                job.computation.status = JobStatus::Completed;
                            }
                        }
                    }
                }
            }

            self.simulator_variables = Some(simulator);

            // Update UI state
            self.simulation_running = false;
            if result {
                self.status_message = Some(String::from("Simulation completed"));
            } else {
                self.status_message = Some(String::from("Simulation failed"));
            }
        }
    }
    

    // Helper method to fetch tasks from URL
    fn get_tasks_from_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Download and parse tasks
        let tasks = download_and_parse_tasks(url)?;

        // Initialize simulator if needed
        if self.simulator_variables.is_none() {
            self.simulator_variables = Some(SimulatorVariables::new(tasks.len(), 0, 10000));
        }

        // Set tasks in simulator
        if let Some(simulator) = &mut self.simulator_variables {
            simulator.set_list_of_tasks(tasks);

            // Update setup values to match task count
            if let Some(setup) = &mut self.setup_values {
                setup.number_of_tasks = simulator.list_of_tasks.as_ref().map_or(0, |tasks| tasks.len());
            }
        }

        // Reset job generation state
        self.jobs_generated = false;

        Ok(())
    }

    // Add method to run analysis
    pub fn do_analysis(&self) -> bool {
        if let Some(simulator) = &self.simulator_variables {
            return simulator.check_schedulability();
        }
        false
    }
    

    
}