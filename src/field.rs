use nalgebra as na;

pub struct Field {
    pub pos: na::Point3<f32>,
    pub number: f32,
}

impl Field {
    pub fn new(pos: na::Point3<f32>) -> Self {
        Self {
            pos,
            number: 0.0_f32,
        }
    }
}
