
#[derive(Debug, Clone)]
pub(crate) enum Deadline {
    Hard,
    Soft,
    Firm,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JobStatus{
    NotStarted,
    InProgress,
    Completed,
    Missed,
}

#[derive(Debug, Clone)]
pub struct Compuation{
    execution_time: usize, // Time it takes to execute the job
    completed_compuation_time: usize, // Time it took to complete the job
    pub(crate) status: JobStatus,
}
impl Compuation{
    fn new(execution_time: usize) -> Self{
        Self{
            execution_time,
            completed_compuation_time: 0,
            status: JobStatus::NotStarted,
        }
    }
    fn start(&mut self){
        self.status = JobStatus::InProgress;
    }
    fn complete(&mut self){
        self.status = JobStatus::Completed;
    }
    fn do_computation(&mut self){
       if self.status == JobStatus::InProgress{
           self.completed_compuation_time += 1;
              if self.completed_compuation_time >= self.execution_time{
                self.complete();
              }
        }
        else {
            // Todo: Handle this error properly - Maybe do a soft fail
            panic!("Cannot do computation on a job that is not in progress");
        }
    }
    fn failed(&mut self){
        self.status = JobStatus::Missed;
    }

}

#[derive(Debug, Clone)]
pub(crate) struct Job {
    pub(crate) id: usize, // Unique identifier // Todo: Remove this? Replace with instance? UUID?
    pub(crate) computation: Compuation, // The computation details
    deadline_type: Deadline, // The deadline for the job
    absolute_deadline: Option<usize>, // The absolute deadline for the job
    pub(crate) release_time: Option<usize>, // The release time for the job
    pub(crate) instance: Option<usize>, // The instance of the job
}
impl Job {
    pub(crate) fn new(id: usize, execution_time: usize, deadline_type: Deadline) -> Self{
        Self{
            id,
            computation: Compuation::new(execution_time),
            deadline_type,
            absolute_deadline: None,
            release_time: None,
            instance: None,
        }
    }
    pub(crate) fn set_absolute_deadline(&mut self, absolute_deadline: usize){
        self.absolute_deadline = Some(absolute_deadline);
    }
    pub(crate) fn set_release_time(&mut self, release_time: usize){
        self.release_time = Some(release_time);
    }
    pub(crate) fn set_instance(&mut self, instance: usize){
        self.instance = Some(instance);
    }
    pub(crate) fn start(&mut self){
        self.computation.start();
    }
    fn complete(&mut self){
        self.computation.complete();
    }
    fn do_computation(&mut self){
        // Do computation if the job is in progress
        if self.computation.status == JobStatus::InProgress{
            self.computation.do_computation();
        }
        else {
            
        }
    }
    fn resolve_deadline(&mut self) {
        self.computation.failed();
        self.computation.status = JobStatus::Missed;
        // Todo - Write notification to the user
    }
    
    pub(crate) fn advance_job_time(&mut self) {
        self.computation.do_computation();
    }
    
    
    pub(crate) fn check_deadline(&mut self, current_time: usize){
        if let Some(absolute_deadline) = self.absolute_deadline{
            if current_time >= absolute_deadline{
                self.resolve_deadline();
            }
        }
    }
}