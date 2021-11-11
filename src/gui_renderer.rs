use re::gl::Gl;
use re::gui::layout::{Origin, Position, Rect};
use re::shader::UniformVariables;
use re::texture::dynamic_texture_atlas::DynamicTextureUV;
use re::texture::image_manager::ImageLoadInfo;
use re::vao::{VaoBuffer, VaoConfig};

use crate::types::*;

const TEX_TITLE_BLOCKING_IO: Rect<i32, u32> = Rect::new_const(88, 0, 190, 38);
const TEX_スペースキーでスタート: Rect<i32, u32> = Rect::new_const(0, 0, 88, 12);
const TEX_勝ち: Rect<i32, u32> = Rect::new_const(0, 12, 64, 12);
const TEX_負け: Rect<i32, u32> = Rect::new_const(0, 24, 64, 12);
const TEX_待機中: Rect<i32, u32> = Rect::new_const(0, 36, 52, 12);
const TEX_接続中: Rect<i32, u32> = Rect::new_const(0, 48, 36, 12);
const TEX_引き分け: Rect<i32, u32> = Rect::new_const(0, 72, 64, 12);

pub struct GuiRenderer {
    buffer: VaoBuffer,
    window_width: u32,
    window_height: u32,
    tex_title: DynamicTextureUV,
    tex_スペースキーでスタート: DynamicTextureUV,
    tex_勝ち: DynamicTextureUV,
    tex_負け: DynamicTextureUV,
    tex_待機中: DynamicTextureUV,
    tex_接続中: DynamicTextureUV,
    tex_引き分け: DynamicTextureUV,
}

impl GuiRenderer {
    pub fn new(window_width: u32, window_height: u32, gui_texture: &ImageLoadInfo) -> Self {
        let tex_title =
            DynamicTextureUV::new(&TEX_TITLE_BLOCKING_IO, gui_texture.width, gui_texture.height);
        let tex_スペースキーでスタート =
            DynamicTextureUV::new(&TEX_スペースキーでスタート, gui_texture.width, gui_texture.height);
        let tex_勝ち = DynamicTextureUV::new(&TEX_勝ち, gui_texture.width, gui_texture.height);
        let tex_負け = DynamicTextureUV::new(&TEX_負け, gui_texture.width, gui_texture.height);
        let tex_待機中 = DynamicTextureUV::new(&TEX_待機中, gui_texture.width, gui_texture.height);
        let tex_接続中 = DynamicTextureUV::new(&TEX_接続中, gui_texture.width, gui_texture.height);
        let tex_引き分け = DynamicTextureUV::new(&TEX_引き分け, gui_texture.width, gui_texture.height);

        Self {
            buffer: VaoBuffer::new(),
            window_width,
            window_height,
            tex_title,
            tex_スペースキーでスタート,
            tex_勝ち,
            tex_負け,
            tex_待機中,
            tex_接続中,
            tex_引き分け,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn change_window_size(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
    }

    pub fn draw_title(&mut self) {
        self.buffer.add_layout_rectangle(
            &self.tex_title,
            self.window_width,
            self.window_height,
            &Origin::Center,
            &Position::Center(0),
            &Position::Center((self.window_height as f32 * -0.2) as i32),
            (self.window_width as f32 * 0.8) as u32,
            (self.window_width as f32 * 0.8) as u32 / *TEX_TITLE_BLOCKING_IO.width()
                * *TEX_TITLE_BLOCKING_IO.height(),
        );
    }

    pub fn draw_スペースキーでスタート(&mut self) {
        self.buffer.add_layout_rectangle(
            &self.tex_スペースキーでスタート,
            self.window_width,
            self.window_height,
            &Origin::Center,
            &Position::Center(0),
            &Position::Negative((self.window_height as f32 * 0.2) as i32),
            (self.window_width as f32 * 0.4) as u32,
            (self.window_width as f32 * 0.4) as u32 / *TEX_スペースキーでスタート.width()
                * *TEX_スペースキーでスタート.height(),
        );
    }

    fn draw_text(
        buffer: &mut VaoBuffer,
        tex: &DynamicTextureUV,
        tex_rect: &Rect<i32, u32>,
        window_width: u32,
        window_height: u32,
    ) {
        buffer.add_layout_rectangle(
            tex,
            window_width,
            window_height,
            &Origin::Center,
            &Position::Center(0),
            &Position::Center(0),
            tex_rect.width() * 3,
            tex_rect.height() * 3,
        );
    }

    pub fn draw_接続中(&mut self) {
        Self::draw_text(
            &mut self.buffer,
            &self.tex_接続中,
            &TEX_接続中,
            self.window_width,
            self.window_height,
        );
    }

    pub fn draw_待機中(&mut self) {
        Self::draw_text(
            &mut self.buffer,
            &self.tex_待機中,
            &TEX_待機中,
            self.window_width,
            self.window_height,
        );
    }

    pub fn draw_勝ち(&mut self) {
        Self::draw_text(
            &mut self.buffer,
            &self.tex_勝ち,
            &TEX_勝ち,
            self.window_width,
            self.window_height,
        );
    }

    pub fn draw_負け(&mut self) {
        Self::draw_text(
            &mut self.buffer,
            &self.tex_負け,
            &TEX_負け,
            self.window_width,
            self.window_height,
        );
    }

    pub fn draw_引き分け(&mut self) {
        Self::draw_text(
            &mut self.buffer,
            &self.tex_引き分け,
            &TEX_引き分け,
            self.window_width,
            self.window_height,
        );
    }

    pub fn render(&self, gl: &Gl, vao_config: &VaoConfig) {
        let vao = self.buffer.build(gl, vao_config);
        let uniforms = {
            let mut uniforms = UniformVariables::new();
            use c_str_macro::c_str;
            use re::shader::Uniform::*;
            uniforms.add(c_str!("uWidth"), Float(self.window_width as f32));
            uniforms.add(c_str!("uHeight"), Float(self.window_height as f32));
            uniforms
        };
        vao.draw_triangles(&uniforms);
    }
}
