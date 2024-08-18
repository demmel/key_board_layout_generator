use clap::Parser;
use keyboard_layout_generator::stats::{process_log, Stats};

#[derive(Parser)]
struct Args {
    log_file: String,
}

fn main() {
    let start = std::time::Instant::now();

    let args = Args::parse();

    let stats = process_log(&args.log_file);
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

    let mut char_counts: Vec<_> = stats.char_counts.iter().collect();
    char_counts.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\nCharacter counts:");
    for (c, count) in char_counts {
        println!("{}: {}", c, count);
    }

    let mut consecutive_char_counts: Vec<_> = stats.consecutive_char_counts.iter().collect();
    consecutive_char_counts.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\nConsecutive character counts:");
    for (chars, count) in consecutive_char_counts {
        println!("{:?} -> {:?}: {}", chars.0, chars.1, count);
    }
}
