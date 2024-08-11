use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use device_query::Keycode;

fn main() {
    let start = std::time::Instant::now();

    let mut log_lines_read = 0;

    let mut individual_key_counts = HashMap::new();
    let mut consectutive_key_counts = HashMap::new();
    let mut simultaneous_key_counts = HashMap::new();

    let mut keys = HashSet::new();

    let reader = BufReader::new(File::open("../keylogger/keys.log").unwrap());

    for line in reader.lines() {
        let prev_keys = keys.clone();
        let line = line.unwrap();
        log_lines_read += 1;
        let (key_code, press) = line.split_once(" ").unwrap();
        let key_code = Keycode::from_str(key_code).unwrap();
        let press = press == "1";
        if press {
            keys.insert(key_code);
            let count = individual_key_counts.entry(key_code).or_insert(0);
            *count += 1;
            for prev in prev_keys {
                let count = consectutive_key_counts.entry((prev, key_code)).or_insert(0);
                *count += 1;
            }
        } else {
            keys.remove(&key_code);
        }

        if !keys.is_empty() {
            let mut held: Vec<_> = keys.iter().cloned().collect();
            held.sort_by_key(|x| x.to_string());
            let count = simultaneous_key_counts.entry(held).or_insert(0);
            *count += 1;
        }
    }

    println!("Individual key counts:");
    for (key, count) in individual_key_counts {
        println!("{:?}: {}", key, count);
    }

    println!("\nConsecutive key counts:");
    for (keys, count) in consectutive_key_counts {
        println!("{:?} -> {:?}: {}", keys.0, keys.1, count);
    }

    println!("\nSimultaneous key counts:");
    for (keys, count) in simultaneous_key_counts {
        println!("{:?}: {}", keys, count);
    }

    let elapsed = start.elapsed();
    println!(
        "\nProcessed {} log lines in {}.{:03} seconds",
        log_lines_read,
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    println!(
        "{} lines per second",
        log_lines_read as f64 / elapsed.as_secs_f64()
    );
}
