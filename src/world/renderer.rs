use super::vao_builder::{VaoBuilderForField, VaoBuilderForPlayer};
use crate::player::{Player, Tagger};
use crate::types::*;
use crate::FIELD_SIZE;
use re::texture::texture_atlas::TextureAtlasPos;
use re::vao::VaoBuffer;

pub const TEX_BLOCK_TOP: TextureAtlasPos = TextureAtlasPos::new(0, 0);
pub const TEX_BLOCK_DANGER: TextureAtlasPos = TextureAtlasPos::new(0, 2);
pub const TEX_BLOCK_SAFE: TextureAtlasPos = TextureAtlasPos::new(0, 3);

pub const TEX_PLAYER_TMP: TextureAtlasPos = TextureAtlasPos::new(0, 1);
pub const TEX_TAGGER: TextureAtlasPos = TextureAtlasPos::new(1, 0);

pub struct FieldRenderer<const X: usize, const Z: usize> {
    pub vao_buffer: VaoBuffer,
    tex_block_top: TextureUV,
    tex_block_danger: TextureUV,
    tex_block_safe: TextureUV,
    diff_up: na::SMatrix<i32, X, Z>,
    diff_down: na::SMatrix<i32, X, Z>,
    diff_left: na::SMatrix<i32, X, Z>,
    diff_right: na::SMatrix<i32, X, Z>,
}

impl FieldRenderer<FIELD_SIZE, FIELD_SIZE> {
    pub fn new() -> Self {
        let tex_block_top = TextureUV::of_atlas(&TEX_BLOCK_TOP);
        let tex_block_danger = TextureUV::of_atlas(&TEX_BLOCK_DANGER);
        let tex_block_safe = TextureUV::of_atlas(&TEX_BLOCK_SAFE);

        let mut vao_buffer = VaoBuffer::with_num_vertex(36 * FIELD_SIZE * FIELD_SIZE); // 立方体は36頂点から成る
        vao_buffer.add_floor(FIELD_SIZE, FIELD_SIZE, &tex_block_top);

        Self {
            vao_buffer,
            tex_block_top,
            tex_block_danger,
            tex_block_safe,
            diff_up: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_down: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_left: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
            diff_right: na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::zeros(),
        }
    }

    pub fn make_diff(&mut self, field: &FieldMatrix) {
        let slide_upward = field.clone().remove_row(0).insert_row(FIELD_SIZE - 2, 0);
        let slide_downward = field.clone().remove_row(FIELD_SIZE - 1).insert_row(0, 0);
        let slide_left = field.clone().remove_column(0).insert_column(FIELD_SIZE - 2, 0);
        let slide_right = field.clone().remove_column(FIELD_SIZE - 1).insert_column(0, 0);

        self.diff_up = field - slide_upward;
        self.diff_down = field - slide_downward;
        self.diff_left = field - slide_right; // ←  slide_leftと間違えているわけではない。1つ右にスライドした行列との差を取ると、各要素について1つ左の要素との差が取れる
        self.diff_right = field - slide_left;
    }

    pub fn render(&mut self, field: &FieldMatrix) {
        // 床以外削除
        self.vao_buffer
            .clear_preserving_first(36 * FIELD_SIZE * FIELD_SIZE);
        for x in 0..FIELD_SIZE {
            for z in 0..FIELD_SIZE {
                self.vao_buffer.add_cell(
                    x as i32,
                    z as i32,
                    field[(x, z)],
                    self.diff_up[(x, z)],
                    self.diff_down[(x, z)],
                    self.diff_left[(x, z)],
                    self.diff_right[(x, z)],
                    &self.tex_block_top,
                    &self.tex_block_danger,
                    &self.tex_block_safe,
                );
            }
        }
    }
}

pub struct PlayerRenderer {
    pub vao_buffer: VaoBuffer,
    tex_player: TextureUV,
    tex_tagger: TextureUV,
    players: Vec<Player>,
    tagger: Option<Tagger>,
}

impl PlayerRenderer {
    pub fn new() -> Self {
        Self {
            vao_buffer: VaoBuffer::with_num_vertex(24 * 2), // 1プレイヤーの頂点数は24、1つのゲームには最大2プレイヤー
            tex_player: TextureUV::of_atlas(&TEX_PLAYER_TMP),
            tex_tagger: TextureUV::of_atlas(&TEX_TAGGER),
            players: Vec::new(),
            tagger: None,
        }
    }

    pub fn calc_world_pos(pos: &Point2i, field: &FieldMatrix) -> Point3 {
        Point3::new(
            pos.x as f32 + 0.5,
            field[(pos.x as usize, pos.y as usize)] as f32 + 1.5,
            pos.y as f32 + 0.5,
        )
    }

    pub fn set_players(&mut self, players: Vec<Player>) {
        self.players = players;
    }

    pub fn set_tagger(&mut self, tagger: Tagger) {
        self.tagger = Some(tagger);
    }

    pub fn render(&mut self, field: &FieldMatrix) {
        self.vao_buffer.clear();
        for player in &self.players {
            let player_pos = Self::calc_world_pos(&player.pos, field);
            self.vao_buffer
                .add_player(&player_pos, &player.name, &self.tex_player);
        }
        if let Some(ref tagger) = self.tagger {
            let tagger_pos = Self::calc_world_pos(&tagger.pos, field);
            self.vao_buffer.add_tagger(&tagger_pos, &self.tex_tagger);
        }
    }
}
