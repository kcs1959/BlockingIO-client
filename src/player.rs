use nalgebra as na;

pub struct Player {
    pub pos: na::Point3<f32>,
}

impl Player {
    pub fn new(pos: na::Point3<f32>) -> Self {
        Self { pos }
    }
}
