use std::{fs::File, io::Write};

use device_query::{DeviceQuery, Keycode};
use keyboard_layout_generator::{Finger, KeyConfig, KeymapConfig, SerializableKeycode};

fn main() {
    let mut keys = vec![];
    for i in 0..68 {
        println!("Press key {}", i);
        let _ = std::io::stdout().flush();
        let key = wait_for_key();
        keys.push(KeyConfig {
            code: SerializableKeycode(key),
            finger: Finger::LeftPinky,
            score_multiplier: 1.0,
        });
    }
    serde_json::to_writer_pretty(
        File::create("keymap.json").unwrap(),
        &KeymapConfig {
            keys,
            ..Default::default()
        },
    )
    .unwrap();
}

fn wait_for_key() -> Keycode {
    let device_state = device_query::DeviceState::new();
    let key = loop {
        let keys = device_state.get_keys();
        if !keys.is_empty() {
            break keys.into_iter().next().unwrap();
        }
    };
    while !device_state.get_keys().is_empty() {}
    key
}
