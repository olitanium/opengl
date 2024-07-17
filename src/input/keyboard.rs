use std::{collections::HashSet, sync::RwLock};

pub use glfw::{Action, Key};
use lazy_static::lazy_static;

use crate::window::Window;

lazy_static! {
    static ref KEYS: RwLock<HashSet<Key>> = RwLock::default();
}

pub struct Keyboard;

impl Keyboard {
    pub(crate) fn new(window: &mut Window) -> Self {
        window.set_key_callback(|key, action| {
            match action {
                Action::Press => match KEYS.write() {
                    Ok(mut set) => {
                        set.insert(key);
                    }
                    Err(_) => {
                        println!("Error setting {key:?} key as pressed");
                    }
                },
                Action::Release => match KEYS.write() {
                    Ok(mut set) => {
                        set.remove(&key);
                    }
                    Err(_) => {
                        println!("Error setting {key:?} key as released");
                    }
                },
                Action::Repeat => {}
            };
        });

        Self
    }

    #[inline]
    pub fn get(&self, key: Key) -> bool {
        KEYS.read().map_or(false, |set| set.contains(&key))
    }
}
