use std::{iter, vec};

use opengl::{
    buffers::{
        framebuffer::{BufferColourType, FrameBuffer},
        vertex_array::VertexArray,
    },
    camera::Camera,
    drawing::{draw::Draw, groups::TempListLights},
    environment::Environment,
    global_state::GlobalState,
    input::keyboard::{Key::*, Keyboard},
    input::mouse::Mouse,
    lighting::{far_light::FarLight, point_light::PointLight, spot_light::SpotLight},
    linear_algebra::{orientation::Orientation, vector::Vector},
    material::Material,
    modelling::model::Model,
    shader_program::ShaderProgram,
    texture::Texture,
    window::Window,
    Result,
};

pub struct State {
    camera: Camera,
    rear_camera: Camera,
    invert_y: bool,
    sensitivity: f32,

    speed: [f32; 3],

    light: Model,
    containers: Vec<Model>,
    player: Model,

    reverse_fbo: FrameBuffer,
    forward_fbo: FrameBuffer,

    point_light: PointLight,
    far_light: FarLight,
    spotlight: SpotLight,

    box_shader: ShaderProgram,
    quad_shader: ShaderProgram,

    screen_quad: Model,
    rear_view_quad: Model,
}

impl GlobalState for State {
    fn poll<'b, 'a: 'b>(
        &'a mut self,
        mouse: &Mouse,
        keyboard: &Keyboard,
        frame_time: f32,
        window: &mut Window,
        default_framebuffer: &'a mut FrameBuffer,
        time: f32,
    ) -> Vec<Draw<'b>> {
        self.controls(mouse, keyboard, frame_time, window);
        self.physics(frame_time, time);

        self.prep_draw(default_framebuffer)
    }

    fn new(environment: &Environment<Self>) -> Result<Self> {
        let speed = [1.0, 1.0, 1.0];

        let invert_y = false;
        
        let (width, height) = environment.get_screendims();
        
        let camera = Camera::builder()
            .centre([-4.0, 0.0, 0.0])
            .orientation(
                Orientation::builder()
                    .fixed_up([0.0, 1.0, 0.0].into())
                    .looking_at([-4.0, 0.0, 0.0].into(), [0.0, 0.0, 0.0].into())
                    .build(),
            )
            .perspective(90.0, width as f32/height as f32, 0.1, 100.0)
            .build();
        
        let sensitivity = 0.5;

        let forward_fbo = FrameBuffer::builder()
            .add_colour(BufferColourType::TexRgb)
            .add_depth()
            .add_dims(width, height)
            .build()?;

        let reverse_fbo = FrameBuffer::builder()
            .add_colour(BufferColourType::TexRgb)
            .add_depth()
            .add_dims(width, height)
            .build()?;

        let container_material = {
            let container_tex = Texture::builder().image("assets/container.png")?.build()?;

            let container_specular = Texture::builder()
                .image("assets/containerspecular.png")?
                .build()?;

            let container_emission = Texture::builder().image("assets/matrix.jpg")?.build()?;

            let container_emission_map = Texture::builder()
                .image("assets/matrix_mask.png")?
                .mag_filter(gl::NEAREST)
                .build()?;

            Material::builder()
                .diffuse(container_tex)
                .specular_map(container_specular)
                .emission(container_emission)
                .emission_map(container_emission_map)
                .build()
        };

        let light_colour = [1.0, 0.0, 0.0];

        let light_material = {
            let light_diffuse = Texture::blank();
            let light_emission_map = Texture::grayscale(1.0, 1.0);
            let light_emission = Texture::builder()
                .monochrome([1.0, 0.0, 0.0, 1.0])
                .build()?;

            Material::builder()
                .diffuse(light_diffuse)
                .emission_map(light_emission_map)
                .emission(light_emission)
                .build()
        };

        let player_material = {
            let awesomeface = Texture::builder()
                .image("assets/awesomeface.png")?
                .build()?;
            Material::builder().diffuse(awesomeface).build()
        };

        let model_builder = Model::cube(1.0, Material::blank())?;

        let container_builder = model_builder
            .clone()
            .set_material(container_material)
            .cull_face(true);

        let light = model_builder
            .clone()
            .set_material(light_material)
            .cull_face(false)
            .scale(0.2)
            .build();

        let player = model_builder
            .clone()
            .set_material(player_material)
            .cull_face(true)
            .scale(0.2)
            .build();

        let cube_positions = [
            [0.0, -2.0, -2.0],
            [2.0, -2.0, -2.0],
            [-2.0, -2.0, -2.0],
            [0.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [-2.0, 0.0, -2.0],
            [0.0, 2.0, -2.0],
            [2.0, 2.0, -2.0],
            [-2.0, 2.0, -2.0],
            [0.0, -2.0, 0.0],
            [2.0, -2.0, 0.0],
            [-2.0, -2.0, 0.0],
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [0.0, -2.0, 2.0],
            [2.0, -2.0, 2.0],
            [-2.0, -2.0, 2.0],
            [0.0, 0.0, 2.0],
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, 2.0],
            [0.0, 2.0, 2.0],
            [2.0, 2.0, 2.0],
            [-2.0, 2.0, 2.0],
        ].map(Vector::new);

        let containers: Vec<Model> = iter::repeat(container_builder)
            .zip(cube_positions)
            .map(|(container_builder, position)| {
                //container.set_model_matrix(Matrix::transform_translate(position));
                container_builder
                    .position(position)
                    .orientation(Orientation::builder()
                        .looking_at(position, Vector::new([2.0, 2.0, 1.0]))
                        .build()
                    )
                    .build()
            })
            .collect();

        let box_shader = ShaderProgram::builder()
            .add_vertex_shader("src/shaders/vertex_shader.vert")
            .add_fragment_shader("src/shaders/fragment_shader.frag")
            .build()?;

        let quad_shader = ShaderProgram::builder()
            .add_vertex_shader("src/shaders/quad_vert.vert")
            .add_fragment_shader("src/shaders/quad_frag.frag")
            .build()?;

        let light_attenuation_array = [1.0, 0.09, 0.032];

        let point_light = PointLight::new(
            Vector::default(),
            light_attenuation_array,
            light_colour.map(|x| x * 0.2),
            light_colour.map(|x| x * 0.5),
            light_colour,
        );

        let sun_colour = [1.0; 3];
        let far_light = FarLight::new(
            Vector::new([0.0, -1.0, 0.0]),
            sun_colour.map(|x| x * 0.0),
            sun_colour.map(|x| x * 0.5),
            sun_colour.map(|x| x * 1.0),
        );

        let spotlight_colour = [1.0; 3];
        let spotlight = SpotLight::new(
            camera.centre(),
            camera.direction(),
            [1.0, 0.09, 0.032],
            spotlight_colour.map(|x| x * 0.2),
            spotlight_colour,
            spotlight_colour,
            20f32.to_radians().cos(),
            25f32.to_radians().cos(),
        );

        let quad_texcoord = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![1.0, 1.0], vec![0.0, 1.0]];

        let screen_corners = vec![
            vec![-1.0, -1.0, 0.0],
            vec![1.0, -1.0, 0.0],
            vec![1.0, 1.0, 0.0],
            vec![-1.0, 1.0, 0.0],
        ];

        let screen_quad = Model::builder()
            .mesh_from(
                VertexArray::builder()
                    .attribute("coords".into(), screen_corners)?
                    .attribute("texcoord".into(), quad_texcoord)?
                    .element_buffer(vec![0, 1, 2, 0, 2, 3])
                    .build()?,
                forward_fbo.as_material()?,
            )
            .build();

        let rear_view_width = 0.5;
        let rear_view_height = 0.25;

        let rear_view_corners = vec![
            vec![-rear_view_width, 1.0 - 2.0 * rear_view_height, -1.0],
            vec![rear_view_width, 1.0 - 2.0 * rear_view_height, -1.0],
            vec![rear_view_width, 1.0, -1.0],
            vec![-rear_view_width, 1.0, -1.0],
        ];
        let rear_view_texcoord = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![1.0, 1.0], vec![0.0, 1.0]];
        
        let rear_view_quad = Model::builder()
        .mesh_from(
            VertexArray::builder()
                .attribute("coords".into(), rear_view_corners)?
                .attribute("texcoord".into(), rear_view_texcoord)?
                .element_buffer(vec![0, 1, 2, 0, 2, 3])
                .build()?,
            reverse_fbo.as_material()?,
        )
        .build();
        
        let mut rear_camera = camera.clone();
        rear_camera.reverse_direction();
        rear_camera.reverse_x();

        Ok(Self {
            camera,
            rear_camera,
            invert_y,
            sensitivity,
            speed,
            light,
            containers,
            player,
            reverse_fbo,
            forward_fbo,
            point_light,
            far_light,
            spotlight,
            box_shader,
            quad_shader,
            screen_quad,
            rear_view_quad,
        })
    }
}

impl State {
    pub fn rear_view_camera(&self) -> Camera {
        let mut output = self.camera.clone();
        output.reverse_direction();
        output.reverse_x();
        output
    }

    pub fn controls(
        &mut self,
        mouse: &Mouse,
        keyboard: &Keyboard,
        frame_time: f32,
        window: &mut Window,
    ) {
        {
            if keyboard.get(Y) {
                self.light.temp_set_all_material(
                    Material::builder()
                        .emission(
                            Texture::builder()
                                .monochrome([0.0, 0.0, 0.0, 0.0])
                                .build()
                                .unwrap(),
                        )
                        .build(),
                );
                self.point_light.set_colour([0.0; 3], [0.0; 3], [0.0; 3])
            }
            if keyboard.get(Escape) {
                window.set_should_close(true);
            }
            if keyboard.get(A) {
                self.camera.move_right(-self.speed[0] * frame_time);
            }
            if keyboard.get(D) {
                self.camera.move_right(self.speed[0] * frame_time);
            }
            if keyboard.get(W) {
                self.camera.move_forward(self.speed[1] * frame_time);
                self.player
                    .temp_set_orientation(self.camera.visual_orientation())
            }
            if keyboard.get(S) {
                self.camera.move_forward(-self.speed[1] * frame_time);
            }
            if keyboard.get(LeftShift) | keyboard.get(RightShift) {
                self.camera.move_up(-self.speed[2] * frame_time);
            }
            if keyboard.get(Space) {
                self.camera.move_up(self.speed[2] * frame_time);
            }
            if keyboard.get(M) {
                self.camera.look_right(1.0 * frame_time);
            }
            if keyboard.get(N) {
                self.camera.look_right(-1.0 * frame_time);
            }
            if keyboard.get(Up) {
                self.camera.look_up(
                    if self.invert_y { -1.0 } else { 1.0 } * self.sensitivity * frame_time,
                );
            }
            if keyboard.get(Down) {
                self.camera.look_up(
                    if self.invert_y { 1.0 } else { -1.0 } * self.sensitivity * frame_time,
                );
            }
            if keyboard.get(Right) {
                self.camera.roll_clockwise(1.0 * frame_time);
            }
            if keyboard.get(Left) {
                self.camera.roll_clockwise(-1.0 * frame_time);
            }
            if keyboard.get(U) {
                self.camera.radius_out(-0.5 * frame_time);
            }
            if keyboard.get(J) {
                self.camera.radius_out(0.5 * frame_time);
            }
        }
        {
            let (dx, dy) = mouse.get_delta();

            self.camera
                .look_right(dx as f32 * self.sensitivity * frame_time);
            self.camera.look_up(
                if self.invert_y { 1.0 } else { -1.0 } * dy as f32 * self.sensitivity * frame_time,
            );
        }
        {
            self.rear_camera = self.rear_view_camera();

        }
    }

    fn physics(&mut self, _frame_time: f32, time: f32) {
        self.spotlight.set_pos(self.camera.centre());
        self.spotlight.set_dir(self.camera.direction());

        let light_pos = Vector::new([2.0 * time.sin(), 1.0, time.cos()]);

        self.light.temp_set_location(light_pos);
        self.point_light.set_pos(light_pos);

        self.player.temp_set_location(self.camera.centre());
        
        /*for container in &mut self.containers {
            let location = container.location();
            container.temp_set_orientation(Orientation::builder().looking_at(location, self.camera.centre()).fixed_up(Vector::new([0.0, 1.0, 0.0])).build());
        }*/
    }

    fn prep_draw<'draw, 'program: 'draw>(
        &'program mut self,
        default_fb: &'program mut FrameBuffer,
    ) -> Vec<Draw<'draw>> {
        // Forward FBO,
        // Rear FBO,
        // default main FBO,
        let mut out = Vec::new();
        for (fbo, camera, shader) in [
            (&mut self.forward_fbo, &self.camera, &self.box_shader),
            (&mut self.reverse_fbo, &self.rear_camera, &self.box_shader),
        ] {
            let mut draw = Draw::new(
                fbo,
                camera,
                TempListLights::new(&self.point_light, &self.far_light, &self.spotlight),
            );

            for model in &self.containers {
                draw.add_model(model, shader);
            }

            draw.add_model(&self.player, shader);
            draw.add_model(&self.light, shader);

            out.push(draw);
        }

        {
            let mut draw = Draw::new_quad(default_fb);
            draw.add_model(&self.screen_quad, &self.quad_shader);
            draw.add_model(&self.rear_view_quad, &self.quad_shader);
            out.push(draw);
        }

        out
    }
}
