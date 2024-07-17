use std::{collections::HashSet, sync::RwLock};

pub use glfw::CursorMode;
pub use glfw::MouseButton as Button;
use lazy_static::lazy_static;

use crate::input::Action;
use crate::window::Window;

pub struct Mouse;

#[derive(Default)]
struct Data {
    first_mouse: bool,
    old_loation: (f64, f64),
    current_location: (f64, f64),
    buttons: HashSet<Button>,
}

impl Data {
    fn new() -> Self {
        Self {
            first_mouse: true,
            ..Default::default()
        }
    }
}

lazy_static! {
    static ref MOUSE_DATA: RwLock<Data> = RwLock::new(Data::new());
}

impl Mouse {
    #[inline]
    pub fn new(window: &mut Window, fix_to_centre: bool) -> Self {
        window.set_cursor_pos_callback(|x, y| {
            let (first_mouse, old_location) = MOUSE_DATA.read().map_or_else(
                |_| {
                    println!("Failed to open the mouse_data mutex for reading during cursor_pos callback");
                    (false, (0.0,0.0))
                },
                |read| (read.first_mouse, read.current_location)
            );

            match MOUSE_DATA.write() {
                Ok(mut write) => {
                    if first_mouse {
                        write.old_loation = (x, y);
                        write.first_mouse = false;
                    } else {
                        write.old_loation = old_location;
                    }

                    write.current_location = (x, y);
                }
                Err(_) => {
                    println!("Failed to open the mouse_data mutex for writing during cursor_pos callback");
                }
            }
        });

        window.set_mouse_button_callback(
            |mouse_button, action: Action| match action {
                Action::Press => {
                    match MOUSE_DATA.write() {
                        Ok(mut write) => {write.buttons.insert(mouse_button);},
                        Err(_) => {
                            println!("Failed to open the mouse_data mutex for writing during mouse_button press callback");
                        }
                    }
                }
                Action::Release => {
                    match MOUSE_DATA.write() {
                        Ok(mut write) => {write.buttons.remove(&mouse_button);},
                        Err(_) => {
                            println!("Failed to open the mouse_data mutex for writing during mouse_button release callback");
                        }
                    }
                }
                Action::Repeat => {}
            },
        );
        if fix_to_centre {
            window.set_cursor_mode(CursorMode::Disabled);
        }

        Self
    }

    #[inline]
    pub fn get_delta(&self) -> (f64, f64) {
        let output = MOUSE_DATA.read().map_or_else(
            |_| {
                println!(
                    "Failed to open the mouse_data mutex for reading when getting mouse movement"
                );
                (0.0, 0.0)
            },
            |read| {
                (
                    read.current_location.0 - read.old_loation.0,
                    read.current_location.1 - read.old_loation.1,
                )
            },
        );

        match MOUSE_DATA.write() {
            Ok(mut write) => write.current_location = write.old_loation,
            Err(_) => println!("Failed to open the mouse_data mutex for writing when setting the current mouse data"),
        }

        output
    }

    #[inline]
    pub fn get_button(&self, button: &Button) -> bool {
        MOUSE_DATA.read().map_or_else(
            |_| {
                println!(
                    "Failed to open the mouse_data mutex for reading when getting mouse button"
                );
                false
            },
            |read| read.buttons.contains(button),
        )
    }
}
