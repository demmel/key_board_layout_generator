use clap::Parser;
use device_query::Keycode;
use genetic::{Crossover, DiversifyStrategy, Gen, Mutate};
use keyboard_layout_generator::{
    layout_format::{map_keycode_to_str, parse_keymap_config, write_grid, GridItem},
    stats::{process_log, Stats},
    Finger, FingerKind, KeymapConfig, PhysicalKey,
};
use rand::{seq::SliceRandom, Rng};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    io::{BufWriter, Write},
};

/// A tool to explore different keyboard layouts using a
/// genetic algorithm and simulated annealing.
///
/// Every iteration, the program will output the max, mean,
/// and min scores of the current population, as well as the
/// diversity of the population.  The program will also save
/// the best layout to a file called `best.txt`.
///
/// The program will run indefinitely, so you will need to
/// manually stop it when you are satisfied with the results.
#[derive(Parser)]
struct Args {
    /// Path to the log file created by the keylogger.
    log_file: String,
    /// Path to a keymap configuration file describing the
    /// layout of the physical keyboard.  See the README for
    /// more information.
    keymap_config: String,
}

fn main() {
    // Leave one core so my UI doesn't lag
    rayon::ThreadPoolBuilder::new()
        .num_threads(31)
        .build_global()
        .unwrap();

    let args = Args::parse();
    let stats = process_log(&args.log_file);
    println!("Max Possible Score: {}", max_possible_score(&stats));
    let keymap_str = std::fs::read_to_string(&args.keymap_config).unwrap();
    let keymap_config = parse_keymap_config(&keymap_str);
    let mut population = (0..1000)
        .map(|_| Layout::gen(&mut rand::thread_rng(), &keymap_config))
        .collect::<Vec<_>>();
    loop {
        println!("Annealing");
        population.par_iter_mut().for_each(|layout| {
            *layout = simmulated_annealing(&stats, &keymap_config, 0.0001, layout.clone());
        });
        println!("Genetic");
        let (new_population, gstats) = genetic::evolve(
            &population,
            &keymap_config,
            |layout| layout_score(layout, &stats, &keymap_config) as f32,
            layout_similarity,
            DiversifyStrategy::HalfAreRandom,
        );
        let best = &new_population[0];
        save_best(&keymap_config, best);
        println!(
            "Max: {}, Mean: {}, Min: {}, Div: {}",
            gstats.max, gstats.mean, gstats.min, gstats.diversity,
        );
        population = new_population;
    }
}

fn save_best(keymap_config: &KeymapConfig, best: &Layout) {
    let rows = keymap_config
        .keys
        .keys()
        .iter()
        .map(|key| key.position.1 as u8)
        .max()
        .unwrap()
        + 1;
    let cols = keymap_config
        .keys
        .keys()
        .iter()
        .map(|key| key.position.0 as u8)
        .max()
        .unwrap()
        + 1;

    let mut best_grid = vec![vec![None; cols as usize]; rows as usize];
    for (key, config) in best.keys().iter().zip(keymap_config.keys.keys().iter()) {
        best_grid[config.position.1 as usize][config.position.0 as usize] = Some(key.clone());
    }

    let mut best_str = String::new();
    write_grid(best_grid, &mut best_str, cols).unwrap();
    let mut writer = BufWriter::new(std::fs::File::create(format!("best.txt")).unwrap());
    writer.write_all(best_str.as_bytes()).unwrap();
}

fn simmulated_annealing(
    stats: &Stats,
    keymap_config: &KeymapConfig,
    min_temperature: f64,
    initial_layout: Layout,
) -> Layout {
    let mut rng = rand::thread_rng();
    let mut layout = initial_layout;
    let mut score = layout_score(&layout, &stats, &keymap_config);
    let mut temperature = 1.0;
    let mut best_layout = layout.clone();
    let mut best_score = score;
    loop {
        let mut new_layout = layout.clone();
        let i = rng.gen_range(0..new_layout.keys().len());
        let j = rng.gen_range(0..new_layout.keys().len());
        new_layout.swap(i, j);
        let new_score = layout_score(&new_layout, &stats, &keymap_config);
        if new_score > best_score {
            best_layout = new_layout.clone();
            best_score = new_score;
        }
        let delta = new_score - score;
        let normalized_delta = delta / max_possible_score(&stats);
        if delta > 0.0 || rng.gen_bool((normalized_delta / temperature).exp()) {
            layout = new_layout;
            score = new_score;
        }
        temperature *= 0.9999;
        if temperature < min_temperature {
            break;
        }
    }
    best_layout
}

fn max_possible_score(stats: &Stats) -> f64 {
    let mut score = 0.0;
    for count in stats.individual_key_counts.values() {
        score += *count as f64;
    }
    for count in stats.consectutive_key_counts.values() {
        score += *count as f64;
    }
    let n_intuitions = intuitions().len() as f64;
    score += n_intuitions * 100.0;
    score
}

fn layout_similarity(l1: &Layout, l2: &Layout) -> f32 {
    let mut score = 0.0;
    for (key1, key2) in l1.keys().iter().zip(l2.keys().iter()) {
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
    score / (l1.keys().len() * 2) as f32
}

fn layout_score(layout: &Layout, stats: &Stats, keymap_config: &KeymapConfig) -> f64 {
    let individual_key_score = layout_individual_key_score(layout, stats, keymap_config);
    let consecutive_key_score = layout_consecutive_key_score(layout, stats, keymap_config);
    let intuition_score = intuition_score(layout, keymap_config, &intuitions());

    individual_key_score + consecutive_key_score + 100.0 * intuition_score
}

struct IntuitionPair(Key, Key);

impl IntuitionPair {
    fn physical_keys<'a>(
        &self,
        layout: &Layout,
        keymap_config: &'a KeymapConfig,
    ) -> (&'a PhysicalKey, &'a PhysicalKey) {
        (
            get_physical_key_for_key(layout, keymap_config, &self.0),
            get_physical_key_for_key(layout, keymap_config, &self.1),
        )
    }
}

enum Intuition {
    Close(IntuitionPair),
    Symmetric(IntuitionPair),
    SameRow(IntuitionPair),
    SameColumn(IntuitionPair),
    LeftOf(IntuitionPair),
    RightOf(IntuitionPair),
    Above(IntuitionPair),
    Below(IntuitionPair),
    Or(Box<Intuition>, Box<Intuition>),
    And(Box<Intuition>, Box<Intuition>),
}

impl Intuition {
    fn satisfied(&self, layout: &Layout, keymap_config: &KeymapConfig) -> bool {
        match self {
            Intuition::Close(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                distance(key1, key2) < 1.1
            }
            Intuition::Symmetric(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                are_symmetric(keymap_config, key1.position, key2.position)
            }
            Intuition::SameRow(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.1 == key2.position.1
            }
            Intuition::SameColumn(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.0 == key2.position.0
            }
            Intuition::LeftOf(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.0 < key2.position.0
            }
            Intuition::RightOf(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.0 > key2.position.0
            }
            Intuition::Above(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.1 < key2.position.1
            }
            Intuition::Below(pair) => {
                let (key1, key2) = pair.physical_keys(layout, keymap_config);
                key1.position.1 > key2.position.1
            }
            Intuition::Or(a, b) => {
                a.satisfied(layout, keymap_config) || b.satisfied(layout, keymap_config)
            }
            Intuition::And(a, b) => {
                a.satisfied(layout, keymap_config) && b.satisfied(layout, keymap_config)
            }
        }
    }
}

fn intuition_score(layout: &Layout, keymap_config: &KeymapConfig, intuitions: &[Intuition]) -> f64 {
    let mut score = 0.0;
    for intuition in intuitions {
        if intuition.satisfied(layout, keymap_config) {
            score += 1.0;
        }
    }
    score
}

fn close(key1: Key, key2: Key) -> Intuition {
    Intuition::Close(IntuitionPair(key1, key2))
}

fn symmetric(key1: Key, key2: Key) -> Intuition {
    Intuition::Symmetric(IntuitionPair(key1, key2))
}

fn same_row(key1: Key, key2: Key) -> Intuition {
    Intuition::SameRow(IntuitionPair(key1, key2))
}

fn same_column(key1: Key, key2: Key) -> Intuition {
    Intuition::SameColumn(IntuitionPair(key1, key2))
}

fn left_of(key1: Key, key2: Key) -> Intuition {
    Intuition::LeftOf(IntuitionPair(key1, key2))
}

fn right_of(key1: Key, key2: Key) -> Intuition {
    Intuition::RightOf(IntuitionPair(key1, key2))
}

fn above(key1: Key, key2: Key) -> Intuition {
    Intuition::Above(IntuitionPair(key1, key2))
}

fn below(key1: Key, key2: Key) -> Intuition {
    Intuition::Below(IntuitionPair(key1, key2))
}

fn or(a: Intuition, b: Intuition) -> Intuition {
    Intuition::Or(Box::new(a), Box::new(b))
}

fn and(a: Intuition, b: Intuition) -> Intuition {
    Intuition::And(Box::new(a), Box::new(b))
}

fn key(c: char) -> Key {
    Key::from_char_default_shifted(c)
}

fn intuitions() -> Vec<Intuition> {
    use Key::*;
    vec![
        and(same_row(Left, Right), left_of(Left, Right)),
        and(same_column(Up, Down), above(Up, Down)),
        or(close(Left, Right), symmetric(Left, Right)),
        or(close(Up, Down), symmetric(Up, Down)),
        or(close(PageUp, PageDown), symmetric(PageUp, PageDown)),
        or(close(key('['), key(']')), symmetric(key('['), key(']'))),
        symmetric(LShift, RShift),
        symmetric(LCtrl, RCtrl),
        close(key('1'), key('2')),
        close(key('2'), key('3')),
        close(key('4'), key('5')),
        close(key('5'), key('6')),
        close(key('7'), key('8')),
        close(key('8'), key('9')),
        close(key('1'), key('4')),
        close(key('2'), key('5')),
        close(key('3'), key('6')),
        close(key('4'), key('7')),
        close(key('5'), key('8')),
        close(key('6'), key('9')),
    ]
}

fn are_symmetric(config: &KeymapConfig, pos1: (f64, f64), pos2: (f64, f64)) -> bool {
    if pos1.1 != pos2.1 {
        return false;
    }

    let max = config
        .keys
        .keys()
        .iter()
        .map(|k| k.position.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let smaller = pos1.0.min(pos2.0);
    let larger = pos1.0.max(pos2.0);

    (max - larger - smaller).abs() < 0.01
}

fn get_physical_key_for_key<'a>(
    layout: &Layout,
    config: &'a KeymapConfig,
    key: &Key,
) -> &'a PhysicalKey {
    &config.keys.keys()[layout.get(key)]
}

fn layout_consecutive_key_score(
    layout: &Layout,
    stats: &Stats,
    keymap_config: &KeymapConfig,
) -> f64 {
    let mut score = 0.0;
    for ((i, key1), (j, key2)) in layout
        .keys()
        .iter()
        .enumerate()
        .zip(layout.keys().iter().enumerate().skip(1))
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
    for (key, config) in layout.keys().iter().zip(keymap_config.keys.keys().iter()) {
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

    let synergy = match (f1.finger, f2.finger) {
        (FingerKind::Pinky, FingerKind::Pinky) => 0.1,
        (FingerKind::Pinky, FingerKind::Ring) | (FingerKind::Ring, FingerKind::Pinky) => 0.2,
        (FingerKind::Pinky, FingerKind::Middle) | (FingerKind::Middle, FingerKind::Pinky) => 0.2,
        (FingerKind::Pinky, FingerKind::Index) | (FingerKind::Index, FingerKind::Pinky) => 0.5,
        (FingerKind::Pinky, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Pinky) => 0.6,
        (FingerKind::Ring, FingerKind::Ring) => 0.1,
        (FingerKind::Ring, FingerKind::Middle) | (FingerKind::Middle, FingerKind::Ring) => 0.3,
        (FingerKind::Ring, FingerKind::Index) | (FingerKind::Index, FingerKind::Ring) => 0.3,
        (FingerKind::Ring, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Ring) => 0.2,
        (FingerKind::Middle, FingerKind::Middle) => 0.2,
        (FingerKind::Middle, FingerKind::Index) | (FingerKind::Index, FingerKind::Middle) => 0.7,
        (FingerKind::Middle, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Middle) => 0.7,
        (FingerKind::Index, FingerKind::Index) => 0.3,
        (FingerKind::Index, FingerKind::Thumb) | (FingerKind::Thumb, FingerKind::Index) => 0.9,
        (FingerKind::Thumb, FingerKind::Thumb) => 0.3,
    };

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
    let distance_score = 1.0 * (1.0 - distance_importance) + raw_score * distance_importance;
    distance_score * synergy
}

#[derive(Clone, Debug)]
struct Layout {
    keys: Vec<Key>,
    key_map: HashMap<Key, usize>,
}

impl Layout {
    fn new(keys: Vec<Key>) -> Self {
        let mut key_map = HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            key_map.insert(*key, i);
        }
        Self { keys, key_map }
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.keys.swap(i, j);
        self.key_map.insert(self.keys[i], i);
        self.key_map.insert(self.keys[j], j);
    }

    fn get(&self, key: &Key) -> usize {
        *self.key_map.get(key).unwrap()
    }

    fn keys(&self) -> &[Key] {
        &self.keys
    }
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
                    normal: keycode_to_char(p.code),
                    shifted: default_shifted(keycode_to_char(p.code)),
                },
            })
            .collect::<Vec<_>>();
        pool.shuffle(rng);
        Self::new(pool)
    }
}

impl Crossover for Layout {
    fn crossover<R: rand::Rng>(&self, rng: &mut R, other: &Self) -> (Self, Self) {
        let mut child1 = Vec::with_capacity(self.keys().len());
        let mut child2 = Vec::with_capacity(self.keys().len());
        for (key1, key2) in self.keys().iter().zip(other.keys().iter()) {
            let (child1_key, child2_key) = {
                if rng.gen_bool(0.5) {
                    (*key1, *key2)
                } else {
                    (*key2, *key1)
                }
            };
            child1.push(child1_key);
            child2.push(child2_key);
        }
        fix_missing_keys(&mut child1, self.keys());
        fix_missing_keys(&mut child2, self.keys());
        (Layout::new(child1), Layout::new(child2))
    }
}

fn fix_missing_keys(child: &mut Vec<Key>, parent: &[Key]) {
    let all_keys: HashSet<Key> = parent.iter().cloned().collect();
    let child_keys: HashSet<Key> = child.iter().cloned().collect();
    let missing_keys = all_keys.difference(&child_keys);
    for key in missing_keys {
        let dupe_i = find_duplicate_key_index(&child);
        child[dupe_i] = *key;
    }
}

fn find_duplicate_key_index(keys: &Vec<Key>) -> usize {
    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            if keys[i] == keys[j] {
                return j;
            }
        }
    }
    panic!("No duplicate key found");
}

impl Mutate for Layout {
    fn mutate<R: rand::Rng>(&mut self, rng: &mut R, rate: f32) {
        for i in 0..self.keys().len() {
            for j in (i + 1)..self.keys().len() {
                if rng.gen_bool(rate as f64) {
                    self.swap(i, j);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    fn from_char_default_shifted(c: char) -> Self {
        Key::Normal {
            normal: c,
            shifted: default_shifted(c),
        }
    }

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

impl GridItem for Key {
    fn num_items() -> usize {
        2
    }

    fn get_item(&self, i: usize) -> Option<String> {
        match i {
            0 => {
                let code = self.keycode(false);
                Some(map_keycode_to_str(code).unwrap().to_string())
            }
            1 => {
                let code = self.keycode(true);
                Some(map_keycode_to_str(code).unwrap().to_string())
            }
            _ => None,
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

fn keycode_to_char(code: Keycode) -> char {
    match code {
        Keycode::A => 'a',
        Keycode::B => 'b',
        Keycode::C => 'c',
        Keycode::D => 'd',
        Keycode::E => 'e',
        Keycode::F => 'f',
        Keycode::G => 'g',
        Keycode::H => 'h',
        Keycode::I => 'i',
        Keycode::J => 'j',
        Keycode::K => 'k',
        Keycode::L => 'l',
        Keycode::M => 'm',
        Keycode::N => 'n',
        Keycode::O => 'o',
        Keycode::P => 'p',
        Keycode::Q => 'q',
        Keycode::R => 'r',
        Keycode::S => 's',
        Keycode::T => 't',
        Keycode::U => 'u',
        Keycode::V => 'v',
        Keycode::W => 'w',
        Keycode::X => 'x',
        Keycode::Y => 'y',
        Keycode::Z => 'z',
        Keycode::Key0 => '0',
        Keycode::Key1 => '1',
        Keycode::Key2 => '2',
        Keycode::Key3 => '3',
        Keycode::Key4 => '4',
        Keycode::Key5 => '5',
        Keycode::Key6 => '6',
        Keycode::Key7 => '7',
        Keycode::Key8 => '8',
        Keycode::Key9 => '9',
        Keycode::Minus => '-',
        Keycode::Equal => '=',
        Keycode::LeftBracket => '[',
        Keycode::RightBracket => ']',
        Keycode::BackSlash => '\\',
        Keycode::Semicolon => ';',
        Keycode::Apostrophe => '\'',
        Keycode::Comma => ',',
        Keycode::Dot => '.',
        Keycode::Slash => '/',
        Keycode::Grave => '`',
        Keycode::Space => ' ',
        code => unimplemented!("{:?}", code),
    }
}

fn default_shifted(c: char) -> char {
    match c {
        'a' => 'A',
        'b' => 'B',
        'c' => 'C',
        'd' => 'D',
        'e' => 'E',
        'f' => 'F',
        'g' => 'G',
        'h' => 'H',
        'i' => 'I',
        'j' => 'J',
        'k' => 'K',
        'l' => 'L',
        'm' => 'M',
        'n' => 'N',
        'o' => 'O',
        'p' => 'P',
        'q' => 'Q',
        'r' => 'R',
        's' => 'S',
        't' => 'T',
        'u' => 'U',
        'v' => 'V',
        'w' => 'W',
        'x' => 'X',
        'y' => 'Y',
        'z' => 'Z',
        '0' => ')',
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        '`' => '~',
        _ => unimplemented!(),
    }
}
