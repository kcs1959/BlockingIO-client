use super::renderer::{FieldRenderer, PlayerRenderer};
use crate::{
    player::{Player, Tagger},
    types::*,
    FIELD_SIZE,
};
use re::vao::VaoBuffer;

pub struct World<const X: usize, const Z: usize> {
    /// 各地点のブロックの高さを保持する
    ///
    /// ^ X軸  \
    /// |  \
    /// |  \
    /// |  \
    /// +---------> Z軸
    field: na::SMatrix<i32, X, Z>,
    /// 最後にフィールドを描画したときからフィールドに変更があったかどうか
    field_updated: bool,
    /// 最後にプレイヤーを描画したときからプレイヤーに変更があったかどうか
    players_updated: bool,
    field_renderer: FieldRenderer<X, Z>,
    player_renderer: PlayerRenderer,
}

impl World<FIELD_SIZE, FIELD_SIZE> {
    pub fn new() -> Self {
        Self {
            field: FieldMatrix::zeros(),
            field_updated: true,
            players_updated: true,
            field_renderer: FieldRenderer::new(),
            player_renderer: PlayerRenderer::new(),
        }
    }

    pub fn player_world_pos(&self, player: &Player) -> Point3 {
        PlayerRenderer::calc_world_pos(&player.pos, &self.field)
    }

    pub fn update(&mut self, height_map: FieldMatrix) {
        self.field = height_map;
        self.field_renderer.make_diff(&self.field);
        self.field_updated = true;
    }

    pub fn set_players(&mut self, players: Vec<Player>) {
        self.player_renderer.set_players(players);
        self.players_updated = true;
    }

    pub fn set_tagger(&mut self, tagger: Tagger) {
        self.player_renderer.set_tagger(tagger);
        self.players_updated = true;
    }

    pub fn set_no_tagger(&mut self) {
        self.player_renderer.set_no_tagger();
        self.players_updated = true;
    }

    pub fn render_field(&mut self) -> &VaoBuffer {
        self.field_updated = false;
        self.field_renderer.render(&self.field);
        &self.field_renderer.vao_buffer
    }

    pub fn render_players(&mut self) -> &VaoBuffer {
        self.players_updated = false;
        self.player_renderer.render(&self.field);
        &self.player_renderer.vao_buffer
    }

    pub fn field_updated(&self) -> bool {
        self.field_updated
    }

    pub fn players_updated(&self) -> bool {
        self.players_updated
    }
}
