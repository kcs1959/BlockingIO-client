pub use nalgebra as na;
pub use reverie_engine as re;

use crate::{TEX_ATLAS_H, TEX_ATLAS_W, TEX_H, TEX_W};

pub type TextureUV = re::texture::texture_atlas::TextureUV<TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
pub type CuboidTextures<'a> =
    re::vao::vao_builder::CuboidTextures<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
pub type VaoBuilder<'a> =
    re::vao::vao_builder::VaoBuilder<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;

pub type Vector3 = na::Vector3<f32>;
pub type Matrix4 = na::Matrix4<f32>;
pub type Point3 = na::Point3<f32>;

// prelude
pub use crate::socketio_encoding::ToUtf8String;
pub use crate::vao_ex::VaoBuilderEx;
