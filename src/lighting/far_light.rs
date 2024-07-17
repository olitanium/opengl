use crate::{linear_algebra::vector::Vector, shader_program::ShaderProgram};

#[derive(Clone, Copy, Default, Debug)]
pub struct FarLight {
    direction: Vector<3>,

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl FarLight {
    #[inline]
    pub fn new(
        direction: Vector<3>,
        ambient: [f32; 3],
        diffuse: [f32; 3],
        specular: [f32; 3],
    ) -> Self {
        Self {
            direction: direction.normalize(),
            ambient,
            diffuse,
            specular,
        }
    }

    #[inline]
    pub fn set_dir(&mut self, direction: Vector<3>) {
        self.direction = direction;
    }

    #[inline]
    pub fn set_colour(&mut self, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) {
        self.ambient = ambient;
        self.diffuse = diffuse;
        self.specular = specular;
    }

    pub(crate) fn bind_to(&self, shader: &ShaderProgram, name: &str) -> crate::Result<()> {
        shader.set_uniform_fv(&format!("{name}.direction"), self.direction.into())?;
        shader.set_uniform_fv(&format!("{name}.ambient"), self.ambient)?;
        shader.set_uniform_fv(&format!("{name}.diffuse"), self.diffuse)?;
        shader.set_uniform_fv(&format!("{name}.specular"), self.specular)?;
        Ok(())
    }
}
