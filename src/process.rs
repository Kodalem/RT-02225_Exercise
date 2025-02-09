
// Importing the necessary libraries, i.e. job.rs and task.rs

use crate::job::{Job, JobStatus};
use colored::*;

struct JobHistory{
    done_jobs: Vec<Job>,
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

struct JobQueue{
    size: usize, // The size of the job queue
    jobs: Vec<Job>,
}
impl JobQueue {
    fn new() -> Self{
        Self{
            size: 0,
            jobs: Vec::new(),
        }
    }
}

pub(crate) struct ProcessorUnit{
    id: usize, // Unique identifier
    blocked: bool, // Is the processor blocked
    pub(crate) job: Option<Job>, // The job that is currently being processed
    job_queue: Option<JobQueue>, // The job queue
    clock_period: usize, // The clock period of the processor
    job_history: Option<JobHistory>, // The history of the jobs that have been processed
}

impl ProcessorUnit{
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
        if let Some(job) = &self.job{
            if let Some(job_queue) = &mut self.job_queue{
                job_queue.jobs.push(job.clone());
                job_queue.size += 1;
            }
            else {
                self.job_queue = Some(JobQueue{size: 1, jobs: vec![job.clone()]});
            }
        }
        self.job = Some(job);
    }
    
    // Only remove the job which is/was being processed
    fn remove_job(&mut self){
        // Add the job to the job history
        if let Some(job) = &self.job{
            println!("Job: {} completed", job.id.to_string().blue());
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
        // Blocked processor will only compute the job which is being processed
        if self.blocked {
            if let Some(job) = &mut self.job {
                job.advance_job_time();
                if job.computation.status == JobStatus::Completed {
                    self.unblock();
                    self.remove_job();
                    // Prioritize the job queue // Todo: Create some priority system?
                    self.add_job_from_queue();
                }
            }
        }
        // Unblocked processor will wait for a job or try to put a job from the job queue
        if !self.blocked {
            self.add_job_from_queue();
            self.start_job();
        }
        // Always check for missed deadlines from any job in the processor, whether blocked or not,
        // also whether current process or from the job queue // TODO: Async?
        if let Some(job) = &mut self.job {
            job.check_deadline(time);
        }
        if let Some(ref mut job_queue) = self.job_queue {
            for job in &mut job_queue.jobs {
                job.check_deadline(time);
            }
        }
    }
    
    // Debug print the processor unit details
    pub(crate) fn debug_print(&self, include_job_details: bool, include_history: bool){
        println!("Processor Unit: {}", self.id.to_string().green());
        if let Some(job) = &self.job{
            println!("Job: {}", job.id.to_string().blue());
            if include_job_details{
                job.debug_print();
            }
        }
        if let Some(job_queue) = &self.job_queue{
            println!("Job Queue Size: {}", job_queue.size.to_string().yellow());
            for job in &job_queue.jobs{
                println!("Job: {}", job.id.to_string().blue());
                if include_job_details{
                    job.debug_print();
                }
            }
        }
        if self.blocked{
            println!("Processor Status: {}", "Blocked".red());
        }
        else{
            println!("Processor Status: {}", "Unblocked".green());
        }
        // Print the job history
        if let Some(job_history) = &self.job_history{
            println!("{}", "Job History:".cyan().bold().italic());
            if include_history{
                for job in &job_history.done_jobs{
                    println!("Job: {}", job.id.to_string().blue());
                    if include_job_details{
                        job.debug_print();
                    }
                }
            }
        }
    }
}

struct Processor{
    id: usize, // Unique identifier
    processor_units: Vec<ProcessorUnit>, // The processor units
}