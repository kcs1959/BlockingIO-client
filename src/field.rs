use crate::{types::*, FIELD_SIZE, TEX_BLOCK_DANGER, TEX_BLOCK_SAFE, TEX_BLOCK_TOP};
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
        for x in 0..FIELD_SIZE {
            for z in 0..FIELD_SIZE {
                self.vao_buffer.add_cell(
                    x as i32,
                    z as i32,
                    map[(x, z)],
                    self.diff_up[(x, z)],
                    self.diff_down[(x, z)],
                    self.diff_left[(x, z)],
                    self.diff_right[(x, z)],
                );
            }
        }
    }
}

/// ReverieEngineのVaoBufferに、フィールド描画の機能を追加するためのトレイト
pub trait VaoBuilderOfField {
    /// ステージの床を追加する
    fn add_floor(&mut self, width: usize, height: usize);

    fn add_cell(
        &mut self,
        x: i32,
        z: i32,
        height: i32,
        diff_up: i32,
        diff_down: i32,
        diff_left: i32,
        diff_right: i32,
    );
}

fn add_block(
    vao_builder: &mut VaoBuffer,
    x: f32,
    y: f32,
    z: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    textures: &CuboidTextures,
) {
    vao_builder.add_cuboid(
        &Point3::new(x, y, z),
        &Point3::new(x + dx, y + dy, z + dz),
        &textures,
    );
}

impl VaoBuilderOfField for VaoBuffer {
    fn add_floor(&mut self, width: usize, height: usize) {
        let textures = CuboidTextures {
            top: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            bottom: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            south: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            north: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            west: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            east: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
        };
        for x in 0..width as i32 {
            for z in 0..height as i32 {
                add_block(self, x as f32, 0.0, z as f32, 1.0, 1.0, 1.0, &textures);
            }
        }
    }

    /// マス目を描画
    ///
    /// * `diff_up` - 1つ上のマス目に比べて、描画するマス目がどれだけ高いか
    /// * `diff_down` - 1つ下のマス目に比べて、描画するマス目がどれだけ高いか
    fn add_cell(
        &mut self,
        x: i32,
        z: i32,
        height: i32,
        diff_up: i32,
        diff_down: i32,
        diff_left: i32,
        diff_right: i32,
    ) {
        let p_lu = Point3::new((x + 1) as f32, (height + 1) as f32, z as f32); // 上面の左上の点
        let p_ld = Point3::new(x as f32, (height + 1) as f32, z as f32); // 左下
        let p_rd = Point3::new(x as f32, (height + 1) as f32, (z + 1) as f32); // 右下
        let p_ru = Point3::new((x + 1) as f32, (height + 1) as f32, (z + 1) as f32); // 右上

        // 上面(カメラから見て正面にある面)
        self.add_face(&p_lu, &p_ld, &p_rd, &p_ru, &TextureUV::of_atlas(&TEX_BLOCK_TOP));

        // 上の面(カメラから見て上にある面)
        if diff_up > 0 {
            let diff_up_vec = Vector3::new(0.0, diff_up as f32, 0.0);
            self.add_face(
                &p_ru,
                &(p_ru - diff_up_vec),
                &(p_lu - diff_up_vec),
                &p_lu,
                &TextureUV::of_atlas(if diff_up == 1 {
                    &TEX_BLOCK_SAFE
                } else {
                    &TEX_BLOCK_DANGER
                }),
            );
        }

        // 下の面
        if diff_down > 0 {
            let diff_down_vec = Vector3::new(0.0, diff_down as f32, 0.0);
            self.add_face(
                &p_ld,
                &(p_ld - diff_down_vec),
                &(p_rd - diff_down_vec),
                &p_rd,
                &TextureUV::of_atlas(if diff_down == 1 {
                    &TEX_BLOCK_SAFE
                } else {
                    &TEX_BLOCK_DANGER
                }),
            );
        }

        // 左の面
        if diff_left > 0 {
            let diff_left_vec = Vector3::new(0.0, diff_left as f32, 0.0);
            self.add_face(
                &p_lu,
                &(p_lu - diff_left_vec),
                &(p_ld - diff_left_vec),
                &p_ld,
                &TextureUV::of_atlas(if diff_left == 1 {
                    &TEX_BLOCK_SAFE
                } else {
                    &TEX_BLOCK_DANGER
                }),
            );
        }

        // 右の面
        if diff_right > 0 {
            let diff_right_vec = Vector3::new(0.0, diff_right as f32, 0.0);
            self.add_face(
                &p_rd,
                &(p_rd - diff_right_vec),
                &(p_ru - diff_right_vec),
                &p_ru,
                &TextureUV::of_atlas(if diff_right == 1 {
                    &TEX_BLOCK_SAFE
                } else {
                    &TEX_BLOCK_DANGER
                }),
            );
        }
    }
}
