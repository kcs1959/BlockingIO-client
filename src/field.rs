use crate::types::*;
use re::vao::vao_builder::VaoBuffer;

/// 各地点のブロックの高さを保持する構造体
///
/// ^ X軸  \
/// |  \
/// |  \
/// |  \
/// +---------> Z軸
pub struct Field<const X: usize, const Z: usize> {
    map: [[u32; X]; Z],
}

impl<const X: usize, const Z: usize> Field<X, Z> {
    pub fn new() -> Self {
        Self { map: [[0_u32; X]; Z] }
    }

    pub fn set_height(&mut self, height: u32, x: usize, z: usize) {
        self.map[x][z] = height;
    }

    pub fn update(&mut self, height_map: [[u32; X]; Z]) {
        self.map = height_map;
    }

    pub fn add_to(&self, vao_builder: &mut VaoBuffer) {
        for x in 0..X {
            for z in 0..Z {
                vao_builder.add_block_with_height(x as i32, z as i32, self.map[x][z] as i32);
            }
        }
    }
}
