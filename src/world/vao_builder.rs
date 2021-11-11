use crate::types::*;
use re::vao::VaoBuffer;

/// ReverieEngineのVaoBufferに、フィールド描画の機能を追加するためのトレイト
pub trait VaoBuilderForField {
    /// ステージの床を追加する
    fn add_floor(&mut self, width: usize, height: usize, texture: &TextureUV);

    fn add_cell(
        &mut self,
        x: i32,
        z: i32,
        height: i32,
        diff_up: i32,
        diff_down: i32,
        diff_left: i32,
        diff_right: i32,
        tex_top: &TextureUV,
        tex_danger: &TextureUV,
        tex_safe: &TextureUV,
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

impl VaoBuilderForField for VaoBuffer {
    fn add_floor(&mut self, width: usize, height: usize, texture: &TextureUV) {
        let textures = CuboidTextures {
            top: texture,
            bottom: texture,
            south: texture,
            north: texture,
            west: texture,
            east: texture,
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
        tex_top: &TextureUV,
        tex_danger: &TextureUV,
        tex_safe: &TextureUV,
    ) {
        let p_lu = Point3::new((x + 1) as f32, (height + 1) as f32, z as f32); // 上面の左上の点
        let p_ld = Point3::new(x as f32, (height + 1) as f32, z as f32); // 左下
        let p_rd = Point3::new(x as f32, (height + 1) as f32, (z + 1) as f32); // 右下
        let p_ru = Point3::new((x + 1) as f32, (height + 1) as f32, (z + 1) as f32); // 右上

        // 上面(カメラから見て正面にある面)
        self.add_face(&p_lu, &p_ld, &p_rd, &p_ru, tex_top);

        // 上の面(カメラから見て上にある面)
        if diff_up > 0 {
            let diff_up_vec = Vector3::new(0.0, diff_up as f32, 0.0);
            self.add_face(
                &p_ru,
                &(p_ru - diff_up_vec),
                &(p_lu - diff_up_vec),
                &p_lu,
                if diff_up == 1 { tex_safe } else { tex_danger },
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
                if diff_down == 1 { tex_safe } else { tex_danger },
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
                if diff_left == 1 { tex_safe } else { tex_danger },
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
                if diff_right == 1 { tex_safe } else { tex_danger },
            );
        }
    }
}

/// ReverieEngineのVaoBufferに、プレイヤー描画の機能を追加するためのトレイト
pub trait VaoBuilderForPlayer {
    fn add_player(&mut self, player_pos: &Point3, player_name: &str, tex_player: &TextureUV);
    fn add_tagger(&mut self, tagger_pos: &Point3, tex_tagger: &TextureUV);
}

impl VaoBuilderForPlayer for VaoBuffer {
    fn add_player(&mut self, player_pos: &Point3, _player_name: &str, tex_player: &TextureUV) {
        self.add_octahedron(&player_pos, 0.5, tex_player)
    }

    fn add_tagger(&mut self, tagger_pos: &Point3, tex_tagger: &TextureUV) {
        self.add_octahedron(&tagger_pos, 0.5, tex_tagger)
    }
}
