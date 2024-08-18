//! This parses a special layout file that simiplifies the definition of a keyboard layout.
//! They look like this:
//!
//! ```plaintext
//! Fingers
//! LP: 70
//! LR: 50
//! LM: 80
//! LI: 100
//! LT: 100
//! RT: 100
//! RI: 100
//! RM: 80
//! RR: 50
//! RP: 70
//!
//! Keys
//! -----------------------------------------------------------------
//! | = | 1 | 2 | 3 | 4 | 5 |   |   |   |   | 6 | 7 | 8 | 9 | 0 | - |
//! |LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
//! |25 |35 |45 |50 |50 |50 |   |   |   |   |50 |50 |50 |45 |35 |25 |
//! -----------------------------------------------------------------
//! |Tab| Q | W | E | R | T |   |   |   |   | Y | U | I | O | P | \ |
//! |LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
//! |35 |75 |75 |75 |75 |75 |   |   |   |   |75 |75 |75 |75 |75 |35 |
//! -----------------------------------------------------------------
//! |Esc| A | S | D | F | G |   |   |   |   | H | J | K | L | ; | ' |
//! |LP |LP |LR |LM |LI |LI |   |   |   |   |RI |RI |RM |RR |RP |RP |
//! |75 |100|100|100|100|100|   |   |   |   |100|100|100|100|100|75 |
//! -----------------------------------------------------------------
//! |LSh| Z | X | C | V | B |LCt|LAt|LMt|RCt| N | M | , | . | / |RSh|
//! |LP |LP |LR |LM |LI |LI |LT |LT |RT |RT |RI |RI |RM |RR |RP |RP |
//! |65 |85 |85 |85 |85 |85 |70 |50 |50 |70 |85 |85 |85 |85 |85 |65 |
//! -----------------------------------------------------------------
//! |   | ~ |Cap|<--|-->|Bks|Del|Hom|PUp|Etr|Spc| Up| Dn| [ | ] |   |
//! |   |LP |LR |LM |LI |LT |LT |LT |RT |RT |RT |RI |RI |RP |RP |   |
//! |   |50 |50 |50 |50 |100|100|70 |70 |70 |100|100|50 |50 |50 |   |
//! -----------------------------------------------------------------
//! |   |   |   |   |   |   |   |End|PDn|   |   |   |   |   |   |   |
//! |   |   |   |   |   |   |   |LT |RT |   |   |   |   |   |   |   |
//! |   |   |   |   |   |   |   |80 |80 |   |   |   |   |   |   |   |
//! -----------------------------------------------------------------
//! ```
//!
//! The first section is the finger section. It defines the score for each finger.
//! The second section is the key section. It defines the layout of the keyboard.
//! Each key is defined by a character, a finger, and a score.
//! The layout is defined by a grid of keys.
//!
//! This layout makes it easy to define a layout for a keyboard without having to
//! consider the position of each key while writing something like JSON.

use crate::{Finger, FingerConfig, FingerKind, Hand, KeymapConfig, PhysicalKey, PhysicalKeyboard};
use device_query::Keycode;

pub fn parse_keymap_config(layout: &str) -> KeymapConfig {
    let mut lines = layout.lines();
    let fingers = parser_fingers(&mut lines);
    let keys = parser_keys(&mut lines);
    KeymapConfig { fingers, keys }
}

fn parser_fingers(lines: &mut std::str::Lines) -> Vec<FingerConfig> {
    let mut fingers = vec![];
    let mut lines = lines.skip_while(|line| line.trim() != "Fingers");
    let _ = lines.next();
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line == "Keys" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let (finger, score) = line.split_once(":").unwrap();
        let finger = finger.trim();
        let finger = parse_finger(finger);
        let score = score.trim();
        let score = parse_score(score);
        fingers.push(FingerConfig { finger, score });
    }
    fingers
}

fn parse_score(score: &str) -> f64 {
    score.parse::<f64>().unwrap() / 100.0
}

fn parse_finger(finger: &str) -> Finger {
    let finger = match finger {
        "LP" => Finger {
            hand: Hand::Left,
            finger: FingerKind::Pinky,
        },
        "LR" => Finger {
            hand: Hand::Left,
            finger: FingerKind::Ring,
        },
        "LM" => Finger {
            hand: Hand::Left,
            finger: FingerKind::Middle,
        },
        "LI" => Finger {
            hand: Hand::Left,
            finger: FingerKind::Index,
        },
        "LT" => Finger {
            hand: Hand::Left,
            finger: FingerKind::Thumb,
        },
        "RT" => Finger {
            hand: Hand::Right,
            finger: FingerKind::Thumb,
        },
        "RI" => Finger {
            hand: Hand::Right,
            finger: FingerKind::Index,
        },
        "RM" => Finger {
            hand: Hand::Right,
            finger: FingerKind::Middle,
        },
        "RR" => Finger {
            hand: Hand::Right,
            finger: FingerKind::Ring,
        },
        "RP" => Finger {
            hand: Hand::Right,
            finger: FingerKind::Pinky,
        },
        _ => panic!("Invalid finger"),
    };
    finger
}

fn parser_keys(lines: &mut std::str::Lines) -> PhysicalKeyboard {
    let mut keys = PhysicalKeyboard::new();
    let mut r = 0;
    while let Some(row) = parse_row(lines) {
        for (key, c) in row {
            let Some((code, finger, score)) = key else {
                continue;
            };
            keys.add_key(PhysicalKey {
                code,
                finger,
                score,
                position: (c as f64, r as f64),
            });
        }
        r += 1;
    }
    keys
}

fn parse_row(lines: &mut std::str::Lines) -> Option<Vec<(Option<(Keycode, Finger, f64)>, u8)>> {
    parse_hr(lines);
    let line = lines.next()?;
    let codes = parse_keycodes(line);
    let line = lines.next()?;
    let fingers = parse_key_fingers(line);
    let line = lines.next()?;
    let scores = parse_key_scores(line);

    assert!(
        codes.len() == fingers.len() && fingers.len() == scores.len(),
        "Length mismatch {:?} {:?} {:?}",
        codes,
        fingers,
        scores
    );

    let mut row = vec![];
    let mut col = 0;

    for (key, (finger, score)) in codes
        .into_iter()
        .zip(fingers.into_iter().zip(scores.into_iter()))
    {
        if key.is_none() {
            row.push((None, col));
        } else {
            row.push((Some((key.unwrap(), finger.unwrap(), score.unwrap())), col));
        }
        col += 1;
    }

    Some(row)
}

fn parse_hr(lines: &mut std::str::Lines) {
    let line = lines.next().unwrap();
    assert!(line.starts_with("-"));
}

fn parse_keycodes(line: &str) -> Vec<Option<Keycode>> {
    let codes = parse_pipe_separated(line)
        .map(|s| {
            Some(match s {
                "A" => Keycode::A,
                "B" => Keycode::B,
                "C" => Keycode::C,
                "D" => Keycode::D,
                "E" => Keycode::E,
                "F" => Keycode::F,
                "G" => Keycode::G,
                "H" => Keycode::H,
                "I" => Keycode::I,
                "J" => Keycode::J,
                "K" => Keycode::K,
                "L" => Keycode::L,
                "M" => Keycode::M,
                "N" => Keycode::N,
                "O" => Keycode::O,
                "P" => Keycode::P,
                "Q" => Keycode::Q,
                "R" => Keycode::R,
                "S" => Keycode::S,
                "T" => Keycode::T,
                "U" => Keycode::U,
                "V" => Keycode::V,
                "W" => Keycode::W,
                "X" => Keycode::X,
                "Y" => Keycode::Y,
                "Z" => Keycode::Z,
                "1" => Keycode::Key1,
                "2" => Keycode::Key2,
                "3" => Keycode::Key3,
                "4" => Keycode::Key4,
                "5" => Keycode::Key5,
                "6" => Keycode::Key6,
                "7" => Keycode::Key7,
                "8" => Keycode::Key8,
                "9" => Keycode::Key9,
                "0" => Keycode::Key0,
                "~" => Keycode::Grave,
                "Tab" => Keycode::Tab,
                "Esc" => Keycode::Escape,
                "LSh" => Keycode::LShift,
                "LCt" => Keycode::LControl,
                "LAt" => Keycode::LAlt,
                "LMt" => Keycode::LMeta,
                "RSh" => Keycode::RShift,
                "RCt" => Keycode::RControl,
                "RAt" => Keycode::RAlt,
                "RMt" => Keycode::RMeta,
                "Cap" => Keycode::CapsLock,
                "<--" => Keycode::Left,
                "-->" => Keycode::Right,
                "Bks" => Keycode::Backspace,
                "Del" => Keycode::Delete,
                "Hom" => Keycode::Home,
                "End" => Keycode::End,
                "PDn" => Keycode::PageDown,
                "PUp" => Keycode::PageUp,
                "Etr" => Keycode::Enter,
                "Spc" => Keycode::Space,
                "Up" => Keycode::Up,
                "Dn" => Keycode::Down,
                "[" => Keycode::LeftBracket,
                "]" => Keycode::RightBracket,
                "," => Keycode::Comma,
                "." => Keycode::Dot,
                "/" => Keycode::Slash,
                ";" => Keycode::Semicolon,
                "'" => Keycode::Apostrophe,
                "\\" => Keycode::BackSlash,
                "-" => Keycode::Minus,
                "=" => Keycode::Equal,
                "" => return None,
                x => panic!("Invalid key: {}", x),
            })
        })
        .collect();
    codes
}

fn parse_key_fingers(line: &str) -> Vec<Option<Finger>> {
    parse_pipe_separated(line)
        .map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(parse_finger(s))
            }
        })
        .collect()
}

fn parse_key_scores(line: &str) -> Vec<Option<f64>> {
    parse_pipe_separated(line)
        .map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(parse_score(s))
            }
        })
        .collect()
}

fn parse_pipe_separated(line: &str) -> impl Iterator<Item = &str> {
    let (_, line) = line.split_once("|").unwrap();
    let (line, _) = line.rsplit_once("|").unwrap();
    line.split("|").map(|s| s.trim())
}
