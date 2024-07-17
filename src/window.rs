use glfw::{Context, PWindow};

use crate::input::{
    keyboard::Key,
    mouse::{Button, CursorMode},
    Action,
};

pub struct Window(PWindow);

impl Window {
    #[must_use]
    #[inline]
    pub(crate) const fn new(window: PWindow) -> Self {
        Self(window)
    }

    #[must_use]
    #[inline]
    pub fn get_aspect(&self) -> f32 {
        let (x, y) = self.0.get_size();
        #[expect(clippy::cast_precision_loss, reason = "Unlikely to overflow, otherwise the aspect ratio will mostly be correct")]
        (x as f32 / y as f32)
    }

    #[must_use]
    #[inline]
    pub fn get_framebuffer_size(&self) -> (i32, i32) {
        self.0.get_framebuffer_size()
    }

    #[inline]
    pub fn set_should_close(&mut self, value: bool) {
        self.0.set_should_close(value);
    }

    #[must_use]
    #[inline]
    pub fn should_close(&self) -> bool {
        self.0.should_close()
    }

    #[inline]
    pub fn swap_buffers(&mut self) {
        self.0.swap_buffers();
    }

    #[inline]
    pub fn set_key_callback<T: FnMut(Key, Action) + 'static>(&mut self, mut callback: T) {
        self.0
            .set_key_callback(move |_window, key, _scancode, action, _modifiers| {
                callback(key, action);
            });
    }

    #[inline]
    pub fn set_cursor_pos_callback<T: FnMut(f64, f64) + 'static>(&mut self, mut callback: T ) {
        self.0
            .set_cursor_pos_callback(move |_window, x, y| callback(x, y));
    }

    #[inline]
    pub fn set_mouse_button_callback<T: FnMut(Button, Action) + 'static>(
        &mut self,
        mut callback: T,
    ) {
        self.0
            .set_mouse_button_callback(move |_window, mouse_button, action, _modifiers| {
                callback(mouse_button, action);
            });
    }

    #[inline]
    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.0.set_cursor_mode(mode);
    }
}
