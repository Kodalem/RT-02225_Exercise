use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use crate::task::Task;

pub(crate) fn parse_csv_to_tasks<P: AsRef<Path>>(path: P) -> Result<Vec<Task>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    let mut is_header = true;

    for line in reader.lines() {
        let line = line?;

        // Skip header row
        if is_header {
            is_header = false;
            continue;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse CSV line
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 5 {
            let task_name = parts[0].trim();

            // Parse numeric values, using unwrap_or for error handling
            let wcet = parts[2].trim().parse::<usize>().unwrap_or(0);
            let bcet = parts[1].trim().parse::<usize>().unwrap_or(0);
            let period = parts[3].trim().parse::<usize>().unwrap_or(0);
            let deadline = parts[4].trim().parse::<usize>().unwrap_or(0);

            // Extract the task ID from the name (e.g., "T1" -> 1)
            let id = if task_name.starts_with('T') && task_name.len() > 1 {
                task_name[1..].parse::<usize>().unwrap_or(tasks.len() + 1)
            } else {
                tasks.len() + 1
            };

            // Create and add the task
            let task = Task::new(id, wcet, bcet, deadline, period);
            tasks.push(task);
        }
    }

    Ok(tasks)
}

// Function to download and parse tasks from a URL
#[tokio::main]
pub(crate) async fn download_and_parse_tasks(url: &str) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    // Create a temporary directory to store the downloaded file
    let tmp_dir = tempfile::Builder::new().prefix("temp_dir").tempdir()?;

    // Download the file
    let response = reqwest::get(url).await?;
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tasks.csv");

    let file_path = tmp_dir.path().join(fname);
    let mut file = File::create(&file_path)?;

    // Write the content to the file
    let content = response.bytes().await?;
    file.write_all(&content)?;

    // Parse the downloaded CSV file
    let tasks = parse_csv_to_tasks(file_path)?;

    Ok(tasks)
}

// Function to set up simulator with tasks from a URL

// Allow dead code
#[allow(dead_code)]
pub(crate) fn setup_simulator_from_url(simulator: &mut crate::simulator_variables::SimulatorVariables, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = download_and_parse_tasks(url)?;

    // Set the tasks in the simulator
    simulator.set_list_of_tasks(tasks);

    Ok(())
}