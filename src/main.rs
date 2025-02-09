
// Include the process.rs from rt-props folder

mod analysis_functions;
mod process;
mod task;
mod job;
mod simulator_variables;

fn main() {
    // Simulate the system of just only two tasks and 2 cycles
    let mut simulator = simulator_variables::SimulatorVariables::new(2, 2, 1000);
    // Set the list of tasks
    simulator.set_list_of_tasks(vec![
        task::Task::new(1, 10, 5,
                        20, 40),
        task::Task::new(2, 20, 10,
                        40, 40),
    ]);
    // Generate jobs for the tasks
    simulator.generate_jobs();
    
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
}