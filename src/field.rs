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
    map: na::SMatrix<i32, X, Z>,
    renderer: FieldRenderer<X, Z>,
}

impl Field<FIELD_SIZE, FIELD_SIZE> {
    pub fn new() -> Self {
        Self {
            map: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            renderer: FieldRenderer::new(),
        }
    }

    pub fn update(&mut self, height_map: na::SMatrix<i32, FIELD_SIZE, FIELD_SIZE>) {
        self.map = height_map;
        self.renderer.make_diff(&self.map);
    }

    pub fn render(&mut self) -> &VaoBuffer {
        self.renderer.render(&self.map);
        &self.renderer.vao_buffer
    }
}

struct FieldRenderer<const X: usize, const Z: usize> {
    vao_buffer: VaoBuffer,
    diff_up: na::SMatrix<i32, X, Z>,
    diff_down: na::SMatrix<i32, X, Z>,
    diff_left: na::SMatrix<i32, X, Z>,
    diff_right: na::SMatrix<i32, X, Z>,
}

impl FieldRenderer<FIELD_SIZE, FIELD_SIZE> {
    pub fn new() -> Self {
        let mut vao_buffer = VaoBuffer::with_num_vertex(36 * FIELD_SIZE * FIELD_SIZE); // 立方体は36頂点から成る
        vao_buffer.add_floor(FIELD_SIZE, FIELD_SIZE);
        Self {
            vao_buffer,
            diff_up: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_down: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_left: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_right: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
        }
    }

    pub fn make_diff(&mut self, map: &na::SMatrix<i32, FIELD_SIZE, FIELD_SIZE>) {
        let slide_upward = map.clone().remove_row(0).insert_row(FIELD_SIZE - 2, 0);
        let slide_downward = map.clone().remove_row(FIELD_SIZE - 1).insert_row(0, 0);
        let slide_left = map.clone().remove_column(0).insert_column(FIELD_SIZE - 2, 0);
        let slide_right = map.clone().remove_column(FIELD_SIZE - 1).insert_column(0, 0);

        self.diff_up = map - slide_upward;
        self.diff_down = map - slide_downward;
        self.diff_left = map - slide_right; // ←  slide_leftと間違えているわけではない。1つ右にスライドした行列との差を取ると、各要素について1つ左の要素との差が取れる
        self.diff_right = map - slide_left;
    }

    pub fn render(&mut self, map: &na::SMatrix<i32, FIELD_SIZE, FIELD_SIZE>) {
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
