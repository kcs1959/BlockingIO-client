use crate::types::*;
use crate::{TEX_BLOCK_DANGER, TEX_BLOCK_SAFE, TEX_BLOCK_TOP};

// ReverieEngineのVaoBuilderに、Blocking.io特有の機能を追加するためのトレイト
pub trait VaoBuilderEx {
    /// ステージの床を追加する
    fn add_floor(&mut self, width: usize, height: usize);

    /// 高さに応じてブロックを追加する
    fn add_block_with_height(&mut self, x: i32, z: i32, height: i32);

    fn add_tall_block(&mut self, x: i32, z: i32, height: i32);

    /// 乗り越えられるブロックを追加する
    fn add_short_block(&mut self, x: i32, z: i32);
}

fn add_block(
    vao_builder: &mut VaoBuilder,
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

impl VaoBuilderEx for VaoBuilder {
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
