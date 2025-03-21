
// Importing the necessary libraries, i.e. job.rs and task.rs

use crate::job::{Job, JobStatus};
use colored::*;

pub(crate) struct JobHistory{
    pub(crate) done_jobs: Vec<Job>,
    // Todo - Add something more?? Remove this todo?
}
impl JobHistory{
    fn add (&mut self, job: Job){
        // Create if it does not exist
        if self.done_jobs.is_empty(){
            self.done_jobs = Vec::new();
        }
        self.done_jobs.push(job);
    }
}

pub(crate) struct JobQueue{
    size: usize, // The size of the job queue
    pub(crate) jobs: Vec<Job>,
}
impl JobQueue {
    #[allow(dead_code)]
    fn new() -> Self{
        Self{
            size: 0,
            jobs: Vec::new(),
        }
    }
}

pub(crate) struct ProcessorUnit{
    #[allow(dead_code)]
    pub(crate) id: usize, // Unique identifier
    pub(crate) blocked: bool, // Is the processor blocked
    pub(crate) job: Option<Job>, // The job that is currently being processed
    pub(crate) job_queue: Option<JobQueue>, // The job queue
    clock_period: usize, // The clock period of the processor
    pub(crate) job_history: Option<JobHistory>, // The history of the jobs that have been processed
}

impl ProcessorUnit{
    // Todo: Remove this function? Not used anywhere?? Overthinking overkill?
    #[allow(dead_code)]
    fn clock_speed(&self) -> usize{
        1 / self.clock_period
    }
    
    pub(crate) fn new(id: usize, clock_period: usize) -> Self{
        Self{
            id,
            blocked: false,
            job: None,
            job_queue: None,
            clock_period, // Todo: Utilize the clock period or bin it
            job_history: None,
        }
    }
    
    fn add_job_to_history(&mut self, job: Job){
        if let Some(job_history) = &mut self.job_history{
            job_history.add(job);
        }
        else{
            self.job_history = Some(JobHistory{done_jobs: vec![job]});
        }
    }
    
    fn block(&mut self){
        self.blocked = true;
    }
    fn unblock(&mut self){
        self.blocked = false;
    }
    
    // Add a job to the processor unit, put it onto the job queue if there is some job already being processed
    pub(crate) fn add_job(&mut self, job: Job){
        // If there's already a job in progress, put the new job in the queue
        if self.job.is_some() {
            if let Some(job_queue) = &mut self.job_queue {
                job_queue.jobs.push(job);
                job_queue.size += 1;
            } else {
                self.job_queue = Some(JobQueue{size: 1, jobs: vec![job]});
            }
        } else {
            // Otherwise, set it as the current job
            self.job = Some(job);
        }
    }
    
    // Only remove the job which is/was being processed
    fn remove_job(&mut self){
        // Add the job to the job history
        if let Some(job) = &self.job{
            //println!("Job: {} completed", job.id.to_string().blue());
            self.add_job_to_history(job.clone());
        }
        self.job = None;
    }
    
    // Add a job from the job queue to the processor 
    fn add_job_from_queue(&mut self){
        if let Some(job_queue) = &mut self.job_queue{
            if job_queue.size > 0{
                self.job = Some(job_queue.jobs.remove(0));
                job_queue.size -= 1;
                // Set the job to in progress
                //if let Some(job) = &mut self.job{
                //    job.start();
                //    self.block();
                //}
            }
            // Panic if the current job is in progress
            else if self.job.is_some(){
                panic!("Job is in progress, cannot add job from queue");
            }
        }
    }
    
    // Start a job if the processor is not blocked
    fn start_job(&mut self){
        if !self.blocked{
            // Only start the job if there is a job to start
            if let Some(job) = &mut self.job{
                job.start();
                // Block the processor 
                self.block();
            }
        }
    }

    pub(crate) fn advance_time(&mut self, time: usize) {
        // Log processor status and time
        //println!("Processor {} - Time: {}, Blocked: {}", self.id, time, self.blocked);

        // Blocked processor will only compute the job which is being processed
        if self.blocked {
            if let Some(job) = &mut self.job {
                //println!("Job {} - Status: {:?}, Time: {}", job.id, job.computation.status, time);
                job.advance_job_time();
                if job.computation.status == JobStatus::Completed {
                    //println!("Job {} completed at Time: {}", job.id, time);
                    self.unblock();
                    self.remove_job();
                    // Prioritize the job queue // Todo: Create some priority system?
                    self.add_job_from_queue();
                }
            }
        }
        // Unblocked processor will wait for a job or try to put a job from the job queue
        if !self.blocked {
            if self.job.is_none() {
                self.add_job_from_queue();
            }
            self.start_job();
        }
        // Always check for missed deadlines from any job in the processor, whether blocked or not,
        // also whether current process or from the job queue // TODO: Async?
        if let Some(job) = &mut self.job {
            if job.check_deadline(time) {
                //println!("Job {} missed deadline at Time: {}", job.id, time);
                self.unblock();
                self.remove_job();
                // Prioritize the job queue // Todo: Create some priority system?
                //self.add_job_from_queue();
            }
        }
        let mut indices_to_remove = Vec::new();

        // In your loop over job_queue.jobs:
        if let Some(job_queue) = &mut self.job_queue {
            // First, identify jobs that miss deadlines
            for (idx, job) in job_queue.jobs.iter_mut().enumerate() {
                if job.check_deadline(time) {
                    // Just mark for removal, don't modify collection yet
                    indices_to_remove.push((idx, job.clone()));
                }
            }

            // Now process missed jobs and remove them from queue
            for (_, job) in indices_to_remove.iter().rev() {
                if let Some(job_history) = &mut self.job_history{
                    job_history.add(job.clone());
                }
                else{
                    self.job_history = Some(JobHistory{done_jobs: vec![job.clone()]});
                }
            }

            // Remove jobs in reverse order to maintain correct indices
            indices_to_remove.sort_by(|a, b| b.0.cmp(&a.0)); // Sort in descending order by index
            for (idx, _) in indices_to_remove {
                job_queue.jobs.remove(idx);
            }
        }
    }
    
    
    // Debug print the processor unit details
    #[allow(dead_code)]
    pub(crate) fn debug_print(&self, include_job_details: bool, include_history: bool){
        //println!("Processor Unit: {}", self.id.to_string().green());
        if let Some(job) = &self.job{
            println!("Job: {}", job.id.to_string().blue());
            if include_job_details{
                job.debug_print();
            }
        }
        if let Some(job_queue) = &self.job_queue{
            //println!("Job Queue Size: {}", job_queue.size.to_string().yellow());
            for job in &job_queue.jobs{
                println!("Job: {}", job.id.to_string().blue());
                if include_job_details{
                    job.debug_print();
                }
            }
        }
        if self.blocked{
            //println!("Processor Status: {}", "Blocked".red());
        }
        else{
            //println!("Processor Status: {}", "Unblocked".green());
        }
        // Print the job history
        if let Some(job_history) = &self.job_history{
            //println!("{}", "Job History:".cyan().bold().italic());
            if include_history{
                for job in &job_history.done_jobs{
                    println!("Job: {}", job.id.to_string().blue());
                    if include_job_details{
                        job.debug_print();
                    }
                }
            }
        }
        // Panic if amount of jobs in the history does not match the amount of jobs in total (number of cycles * number of tasks)
        if include_history && let Some(job_history) = &self.job_history {
            if let Some(expected_jobs) = std::env::args().find_map(|arg| {
                if arg.starts_with("--expected-jobs=") {
                    arg.strip_prefix("--expected-jobs=").and_then(|s| s.parse::<usize>().ok())
                } else {
                    None
                }
            }) {
                if job_history.done_jobs.len() != expected_jobs {
                    panic!("Mismatch between job history ({}) and expected jobs ({})",
                           job_history.done_jobs.len(), expected_jobs);
                }
            }
        }
    }
}
