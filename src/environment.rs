use std::{
    ffi::{c_void, CStr},
    ptr,
};

use glfw::{fail_on_errors, Context};

use crate::{
    buffers::framebuffer::FrameBuffer, drawing::draw::Draw, global_state::GlobalState,
    input::keyboard::Keyboard, input::mouse::Mouse, window::Window, EngineError, Result,
};

#[expect(unused_variables)]
extern "system" fn debug_callback(
    source: u32,
    gltype: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut c_void,
) {
    let message = unsafe { CStr::from_ptr(message) };
    let source = match source {
        gl::DEBUG_SOURCE_API => "DEBUG_SOURCE_API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "DEBUG_SOURCE_WINDOW_SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "DEBUG_SOURCE_SHADER_COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "DEBUG_SOURCE_THIRD_PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "DEBUG_SOURCE_APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "DEBUG_SOURCE_OTHER",
        _ => "ERROR_SOURCE_NOT_FOUND",
    };
    let gltype = match gltype {
        gl::DEBUG_TYPE_ERROR => "DEBUG_TYPE_ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEBUG_TYPE_DEPRECATED_BEHAVIOR",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "DEBUG_TYPE_UNDEFINED_BEHAVIOR",
        gl::DEBUG_TYPE_PORTABILITY => "DEBUG_TYPE_PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "DEBUG_TYPE_PERFORMANCE",
        gl::DEBUG_TYPE_MARKER => "DEBUG_TYPE_MARKER",
        gl::DEBUG_TYPE_PUSH_GROUP => "DEBUG_TYPE_PUSH_GROUP",
        gl::DEBUG_TYPE_POP_GROUP => "DEBUG_TYPE_POP_GROUP",
        gl::DEBUG_TYPE_OTHER => "DEBUG_TYPE_OTHER",
        _ => "ERROR TYPE NOT FOUND",
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "DEBUG_SEVERITY_HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "DEBUG_SEVERITY_MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "DEBUG_SEVERITY_LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => "DEBUG_SEVERITY_NOTIFICATION",
        _ => "ERROR SEVERITY NOT FOUND",
    };

    println!(
        "GL Callback:\n\tsource =  \t{source}\n\ttype =    \t{gltype}\n\tseverity =\t{severity}\n\tmessage = \t{message:?}",
    );
}

pub struct Environment<G: GlobalState> {
    global_state: Option<G>,

    mouse: Mouse,
    keyboard: Keyboard,
    default_framebuffer: FrameBuffer,

    old_frame: f64,
    frametime: f32,

    glfw: glfw::Glfw,
    window: Window,
    _events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl<G: GlobalState> Environment<G> {
    /// # Errors
    /// As string    
    #[inline]
    pub fn new(
        gl_version: (u32, u32),
        screen_dims: (u32, u32),
        title: &str,
        mouse_fix_to_centre: bool,
        //initial_state: G,
    ) -> crate::Result<Self> {
        let mut glfw = glfw::init(fail_on_errors!())
            .map_err(|_| crate::EngineError::GlfwErr("Error creating glfw".to_string()))?;

        let (gl_major, gl_minor) = gl_version;
        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(gl_major));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(gl_minor));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        // Create a windowed mode window and its OpenGL context
        let (mut internal_window, events) = glfw
            .create_window(
                screen_dims.0,
                screen_dims.1,
                title,
                glfw::WindowMode::Windowed,
            )
            .ok_or_else(|| EngineError::GlfwErr("Failed to create GLFW window".to_string()))?;

        internal_window.set_framebuffer_size_callback(|_, width, height| unsafe {
            gl::Viewport(0, 0, width, height);
        });

        internal_window.make_current();
        internal_window.set_key_polling(true);

        gl::load_with(|s| glfw.get_proc_address_raw(s));
        // Configure glfw

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::STENCIL_TEST);
            gl::Enable(gl::DEPTH_TEST);

            gl::DebugMessageCallback(Some(debug_callback), ptr::null_mut());
        };

        let mut window = Window::new(internal_window);
        let mouse = Mouse::new(&mut window, mouse_fix_to_centre);
        let keyboard = Keyboard::new(&mut window);

        let default_framebuffer = FrameBuffer::new_default();

        let mut out = Self {
            glfw,
            window,
            _events: events,
            global_state: None,
            default_framebuffer,

            mouse,
            keyboard,

            old_frame: 0.0,
            frametime: 0.0,
        };

        let state = G::new(&out)?;
        out.global_state = Some(state);

        Ok(out)
    }

    #[inline]
    fn window_open(&self) -> bool {
        !self.window.should_close()
    }

    #[inline]
    pub fn set_global_state(&mut self, state: G) {
        self.global_state = Some(state);
    }

    #[inline]
    pub fn global_state(&mut self) -> &mut G {
        self.global_state.as_mut().unwrap()
    }

    #[inline]
    pub fn get_screendims(&self) -> (i32, i32) {
        self.window.get_framebuffer_size()
    }

    fn poll(&mut self) -> Vec<Draw> {
        self.global_state.as_mut().unwrap().poll(
            &self.mouse,
            &self.keyboard,
            self.frametime,
            &mut self.window,
            &mut self.default_framebuffer,
            self.glfw.get_time() as f32,
        )
    }

    #[inline]
    pub fn run(&mut self) -> Result<()> {
        // Substitute for `for to_draw in env.iter() {`
        let mut frame_iter = self.iter();
        while let Some(to_draw) = frame_iter.next() {
            // Begin rendering code
            for draw in to_draw {
                draw.draw()?;
            }
        }

        Ok(())
    }

    fn end_render(&mut self) {
        // Poll for and process events
        self.glfw.poll_events();
        // Swap front and back buffers
        self.window.swap_buffers();
        // Now clear the old buffer ready to be rewritten
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    fn calculate_frametime(&mut self) {
        let curr_time = self.glfw.get_time();
        self.frametime = (curr_time - self.old_frame) as f32;
        self.old_frame = curr_time;
    }

    #[expect(clippy::iter_not_returning_iterator)]
    #[inline]
    pub fn iter(&mut self) -> FrameIter<G> {
        FrameIter::new(self)
    }
}

pub struct FrameIter<'a, G: GlobalState> {
    env: &'a mut Environment<G>,
}

impl<'env, G: GlobalState> FrameIter<'env, G> {
    fn new(env: &'env mut Environment<G>) -> Self {
        Self { env }
    }

    #[inline]
    pub fn next<'fi, 'draw>(&'fi mut self) -> Option<Vec<Draw<'draw>>>
    where
        'env: 'fi,
        'fi: 'draw,
    {
        if self.env.window_open() {
            self.env.end_render();
            self.env.calculate_frametime();
            let out = self.env.poll();

            Some(out)
        } else {
            None
        }
    }
}
