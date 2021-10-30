//! モックAPI
//! APIが完成したらhttpやwebsocketで通信するコードを書く

use std::collections::VecDeque;

use nalgebra::Point3;
use rust_socketio::{Payload, Socket, SocketBuilder};

use crate::{player::Player, socketio_encoding::ToUtf8String};

pub struct Api {
    socket: Option<Socket>,
    unhandled_events: VecDeque<ApiEvent>,
}

impl Api {
    pub fn new() -> Api {
        Api {
            socket: None,
            unhandled_events: VecDeque::new(),
        }
    }

    pub fn connect(&mut self) -> Result<(), rust_socketio::error::Error> {
        const URL: &str = "http://localhost:3000";
        // const URL: &str = "http://13.114.119.94:3000";
        self.socket = Some(
            SocketBuilder::new(URL)
                .on("error", |err, _| eprintln!("Error: {:#?}", err))
                .on(event::ROOM_STATE, |payload, _socket| {
                    println!("room-state event");
                    print_payload(&payload);
                })
                .on(event::UPDATE_FIELD, |payload, _| {
                    println!("update-field event");
                    print_payload(&payload);
                })
                .connect()?,
        );
        Ok(())
    }

    pub fn update(&mut self) {}

    /// 未処理のイベントのうち最初のものを削除して返す
    ///
    /// 未処理のイベントがなかったらNoneを返す
    pub fn dequeue_event(&mut self) -> Option<ApiEvent> {
        self.unhandled_events.pop_front()
    }

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
    UpdateField { players: Vec<Player> },
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
