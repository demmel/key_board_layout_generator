pub mod layout_format;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use device_query::Keycode;
use serde::{Deserialize, Serialize};

pub struct Stats {
    pub total_log_lines: u64,
    pub char_counts: HashMap<char, u64>,
    pub consecutive_char_counts: HashMap<(char, char), u64>,
    pub individual_key_counts: HashMap<Keycode, u64>,
    pub consectutive_key_counts: HashMap<(Keycode, Keycode), u64>,
    pub simultaneous_key_counts: HashMap<Vec<Keycode>, u64>,
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

pub fn process_log(path: &str) -> Stats {
    let mut stats = Stats::new();
    let mut key_processor = KeyProcessor::new();

    let reader = BufReader::new(File::open(path).unwrap());

    for line in reader.lines() {
        stats.total_log_lines += 1;
        let line = line.unwrap();
        let (key_code, press) = line.split_once(" ").unwrap();
        let key_code = Keycode::from_str(key_code).unwrap();
        let press = press == "1";
        key_processor.process_key(key_code, press, &mut stats);
    }

    stats
}

struct KeyProcessor {
    prev_keys: HashSet<Keycode>,
    keys: HashSet<Keycode>,
    prev_char: Option<char>,
}

impl KeyProcessor {
    fn new() -> Self {
        Self {
            prev_keys: HashSet::new(),
            keys: HashSet::new(),
            prev_char: None,
        }
    }

    fn process_key(&mut self, key_code: Keycode, press: bool, stats: &mut Stats) {
        {}
        if press {
            self.keys.insert(key_code);
            let count = stats.individual_key_counts.entry(key_code).or_insert(0);
            *count += 1;
            let prev_keys = std::mem::replace(&mut self.prev_keys, self.keys.clone());
            for prev in prev_keys {
                let count = stats
                    .consectutive_key_counts
                    .entry((prev, key_code))
                    .or_insert(0);
                *count += 1;
            }
            let shift_held =
                self.keys.contains(&Keycode::LShift) || self.keys.contains(&Keycode::RShift);
            if let Some(c) = translate_key_to_char(&key_code, shift_held) {
                let count = stats.char_counts.entry(c).or_insert(0);
                *count += 1;

                if let Some(prev_char) = self.prev_char {
                    let count = stats
                        .consecutive_char_counts
                        .entry((prev_char, c))
                        .or_insert(0);
                    *count += 1;
                }
                self.prev_char = Some(c);
            }
        } else {
            self.keys.remove(&key_code);
        }

        if self.keys.len() > 1 {
            let mut held: Vec<_> = self.keys.iter().cloned().collect();
            held.sort_by_key(|x| x.to_string());
            let count = stats.simultaneous_key_counts.entry(held).or_insert(0);
            *count += 1;
        }
    }
}

fn translate_key_to_char(key: &Keycode, shift_held: bool) -> Option<char> {
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
        _ => return None,
    };
    Some(c)
}

#[derive(Debug)]
pub struct KeymapConfig {
    pub fingers: Vec<FingerConfig>,
    pub keys: PhysicalKeyboard,
}

#[derive(Debug)]
pub struct PhysicalKeyboard(Vec<PhysicalKey>);

impl PhysicalKeyboard {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_key(&mut self, key: PhysicalKey) {
        self.0.push(key);
    }

    pub fn keys(&self) -> &[PhysicalKey] {
        &self.0
    }
}

#[derive(Debug)]
pub struct PhysicalKey {
    pub code: Keycode,
    pub finger: Finger,
    pub score: f64,
    pub position: (f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Finger {
    pub hand: Hand,
    pub finger: FingerKind,
}

impl Finger {
    pub fn all() -> Vec<Self> {
        let mut fingers = vec![];
        for &hand in &[Hand::Left, Hand::Right] {
            for &finger in &[
                FingerKind::Pinky,
                FingerKind::Ring,
                FingerKind::Middle,
                FingerKind::Index,
                FingerKind::Thumb,
            ] {
                fingers.push(Finger { hand, finger });
            }
        }
        fingers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FingerKind {
    Pinky,
    Ring,
    Middle,
    Index,
    Thumb,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FingerConfig {
    pub finger: Finger,
    pub score: f64,
}
