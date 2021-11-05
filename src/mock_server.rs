//! モックAPI
//! APIが完成したらhttpやwebsocketで通信するコードを書く

use std::{
    collections::VecDeque,
    error::Error,
    sync::{Arc, Mutex},
};

use nalgebra::Point3;
use rust_socketio::{Payload, Socket, SocketBuilder};
use uuid::Uuid;

use crate::{
    api::json::{DirectionJson, OnUpdateUserJson, SetupUidJson, SquareJson, UpdateFieldJson},
    player::Player,
    socketio_encoding::ToUtf8String,
    FIELD_SIZE,
};

pub struct Api {
    socket: Option<Socket>,
}

impl Api {
    pub fn new() -> Api {
        Api { socket: None }
    }

    pub fn connect(
        &mut self,
        queue: &Arc<Mutex<VecDeque<ApiEvent>>>,
    ) -> Result<(), rust_socketio::error::Error> {
        let queue_update_user = Arc::clone(queue);
        let queue_update_field = Arc::clone(queue);

        const URL: &str = "http://localhost:3000";
        // const URL: &str = "http://13.114.119.94:3000";
        self.socket = Some(
            SocketBuilder::new(URL)
                .on("error", |err, _| eprintln!("Error: {:#?}", err))
                .on(event::UPDATE_USER, move |payload, _| {
                    println!("on-update-user event");
                    let json: OnUpdateUserJson = serde_json::from_slice(&payload.to_utf8_bytes())
                        .expect("parsing error (on-update-user)");
                    let event = ApiEvent::UpdateUser {
                        uid: json.uid,
                        name: json.name,
                    };
                    queue_update_user.lock().unwrap().push_back(event);
                })
                .on(event::ROOM_STATE, |payload, _socket| {
                    println!("room-state event");
                    print_payload(&payload);
                })
                .on(event::UPDATE_FIELD, move |payload, _| {
                    println!("update-field event");
                    let json: UpdateFieldJson = serde_json::from_slice(&payload.to_utf8_bytes())
                        .expect("parsing erro (update-field event)");

                    let mut players = Vec::new();
                    for player in &json.player_list {
                        let player = Player::new(nalgebra::Point3::<f32>::new(
                            (FIELD_SIZE - 1) as f32 - player.position.row + 0.5,
                            1.5,
                            player.position.column + 0.5,
                        ));
                        players.push(player);
                    }

                    assert_eq!(json.battle_field.length, FIELD_SIZE as i32);
                    let field = make_height_array(json.battle_field.squares);

                    queue_update_field
                        .lock()
                        .unwrap()
                        .push_back(ApiEvent::UpdateField { players, field });
                })
                .connect()?,
        );
        Ok(())
    }

    pub fn setup_uid(&mut self, uid: Uuid) -> Result<(), Box<dyn Error>> {
        println!("setup-uid");
        self.socket.as_mut().unwrap().emit(
            event::SETUP_UID,
            serde_json::to_string(&SetupUidJson { user_id: uid })?,
        )?;
        Ok(())
    }

    pub fn update(&mut self) {}

    pub fn join_room(&mut self, _id: &str) -> Result<(), rust_socketio::error::Error> {
        println!("emitting join-room event");
        self.socket
            .as_mut()
            .unwrap()
            .emit(event::JOIN_ROOM, serde_json::json!("{}"))?;
        println!("emitted join-room event");
        Ok(())
    }

    pub fn try_move(&mut self, direction: &DirectionJson) {
        println!("try-move");
        self.socket
            .as_mut()
            .expect("no server")
            .emit(event::TRY_MOVE, serde_json::to_string(&direction).unwrap())
            .unwrap();
    }
}

pub enum ApiEvent {
    UpdateUser {
        uid: Uuid,
        name: String,
    },
    JoinRoom,
    UpdateRoomState,
    UpdateField {
        players: Vec<Player>,
        field: [[u32; FIELD_SIZE]; FIELD_SIZE],
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

fn make_height_array(
    squares: [[SquareJson; FIELD_SIZE]; FIELD_SIZE],
) -> [[u32; FIELD_SIZE]; FIELD_SIZE] {
    use array_macro::array;
    array![x => array![z => squares[FIELD_SIZE - 1 - x][z].height; FIELD_SIZE]; FIELD_SIZE]
}

fn print_payload(payload: &Payload) {
    match payload {
        Payload::String(str) => {
            println!("  Received: {}", str);
            let bytes = payload.to_utf8_bytes();
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                println!("  Json: {}", json);
            } else {
                println!("  Not a json");
            }
        }
        Payload::Binary(bin_data) => println!("  Received bytes: {:#?}", bin_data),
    }
}
