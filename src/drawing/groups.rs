use crate::{
    lighting::{far_light::FarLight, point_light::PointLight, spot_light::SpotLight},
    modelling::model::Model,
    shader_program::ShaderProgram,
    Result,
};

pub(crate) struct ModelGroup<'a> {
    pub model: &'a Model,
    pub shader_program: &'a ShaderProgram,
}

impl<'a> ModelGroup<'a> {
    pub(crate) fn draw(&self) -> crate::Result<()> {
        self.shader_program.use_program();
        self.model.draw(self.shader_program)?;

        Ok(())
    }
}

pub(crate) struct ListModelGroup<'a>(Vec<ModelGroup<'a>>);

impl<'a> ListModelGroup<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push_group(&mut self, group: ModelGroup<'a>) {
        self.0.push(group);
    }

    pub fn push_simple(&mut self, model: &'a Model, shader_program: &'a ShaderProgram) {
        self.push_group(ModelGroup {
            model,
            shader_program,
        });
    }

    pub fn as_vec(&self) -> &Vec<ModelGroup<'a>> {
        &self.0
    }
}

impl<'a> Extend<ModelGroup<'a>> for ListModelGroup<'a> {
    fn extend<T: IntoIterator<Item = ModelGroup<'a>>>(&mut self, iter: T) {
        Vec::extend(&mut self.0, iter);
    }
}

#[derive(Clone, Debug)]
pub struct TempListLights<'a> {
    pub point: &'a PointLight,
    pub far: &'a FarLight,
    pub spot: &'a SpotLight,
}

impl<'a> TempListLights<'a> {
    #[inline]
    pub fn new(point: &'a PointLight, far: &'a FarLight, spot: &'a SpotLight) -> Self {
        TempListLights { point, far, spot }
    }

    #[inline]
    pub fn bind(&self, shader_program: &ShaderProgram) -> Result<()> {
        self.point.bind_to(shader_program, "light")?;
        self.spot.bind_to(shader_program, "torch")?;
        self.far.bind_to(shader_program, "sun")?;

        Ok(())
    }
}
