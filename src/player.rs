use nalgebra as na;
use uuid::Uuid;

pub struct Player {
    pub pos: na::Point3<f32>,
    pub uid: Uuid,
    pub name: String,
}

impl Player {
    pub fn new(pos: na::Point3<f32>, uid: Uuid, name: String) -> Self {
        Self { pos, uid, name }
    }
}
