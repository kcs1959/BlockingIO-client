use crate::types::*;
use uuid::Uuid;

pub struct Player {
    pub pos: Point3,
    pub uid: Uuid,
    pub name: String,
}

impl Player {
    pub fn new(pos: Point3, uid: Uuid, name: String) -> Self {
        Self { pos, uid, name }
    }
}
