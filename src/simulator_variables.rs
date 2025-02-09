
// Include the process.rs, job.rs, and task.rs files
use crate::task::{Task};
use crate::process::{ProcessorUnit};
use crate::job::{Job};
use crate::analysis_functions::{response_time_analysis};
use rand::Rng;
use colored::*;

// Todo: Write enum for the task traits, ie. periodic, sporadic, aperiodic  etc.
// also, the analysis results, ie. schedulable, unschedulable, etc.

struct SimulatorTime {
    time: u128, // The current time of the simulator
    // Todo - Add more fields to the struct
}

pub(crate) struct SimulatorVariables {
    number_of_tasks: usize, // The number of tasks in the system
    number_of_tasks_per_cycle: usize, // The number of tasks that can be processed in a cycle, aka number of jobs within a task
    maximum_time_to_run: usize, // The maximum time the simulator can run for
    current_time: SimulatorTime, // The current time of the simulator
    list_of_tasks: Option<Vec<Task>>, // The list of tasks in the system
    maximum_job_time: usize, // The maximum time a job can run for
    processor: Option<ProcessorUnit>, // Mono-processor system for now

    // Todo: Write randomizer booleans
}


impl SimulatorVariables {
    pub(crate) fn new(number_of_tasks: usize, number_of_tasks_per_cycle: usize, maximum_time_to_run: usize) -> Self {
        Self {
            number_of_tasks,
            number_of_tasks_per_cycle,
            maximum_time_to_run,
            current_time: SimulatorTime { time: 0 },
            list_of_tasks: None,
            maximum_job_time: 100,
            processor: None,
        }
    }

    // Set the list of tasks
    pub(crate) fn set_list_of_tasks(&mut self, list_of_tasks: Vec<Task>) {
        self.list_of_tasks = Some(list_of_tasks);
    }
    // Generate jobs for the tasks
    pub(crate) fn generate_jobs(&mut self) {
        if let Some(tasks) = &mut self.list_of_tasks {
            for task in tasks.iter_mut() {
                task.randomly_generate_jobs(self.number_of_tasks_per_cycle);
            }
        }
    }
    
    // Advance the time of the simulator
    fn advance_time(&mut self) {
        self.current_time.time += 1;
    }

    // Return a job if the release time is equal to the current time
    fn poll_jobs(&mut self) -> Option<Job> {
        let job = None;
        if let Some(tasks) = &self.list_of_tasks {
            for task in tasks {
                if let Some(jobs) = &task.jobs {
                    for job in jobs {
                        if let Some(release_time) = job.release_time {
                            if release_time == self.current_time.time as usize {
                                return Some(job.clone());
                            }
                        }
                    }
                }
            }
        }
        job
    }

    // Run the simulator
    pub(crate) fn run(&mut self) {
        
        // Check if the simulator is ready to run
        if !self.is_ready() {
            panic!("Simulator is not ready to run!");
        }
        
        while self.current_time.time < self.maximum_time_to_run as u128 {

            // Add a job to the processor from polling the jobs
            if let Some(job) = self.poll_jobs() {
                if let Some(processor) = &mut self.processor {
                    processor.add_job(job);
                }
                // Panic if processor is not set
                else {
                    panic!("Processor not set!");
                }
            }

            // Advance the time in the processor
            if let Some(processor) = &mut self.processor {
                processor.advance_time(self.current_time.time as usize);
            }

            self.advance_time();
        }
    }

    // Randomly generate a list of tasks, making sure that WCET is greater than BCET
    fn generate_tasks(&self, rand_range: usize) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut rng = rand::rng();

        for i in 0..self.number_of_tasks {
            let wcet = rng.random_range(2..rand_range);
            let bcet = rng.random_range(1..wcet);
            let relative_deadline = rng.random_range(wcet..wcet + rand_range);
            // As for now, the period is set to be equal to the relative deadline
            let period = relative_deadline;

            tasks.push(Task::new(i, wcet, bcet, relative_deadline, period));
        }

        tasks
    }
    pub(crate) fn setup_processor(&mut self, id: usize, clock_period: usize) {
        self.processor = Some(ProcessorUnit::new(id, clock_period));
        // Debug print the processor
        if let Some(processor) = &self.processor {
            processor.debug_print(true, true);
        }
    }

    // Check if the list of tasks are schedulable by RT analysis
    pub(crate) fn check_schedulability(&self) -> bool {
        if let Some(tasks) = &self.list_of_tasks {
            let schedulable_tasks = response_time_analysis(tasks);
            if schedulable_tasks.len() == tasks.len() {
                return true;
            }
            // Print the tasks that are schedulable
            for task in schedulable_tasks {
                println!("Task {} is schedulable", task.id);
                // Debug print the task
                task.debug_print(true);
            }
        }
        false
    }

    // Debug print the simulator variables
    pub(crate) fn debug_print(&self, include_tasks: bool, include_processor: bool) {
        println!("Number of tasks: {}", self.number_of_tasks.to_string().cyan());
        println!("Number of tasks per cycle: {}", self.number_of_tasks_per_cycle.to_string().cyan());
        println!("Maximum time to run: {}", self.maximum_time_to_run.to_string().cyan());
        println!("Current time: {}", self.current_time.time.to_string().cyan());
        if let Some(tasks) = &self.list_of_tasks {
            if include_tasks {
                for task in tasks {
                    task.debug_print(true);
                }
            }
        }
        if let Some(processor) = &self.processor {
            if include_processor {
                processor.debug_print(true, true);
            }
        }
    }
    
    // Function which ensures that the simulator is ready to simulate, i.e. there is a processor, tasks and jobs
    pub(crate) fn is_ready(&self) -> bool {
        if self.processor.is_some() && self.list_of_tasks.is_some() {
            // Check if the tasks have jobs
            if let Some(tasks) = &self.list_of_tasks {
                for task in tasks {
                    if task.jobs.is_none() {
                        println!("Task {} has no jobs!", task.id.to_string().red().bold());
                        return false;
                    }
                }
                return true;
            }
        }
        println!("{}", "Processor and/or Tasks is not ready!".red().bold());
        false
    }

}

struct Simulator {
    variables: SimulatorVariables,
}

impl Simulator {
    fn new(number_of_tasks: usize, number_of_tasks_per_cycle: usize, maximum_time_to_run: usize) -> Self {
        let variables = SimulatorVariables::new(number_of_tasks, number_of_tasks_per_cycle, maximum_time_to_run);
        Self {
            variables,
        }
    }
    fn setup_tasks_from_gen(&mut self, rand_range: usize) {
        let tasks = self.variables.generate_tasks(rand_range);
        self.variables.set_list_of_tasks(tasks);
    }
    fn run(&mut self) {
        self.variables.run();
    }
    fn setup_processor(&mut self, id: usize, clock_period: usize) {
        self.variables.setup_processor(id, clock_period);
    }

    fn do_analysis(&self) {
        self.variables.check_schedulability();
    }
    
    fn check_schedulability(&self)  {
        // Print the results
        if self.variables.check_schedulability() {
            println!("The system is schedulable!");
        } else {
            println!("The system is not schedulable!");
        }
    }

    fn debug_print(&self, include_tasks: bool, include_processor: bool) {
        self.variables.debug_print(include_tasks, include_processor);
    }
}
