// Items can be

// Opaque (or not)
// Face Cull (or not)
// Outline Groups -> Can be many and each have their own outline colour and width
// Sky cube (only one)

use crate::{
    buffers::framebuffer::FrameBuffer, camera::Camera, modelling::model::Model, shader_program::ShaderProgram,
    Result,
};

use super::groups::{ListModelGroup, TempListLights};

pub struct Draw<'a> {
    framebuffer: &'a mut FrameBuffer,
    camera: Option<&'a Camera>,
    pub lights: Option<TempListLights<'a>>,
    opaque: ListModelGroup<'a>,
}

impl<'a> Draw<'a> {
    #[inline]
    pub fn new(
        framebuffer: &'a mut FrameBuffer,
        camera: &'a Camera,
        lights: TempListLights<'a>,
    ) -> Self {
        Self {
            framebuffer,
            camera: Some(camera),
            lights: Some(lights),
            opaque: ListModelGroup::new(),
        }
    }

    #[inline]
    pub fn new_quad(framebuffer: &'a mut FrameBuffer) -> Self {
        Self {
            framebuffer,
            camera: None,
            lights: None,
            opaque: ListModelGroup::new(),
        }
    }

    #[inline]
    pub fn add_model(&mut self, model: &'a Model, shader_program: &'a ShaderProgram) {
        self.opaque.push_simple(model, shader_program);
    }

    #[inline]
    pub fn draw(self) -> Result<()> {
        self.framebuffer.bind();

        for model in self.opaque.as_vec() {
            if let Some(lightlist) = &self.lights {
                lightlist.bind(model.shader_program)?;
            }
            if let Some(camera) = self.camera {
                model
                    .shader_program
                    .set_uniform_mat4f("projtimesview", camera.look_at())?;
                model
                    .shader_program
                    .set_uniform_fv("camera_position", camera.position().into())?;
            }
            model.draw()?;
        }
        Ok(())
    }
}
