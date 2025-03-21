# Real-Time Task Simulator

A simulator for real-time task scheduling and analysis.

## Building and Running

```bash
# Clone the repository
git clone <repository-url>
cd <repository-directory>

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

### Interface Navigation
- Press `s` to switch to Simulator screen
- Press `a` for Analysis screen
- Press `p` for Properties screen
- Press `q` to exit

### Importing Tasks
1. Press `p` to navigate to Properties screen
2. Press `e` to enter edit mode
3. Enter a URL to a raw CSV file
4. Press `Enter` to import

### CSV Format
```
Task,BCET,WCET,Period,Deadline,Priority
T1,1,3,40,40,1
T2,2,7,80,80,2
```

### Running Simulation
1. After importing tasks, enter the number of cycles
2. Press `g` to generate jobs
3. Press `Space` to run the simulation

### Example URLs
- Use GitHub raw content URLs: `https://raw.githubusercontent.com/username/repo/branch/path/to/file.csv`
- For testing, you can use services like Gist or Pastebin that provide raw file access

## License
See the [LICENSE](LICENSE) file for details.