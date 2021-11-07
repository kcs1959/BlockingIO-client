use crate::types::*;
use crate::{TEX_BLOCK_DANGER, TEX_BLOCK_SAFE, TEX_BLOCK_TOP};
use re::vao::VaoBuffer;

// TODO: vao_exをfieldに移動する(VaoFieldBuilderという名前にする)

// ReverieEngineのVaoBufferに、Blocking.io特有の機能を追加するためのトレイト
pub trait VaoBuilderEx {
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

    /// 高さに応じてブロックを追加する
    fn add_block_with_height(&mut self, x: i32, z: i32, height: i32);

    fn add_tall_block(&mut self, x: i32, z: i32, height: i32);

    /// 乗り越えられるブロックを追加する
    fn add_short_block(&mut self, x: i32, z: i32);
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

impl VaoBuilderEx for VaoBuffer {
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

    fn add_block_with_height(&mut self, x: i32, z: i32, height: i32) {
        if height <= 0 {
            // pass
        } else if height == 1 {
            self.add_short_block(x, z);
        } else {
            self.add_tall_block(x, z, height);
        }
    }

    fn add_tall_block(&mut self, x: i32, z: i32, height: i32) {
        let textures = CuboidTextures {
            top: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            bottom: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            south: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            north: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            west: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            east: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
        };
        // 1ブロックは立方体の半分なので高さを1/2にする
        let height: f32 = height as f32 * 0.5;
        add_block(self, x as f32, 1.0, z as f32, 1.0, height, 1.0, &textures);
    }

    fn add_short_block(&mut self, x: i32, z: i32) {
        let textures = CuboidTextures {
            top: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            bottom: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            south: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            north: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            west: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            east: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
        };
        add_block(self, x as f32, 1.0, z as f32, 1.0, 0.5, 1.0, &textures);
    }
}
