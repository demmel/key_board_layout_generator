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
use std::fmt::{Display, Write};

macro_rules! enum_strings {
    ($type:ty,$($variant:ident:$str:literal),*) => {
        paste::paste! {
            fn [<map_ $type:lower _to_str>] (variant: $type) -> Option<&'static str> {
                match variant {
                    $($type::$variant => Some($str),)*
                    _ => None,
                }
            }

            fn [<map_str_to_ $type:lower>](s: &str) -> Option<$type> {
                match s {
                    $($str => Some(<$type>::$variant),)*
                    _ => None,
                }
            }
        }
    }
}

enum_strings! {
    Keycode,
    A: "A",
    B: "B",
    C: "C",
    D: "D",
    E: "E",
    F: "F",
    G: "G",
    H: "H",
    I: "I",
    J: "J",
    K: "K",
    L: "L",
    M: "M",
    N: "N",
    O: "O",
    P: "P",
    Q: "Q",
    R: "R",
    S: "S",
    T: "T",
    U: "U",
    V: "V",
    W: "W",
    X: "X",
    Y: "Y",
    Z: "Z",
    Key1: "1",
    Key2: "2",
    Key3: "3",
    Key4: "4",
    Key5: "5",
    Key6: "6",
    Key7: "7",
    Key8: "8",
    Key9: "9",
    Key0: "0",
    Grave: "~",
    Tab: "Tab",
    Escape: "Esc",
    LShift: "LSh",
    LControl: "LCt",
    LAlt: "LAt",
    LMeta: "LMt",
    RShift: "RSh",
    RControl: "RCt",
    RAlt: "RAt",
    RMeta: "RMt",
    CapsLock: "Cap",
    Left: "<--",
    Right: "-->",
    Backspace: "Bks",
    Delete: "Del",
    Home: "Hom",
    End: "End",
    PageDown: "PDn",
    PageUp: "PUp",
    Enter: "Etr",
    Space: "Spc",
    Up: "Up",
    Down: "Dn",
    LeftBracket: "[",
    RightBracket: "]",
    Comma: ",",
    Dot: ".",
    Slash: "/",
    Semicolon: ";",
    Apostrophe: "'",
    BackSlash: "\\",
    Minus: "-",
    Equal: "="
}

enum_strings! {
    FingerKind,
    Pinky: "P",
    Ring: "R",
    Middle: "M",
    Index: "I",
    Thumb: "T"
}

enum_strings! {
    Hand,
    Left: "L",
    Right: "R"
}

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
    let (hand, finger) = finger.split_at(1);
    let finger = finger.trim();
    let hand = hand.trim();
    let finger = map_str_to_fingerkind(finger).unwrap();
    let hand = map_str_to_hand(hand).unwrap();
    Finger { hand, finger }
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
            if s.is_empty() {
                None
            } else {
                Some(map_str_to_keycode(s).unwrap())
            }
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

pub fn keymap_config_to_str(config: &KeymapConfig) -> Result<String, std::fmt::Error> {
    let mut s = String::new();
    writeln!(s, "Fingers")?;
    for finger in &config.fingers {
        writeln!(
            s,
            "{}{}: {}",
            map_hand_to_str(finger.finger.hand).unwrap(),
            map_fingerkind_to_str(finger.finger.finger).unwrap(),
            (finger.score * 100.0) as i32
        )?;
    }
    writeln!(s)?;
    writeln!(s, "Keys")?;
    let keys = &config.keys;
    let rows = keys
        .keys()
        .iter()
        .map(|key| key.position.1 as u8)
        .max()
        .unwrap()
        + 1;
    let cols = keys
        .keys()
        .iter()
        .map(|key| key.position.0 as u8)
        .max()
        .unwrap()
        + 1;

    let mut grid = vec![vec![None; cols as usize]; rows as usize];
    for key in keys.keys() {
        grid[key.position.1 as usize][key.position.0 as usize] = Some(key);
    }

    for row in grid {
        write!(s, "-")?;
        for _ in 0..row.len() {
            write!(s, "----")?;
        }
        writeln!(s)?;

        fn write_separated(
            s: &mut String,
            items: impl Iterator<Item = Option<impl Display>>,
        ) -> Result<(), std::fmt::Error> {
            write!(s, "|")?;
            for item in items {
                if let Some(item) = item {
                    let item = item.to_string();
                    if item.len() == 1 {
                        write!(s, " {} |", item)?;
                    } else {
                        write!(s, "{:<3}|", item)?;
                    }
                } else {
                    write!(s, "   |")?;
                }
            }
            writeln!(s)
        }

        write_separated(
            &mut s,
            row.iter()
                .map(|key| key.map(|key| map_keycode_to_str(key.code).unwrap())),
        )?;
        write_separated(
            &mut s,
            row.iter().map(|key| {
                key.map(|key| {
                    format!(
                        "{}{}",
                        map_hand_to_str(key.finger.hand).unwrap(),
                        map_fingerkind_to_str(key.finger.finger).unwrap()
                    )
                })
            }),
        )?;
        write_separated(
            &mut s,
            row.iter()
                .map(|key| key.map(|key| (key.score * 100.0) as i32)),
        )?;
    }

    write!(s, "-")?;
    for _ in 0..cols {
        write!(s, "----")?;
    }
    writeln!(s)?;

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e() {
        let s = include_str!("../kinesis.layout");
        let config = parse_keymap_config(s);
        let s2 = keymap_config_to_str(&config).unwrap();
        assert_eq!(s, s2);
    }
}
