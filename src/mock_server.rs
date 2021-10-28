//! モックAPI
//! APIが完成したらhttpやwebsocketで通信するコードを書く

use nalgebra::Point3;

use crate::player::Player;

pub struct Api {
    frames: u64,
}

impl Api {
    pub fn new() -> Api {
        Api { frames: 0 }
    }

    pub fn update(&mut self) {
        self.frames += 1;
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn join_room(&mut self, _id: &str) -> Player {
        Player::new(Point3::<f32>::new(0.5, 1.5, 1.5))
    }

    pub fn try_move(&mut self, direction: &Direction, player: &Player) -> Option<Point3<f32>> {
        if self.frames % 12 == 0 {
            // 本来はブロックの高さなどの判定も行うが、このコードでは無条件に移動可能
            match *direction {
                Direction::Up => Some(Point3::<f32>::new(
                    player.pos.x + 1.0,
                    player.pos.y,
                    player.pos.z,
                )),
                Direction::Down => Some(Point3::<f32>::new(
                    player.pos.x - 1.0,
                    player.pos.y,
                    player.pos.z,
                )),
                Direction::Right => Some(Point3::<f32>::new(
                    player.pos.x,
                    player.pos.y,
                    player.pos.z + 1.0,
                )),
                Direction::Left => Some(Point3::<f32>::new(
                    player.pos.x,
                    player.pos.y,
                    player.pos.z - 1.0,
                )),
            }
        } else {
            None
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}
