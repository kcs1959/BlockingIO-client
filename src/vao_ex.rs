type VaoBuilder<'a> = re::vao::vao_builder::VaoBuilder<
    'a,
    { crate::TEX_W },
    { crate::TEX_H },
    { crate::TEX_ATLAS_W },
    { crate::TEX_ATLAS_H },
>;
type CuboidTextures<'a> = re::vao::vao_builder::CuboidTextures<
    'a,
    { crate::TEX_W },
    { crate::TEX_H },
    { crate::TEX_ATLAS_W },
    { crate::TEX_ATLAS_H },
>;
type TextureUV = re::texture::texture_atlas::TextureUV<
    { crate::TEX_W },
    { crate::TEX_H },
    { crate::TEX_ATLAS_W },
    { crate::TEX_ATLAS_H },
>;
use reverie_engine as re;

type Point3 = nalgebra::Point3<f32>;

use crate::types::BlockUnit;
use crate::{TEX_BLOCK_DANGER, TEX_BLOCK_SAFE, TEX_BLOCK_TOP};

// ReverieEngineのVaoBuilderに、Blocking.io特有の機能を追加するためのトレイト
pub trait VaoBuilderEx {
    /// ステージの床を追加する
    fn add_floor(&mut self, width: u32, height: u32);

    /// 高さに応じてブロックを追加する
    fn add_block_with_height(&mut self, x: i32, y: i32, z: i32, height: i32);

    fn add_tall_block(&mut self, x: i32, y: i32, z: i32, height: i32);

    /// 乗り越えられるブロックを追加する
    fn add_short_block(&mut self, x: i32, y: i32, z: i32);
}

fn add_block(
    vao_builder: &mut VaoBuilder,
    x: i32,
    y: i32,
    z: i32,
    dx: BlockUnit,
    dy: BlockUnit,
    dz: BlockUnit,
    textures: &CuboidTextures,
) {
    let x: BlockUnit = x.into();
    let y: BlockUnit = y.into();
    let z: BlockUnit = z.into();
    let dx: BlockUnit = dx.into();
    let dy: BlockUnit = dy.into();
    let dz: BlockUnit = dz.into();
    vao_builder.add_cuboid(
        &Point3::new(
            x.into_render_value(),
            y.into_render_value(),
            z.into_render_value(),
        ),
        &Point3::new(
            (x + dx).into_render_value(),
            (y + dy).into_render_value(),
            (z + dz).into_render_value(),
        ),
        &textures,
    );
}

impl<'a> VaoBuilderEx for VaoBuilder<'a> {
    fn add_floor(&mut self, width: u32, height: u32) {
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
                add_block(self, x, 0, z, 1.0.into(), 1.0.into(), 1.0.into(), &textures);
            }
        }
    }

    fn add_block_with_height(&mut self, x: i32, y: i32, z: i32, height: i32) {
        if height <= 0 {
            // pass
        } else if height == 1 {
            self.add_short_block(x, y, z);
        } else {
            self.add_tall_block(x, y, z, height);
        }
    }

    fn add_tall_block(&mut self, x: i32, y: i32, z: i32, height: i32) {
        let textures = CuboidTextures {
            top: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            bottom: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            south: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            north: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            west: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
            east: &TextureUV::of_atlas(&TEX_BLOCK_DANGER),
        };
        // 1ブロックは立方体の半分なので高さを1/2にする
        let height: BlockUnit = (height as f32 * 0.5).into();
        add_block(self, x, y, z, 1.0.into(), height, 1.0.into(), &textures);
    }

    fn add_short_block(&mut self, x: i32, y: i32, z: i32) {
        let textures = CuboidTextures {
            top: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            bottom: &TextureUV::of_atlas(&TEX_BLOCK_TOP),
            south: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            north: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            west: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
            east: &TextureUV::of_atlas(&TEX_BLOCK_SAFE),
        };
        add_block(self, x, y, z, 1.0.into(), 0.5.into(), 1.0.into(), &textures);
    }
}
