pub use reverie_engine as re;

pub use re::types::*;

use crate::{TEX_ATLAS_H, TEX_ATLAS_W, TEX_H, TEX_W};

pub type TextureUV = re::texture::texture_atlas::TextureUV<TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
pub type CuboidTextures<'a> =
    re::vao::vao_builder::CuboidTextures<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
pub type VaoBuilder = re::vao::vao_builder::VaoBuilder<TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;

// prelude
pub use crate::socketio_encoding::ToUtf8String;
pub use crate::vao_ex::VaoBuilderEx;
pub use tracing_unwrap::{OptionExt, ResultExt};
