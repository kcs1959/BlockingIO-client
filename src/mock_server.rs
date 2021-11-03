//! モックAPI
//! APIが完成したらhttpやwebsocketで通信するコードを書く

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use nalgebra::Point3;
use rust_socketio::{Payload, Socket, SocketBuilder};

use crate::{
    api::json::{SquareJson, UpdateFieldJson},
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
        unhandled_events: Arc<Mutex<VecDeque<ApiEvent>>>,
    ) -> Result<(), rust_socketio::error::Error> {
        const URL: &str = "http://localhost:3000";
        // const URL: &str = "http://13.114.119.94:3000";
        self.socket = Some(
            SocketBuilder::new(URL)
                .on("error", |err, _| eprintln!("Error: {:#?}", err))
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

                    unhandled_events
                        .lock()
                        .unwrap()
                        .push_back(ApiEvent::UpdateField { players, field });
                })
                .connect()?,
        );
        Ok(())
    }

    pub fn update(&mut self) {}

    pub fn join_room(&mut self, _id: &str) -> Result<Player, rust_socketio::error::Error> {
        println!("emitting join-room event");
        self.socket
            .as_mut()
            .unwrap()
            .emit(event::JOIN_ROOM, serde_json::json!("{}"))?;
        println!("emitted join-room event");
        Ok(Player::new(Point3::<f32>::new(0.5, 1.5, 1.5)))
    }

    pub fn try_move(
        &mut self,
        direction: &Direction,
        player: &Player,
        frames: u64,
    ) -> Option<Point3<f32>> {
        if frames % 12 == 0 {
            // 本来はブロックの高さなどの判定も行うが、このコードでは無条件に移動可能
            match *direction {
                Direction::Up => Some(Point3::<f32>::new(
                    player.pos.x + 1.0,
                    player.pos.y,
                    player.pos.z,
                )),
                Direction::Down => Some(Point3::<f32>::new(
                    player.pos.x - 1.0,
                    player.pos.y,
                    player.pos.z,
                )),
                Direction::Right => Some(Point3::<f32>::new(
                    player.pos.x,
                    player.pos.y,
                    player.pos.z + 1.0,
                )),
                Direction::Left => Some(Point3::<f32>::new(
                    player.pos.x,
                    player.pos.y,
                    player.pos.z - 1.0,
                )),
            }
        } else {
            None
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

pub enum ApiEvent {
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
