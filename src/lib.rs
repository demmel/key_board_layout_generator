pub mod layout_format;
pub mod stats;

use device_query::Keycode;
use serde::{Deserialize, Serialize};

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
