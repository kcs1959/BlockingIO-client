use nalgebra as na;

pub struct Player {
    pub pos_grid: na::Point3<f32>,
    pub pos_smooth: na::Point3<f32>,
}

impl Player {
    pub fn new(pos: na::Point3<f32>) -> Self {
        Self {
            pos_grid: pos,
            pos_smooth: pos,
        }
    }
}
