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

struct Stats {
    total_log_lines: u64,
    char_counts: HashMap<char, u64>,
    consecutive_char_counts: HashMap<(char, char), u64>,
    individual_key_counts: HashMap<Keycode, u64>,
    consectutive_key_counts: HashMap<(Keycode, Keycode), u64>,
    simultaneous_key_counts: HashMap<Vec<Keycode>, u64>,
}

impl Stats {
    fn new() -> Self {
        Self {
            total_log_lines: 0,
            char_counts: HashMap::new(),
            consecutive_char_counts: HashMap::new(),
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

    let mut prev_char = None;

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

            let shift_held = keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift);
            for key in keys.iter() {
                let c = match (key, shift_held) {
                    (Keycode::A, true) => 'A',
                    (Keycode::B, true) => 'B',
                    (Keycode::C, true) => 'C',
                    (Keycode::D, true) => 'D',
                    (Keycode::E, true) => 'E',
                    (Keycode::F, true) => 'F',
                    (Keycode::G, true) => 'G',
                    (Keycode::H, true) => 'H',
                    (Keycode::I, true) => 'I',
                    (Keycode::J, true) => 'J',
                    (Keycode::K, true) => 'K',
                    (Keycode::L, true) => 'L',
                    (Keycode::M, true) => 'M',
                    (Keycode::N, true) => 'N',
                    (Keycode::O, true) => 'O',
                    (Keycode::P, true) => 'P',
                    (Keycode::Q, true) => 'Q',
                    (Keycode::R, true) => 'R',
                    (Keycode::S, true) => 'S',
                    (Keycode::T, true) => 'T',
                    (Keycode::U, true) => 'U',
                    (Keycode::V, true) => 'V',
                    (Keycode::W, true) => 'W',
                    (Keycode::X, true) => 'X',
                    (Keycode::Y, true) => 'Y',
                    (Keycode::Z, true) => 'Z',
                    (Keycode::A, false) => 'a',
                    (Keycode::B, false) => 'b',
                    (Keycode::C, false) => 'c',
                    (Keycode::D, false) => 'd',
                    (Keycode::E, false) => 'e',
                    (Keycode::F, false) => 'f',
                    (Keycode::G, false) => 'g',
                    (Keycode::H, false) => 'h',
                    (Keycode::I, false) => 'i',
                    (Keycode::J, false) => 'j',
                    (Keycode::K, false) => 'k',
                    (Keycode::L, false) => 'l',
                    (Keycode::M, false) => 'm',
                    (Keycode::N, false) => 'n',
                    (Keycode::O, false) => 'o',
                    (Keycode::P, false) => 'p',
                    (Keycode::Q, false) => 'q',
                    (Keycode::R, false) => 'r',
                    (Keycode::S, false) => 's',
                    (Keycode::T, false) => 't',
                    (Keycode::U, false) => 'u',
                    (Keycode::V, false) => 'v',
                    (Keycode::W, false) => 'w',
                    (Keycode::X, false) => 'x',
                    (Keycode::Y, false) => 'y',
                    (Keycode::Z, false) => 'z',
                    (Keycode::Key1, true) => '!',
                    (Keycode::Key2, true) => '@',
                    (Keycode::Key3, true) => '#',
                    (Keycode::Key4, true) => '$',
                    (Keycode::Key5, true) => '%',
                    (Keycode::Key6, true) => '^',
                    (Keycode::Key7, true) => '&',
                    (Keycode::Key8, true) => '*',
                    (Keycode::Key9, true) => '(',
                    (Keycode::Key0, true) => ')',
                    (Keycode::Key1, false) => '1',
                    (Keycode::Key2, false) => '2',
                    (Keycode::Key3, false) => '3',
                    (Keycode::Key4, false) => '4',
                    (Keycode::Key5, false) => '5',
                    (Keycode::Key6, false) => '6',
                    (Keycode::Key7, false) => '7',
                    (Keycode::Key8, false) => '8',
                    (Keycode::Key9, false) => '9',
                    (Keycode::Key0, false) => '0',
                    (Keycode::Space, _) => ' ',
                    (Keycode::Comma, true) => '<',
                    (Keycode::Comma, false) => ',',
                    (Keycode::Dot, true) => '>',
                    (Keycode::Dot, false) => '.',
                    (Keycode::Slash, true) => '?',
                    (Keycode::Slash, false) => '/',
                    (Keycode::Semicolon, true) => ':',
                    (Keycode::Semicolon, false) => ';',
                    (Keycode::Apostrophe, true) => '"',
                    (Keycode::Apostrophe, false) => '\'',
                    (Keycode::LeftBracket, true) => '{',
                    (Keycode::LeftBracket, false) => '[',
                    (Keycode::RightBracket, true) => '}',
                    (Keycode::RightBracket, false) => ']',
                    (Keycode::BackSlash, true) => '|',
                    (Keycode::BackSlash, false) => '\\',
                    (Keycode::Minus, true) => '_',
                    (Keycode::Minus, false) => '-',
                    (Keycode::Equal, true) => '+',
                    (Keycode::Equal, false) => '=',
                    (Keycode::Grave, true) => '~',
                    (Keycode::Grave, false) => '`',
                    _ => continue,
                };
                let count = stats.char_counts.entry(c).or_insert(0);
                *count += 1;

                if let Some(prev_char) = prev_char {
                    let count = stats
                        .consecutive_char_counts
                        .entry((prev_char, c))
                        .or_insert(0);
                    *count += 1;
                }
                prev_char = Some(c);
            }
        }
    }

    stats
}
