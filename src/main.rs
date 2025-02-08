
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
        task::Task::new(1, 10, 5, 20, 40),
        task::Task::new(2, 20, 10, 40, 80),
    ]);
    // Run the simulator
    simulator.run();
}