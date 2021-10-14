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

    pub fn join_room(&mut self, _id: &str) -> Player {
        Player::new(Point3::<f32>::new(0.25, 0.75, 0.75))
    }

    pub fn try_move(&mut self, direction: &Direction, player: &Player) -> Point3<f32> {
        if self.frames % 30 == 0 {
            // 本来はブロックの高さなどの判定も行うが、このコードでは無条件に移動可能
            match *direction {
                Direction::Up => Point3::<f32>::new(
                    player.pos_grid.x + 0.5,
                    player.pos_grid.y,
                    player.pos_grid.z,
                ),
                Direction::Down => Point3::<f32>::new(
                    player.pos_grid.x - 0.5,
                    player.pos_grid.y,
                    player.pos_grid.z,
                ),
                Direction::Right => Point3::<f32>::new(
                    player.pos_grid.x,
                    player.pos_grid.y,
                    player.pos_grid.z + 0.5,
                ),
                Direction::Left => Point3::<f32>::new(
                    player.pos_grid.x,
                    player.pos_grid.y,
                    player.pos_grid.z - 0.5,
                ),
            }
        } else {
            player.pos_grid
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}
