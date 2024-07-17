use std::iter;

use crate::{
    buffers::vertex_array::VertexArray, linear_algebra::{matrix::Matrix, orientation::Orientation, vector::Vector}, material::Material, shader_program::ShaderProgram, some_builder, Result
};

#[derive(Debug, Clone)]
pub struct Mesh {
    vertex_array: VertexArray,
    material: Material,
}

impl Mesh {
    #[must_use]
    #[inline]
    pub fn new(vertex_array: VertexArray, material: Material) -> Self {
        Self {
            vertex_array,
            material,
        }
    }

    pub(crate) fn draw(&self, shader_program: &ShaderProgram) -> Result<()> {
        self.material.bind_to(shader_program, "material")?;
        self.vertex_array.draw();
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Model {
    meshes: Vec<Mesh>,
    cull_face: bool,
    position: Vector<3>,
    orientation: Orientation,
    scale: f32,
}

impl Model {
    #[inline]
    pub fn new(
        mesh_iter: Vec<Mesh>,
        cull_face: bool,
        scale: f32,
        position: Vector<3>,
        orientation: Option<Orientation>,
    ) -> Self {
        Self {
            meshes: mesh_iter,
            cull_face,
            position,
            orientation: orientation.unwrap_or_default(),
            scale,
        }
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    #[inline]
    pub fn temp_set_location(&mut self, loc: Vector<3>) {
        self.position = loc;
    }

    #[inline]
    pub fn temp_set_all_material(&mut self, mat: Material) {
        self.meshes
            .iter_mut()
            .zip(iter::repeat(mat)) // is this an optimisation?
            .for_each(|(mesh, mat)| mesh.material = mat);
    }

    #[inline]
    pub fn temp_set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    #[must_use]
    #[inline]
    pub fn model_matrix(&self) -> Matrix<4, 4> {
        // translation * orientation * scale

        Matrix::transform_translate(self.position)
            * self.orientation.as_matrix(None)
            * Matrix::transform_scale(self.scale, self.scale, self.scale)
    }

    #[must_use]
    #[inline]
    pub const fn location(&self) -> Vector<3> {
        self.position
    }

    /// # Errors
    #[inline]
    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<()> {
        unsafe {
            if self.cull_face {
                gl::Enable(gl::CULL_FACE);
            } else {
                gl::Disable(gl::CULL_FACE);
            }
        }

        shader_program.set_uniform_mat4f("model", self.model_matrix())?;

        for mesh in &self.meshes {
            mesh.draw(shader_program)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    meshes: Vec<Mesh>,
    cull_face: bool,
    position: Option<Vector<3>>,
    orientation: Option<Orientation>,
    scale: Option<f32>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mesh(mut self, mesh: Mesh) -> Self {
        self.meshes.push(mesh);
        self
    }

    pub fn mesh_from(self, vertex_array: VertexArray, material: Material) -> Self {
        self.mesh(Mesh::new(vertex_array, material))
    }

    pub fn all_meshes(mut self, all_meshes: Vec<Mesh>) -> Self {
        self.meshes = all_meshes;
        self
    }

    some_builder!(position: Vector<3>);
    some_builder!(orientation: Orientation);
    some_builder!(scale: f32);
    
    pub fn cull_face(mut self, cull_face: bool) -> Self {
        self.cull_face = cull_face;
        self
    }

    pub fn set_material(mut self, material: Material) -> Self {
        for mesh in &mut self.meshes {
            mesh.material = material.clone();
        }
        self
    }

    pub fn build(self) -> Model {
        Model {
            meshes: self.meshes,
            cull_face: self.cull_face,
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            scale: self.scale.unwrap_or(1.0)
        }
    }
}

impl Model {
    pub fn cube(side_length: f32, material: Material) -> Result<Builder> {
        let out = Self::builder()
            .all_meshes(
                VertexArray::cube(side_length)?
                    .into_iter()
                    .map(|vao| Mesh::new(vao, material.clone()))
                    .collect()
            );

        Ok(out)
    }

    pub fn quad() -> QuadBuilder {
        QuadBuilder(Model::builder())
    }
}

pub struct QuadBuilder(Builder);

/*impl QuadBuilder {
    pub fn whole_screen(self) -> Self {
        self.0.mesh(mesh)
    }
}*/
