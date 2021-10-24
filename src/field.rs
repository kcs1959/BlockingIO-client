use crate::{vao_ex::VaoBuilderEx, VaoBuilder};

/// 各地点のブロックの高さを保持する構造体
pub struct Field<const X: usize, const Z: usize> {
    map: [[u32; X]; Z],
}

impl<const X: usize, const Z: usize> Field<X, Z> {
    pub fn new() -> Self {
        Self {
            map: [[0_u32; X]; Z],
        }
    }

    pub fn set_height(&mut self, height: u32, x: usize, z: usize) {
        self.map[x][z] = height;
    }

    pub fn add_to(&self, vao_builder: &mut VaoBuilder) {
        for x in 0..X {
            for z in 0..Z {
                vao_builder.add_block_with_height(x as i32, 1, z as i32, self.map[x][z] as i32);
            }
        }
    }
}
