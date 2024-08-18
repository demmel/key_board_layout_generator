use clap::Parser;
use device_query::Keycode;
use genetic::{Crossover, DiversifyStrategy, Gen, Mutate};
use keyboard_layout_generator::{
    layout_format::parse_keymap_config,
    stats::{process_log, Stats},
    Finger, FingerKind, KeymapConfig, PhysicalKey,
};
use rand::seq::SliceRandom;

#[derive(Parser)]
struct Args {
    log_file: String,
    keymap_config: String,
}

fn main() {
    let args = Args::parse();
    let stats = process_log(&args.log_file);
    println!("Max Possible Score: {}", max_possible_score(&stats));
    let keymap_str = std::fs::read_to_string(&args.keymap_config).unwrap();
    let keymap_config = parse_keymap_config(&keymap_str);
    let mut rng = rand::thread_rng();
    let mut population = (0..1000)
        .map(|_| Layout::gen(&mut rng, &keymap_config))
        .collect::<Vec<_>>();
    let mut generation = 0;
    loop {
        let (new_population, stats) = genetic::evolve(
            &mut population,
            &keymap_config,
            |l| layout_score(l, &stats, &keymap_config) as f32,
            |l1, l2| layout_similarity(l1, l2),
            DiversifyStrategy::None,
        );
        population = new_population;
        println!(
            "Generation: {}, Mean: {}, Max: {}, Min: {}, Std Dev: {}, Diversity: {}",
            generation, stats.mean, stats.max, stats.min, stats.std_dev, stats.diversity
        );
        generation += 1;
    }
}

fn max_possible_score(stats: &Stats) -> f64 {
    let mut score = 0.0;
    for count in stats.individual_key_counts.values() {
        score += *count as f64;
    }
    for count in stats.consectutive_key_counts.values() {
        score += *count as f64;
    }
    score * 100.0
}

fn layout_similarity(l1: &Layout, l2: &Layout) -> f32 {
    let mut score = 0.0;
    for (key1, key2) in l1.keys.iter().zip(l2.keys.iter()) {
        let key1_code = key1.keycode(false);
        let key2_code = key2.keycode(false);
        if key1_code == key2_code {
            score += 1.0;
        }
        let shift_key1_code = key1.keycode(true);
        let shift_key2_code = key2.keycode(true);
        if shift_key1_code == shift_key2_code {
            score += 1.0;
        }
    }
    score / (l1.keys.len() * 2) as f32
}

fn layout_score(layout: &Layout, stats: &Stats, keymap_config: &KeymapConfig) -> f64 {
    let individual_key_score = layout_individual_key_score(layout, stats, keymap_config);
    let consecutive_key_score = layout_consecutive_key_score(layout, stats, keymap_config);

    individual_key_score + consecutive_key_score
}

fn layout_consecutive_key_score(
    layout: &Layout,
    stats: &Stats,
    keymap_config: &KeymapConfig,
) -> f64 {
    let mut score = 0.0;
    for ((i, key1), (j, key2)) in layout
        .keys
        .iter()
        .enumerate()
        .zip(layout.keys.iter().enumerate().skip(1))
    {
        let key1_code = key1.keycode(false);
        let key2_code = key2.keycode(false);
        let pkey1 = &keymap_config.keys.keys()[i];
        let pkey2 = &keymap_config.keys.keys()[j];
        let count = stats
            .consectutive_key_counts
            .get(&(key1_code, key2_code))
            .unwrap_or(&0);
        let distance = distance(&pkey1, &pkey2);
        score += *count as f64 * consecutive_finger_score(pkey1.finger, pkey1.finger, distance);
    }
    score
}

fn distance(key1: &PhysicalKey, key2: &PhysicalKey) -> f64 {
    let key1_pos = key1.position;
    let key2_pos = key2.position;
    ((key1_pos.0 - key2_pos.0).powi(2) + (key1_pos.1 - key2_pos.1).powi(2)).sqrt()
}

fn layout_individual_key_score(
    layout: &Layout,
    stats: &Stats,
    keymap_config: &KeymapConfig,
) -> f64 {
    let mut score = 0.0;
    for (key, config) in layout.keys.iter().zip(keymap_config.keys.keys().iter()) {
        let count = stats
            .individual_key_counts
            .get(&key.keycode(false))
            .unwrap_or(&0);
        score += config.score
            * keymap_config
                .fingers
                .iter()
                .find(|c| c.finger == config.finger)
                .unwrap()
                .score
            * *count as f64;
    }
    score
}

fn consecutive_finger_score(f1: Finger, f2: Finger, distance: f64) -> f64 {
    if distance == 0.0 {
        return 1.0;
    }

    if f1.hand != f2.hand {
        return 1.0;
    }

    let distance_importance = match (f1.finger, f2.finger) {
        (FingerKind::Pinky, FingerKind::Pinky) => 1.0,
        (FingerKind::Pinky, FingerKind::Ring) | (FingerKind::Ring, FingerKind::Pinky) => 0.9,
        (FingerKind::Pinky, FingerKind::Middle) | (FingerKind::Middle, FingerKind::Pinky) => 0.8,
        (FingerKind::Pinky, FingerKind::Index) | (FingerKind::Index, FingerKind::Pinky) => 0.2,
        (FingerKind::Pinky, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Pinky) => 0.1,
        (FingerKind::Ring, FingerKind::Ring) => 1.0,
        (FingerKind::Ring, FingerKind::Middle) | (FingerKind::Middle, FingerKind::Ring) => 0.9,
        (FingerKind::Ring, FingerKind::Index) | (FingerKind::Index, FingerKind::Ring) => 0.5,
        (FingerKind::Ring, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Ring) => 0.1,
        (FingerKind::Middle, FingerKind::Middle) => 1.0,
        (FingerKind::Middle, FingerKind::Index) | (FingerKind::Index, FingerKind::Middle) => 0.7,
        (FingerKind::Middle, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Middle) => 0.1,
        (FingerKind::Index, FingerKind::Index) => 1.0,
        (FingerKind::Index, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Index) => 0.2,
        (FingerKind::Thumb, FingerKind::Thumb) => 1.0,
    };

    let raw_score = 1.0 / (distance as f64 + 1.0);
    1.0 * (1.0 - distance_importance) + raw_score * distance_importance
}

#[derive(Clone, Debug)]
struct Layout {
    keys: Vec<Key>,
}

impl Gen for Layout {
    type Config = KeymapConfig;

    fn gen<R: rand::Rng>(rng: &mut R, config: &Self::Config) -> Self {
        let mut pool = config
            .keys
            .keys()
            .iter()
            .map(|p| match p.code {
                Keycode::Backspace => Key::Backspace,
                Keycode::Tab => Key::Tab,
                Keycode::Enter => Key::Enter,
                Keycode::CapsLock => Key::CapsLock,
                Keycode::LShift => Key::LShift,
                Keycode::RShift => Key::RShift,
                Keycode::LControl => Key::LCtrl,
                Keycode::RControl => Key::RCtrl,
                Keycode::LAlt => Key::LAlt,
                Keycode::RAlt => Key::RAlt,
                Keycode::LMeta => Key::LMeta,
                Keycode::RMeta => Key::RMeta,
                Keycode::Space => Key::Space,
                Keycode::Escape => Key::Escape,
                Keycode::Home => Key::Home,
                Keycode::End => Key::End,
                Keycode::PageUp => Key::PageUp,
                Keycode::PageDown => Key::PageDown,
                Keycode::Left => Key::Left,
                Keycode::Right => Key::Right,
                Keycode::Up => Key::Up,
                Keycode::Down => Key::Down,
                Keycode::Delete => Key::Delete,
                _ => Key::Normal {
                    normal: keycode_to_char(p.code, false),
                    shifted: keycode_to_char(p.code, true),
                },
            })
            .collect::<Vec<_>>();
        pool.shuffle(rng);
        Self { keys: pool }
    }
}

impl Crossover for Layout {
    fn crossover<R: rand::Rng>(&self, rng: &mut R, other: &Self) -> (Self, Self) {
        let mut child1 = Vec::with_capacity(self.keys.len());
        let mut child2 = Vec::with_capacity(self.keys.len());
        for (key1, key2) in self.keys.iter().zip(other.keys.iter()) {
            let (child1_key, child2_key) = {
                match (key1, key2) {
                    (
                        Key::Normal {
                            normal: p1_normal,
                            shifted: p1_shifted,
                        },
                        Key::Normal {
                            normal: p2_normal,
                            shifted: p2_shifted,
                        },
                    ) => {
                        let (normal1, noraml2) = if rng.gen_bool(0.5) {
                            (p1_normal, p2_normal)
                        } else {
                            (p2_normal, p1_normal)
                        };
                        let (shifted1, shifted2) = if rng.gen_bool(0.5) {
                            (p1_shifted, p2_shifted)
                        } else {
                            (p2_shifted, p1_shifted)
                        };
                        (
                            Key::Normal {
                                normal: *normal1,
                                shifted: *shifted1,
                            },
                            Key::Normal {
                                normal: *noraml2,
                                shifted: *shifted2,
                            },
                        )
                    }
                    _ => {
                        if rng.gen_bool(0.5) {
                            (*key1, *key2)
                        } else {
                            (*key2, *key1)
                        }
                    }
                }
            };
            child1.push(child1_key);
            child2.push(child2_key);
        }
        (Self { keys: child1 }, Self { keys: child2 })
    }
}

impl Mutate for Layout {
    fn mutate<R: rand::Rng>(&mut self, rng: &mut R, rate: f32) {
        for i in 0..self.keys.len() {
            for j in (i + 1)..self.keys.len() {
                let (_, rest) = self.keys.split_at_mut(i);
                let (first, rest) = rest.split_first_mut().unwrap();
                let (_, rest) = rest.split_at_mut(j - i - 1);
                let (second, _) = rest.split_first_mut().unwrap();
                match (first, second) {
                    (
                        Key::Normal {
                            normal: p1_normal,
                            shifted: p1_shifted,
                        },
                        Key::Normal {
                            normal: p2_normal,
                            shifted: p2_shifted,
                        },
                    ) => {
                        if rng.gen_bool(rate as f64) {
                            std::mem::swap(p1_normal, p2_normal);
                        }
                        if rng.gen_bool(rate as f64) {
                            std::mem::swap(p1_shifted, p2_shifted);
                        }
                    }
                    (first, second) => {
                        if rng.gen_bool(rate as f64) {
                            std::mem::swap(first, second);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Key {
    Normal { normal: char, shifted: char },
    Backspace,
    Tab,
    Enter,
    CapsLock,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LMeta,
    RMeta,
    Space,
    Escape,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Up,
    Down,
    Delete,
}

impl Key {
    fn keycode(&self, shift: bool) -> Keycode {
        match self {
            Key::Normal { normal, shifted } => {
                if shift {
                    char_to_keycode(*shifted)
                } else {
                    char_to_keycode(*normal)
                }
            }
            Key::Backspace => Keycode::Backspace,
            Key::Tab => Keycode::Tab,
            Key::Enter => Keycode::Enter,
            Key::CapsLock => Keycode::CapsLock,
            Key::LShift => Keycode::LShift,
            Key::RShift => Keycode::RShift,
            Key::LCtrl => Keycode::LControl,
            Key::RCtrl => Keycode::RControl,
            Key::LAlt => Keycode::LAlt,
            Key::RAlt => Keycode::RAlt,
            Key::LMeta => Keycode::LMeta,
            Key::RMeta => Keycode::RMeta,
            Key::Space => Keycode::Space,
            Key::Escape => Keycode::Escape,
            Key::Home => Keycode::Home,
            Key::End => Keycode::End,
            Key::PageUp => Keycode::PageUp,
            Key::PageDown => Keycode::PageDown,
            Key::Left => Keycode::Left,
            Key::Right => Keycode::Right,
            Key::Up => Keycode::Up,
            Key::Down => Keycode::Down,
            Key::Delete => Keycode::Delete,
        }
    }
}

fn char_to_keycode(c: char) -> Keycode {
    match c {
        'a' => Keycode::A,
        'b' => Keycode::B,
        'c' => Keycode::C,
        'd' => Keycode::D,
        'e' => Keycode::E,
        'f' => Keycode::F,
        'g' => Keycode::G,
        'h' => Keycode::H,
        'i' => Keycode::I,
        'j' => Keycode::J,
        'k' => Keycode::K,
        'l' => Keycode::L,
        'm' => Keycode::M,
        'n' => Keycode::N,
        'o' => Keycode::O,
        'p' => Keycode::P,
        'q' => Keycode::Q,
        'r' => Keycode::R,
        's' => Keycode::S,
        't' => Keycode::T,
        'u' => Keycode::U,
        'v' => Keycode::V,
        'w' => Keycode::W,
        'x' => Keycode::X,
        'y' => Keycode::Y,
        'z' => Keycode::Z,
        'A' => Keycode::A,
        'B' => Keycode::B,
        'C' => Keycode::C,
        'D' => Keycode::D,
        'E' => Keycode::E,
        'F' => Keycode::F,
        'G' => Keycode::G,
        'H' => Keycode::H,
        'I' => Keycode::I,
        'J' => Keycode::J,
        'K' => Keycode::K,
        'L' => Keycode::L,
        'M' => Keycode::M,
        'N' => Keycode::N,
        'O' => Keycode::O,
        'P' => Keycode::P,
        'Q' => Keycode::Q,
        'R' => Keycode::R,
        'S' => Keycode::S,
        'T' => Keycode::T,
        'U' => Keycode::U,
        'V' => Keycode::V,
        'W' => Keycode::W,
        'X' => Keycode::X,
        'Y' => Keycode::Y,
        'Z' => Keycode::Z,
        '0' => Keycode::Key0,
        '1' => Keycode::Key1,
        '2' => Keycode::Key2,
        '3' => Keycode::Key3,
        '4' => Keycode::Key4,
        '5' => Keycode::Key5,
        '6' => Keycode::Key6,
        '7' => Keycode::Key7,
        '8' => Keycode::Key8,
        '9' => Keycode::Key9,
        '!' => Keycode::Key1,
        '@' => Keycode::Key2,
        '#' => Keycode::Key3,
        '$' => Keycode::Key4,
        '%' => Keycode::Key5,
        '^' => Keycode::Key6,
        '&' => Keycode::Key7,
        '*' => Keycode::Key8,
        '(' => Keycode::Key9,
        ')' => Keycode::Key0,
        '-' => Keycode::Minus,
        '_' => Keycode::Minus,
        '=' => Keycode::Equal,
        '+' => Keycode::Equal,
        '[' => Keycode::LeftBracket,
        '{' => Keycode::LeftBracket,
        ']' => Keycode::RightBracket,
        '}' => Keycode::RightBracket,
        '\\' => Keycode::BackSlash,
        '|' => Keycode::BackSlash,
        ';' => Keycode::Semicolon,
        ':' => Keycode::Semicolon,
        '\'' => Keycode::Apostrophe,
        '"' => Keycode::Apostrophe,
        ',' => Keycode::Comma,
        '<' => Keycode::Comma,
        '.' => Keycode::Dot,
        '>' => Keycode::Dot,
        '/' => Keycode::Slash,
        '?' => Keycode::Slash,
        ' ' => Keycode::Space,
        '`' => Keycode::Grave,
        '~' => Keycode::Grave,
        _ => unimplemented!(),
    }
}

fn keycode_to_char(code: Keycode, shift: bool) -> char {
    match (code, shift) {
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
        (Keycode::Key0, false) => '0',
        (Keycode::Key1, false) => '1',
        (Keycode::Key2, false) => '2',
        (Keycode::Key3, false) => '3',
        (Keycode::Key4, false) => '4',
        (Keycode::Key5, false) => '5',
        (Keycode::Key6, false) => '6',
        (Keycode::Key7, false) => '7',
        (Keycode::Key8, false) => '8',
        (Keycode::Key9, false) => '9',
        (Keycode::Key0, true) => ')',
        (Keycode::Key1, true) => '!',
        (Keycode::Key2, true) => '@',
        (Keycode::Key3, true) => '#',
        (Keycode::Key4, true) => '$',
        (Keycode::Key5, true) => '%',
        (Keycode::Key6, true) => '^',
        (Keycode::Key7, true) => '&',
        (Keycode::Key8, true) => '*',
        (Keycode::Key9, true) => '(',
        (Keycode::Minus, false) => '-',
        (Keycode::Minus, true) => '_',
        (Keycode::Equal, false) => '=',
        (Keycode::Equal, true) => '+',
        (Keycode::LeftBracket, false) => '[',
        (Keycode::LeftBracket, true) => '{',
        (Keycode::RightBracket, false) => ']',
        (Keycode::RightBracket, true) => '}',
        (Keycode::BackSlash, false) => '\\',
        (Keycode::BackSlash, true) => '|',
        (Keycode::Semicolon, false) => ';',
        (Keycode::Semicolon, true) => ':',
        (Keycode::Apostrophe, false) => '\'',
        (Keycode::Apostrophe, true) => '"',
        (Keycode::Comma, false) => ',',
        (Keycode::Comma, true) => '<',
        (Keycode::Dot, false) => '.',
        (Keycode::Dot, true) => '>',
        (Keycode::Slash, false) => '/',
        (Keycode::Slash, true) => '?',
        (Keycode::Space, _) => ' ',
        (Keycode::Grave, false) => '`',
        (Keycode::Grave, true) => '~',
        (code, _) => unimplemented!("{:?}", code),
    }
}
