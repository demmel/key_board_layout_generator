use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use device_query::Keycode;

fn main() {
    let start = std::time::Instant::now();

    let stats = process_log("../keylogger/keys.log");
    print_statistics(&stats);

    let elapsed = start.elapsed();

    println!(
        "\nProcessed {} log lines in {}.{:03} seconds",
        stats.total_log_lines,
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    println!(
        "{} lines per second",
        stats.total_log_lines as f64 / elapsed.as_secs_f64()
    );
}

fn print_statistics(stats: &Stats) {
    let mut individual_key_counts: Vec<_> = stats.individual_key_counts.iter().collect();
    individual_key_counts.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("Individual key counts:");
    for (key, count) in individual_key_counts {
        println!("{:?}: {}", key, count);
    }

    let mut consectutive_key_counts: Vec<_> = stats.consectutive_key_counts.iter().collect();
    consectutive_key_counts.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\nConsecutive key counts:");
    for (keys, count) in consectutive_key_counts {
        println!("{:?} -> {:?}: {}", keys.0, keys.1, count);
    }

    let mut simultaneous_key_counts: Vec<_> = stats.simultaneous_key_counts.iter().collect();
    simultaneous_key_counts.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\nSimultaneous key counts:");
    for (keys, count) in simultaneous_key_counts {
        println!("{:?}: {}", keys, count);
    }
}

struct Stats {
    total_log_lines: u64,
    individual_key_counts: HashMap<Keycode, u64>,
    consectutive_key_counts: HashMap<(Keycode, Keycode), u64>,
    simultaneous_key_counts: HashMap<Vec<Keycode>, u64>,
}

impl Stats {
    fn new() -> Self {
        Self {
            total_log_lines: 0,
            individual_key_counts: HashMap::new(),
            consectutive_key_counts: HashMap::new(),
            simultaneous_key_counts: HashMap::new(),
        }
    }
}

fn process_log(path: &str) -> Stats {
    let mut stats = Stats::new();

    let mut prev_keys = HashSet::new();
    let mut keys = HashSet::new();

    let reader = BufReader::new(File::open(path).unwrap());

    for line in reader.lines() {
        stats.total_log_lines += 1;
        let line = line.unwrap();
        let (key_code, press) = line.split_once(" ").unwrap();
        let key_code = Keycode::from_str(key_code).unwrap();
        let press = press == "1";
        if press {
            keys.insert(key_code);
            let count = stats.individual_key_counts.entry(key_code).or_insert(0);
            *count += 1;
            for prev in prev_keys {
                let count = stats
                    .consectutive_key_counts
                    .entry((prev, key_code))
                    .or_insert(0);
                *count += 1;
            }
            prev_keys = keys.clone();
        } else {
            keys.remove(&key_code);
        }

        if keys.len() > 1 {
            let mut held: Vec<_> = keys.iter().cloned().collect();
            held.sort_by_key(|x| x.to_string());
            let count = stats.simultaneous_key_counts.entry(held).or_insert(0);
            *count += 1;
        }
    }

    stats
}
