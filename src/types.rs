pub use reverie_engine as re;

pub use re::types::*;

pub type Point2i = na::Point2<i32>;

use crate::{TEX_ATLAS_H, TEX_ATLAS_W, TEX_H, TEX_W};

pub type TextureUV = re::texture::texture_atlas::TextureUV<TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
pub type CuboidTextures<'a> = re::vao::CuboidTextures<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;

// prelude
pub use crate::socketio_encoding::ToUtf8String;
pub use re::vao::VaoBuilder3DGeometry;
pub use tracing_unwrap::{OptionExt, ResultExt};
