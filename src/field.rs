use crate::{types::*, FIELD_SIZE};
use re::vao::VaoBuffer;

/// 各地点のブロックの高さを保持する構造体
///
/// ^ X軸  \
/// |  \
/// |  \
/// |  \
/// +---------> Z軸
pub struct Field<const X: usize, const Z: usize> {
    map: [[u32; X]; Z],
    renderer: FieldRenderer<X, Z>,
}

impl<const X: usize, const Z: usize> Field<X, Z> {
    pub fn new() -> Self {
        Self {
            map: [[0_u32; X]; Z],
            renderer: FieldRenderer::new(),
        }
    }

    pub fn set_height(&mut self, height: u32, x: usize, z: usize) {
        self.map[x][z] = height;
    }

    pub fn update(&mut self, height_map: [[u32; X]; Z]) {
        self.map = height_map;
    }

    pub fn render(&mut self) -> &VaoBuffer {
        self.renderer.render(&self.map);
        &self.renderer.vao_buffer
    }
}

struct FieldRenderer<const X: usize, const Z: usize> {
    vao_buffer: VaoBuffer,
}

impl<const X: usize, const Z: usize> FieldRenderer<X, Z> {
    pub fn new() -> Self {
        let mut vao_buffer = VaoBuffer::with_num_vertex(36 * FIELD_SIZE * FIELD_SIZE); // 立方体は36頂点から成る
        vao_buffer.add_floor(FIELD_SIZE, FIELD_SIZE);
        Self { vao_buffer }
    }

    pub fn render(&mut self, map: &[[u32; X]; Z]) {
        // 床以外削除
        self.vao_buffer
            .clear_preserving_first(36 * FIELD_SIZE * FIELD_SIZE);
        for x in 0..X {
            for z in 0..Z {
                self.vao_buffer
                    .add_block_with_height(x as i32, z as i32, map[x][z] as i32);
            }
        }
    }
}
