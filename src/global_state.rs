pub use glfw::{Action::*, Key::*, WindowEvent::Key};

use crate::{
    buffers::framebuffer::FrameBuffer, drawing::draw::Draw, environment::Environment,
    input::keyboard::Keyboard, input::mouse::Mouse, window::Window, Result,
};

pub trait GlobalState: Sized {
    fn poll<'b, 'a: 'b>(
        &'a mut self,
        mouse: &Mouse,
        keyboard: &Keyboard,
        frame_time: f32,
        window: &mut Window,
        default_framebuffer: &'a mut FrameBuffer,
        time: f32,
    ) -> Vec<Draw<'b>>;

    /// # Errors
    fn new(evironment: &Environment<Self>) -> Result<Self>;
}
