use crate::{linear_algebra::{matrix::Matrix, orientation::Orientation, vector::Vector}, some_builder};

#[derive(Clone)]
pub struct Camera {
    centre: Vector<3>, // either the position of the head
    radius: f32,
    orientation: Orientation,
    visual_orientation: Orientation,
    x_reversed: bool,
    perspective: Matrix<4, 4>,
}

#[derive(Default)]
pub struct Builder {
    centre: Option<Vector<3>>,
    radius: f32,
    orientation: Option<Orientation>,
    perspective: Option<Matrix<4, 4>>,
    x_reversed: bool,
}

impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Builder {
    #[inline]
    pub fn centre(mut self, centre: [f32; 3]) -> Self {
        self.centre = Some(centre.into());
        self
    }

    #[inline]
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    some_builder!(orientation: Orientation);

    #[inline]
    pub fn perspective(mut self, fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        self.perspective = Some(Matrix::transform_perspective(fov, aspect, near, far));
        self
    }

    pub fn x_reversed(mut self, x_reversed: bool) -> Self {
        self.x_reversed = x_reversed;
        self
    }

    #[inline]
    pub fn build(self) -> Camera {
        let orientation = self.orientation.unwrap_or_default();
        Camera {
            centre: self.centre.unwrap_or_default(),
            radius: self.radius,
            orientation,
            x_reversed: self.x_reversed,
            visual_orientation: orientation,
            perspective: self
                .perspective
                .unwrap_or_else(|| Matrix::transform_perspective(90.0, 1.0, 0.1, 100.0)),
        }
    }
}

impl Camera {
    #[inline]
    pub fn builder() -> Builder {
        Builder::default()
    }

    #[inline]
    pub fn centre(&self) -> Vector<3> {
        self.centre
    }

    #[inline]
    pub fn reverse_direction(&mut self) {
        self.orientation.reverse_direction();
    }

    pub fn reverse_x(&mut self) {
        self.x_reversed = !self.x_reversed
    }

    #[inline]
    pub fn first_person(&self) -> bool {
        self.radius < 0.01
    }

    #[inline]
    pub fn position(&self) -> Vector<3> {
        if self.first_person() {
            self.centre
        } else {
            self.centre - self.direction().scale(self.radius)
        }
    }

    #[inline]
    pub fn direction(&self) -> Vector<3> {
        self.orientation.forward()
    }

    #[inline]
    pub fn orientaion(&self) -> Orientation {
        self.orientation
    }

    #[inline]
    pub fn visual_orientation(&self) -> Orientation {
        self.visual_orientation
    }

    #[inline]
    pub fn radius_out(&mut self, distance: f32) -> f32 {
        self.radius += distance;
        if self.radius < 0.0 {
            self.radius = 0.0;
        }
        self.radius
    }

    #[inline]
    pub fn move_right(&mut self, distance: f32) {
        let right = self.orientation.right();
        self.centre = self.centre + right.scale(distance);
    }

    #[inline]
    pub fn look_right(&mut self, angle: f32) {
        self.orientation.look_right(angle);
    }

    #[inline]
    pub fn move_up(&mut self, distance: f32) {
        self.centre = self.centre + self.orientation.view_up(None).scale(distance);
    }

    #[inline]
    pub fn look_up(&mut self, angle: f32) {
        self.orientation.look_up(angle);
    }

    #[inline]
    pub fn roll_clockwise(&mut self, angle: f32) {
        self.orientation.roll_clockwise(angle);
    }

    #[inline]
    pub fn move_forward(&mut self, distance: f32) {
        self.centre = self.centre + self.orientation.forward_motion_direction().scale(distance);
        self.visual_orientation = self.orientation.forward_motion_orientation();
    }

    #[inline]
    pub fn move_to(&mut self, position: Vector<3>) {
        self.centre = position;
    }

    #[inline]
    pub fn look_at(&self) -> Matrix<4, 4> {
        let mut camera_right = self.orientation.right();
        let camera_up = self.orientation.view_up(Some(camera_right));
        let direction = -self.direction();

        if self.x_reversed {camera_right = -camera_right;}

        #[rustfmt::skip]
        let lhs = Matrix::from_col_major([
            [ camera_right[0], camera_up[0], direction[0], 0.0,     ],
            [ camera_right[1], camera_up[1], direction[1], 0.0,     ],
            [ camera_right[2], camera_up[2], direction[2], 0.0,     ],
            [             0.0,          0.0,          0.0, 1.0_f32, ],
        ]);

        let camera_pos = self.position();

        let rhs = Matrix::transform_translate(-camera_pos);

        self.perspective * lhs * rhs
    }
}
