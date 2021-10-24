use nalgebra as na;
use reverie_engine::interpolation::Interpolation;

pub struct Player {
    pub pos: na::Point3<f32>,
    pub pos_camera: na::Point3<f32>,
    pub interpolation_x: Interpolation<f32>,
    pub interpolation_y: Interpolation<f32>,
    pub interpolation_z: Interpolation<f32>,
}

impl Player {
    pub fn new(pos: na::Point3<f32>) -> Self {
        Self {
            pos,
            pos_camera: pos,
            interpolation_x: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_y: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_z: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
        }
    }
}
