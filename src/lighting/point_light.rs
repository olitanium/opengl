use crate::{linear_algebra::vector::Vector, shader_program::ShaderProgram};

#[derive(Clone, Copy, Default, Debug)]
pub struct PointLight {
    position: Vector<3>,

    attenuation: [f32; 3],

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl PointLight {
    #[inline]
    pub const fn new(
        position: Vector<3>,
        attenuation: [f32; 3],
        ambient: [f32; 3],
        diffuse: [f32; 3],
        specular: [f32; 3],
    ) -> Self {
        Self {
            position,
            attenuation,
            ambient,
            diffuse,
            specular,
        }
    }

    #[inline]
    pub fn set_colour(&mut self, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) {
        self.ambient = ambient;
        self.diffuse = diffuse;
        self.specular = specular;
    }

    #[inline]
    pub fn set_pos(&mut self, position: Vector<3>) {
        self.position = position;
    }

    pub(crate) fn bind_to(&self, shader: &ShaderProgram, name: &str) -> crate::Result<()> {
        shader.set_uniform_fv(&format!("{name}.position"), self.position.into())?;
        shader.set_uniform_fv(&format!("{name}.attenuation"), self.attenuation)?;
        shader.set_uniform_fv(&format!("{name}.ambient"), self.ambient)?;
        shader.set_uniform_fv(&format!("{name}.diffuse"), self.diffuse)?;
        shader.set_uniform_fv(&format!("{name}.specular"), self.specular)?;
        Ok(())
    }
}
