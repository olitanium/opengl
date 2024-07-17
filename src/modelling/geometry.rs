use crate::linear_algebra::{orientation::Orientation, vector::Vector};

struct Geometry {
    position: Vector<3>,
    orientation: Orientation,
    velocity: Vector<3>,
    angular_velocity: Orientation,
}