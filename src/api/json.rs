use serde::{Deserialize, Serialize};

use crate::FIELD_SIZE;

#[derive(Deserialize)]
pub struct PositionJson {
    pub row: f32,
    pub column: f32,
}

#[derive(Deserialize, Serialize)]
pub enum DirectionJson {
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}

#[derive(Deserialize)]
pub enum PlayerStatusJson {
    #[serde(rename = "alive")]
    Alive,
    #[serde(rename = "dead")]
    Dead,
}

#[derive(Deserialize)]
pub struct PlayerJson {
    pub position: PositionJson,
    pub direction: DirectionJson,
    pub point: i32,
    pub uid: String,
    pub name: String,
    pub status: PlayerStatusJson,
}

#[derive(Deserialize)]
pub struct TaggerJson {
    pub position: PositionJson,
    pub direction: DirectionJson,
    pub name: String,
}

#[derive(Deserialize)]
pub struct SquareJson {
    pub height: u32,
}

#[derive(Deserialize)]
pub struct BattleFieldJson {
    pub length: i32,
    pub squares: [[SquareJson; FIELD_SIZE]; FIELD_SIZE],
}

#[derive(Deserialize)]
pub enum GameStatusJson {
    /// ゲーム開始前
    BeforeStart,
    /// ゲーム開始待ち
    PendingStart,
    /// ゲーム中
    InGame,
    /// ゲームが正常終了した場合
    Finish,
    /// ユーザが退出するなどの理由でゲームが終了した場合
    AbnormalEnd,
}

#[derive(Deserialize)]
pub struct UpdateFieldJson {
    pub winner: Option<PlayerJson>,
    #[serde(rename = "tickCount")]
    pub tick_count: i32,
    #[serde(rename = "battleField")]
    pub battle_field: BattleFieldJson,
    #[serde(rename = "listOfPlayer")]
    pub player_list: Vec<PlayerJson>,
    pub tagger: TaggerJson,
    pub state: GameStatusJson,
}
