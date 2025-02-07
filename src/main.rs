use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

// Randomizer package
use rand::prelude::*;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn main() {
    // Init the simulator
    let vss = VerySimpleSimulator {
        number_of_cycles: 3,
        number_of_tasks: 2,
    };
    let mut tasks = init_empty_set_of_tasks(&vss);
    // Init the tasks
    init_jobs(&mut tasks);
    // Print the jobs
    print_jobs(&tasks);

    // Response time analysis
    //let response_time = response_time_analysis_proven();

    // Response time analysis
    let response_time = response_time_analysis(&tasks.machine[0]);

    println!("Response Time Analysis:");
    // Return a list of schedulable tasks
    for i in response_time {
        println!("  Task {} is schedulable!", i.iteration.to_string().green());
    }
    println!(" ");
    // Init the processor
    let mut core_process = init_processor(1);
    // Run the simulator
    let output_tasks = very_simple_simulator(&mut tasks, &mut core_process);


    // Print the jobs again
    //print_jobs(&output_tasks);


}



struct VerySimpleSimulator {
    number_of_cycles: i64,
    number_of_tasks: i64,
}

struct CoresProcessor {
    cores: Vec<ProcessInfo>,
    current_time: i64,
}

struct ProcessInfo {
    blocked: bool,
    // Borrow the TaskGeneric struct
    used_by: Job,
    stack_queue: Vec<Job>,
}
// ProcessInfo struct functions
impl ProcessInfo {
    fn unblock(&mut self) {
        //println!("Unblocking the process!");
        self.blocked = false;
    }

    fn do_task(&mut self) {
        self.used_by.compute_progress += 1;
        self.used_by.do_task();
        // Check if the task is done
        if self.used_by.is_done {
            //println!("Task done!");
            self.unblock();
        }
    }

    fn pick_from_stack_queue(&mut self) {
        if !self.stack_queue.is_empty() && !self.blocked {
            //println!("Picking from stack queue!");
            self.used_by = self.stack_queue.remove(0);
            self.blocked = true;
        }
    }

}

#[derive(Debug, Clone)]
struct Job {
    execution_time: i64,
    absolute_deadline: i64,
    release_time: i64,
    is_done: bool,
    iteration: i64,
    compute_progress: i64,
    progress_bar: ProgressBar,
}
impl Job {
    fn do_task(&mut self) {
        if !self.is_done {
            if self.compute_progress >= self.execution_time {
                self.is_done = true;
                // Update the progress bar
                self.progress_bar.finish_with_message("Job completed!");
            }
            self.progress_bar.set_position(self.compute_progress as u64);
        }
    }
}

#[derive(Debug, Clone)]
struct TaskGeneric {
    worst_case_execution_time: i64,
    best_case_execution_time: i64,
    phase_of_task: i64,
    period_of_task: i64,
    relative_deadline: i64,
    jobs: Vec<Job>,
    iteration: i64,
    number_of_jobs: i64,
    
    // Task progress bar
    progress_bar: ProgressBar,
}
// Add assertion of the TaskGeneric struct that if the completed time is equal to the worst case execution time, then the task is done.
impl TaskGeneric {
    fn completed_task(&mut self) {
        for job in &mut self.jobs {
            if job.execution_time <= 0 {
                job.is_done = true;
                job.progress_bar.finish_with_message("Job completed!");
            }
        }
    }
    
    fn update_task_progress_bar(&mut self, time: u64){
        self.progress_bar.set_position(time);
        // If the task is done, finish the progress bar

    }
    fn finish_progress_bar(&mut self, time: u64) -> bool {
        if self.jobs[self.jobs.len() - 1].absolute_deadline as u64 <= time {
            self.progress_bar.finish_with_message("Task completed!");
            return true;
        }
        false
    }
    fn init_task_progress_bar(&mut self) {
        // Set the task progress bar maximum value to be the job of latest absolute deadline
        self.progress_bar = ProgressBar::new(self.jobs[self.jobs.len() - 1].absolute_deadline as u64);
        // Set the style 
        self.progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "Task {}: {{bar:40.blue/cyan}} {{pos:>5}}/{{len:5}} {{msg}}",
                    self.iteration
                ))
                .expect("Failed to set progress bar template")
                .progress_chars("||.")
        );
    }


    fn init_job_progress_bar(&mut self) {
        for job in &mut self.jobs {
            job.progress_bar = ProgressBar::new(job.execution_time as u64);
            job.progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "Task {} || Job {}: {{bar:40.yellow/orange}} {{pos:>5}}/{{len:5}} {{msg}}",
                        self.iteration, job.iteration
                    ))
                    .expect("Failed to set progress bar template")
                    .progress_chars("#>-")
            );
        }
    }
}
#[derive(Debug,Clone)]
struct SetOfTasks {
    //phase_of_task: i64,
    //period_of_task: i64,
    tasks: Vec<TaskGeneric>,
    iteration: i64,
    
    // Multi progress bar
    multi_progress: MultiProgress
}
impl SetOfTasks {
    fn update_job_progress_bar(&mut self, time: u64){
        for task in &mut self.tasks {
            task.update_task_progress_bar(time);
        }
    }
    fn init_task_progress_bar(&mut self) {
        for task in &mut self.tasks {
            task.init_task_progress_bar();
        }
    }
    fn init_job_progress_bar(&mut self) {
        for task in &mut self.tasks {
            task.init_job_progress_bar();
        }
    }
    fn insert_progress_bar(&mut self, progress_bar: ProgressBar) {
        self.multi_progress.add(progress_bar);
    }
}

#[derive(Debug,Clone)]
struct SetOfPeriodicTasks {
    machine: Vec<SetOfTasks>,
}

fn init_empty_set_of_tasks(vss: &VerySimpleSimulator) -> SetOfPeriodicTasks {
    let mut tasks = Vec::new();
    for i in 0..vss.number_of_tasks {
        println!("Creating task {}", i);
        let mut task_generic = TaskGeneric {
            phase_of_task: rand::rng().random_range(0..10),
            worst_case_execution_time: 0,
            best_case_execution_time: 0,
            period_of_task: 1000,
            relative_deadline: 0,
            iteration: i,
            number_of_jobs: vss.number_of_cycles,
            progress_bar: ProgressBar::new(0),
            jobs: Vec::new(),
        };

        for j in 0..task_generic.number_of_jobs {
            println!("Creating job {}", j);
            task_generic.jobs.push(Job {
                execution_time: 0,
                absolute_deadline: 0,
                compute_progress: 0,
                release_time: 0,
                is_done: false,
                iteration: j,
                progress_bar: ProgressBar::new(0),
            });
        }

        tasks.push(SetOfTasks {
            tasks: vec![task_generic],
            iteration: 0,
            multi_progress: MultiProgress::new(),
        });
    }
    SetOfPeriodicTasks { machine: tasks }
}
fn guaranteed_randomizer_task(task: &mut TaskGeneric) {
    // Set the Utilization Limit
    let utilization_limit = 0.68;
    // Set the period to a random value within a reasonable range
    task.period_of_task = rand::rng().random_range(50..1000);
    // Calculate the maximum WCET based on the utilization limit
    let max_wcet = (task.period_of_task as f64 * utilization_limit).floor() as i64;
    // Set the WCET to a random value within the calculated limit
    task.worst_case_execution_time = rand::rng().random_range(1..=max_wcet/4);
    // Set the best-case execution time to a random value less than or equal to WCET
    task.best_case_execution_time = rand::rng().random_range(1..=task.worst_case_execution_time);
    // Set the relative deadline to a random value between WCET and the period
    //task.relative_deadline = rand::rng().random_range(task.worst_case_execution_time..=task.period_of_task);
    task.relative_deadline = task.period_of_task;
    for job in &mut task.jobs {
        job.iteration += 1;
        job.release_time = task.phase_of_task + (job.iteration - 1) * task.period_of_task ;
        job.absolute_deadline = job.release_time + task.period_of_task;
        job.execution_time = rand::rng().random_range(task.best_case_execution_time..=task.worst_case_execution_time);
        job.progress_bar = ProgressBar::new(job.execution_time as u64);
        job.progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos:>5}/{len:5} {msg}")
                .expect("Failed to set progress bar template")
                .progress_chars("#>-")
        );
    }
    // Initialize the progress bar
    task.init_job_progress_bar();
}

fn init_jobs(tasks: &mut SetOfPeriodicTasks) {
    for machine in &mut tasks.machine {
        for task in &mut machine.tasks {
            guaranteed_randomizer_task(task);
            task.init_task_progress_bar();
        }
    }
}

fn print_jobs(tasks: &SetOfPeriodicTasks) {
    println!("Printing the jobs");
    for task in &tasks.machine {
        // Panic if the task has no jobs
        if task.tasks.len() == 0 {
            panic!("Task has no jobs!");
        }
        println!("Machine: {}", task.iteration.to_string().cyan());
        for task in &task.tasks {
            // Print the task properties
            println!("  Task: {}, WCET: {}, BCET: {}, Period: {}, Relative Deadline: {}",
                     task.iteration.to_string().green(),
                     task.worst_case_execution_time.to_string().green(),
                     task.best_case_execution_time.to_string().green(),
                     task.period_of_task.to_string().green(),
                     task.relative_deadline.to_string().green());
            for job in &task.jobs {
                // Print the job properties
                println!("    Job: {}, Release Time:{}, Execution Time: {}, Deadline: {}, Release Time: {}",
                         job.iteration.to_string().yellow(),
                            job.release_time.to_string().yellow(),
                         job.execution_time.to_string().yellow(),
                         job.absolute_deadline.to_string().yellow(),
                         job.release_time.to_string().yellow());
            }
        }
    }
}

fn init_clock_process() -> ProcessInfo {
    ProcessInfo {
        blocked: false,
        used_by: Job {
            execution_time: 0,
            absolute_deadline: 0,
            compute_progress: 0,
            release_time: 0,
            is_done: false,
            iteration: 0,
            progress_bar: ProgressBar::new(0),
        },
        stack_queue: Vec::new(),
    }
}

fn init_processor(num_cores: i64) -> CoresProcessor {
    let mut cores = Vec::new();
    for _i in 0..num_cores {
        cores.push(init_clock_process());
    }
    CoresProcessor { cores, current_time: 0 }
}

fn get_available_core(core_process: &mut CoresProcessor) -> i64 {
    for i in 0..core_process.cores.len() {
        if core_process.cores[i].blocked == false {
            return i as i64;
        }
    }
    // Hacky way but... makes sure to return -1 if no cores are available
    -1
}

fn core_priority_by_task_amount(core_process: &mut CoresProcessor) -> i64 {
    let mut core = 0;
    let mut min = core_process.cores[0].stack_queue.len();
    for i in 0..core_process.cores.len() {
        if core_process.cores[i].stack_queue.len() < min {
            min = core_process.cores[i].stack_queue.len();
            core = i;
        }
    }
    core as i64
}


fn advance_processes(core_process: &mut CoresProcessor, output_tasks: &mut TaskGeneric) {
    for i in 0..core_process.cores.len() {
        if core_process.cores[i].blocked {
            // Hard deadline deletion of current task
            if core_process.cores[i].used_by.absolute_deadline == core_process.current_time {
                core_process.cores[i].used_by.is_done = false;
                core_process.cores[i].used_by.progress_bar.finish_with_message("Job missed deadline!");
                println!("Current Time: {}, Absolute Deadline: {}", core_process.current_time, core_process.cores[i].used_by.absolute_deadline);

                // Unblocking the process
                core_process.cores[i].unblock();
                // Push the task to the output tasks
                output_tasks.jobs.push(core_process.cores[i].used_by.clone());
            } else {
                core_process.cores[i].do_task();
                if core_process.cores[i].used_by.is_done {
                    output_tasks.jobs.push(core_process.cores[i].used_by.clone());
                    core_process.cores[i].unblock();
                }
            }
            core_process.cores[i].pick_from_stack_queue();
        }
    }
}
fn very_simple_simulator(tasks: &mut SetOfPeriodicTasks, core_process: &mut CoresProcessor) -> SetOfPeriodicTasks {
    let mut task_generic = TaskGeneric {
        worst_case_execution_time: 0,
        best_case_execution_time: 0,
        phase_of_task: 0,
        period_of_task: 0,
        relative_deadline: 0,
        jobs: Vec::new(),
        progress_bar: ProgressBar::new(0),
        iteration: 0,
        number_of_jobs: 0,
    };
    let mut set_of_tasks_output = SetOfTasks {
        tasks: Vec::new(),
        iteration: 0,
        multi_progress: MultiProgress::new(),
    };
    let mut output_tasks = SetOfPeriodicTasks {
        machine: Vec::new(),
    };

    for machine in &mut tasks.machine {
        machine.tasks.sort_by_key(|task| task.period_of_task);
    }

    core_process.current_time = 0;
    while core_process.current_time <= 8000 {
        for machine in &mut tasks.machine {
            for task in &mut machine.tasks.clone() {
                for job in &mut task.jobs.clone() {
                    if job.release_time == core_process.current_time {
                        let core = get_available_core(core_process);
                        if core != -1 {
                            //println!("Job {} of Task {} added to core {}, at {}", job.iteration, task.iteration, core, core_process.current_time);
                            core_process.cores[core as usize].used_by = job.clone();
                            core_process.cores[core as usize].blocked = true;
                        } else {
                            //println!("  Job {} of Task {} added to stack queue", job.iteration, task.iteration);
                            let core = core_priority_by_task_amount(core_process);
                            core_process.cores[core as usize].stack_queue.push(job.clone());
                        }
                    }
                }
                //if !task.finish_progress_bar(core_process.current_time as u64){
                //    task.update_task_progress_bar(core_process.current_time as u64);
                //}
                
            }
            advance_processes(core_process, &mut task_generic);
        }
        set_of_tasks_output.tasks.push(task_generic.clone());
        thread::sleep(Duration::from_millis(2));
        core_process.current_time += 1;
    }
    output_tasks.machine.push(set_of_tasks_output);
    output_tasks
}

// Proven working tasks
// Task 1: WCET: 1, Period: 4, Deadline: 3
// Task 2: WCET: 1, Period: 5, Deadline: 4
// Task 3: WCET: 2, Period: 6, Deadline: 5
// Task 4: WCET: 1, Period: 11, Deadline: 10


////// Scheduler Functions //////

fn get_processor_utilization(tasks: &SetOfTasks) -> f64 {
    let mut utilization = 0.0;
    for task in &tasks.tasks {
        utilization += task.worst_case_execution_time as f64 / task.period_of_task as f64;
    }
    utilization
}

fn find_task_highest_response_time (tasks: &SetOfTasks, task: &TaskGeneric, previous_response_time: i64) -> i64 {
    let longest_respoonse_time_at_critical_instant_guess_1 = previous_response_time + task.worst_case_execution_time;
    // Lower bound method
    let longest_respoonse_time_at_critical_instant_guess_2 = task.worst_case_execution_time as f64 / (1.0 - get_processor_utilization(tasks));
    // Pick the largest of the two guesses
    let return_value = longest_respoonse_time_at_critical_instant_guess_1.max(longest_respoonse_time_at_critical_instant_guess_2 as i64);
    return_value
}

fn response_time_analysis(tasks: &SetOfTasks) -> Vec<TaskGeneric> {
    let mut schedulable_tasks = Vec::new();

    for task in &tasks.tasks {
        let mut longest_response_time_at_previous = 0;
        let mut longest_response_time_at_critical_instant = task.worst_case_execution_time;

        while longest_response_time_at_critical_instant > longest_response_time_at_previous {
            if longest_response_time_at_previous == 0 {
                longest_response_time_at_previous = find_task_highest_response_time(&tasks, &task, longest_response_time_at_previous);
            }
            longest_response_time_at_previous = longest_response_time_at_critical_instant;

            let mut interference_measurement = 0;
            for higher_priority_task in &tasks.tasks {
                if higher_priority_task.iteration == task.iteration {
                    continue;
                }

                let division_result = longest_response_time_at_previous as f64 / higher_priority_task.period_of_task as f64;
                let ceiling_result = division_result.ceil() as i64;
                let interference_contribution = ceiling_result * higher_priority_task.worst_case_execution_time;

                interference_measurement += interference_contribution;
            }

            longest_response_time_at_critical_instant = task.worst_case_execution_time + interference_measurement;

            if longest_response_time_at_critical_instant > task.relative_deadline {
                break; // Task is unschedulable, break the loop
            }
        }

        if longest_response_time_at_critical_instant <= task.relative_deadline {
            schedulable_tasks.push(task.clone()); // Task is schedulable, add to the list
        }
    }

    schedulable_tasks
}