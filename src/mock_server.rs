//! モックAPI
//! APIが完成したらhttpやwebsocketで通信するコードを書く

use std::{
    collections::VecDeque,
    error::Error,
    sync::{Arc, Mutex},
};

use rust_socketio::{Payload, Socket, SocketBuilder};
use uuid::Uuid;

use tracing::{debug, error, info, trace};

use crate::{
    api::json::{
        DirectionJson, OnUpdateUserJson, RoomStateEventJson, RoomStateJson, SetupUidJson, SquareJson,
        UpdateFieldJson,
    },
    player::Player,
    types::*,
    GameFinishReason, FIELD_SIZE,
};

pub struct Api {
    socket: Option<Socket>,
    url: String,
}

impl Api {
    pub fn new(url: &str) -> Api {
        Api {
            socket: None,
            url: url.to_string(),
        }
    }

    #[tracing::instrument(name = "connect API", skip_all, fields(url = %self.url))]
    pub fn connect(
        &mut self,
        queue: &Arc<Mutex<VecDeque<ApiEvent>>>,
    ) -> Result<(), rust_socketio::error::Error> {
        let queue_update_user = Arc::clone(queue);
        let queue_room_state = Arc::clone(queue);
        let queue_update_field = Arc::clone(queue);

        info!("connecting server");
        self.socket = Some(
            SocketBuilder::new(&self.url)
                .on("error", |err, _| error!("socket.io {:#?}", err))
                .on(event::UPDATE_USER, move |payload, _| {
                    debug!("{} event", event::UPDATE_USER);
                    let json: OnUpdateUserJson =
                        serde_json::from_slice(&payload.to_utf8_bytes()).unwrap_or_log();
                    let event = ApiEvent::UpdateUser {
                        uid: json.uid,
                        name: json.name,
                    };
                    queue_update_user.lock().unwrap_or_log().push_back(event);
                })
                .on(event::ROOM_STATE, move |payload, _socket| {
                    debug!("{} event", event::ROOM_STATE);
                    let json: RoomStateEventJson =
                        serde_json::from_slice(&payload.to_utf8_bytes()).unwrap_or_log();
                    let event = match json.state {
                        RoomStateJson::Opening => ApiEvent::RoomStateOpening {
                            room_id: json.roomId.unwrap_or_log(),
                            room_name: json.roomname.unwrap_or_log(),
                        },
                        RoomStateJson::Fulfilled => ApiEvent::RoomStateFulfilled {
                            room_id: json.roomId.unwrap_or_log(),
                            room_name: json.roomname.unwrap_or_log(),
                        },
                        RoomStateJson::Empty => ApiEvent::RoomStateEmpty {
                            room_id: json.roomId.unwrap_or_log(),
                            room_name: json.roomname.unwrap_or_log(),
                        },
                        RoomStateJson::notJoining => ApiEvent::RoomStateNotJoined,
                    };
                    queue_room_state.lock().unwrap_or_log().push_back(event);
                })
                .on(event::UPDATE_FIELD, move |payload, _| {
                    debug!("{} event", event::UPDATE_FIELD);
                    let json: UpdateFieldJson =
                        serde_json::from_slice(&payload.to_utf8_bytes()).unwrap_or_log();
                    let event = match json.state {
                        crate::api::json::GameStatusJson::BeforeStart
                        | crate::api::json::GameStatusJson::PendingStart => return,
                        crate::api::json::GameStatusJson::InGame => {
                            let mut players = Vec::new();
                            for player in &json.player_list {
                                let player = Player::new(
                                    Point2i::new(
                                        FIELD_SIZE as i32 - 1 - player.position.row,
                                        player.position.column,
                                    ),
                                    player.uid,
                                    player.name.clone(),
                                );
                                players.push(player);
                            }

                            debug_assert_eq!(json.battle_field.length, FIELD_SIZE as i32);
                            let field = make_height_matrix(json.battle_field.squares);

                            ApiEvent::UpdateField { players, field }
                        }
                        crate::api::json::GameStatusJson::Finish => ApiEvent::GameFinished {
                            reason: GameFinishReason::Normal {
                                winner: json.winner.map(|winner| winner.uid),
                            },
                        },
                        crate::api::json::GameStatusJson::AbnormalEnd => ApiEvent::GameFinished {
                            reason: GameFinishReason::Abnromal,
                        },
                    };

                    queue_update_field.lock().unwrap_or_log().push_back(event);
                })
                .connect()?,
        );
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn setup_uid(&mut self, uid: Uuid) -> Result<(), Box<dyn Error>> {
        info!("emitting");
        self.socket.as_mut().unwrap_or_log().emit(
            event::SETUP_UID,
            serde_json::to_string(&SetupUidJson { user_id: uid })?,
        )?;
        info!("done");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn join_room(&mut self, _id: &str) -> Result<(), rust_socketio::error::Error> {
        info!("emitting");
        self.socket
            .as_mut()
            .unwrap_or_log()
            .emit(event::JOIN_ROOM, serde_json::json!("{}"))?;
        info!("done");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn try_move(&mut self, direction: &DirectionJson) {
        debug!("emitting");
        self.socket
            .as_mut()
            .unwrap_or_log()
            .emit(event::TRY_MOVE, serde_json::to_string(&direction).unwrap_or_log())
            .unwrap_or_log();
    }
}

pub enum ApiEvent {
    UpdateUser {
        uid: Uuid,
        name: String,
    },
    RoomStateOpening {
        room_id: String,
        room_name: String,
    },
    RoomStateFulfilled {
        room_id: String,
        room_name: String,
    },
    /// クライアントに送られてくることはないはず
    RoomStateEmpty {
        room_id: String,
        room_name: String,
    },
    /// 入っていたルームが閉じられるなどして、ルームから追い出された状態
    RoomStateNotJoined,
    UpdateField {
        players: Vec<Player>,
        field: na::SMatrix<i32, FIELD_SIZE, FIELD_SIZE>,
    },
    GameFinished {
        reason: GameFinishReason,
    },
}

// https://github.com/kcs1959/BlockingIO-api/blob/main/src/routes/socketEvents.ts を写しただけ
#[allow(dead_code)]
mod event {
    pub const JOIN_ROOM: &str = "join-room";
    pub const ROOM_STATE: &str = "room-state";
    pub const FIND_AVAILABLE_ROOM: &str = "find-available-room";
    pub const UPDATE_FIELD: &str = "udpate-field"; // API側のタイポ
    pub const TRY_MOVE: &str = "try-move";
    pub const SETUP_UID: &str = "setup-uid";
    pub const UPDATE_USER: &str = "on-update-user";
}

fn make_height_matrix(
    squares: [[SquareJson; FIELD_SIZE]; FIELD_SIZE],
) -> na::SMatrix<i32, FIELD_SIZE, FIELD_SIZE> {
    na::SMatrix::<i32, FIELD_SIZE, FIELD_SIZE>::from_fn(|x, z| {
        squares[FIELD_SIZE - 1 - x][z].height as i32
    })
}

fn print_payload(payload: &Payload) {
    match payload {
        Payload::String(str) => {
            trace!("  Received: {}", str);
            let bytes = payload.to_utf8_bytes();
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                debug!("  Json: {}", json);
            } else {
                debug!("  Not a json");
            }
        }
        Payload::Binary(bin_data) => debug!("  Received bytes: {:#?}", bin_data),
    }
}
