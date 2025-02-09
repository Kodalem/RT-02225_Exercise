use crate::job::{Deadline, Job};
use colored::*;
use rand::Rng;

pub(crate) struct Task {
    pub(crate) id: usize,                        // Unique identifier
    pub(crate) worst_case_execution_time: usize, // The worst case execution time of the task
    best_case_execution_time: usize,             // The best case execution time of the task
    pub(crate) relative_deadline: usize,         // The relative deadline of the task
    pub(crate) period: usize,                    // The period of the task
    phase: usize,                                // The phase of the task

    // List of jobs that the task has
    pub(crate) jobs: Option<Vec<Job>>,
}
impl Task {
    pub(crate) fn new(
        id: usize,
        worst_case_execution_time: usize,
        best_case_execution_time: usize,
        relative_deadline: usize,
        period: usize,
    ) -> Self {
        Self {
            id,
            worst_case_execution_time,
            best_case_execution_time,
            relative_deadline,
            period,
            phase: 0,
            jobs: None,
        }
    }
    fn get_job_by_id(&self, id: usize) -> Option<&Job> {
        // Check if the jobs list is empty
        if let Some(jobs) = &self.jobs {
            // Use binary search to find the job
            return binary_search_job(jobs, id, "id");
        }
        None
    }
    fn get_job_by_instance(&self, instance: usize) -> Option<&Job> {
        // Check if the jobs list is empty
        if let Some(jobs) = &self.jobs {
            // Binary search to find the job
            return binary_search_job(jobs, instance, "instance");
        }
        None
    }
    fn calculate_absolute_deadline_for_job(&self, job: &mut Job) {
        // Check if release time value exists for the job then use one equation, else use another
        if let Some(release_time) = job.release_time {
            // Set the absolute deadline for the job
            job.set_absolute_deadline(release_time + self.period);
        } else {
            // Check if the job has an instanced into some order
            if let Some(instance) = job.instance {
                // Set the absolute deadline for the job
                job.set_absolute_deadline(self.phase + self.period * instance);
            } else {
                panic!("Job has been left uninstanced!")
            }
        }
    }
    fn calculate_release_time_for_job(&self, job: &mut Job) {
        // Check if the job has an instance into some order
        if let Some(instance) = job.instance {
            // Set the release time for the job
            job.set_release_time(self.phase + self.period * (instance - 1));
        } else {
            panic!("Job has been left uninstanced!")
        }
    }

    fn get_random_execution_time(&self) -> usize {
        // Generate a random number between the worst case and best case execution time
        rand::rng().random_range(self.best_case_execution_time..self.worst_case_execution_time)
    }

    pub(crate) fn randomly_generate_jobs(&mut self, num_jobs: usize) {
        // Initialize the jobs list
        let mut jobs = Vec::new();
        // Loop through the number of jobs
        for i in 1..num_jobs+1 {
            // Create a new job
            let mut job = Job::new(i, self.get_random_execution_time(), Deadline::Hard);
            // Set the instance of the job
            job.set_instance(i);
            // Calculate the absolute deadline for the job
            self.calculate_absolute_deadline_for_job(&mut job);
            // Calculate the release time for the job
            self.calculate_release_time_for_job(&mut job); // Push the job into the jobs list
            jobs.push(job);
        }
        // Set the jobs list for the task
        self.jobs = Some(jobs);
    }

    // Debug print the task details
    pub(crate) fn debug_print(&self, include_jobs: bool) {
        println!("  Task ID: {}", self.id.to_string().cyan());
        println!(
            "   Best Case Execution Time: {}",
            self.best_case_execution_time.to_string().green()
        );
        println!(
            "   Worst Case Execution Time: {}",
            self.worst_case_execution_time.to_string().red()
        );
        println!(
            "   Relative Deadline: {}",
            self.relative_deadline.to_string().green()
        );
        println!("  Period: {}", self.period.to_string().green());
        println!("  Phase: {}", self.phase.to_string().green());
        if include_jobs {
            if let Some(jobs) = &self.jobs {
                for job in jobs {
                    job.debug_print();
                }
            }
        }
    }
}

// Binary search helper function to optimize the search for the job, either by id or instance number
fn binary_search_job<'a>(jobs: &'a Vec<Job>, value: usize, search_by: &str) -> Option<&'a Job> {
    // Initialize the start and end index
    let mut start = 0;
    let mut end = jobs.len() - 1;

    // Loop until the start index is less than or equal to the end index
    while start <= end {
        // Calculate the mid-index
        let mid = start + (end - start) / 2;

        // Check if the job id or instance is equal to the mid-index job id or instance
        let mid_value = match search_by {
            "id" => jobs[mid].id,
            "instance" => jobs[mid].instance.unwrap_or(usize::MAX), // Assuming instance is an Option<usize>
            _ => panic!("Invalid search_by value"),
        };

        if mid_value == value {
            return Some(&jobs[mid]);
        } else if mid_value < value {
            start = mid + 1;
        } else {
            end = mid - 1;
        }
    }
    None
}
