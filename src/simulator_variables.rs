
// Include the process.rs, job.rs, and task.rs files
use crate::task::{Task};
use crate::process::{ProcessorUnit};
use crate::job::{Job};
use rand::Rng;

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
    pub(crate) fn set_list_of_tasks(&mut self, list_of_tasks: Vec<Task>) {
        self.list_of_tasks = Some(list_of_tasks);
    }
    fn advance_time(&mut self) {
        self.current_time.time += 1;
    }
    // Return a job if the release time is equal to the current time
    fn poll_jobs(&mut self) -> Option<Job> {
        let mut job = None;
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
    pub(crate) fn run(&mut self) {
        while self.current_time.time < self.maximum_time_to_run as u128 {
            
            // Add a job to the processor from polling the jobs
            if let Some(job) = self.poll_jobs() {
                if let Some(processor) = &mut self.processor {
                    processor.add_job(job);
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
    fn generate_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut rng = rand::rng();

        for i in 0..self.number_of_tasks {
            let wcet = rng.random_range(1..100);
            let bcet = rng.random_range(1..wcet);
            let relative_deadline = rng.random_range(wcet..100);
            // As for now, the period is set to be equal to the relative deadline
            let period = relative_deadline;

            tasks.push(Task::new(i, wcet, bcet, relative_deadline, period));
        }

        tasks
    }
    fn setup_processor(&mut self, id: usize, clock_period: usize) {
        self.processor = Some(ProcessorUnit::new(id, clock_period));
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
    fn setup_tasks(&mut self) {
        let tasks = self.variables.generate_tasks();
        self.variables.set_list_of_tasks(tasks);
    }
    fn run(&mut self) {
        self.variables.run();
    }
    
}
