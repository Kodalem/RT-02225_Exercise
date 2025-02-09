
// Include the process.rs, job.rs, and task.rs files
use crate::task::{Task};


// Todo: Unwrangle the usize spaghetti
pub(crate) fn response_time_analysis(tasks: &Vec<Task>) -> Vec<&Task> {
    let mut schedulable_tasks = Vec::new();

    for task in tasks {
        let mut longest_response_time_at_previous = 0;
        let mut longest_response_time_at_critical_instant = task.worst_case_execution_time;

        while longest_response_time_at_critical_instant > longest_response_time_at_previous {
            if longest_response_time_at_previous == 0 {
                // Todo Rewrite this line
                //longest_response_time_at_previous = find_task_highest_response_time(&tasks, &task, longest_response_time_at_previous);
            }
            longest_response_time_at_previous = longest_response_time_at_critical_instant;

            let mut interference_measurement = 0;
            for higher_priority_task in tasks {
                if higher_priority_task.id == task.id {
                    continue;
                }

                let division_result = longest_response_time_at_previous as f64 / higher_priority_task.period as f64;
                let ceiling_result = division_result.ceil() as i64;
                let interference_contribution = ceiling_result * higher_priority_task.worst_case_execution_time as i64;

                interference_measurement += interference_contribution;
            }

            longest_response_time_at_critical_instant = task.worst_case_execution_time + interference_measurement as usize;

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