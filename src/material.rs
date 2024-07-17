use crate::{shader_program::ShaderProgram, some_builder, texture::Texture, Result};

#[derive(Clone, Debug)]
pub struct Material {
    pub translucent: bool,
    pub shininess: f32,
    pub diffuse: Texture,
    pub specular_map: Texture,
    pub emission: Texture,
    pub emission_map: Texture,
}

impl Material {
    #[must_use]
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[must_use]
    #[inline]
    pub fn blank() -> Self {
        Self::builder().build()
    }

    /*
    getter_clone!(diffuse, Texture);
    getter_clone!(specular_map, Texture);
    getter_clone!(emission, Texture);
    getter_clone!(emission_map, Texture);
    getter_clone!(translucent, bool);
    */
    /// # Errors
    #[inline]
    pub fn bind_to(&self, shader: &ShaderProgram, name: &str) -> Result<()> {
        shader.set_uniform_fv(&format!("{name}.shininess"), [self.shininess])?;
        shader.bind_textures(vec![
            (&self.diffuse, &format!("{name}.diffuse")),
            (&self.specular_map, &format!("{name}.specular_map")),
            (&self.emission, &format!("{name}.emission")),
            (&self.emission_map, &format!("{name}.emission_map")),
        ])?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    translucent: bool,
    shininess: Option<f32>,
    diffuse: Option<Texture>,
    specular_map: Option<Texture>,
    emission: Option<Texture>,
    emission_map: Option<Texture>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    some_builder!(diffuse: Texture);
    some_builder!(specular_map: Texture);
    some_builder!(emission: Texture);
    some_builder!(emission_map: Texture);

    #[must_use]
    #[inline]
    pub const fn shininess(mut self, shininess: f32) -> Self {
        self.shininess = Some(shininess);
        self
    }

    #[must_use]
    #[inline]
    pub const fn is_translucent(mut self) -> Self {
        self.translucent = true;
        self
    }

    #[inline]
    pub fn build(self) -> Material {
        Material {
            translucent: self.translucent,
            shininess: self.shininess.unwrap_or(32.0),
            diffuse: self.diffuse.unwrap_or_else(Texture::blank),
            specular_map: self.specular_map.unwrap_or_else(Texture::blank),
            emission: self.emission.unwrap_or_else(Texture::blank),
            emission_map: self.emission_map.unwrap_or_else(Texture::blank),
        }
    }
}
