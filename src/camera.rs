use nalgebra as na;
use reverie_engine::interpolation::Interpolation;

pub struct Camera {
    pub pos: na::Point3<f32>,
    pub interpolation_x: Interpolation<f32>,
    pub interpolation_y: Interpolation<f32>,
    pub interpolation_z: Interpolation<f32>,
}

impl Camera {
    pub fn new(player_pos: na::Point3<f32>) -> Self {
        Self {
            pos: player_pos,
            interpolation_x: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_y: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_z: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
        }
    }
}
