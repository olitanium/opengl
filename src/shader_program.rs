use std::{ffi::CString, fs, ptr};

use crate::linear_algebra::matrix::Matrix;
use crate::EngineError::ShaderErr;
use crate::{error_fmt, Result};
use crate::{material::Material, texture::Texture};

struct Shader {
    id: u32,
}

impl Shader {
    fn new_generic_shader(type_: u32, source: &str) -> Self {
        let shader_source = CString::new(fs::read_to_string(source).unwrap()).unwrap();
        unsafe {
            let shader_id = gl::CreateShader(type_);
            gl::ShaderSource(shader_id, 1, &shader_source.as_ptr(), ptr::null());
            gl::CompileShader(shader_id);
            Self { id: shader_id }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

pub struct ShaderProgram {
    id: u32,
}

impl ShaderProgram {
    #[must_use]
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[inline]
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// # Errors
    fn get_uniform_location(&self, name: &str) -> Result<i32> {
        unsafe {
            let c_name = CString::new(name).map_err(|_| {
                ShaderErr(error_fmt!(
                    shader_program::Builder,
                    "NUL value found in uniform name"
                ))
            })?;
            Ok(gl::GetUniformLocation(self.id, c_name.as_ptr().cast()))
        }
    }

    /// # Errors
    #[inline]
    pub fn bind_textures(&self, texture_list: Vec<(&Texture, &str)>) -> Result<()> {
        for (index, (tex, name)) in texture_list.into_iter().enumerate() {
            tex.bind_to(index.try_into().expect("Texture index cannot exceed u32"))?;
            
            self.set_uniform_iv(
                name,
                [index.try_into().expect("Texture index cannot exceed u32")],
            )?;

        }
        Ok(())
    }

    /// # Errors
    #[inline]
    pub fn set_uniform_iv<const N: usize>(&self, name: &str, value: [i32; N]) -> Result<()> {
        let uniform_location = self.get_uniform_location(name)?;
        unsafe {
            gl::UseProgram(self.id);
            match N {
                1 => gl::Uniform1i(uniform_location, value[0]),
                2 => gl::Uniform2i(uniform_location, value[0], value[1]),
                3 => gl::Uniform3i(uniform_location, value[0], value[1], value[2]),
                4 => gl::Uniform4i(uniform_location, value[0], value[1], value[2], value[3]),
                x => {
                    return Err(ShaderErr(error_fmt!(
                        shader_program::ShaderProgram,
                        "Cannot pass vector greater than 4 (uniform {name} size is {x})"
                    )))
                }
            }
        }
        Ok(())
    }

    /// # Errors
    #[inline]
    pub fn set_uniform_mat4f(&self, name: &str, value: Matrix<4, 4>) -> Result<()> {
        let uniform_location = self.get_uniform_location(name)?;
        unsafe {
            gl::UseProgram(self.id);
            gl::UniformMatrix4fv(
                uniform_location,
                1,
                gl::FALSE,
                value.col_major().as_ptr().cast(),
            );
        }
        Ok(())
    }

    /// # Errors
    #[inline]
    pub fn set_uniform_fv<const N: usize>(&self, name: &str, value: [f32; N]) -> Result<()> {
        let uniform_location = self.get_uniform_location(name)?;
        unsafe {
            gl::UseProgram(self.id);
            match N {
                1 => gl::Uniform1f(uniform_location, value[0]),
                2 => gl::Uniform2f(uniform_location, value[0], value[1]),
                3 => gl::Uniform3f(uniform_location, value[0], value[1], value[2]),
                4 => gl::Uniform4f(uniform_location, value[0], value[1], value[2], value[3]),
                x => {
                    return Err(ShaderErr(error_fmt!(
                        shader_program::ShaderProgram,
                        "Cannot pass vector greater than 4 (uniform {name} size is {x})"
                    )))
                }
            };
        }
        Ok(())
    }

    /// # Errors
    #[inline]
    pub fn bind_material(&self, material: &Material) -> Result<()> {
        // unsafe {gl::UseProgram(self.id); }

        self.set_uniform_fv("material.shininess", [32.0])?;
        self.bind_textures(vec![
            (&material.diffuse, "material.diffuse"),
            (&material.specular_map, "material.specular"),
            (&material.emission_map, "material.emission_mask"),
            (&material.emission, "material.emission"),
        ])?;

        Ok(())
    }

    #[must_use]
    #[inline]
    pub const fn id(&self) -> u32 {
        self.id
    }
}

impl Drop for ShaderProgram {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[derive(Default)]
pub struct Builder {
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
}

impl Builder {
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[inline]
    pub fn add_vertex_shader(mut self, source: &str) -> Self {
        self.vertex_shader = Some(Shader::new_generic_shader(gl::VERTEX_SHADER, source));
        self
    }

    #[must_use]
    #[inline]
    pub fn add_fragment_shader(mut self, source: &str) -> Self {
        self.fragment_shader = Some(Shader::new_generic_shader(gl::FRAGMENT_SHADER, source));
        self
    }

    /// # Errors
    ///
    #[inline]
    pub fn build(self) -> crate::Result<ShaderProgram> {
        unsafe {
            let program_id = gl::CreateProgram();

            gl::AttachShader(
                program_id,
                self.vertex_shader
                    .ok_or_else(|| {
                        ShaderErr(error_fmt!(shader_program::Builder, "No Vertex Shader"))
                    })?
                    .id,
            );
            gl::AttachShader(
                program_id,
                self.fragment_shader
                    .ok_or_else(|| {
                        ShaderErr(error_fmt!(shader_program::Builder, "No Fragment Shader"))
                    })?
                    .id,
            );

            gl::LinkProgram(program_id);
            Ok(ShaderProgram { id: program_id })
        }
    }
}
