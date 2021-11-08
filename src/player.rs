use crate::types::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Player {
    pub pos: Point2i,
    pub uid: Uuid,
    pub name: String,
}

impl Player {
    pub fn new(pos: Point2i, uid: Uuid, name: String) -> Self {
        Self { pos, uid, name }
    }
}
