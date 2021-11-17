use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::FIELD_SIZE;

#[derive(Deserialize)]
pub struct PositionJson {
    pub row: i32,
    pub column: i32,
}

#[derive(Deserialize, Serialize, Debug)]
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
    pub uid: Uuid,
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
pub enum GameFinishReasonJson {
    Fall,
    Collision,
    Timeup,
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
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<GameFinishReasonJson>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct UserJson {
    pub name: String,
    pub point: i32,
    pub uid: Uuid,
    pub socketId: String,
    pub requestingToStartGame: bool,
}

#[derive(Deserialize)]
#[allow(non_camel_case_types)]
pub enum RoomStateJson {
    Empty,
    Opening,
    Fulfilled,
    notJoining,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
/// `status`が`notJoining`の場合は`status`以外のパラメータを持たない
pub struct RoomStateEventJson {
    pub maxMember: Option<i32>,
    pub assignedUsers: Vec<UserJson>,
    pub roomId: Option<String>,
    pub roomname: Option<String>,
    pub currentGame: Option<serde_json::Value>, // 型の詳細が不明
    pub state: RoomStateJson,
}

#[derive(Serialize)]
pub struct SetupUidJson {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct OnUpdateUserJson {
    pub name: String,
    pub point: i32,
    pub uid: Uuid,
    pub socketId: String,
    pub requestingToStartGame: bool,
}

#[derive(Serialize)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum RequestAfterGameJson {
    restart,
    leave,
}
