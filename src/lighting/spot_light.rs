use crate::{linear_algebra::vector::Vector, shader_program::ShaderProgram};

#[derive(Clone, Copy, Default, Debug)]
pub struct SpotLight {
    position: Vector<3>,
    direction: Vector<3>,

    attenuation: [f32; 3],
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],

    cos_cut_off: f32,
    cos_outer_cut_off: f32,
}

impl SpotLight {
    #[expect(clippy::too_many_arguments)]
    #[inline]
    pub fn new(
        position: Vector<3>,
        direction: Vector<3>,

        attenuation: [f32; 3],
        ambient: [f32; 3],
        diffuse: [f32; 3],
        specular: [f32; 3],

        cos_cut_off: f32,
        cos_outer_cut_off: f32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),

            attenuation,
            ambient,
            diffuse,
            specular,

            cos_cut_off,
            cos_outer_cut_off,
        }
    }

    #[inline]
    pub fn set_pos(&mut self, position: Vector<3>) {
        self.position = position;
    }

    #[inline]
    pub fn set_colour(&mut self, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) {
        self.ambient = ambient;
        self.diffuse = diffuse;
        self.specular = specular;
    }

    #[inline]
    pub fn set_dir(&mut self, direction: Vector<3>) {
        self.direction = direction;
    }

    pub(crate) fn bind_to(&self, shader: &ShaderProgram, name: &str) -> crate::Result<()> {
        shader.set_uniform_fv(&format!("{name}.position"), self.position.into())?;
        shader.set_uniform_fv(&format!("{name}.direction"), self.direction.into())?;
        shader.set_uniform_fv(&format!("{name}.attenuation"), self.attenuation)?;
        shader.set_uniform_fv(&format!("{name}.ambient"), self.ambient)?;
        shader.set_uniform_fv(&format!("{name}.diffuse"), self.diffuse)?;
        shader.set_uniform_fv(&format!("{name}.specular"), self.specular)?;
        shader.set_uniform_fv(&format!("{name}.cos_cut_off"), [self.cos_cut_off])?;

        shader.set_uniform_fv(
            &format!("{name}.cos_outer_cut_off"),
            [self.cos_outer_cut_off],
        )?;

        Ok(())
    }
}
