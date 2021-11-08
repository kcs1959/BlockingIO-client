use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;

use re::gl;
use re::shader::Program;
use re::shader::Shader;
use re::shader::Uniform;
use re::shader::UniformVariables;
use re::texture::texture_atlas::TextureAtlasPos;
use re::vao::VaoConfigBuilder;
use uuid::Uuid;

mod api;
mod camera;
mod engine;
mod mock_server;
mod player;
mod setting_storage;
mod socketio_encoding;
mod types;
mod world;

use crate::api::json::DirectionJson;
use crate::camera::Camera;
use crate::engine::Engine;
use crate::mock_server::Api;
use crate::mock_server::ApiEvent;
use crate::player::Player;
use crate::setting_storage::Setting;
use crate::types::*;
use crate::world::World;

// 64x64ピクセルのテクスチャが4x4個並んでいる
pub const TEX_W: u32 = 64;
pub const TEX_H: u32 = 64;
pub const TEX_ATLAS_W: u32 = TEX_W * 4;
pub const TEX_ATLAS_H: u32 = TEX_H * 4;

const TEX_BLOCK_TOP: TextureAtlasPos = TextureAtlasPos::new(0, 0);
const TEX_PLAYER_TMP: TextureAtlasPos = TextureAtlasPos::new(0, 1);
const TEX_BLOCK_DANGER: TextureAtlasPos = TextureAtlasPos::new(0, 2);
const TEX_BLOCK_SAFE: TextureAtlasPos = TextureAtlasPos::new(0, 3);

const FIELD_SIZE: usize = 32;

fn main() {
    use tracing::{info, warn};

    {
        let level = if std::env::var("BLKIO_TRACE").unwrap_or("".to_string()) == "1" {
            tracing::Level::TRACE
        } else if cfg!(debug_assertions) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    let setting = Setting::load().expect_or_log("設定ファイルの読み込みに失敗");

    let socketio_thread = tokio::runtime::Runtime::new().unwrap_or_log();

    let mut engine = Engine::init();
    let gl = engine.gl().clone();
    let vert_shader = Shader::from_vert_file(gl.clone(), "rsc/shader/shader.vs").unwrap_or_log();
    let frag_shader = Shader::from_frag_file(gl.clone(), "rsc/shader/shader.fs").unwrap_or_log();
    let shader = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap_or_log();
    info!("shader program");

    let main_texture = engine
        .image_manager
        .load_image(Path::new("rsc/textures/atlas/main.png"), "atlas/main", true)
        .expect_or_log("テクスチャの読み込みに失敗");
    debug_assert_eq!(
        main_texture.width, TEX_ATLAS_W,
        "テクスチャのサイズが想定と違います"
    );
    debug_assert_eq!(
        main_texture.height, TEX_ATLAS_H,
        "テクスチャのサイズが想定と違います"
    );
    info!("load texture");

    let mut world = World::<FIELD_SIZE, FIELD_SIZE>::new();
    info!("world");

    let vao_config = VaoConfigBuilder::new(&shader)
        .texture(&main_texture)
        .material_specular(Vector3::new(0.2, 0.2, 0.2))
        .material_shininess(0.1_f32)
        .light_direction(Vector3::new(0.2, 1.0, 0.2))
        .ambient(Vector3::new(0.3, 0.3, 0.3))
        .diffuse(Vector3::new(0.5, 0.5, 0.5))
        .specular(Vector3::new(0.2, 0.2, 0.2))
        .build();
    let mut stage_vao = world.render_field().build(&gl, &vao_config);
    let mut player_vao = world.render_players().build(&gl, &vao_config);
    info!("vao buffer");

    let mut own_player_pos: Point3 = Point3::new(0.0, 0.0, 0.0);
    let mut camera = Camera::new(own_player_pos);

    let mut user_id = setting.uuid;
    let mut user_name: String = "".to_string();

    let mut client_state = ClientState::TitleScreen;

    let mut api = Api::new(&setting.server);
    let unhandled_events = Arc::new(Mutex::new(VecDeque::new()));

    // ゲーム開始から現在までのフレーム数。約60フレームで1秒
    let mut frames: u64 = 0;

    'main: loop {
        frames += 1;

        // OSのイベントを処理
        for event in engine.event_pump.poll_iter() {
            engine.imgui_sdl2.handle_event(&mut engine.imgui, &event);
            if engine.imgui_sdl2.ignore_event(&event) {
                continue;
            }

            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => client_state = ClientState::Quit,
                _ => {}
            }
        }

        let mut world_updated = false;
        // Socket.ioのイベントを処理
        socketio_thread.block_on(async {
            let mut lock = unhandled_events.lock().unwrap_or_log();
            while let Some(event) = lock.pop_front() {
                match event {
                    ApiEvent::UpdateUser { uid, name } => {
                        user_id = uid;
                        user_name = name;
                        client_state = ClientState::JoiningRoom { wait_frames: 0 };
                    }
                    ApiEvent::RoomStateOpening { room_id, room_name } => {
                        info!("room opening id: {}, name: {}", room_id, room_name);
                        client_state = ClientState::WaitingInRoom
                    }
                    ApiEvent::RoomStateFulfilled { room_id, room_name } => {
                        info!("room fulfilled id: {}, name: {}", room_id, room_name);
                        client_state = ClientState::Playing
                    }
                    ApiEvent::RoomStateEmpty { .. } => {
                        warn!("room-stateイベントによるとルームはEmptyです");
                    }
                    ApiEvent::RoomStateNotJoined => {
                        info!("ルームから切断されました。約3秒後に再接続します。");
                        client_state = ClientState::JoiningRoom { wait_frames: 60 * 3 };
                    }
                    ApiEvent::UpdateField { players, field } => {
                        if let Some(own_player) = find_own_player(&players, user_id) {
                            own_player_pos = world.player_world_pos(own_player);
                        } else {
                            // TODO: 自機がいないときのカメラの場所
                        }
                        world.update(field);
                        world.set_players(players);
                        world_updated = true;
                    }
                    ApiEvent::GameFinished { reason } => {
                        client_state = ClientState::GameFinished { reason }
                    }
                }
            }
        });

        // 画面をクリア
        let (width, height) = engine.window().drawable_size();
        unsafe {
            gl.Viewport(0, 0, width as i32, height as i32);

            gl.ClearColor(1.0, 1.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        match client_state {
            ClientState::TitleScreen => {
                let key_state = KeyboardState::new(&engine.event_pump);
                if key_state.is_scancode_pressed(Scancode::Space) {
                    client_state = ClientState::SettingConnection
                }
            }

            ClientState::SettingConnection => {
                // サーバーに接続
                let result = api.connect(&unhandled_events);
                if let Err(error) = result {
                    tracing::error!("サーバーに接続できません: {}", error);
                    client_state = ClientState::TitleScreen;
                    continue;
                }
                let result = api.setup_uid(setting.uuid);
                if let Err(error) = result {
                    tracing::error!("uidを設定できません: {}", error);
                    client_state = ClientState::TitleScreen;
                    continue;
                }
                client_state = ClientState::WaitingSettingUid;
            }

            ClientState::WaitingSettingUid => {}

            ClientState::JoiningRoom { wait_frames: 0 } => {
                match api.join_room("foo") {
                    Ok(_) => {
                        client_state = ClientState::WaitingInRoom;
                    }
                    Err(_) => {
                        warn!("ルームに入れませんでした。約3秒後に再接続します。");
                        client_state = ClientState::JoiningRoom { wait_frames: 60 * 3 };
                    }
                };
            }

            ClientState::JoiningRoom { wait_frames } => {
                tracing::trace!("join-room wait {}", wait_frames);
                client_state = ClientState::JoiningRoom {
                    wait_frames: wait_frames - 1,
                };
            }

            ClientState::WaitingInRoom => {}

            ClientState::Playing => {
                // 入力
                let key_state = KeyboardState::new(&engine.event_pump());
                if key_state.is_scancode_pressed(Scancode::W)
                    || key_state.is_scancode_pressed(Scancode::Up)
                {
                    api.try_move(&DirectionJson::Up);
                }
                if key_state.is_scancode_pressed(Scancode::S)
                    || key_state.is_scancode_pressed(Scancode::Down)
                {
                    api.try_move(&DirectionJson::Down);
                }
                if key_state.is_scancode_pressed(Scancode::D)
                    || key_state.is_scancode_pressed(Scancode::Right)
                {
                    api.try_move(&DirectionJson::Right);
                }
                if key_state.is_scancode_pressed(Scancode::A)
                    || key_state.is_scancode_pressed(Scancode::Left)
                {
                    api.try_move(&DirectionJson::Left);
                }

                // カメラ移動
                if world_updated {
                    camera.shade_to_new_position(own_player_pos, frames, 12);
                }
                camera.update_position(frames);

                // 描画用の変数を準備
                if world_updated {
                    stage_vao = world.render_field().build(&gl, &vao_config);
                    player_vao = world.render_players().build(&gl, &vao_config);
                }
                const SCALE: f32 = 0.5;
                let model_matrix: Matrix4 = Matrix4::identity().scale(SCALE);
                let view_matrix: Matrix4 = camera.view_matrix(SCALE);
                let projection_matrix: Matrix4 = Matrix4::new_perspective(
                    width as f32 / height as f32,
                    std::f32::consts::PI / 4.0f32,
                    0.1,
                    100.0,
                );
                let uniforms = {
                    let mut uniforms = UniformVariables::new();
                    use c_str_macro::c_str;
                    use Uniform::*;
                    uniforms.add(c_str!("uModel"), Matrix4(&model_matrix));
                    uniforms.add(c_str!("uView"), Matrix4(&view_matrix));
                    uniforms.add(c_str!("uProjection"), Matrix4(&projection_matrix));
                    uniforms.add(
                        c_str!("uViewPosition"),
                        TripleFloat(camera.pos().x, camera.pos().y, camera.pos().z),
                    );
                    uniforms
                };

                // 描画
                stage_vao.draw_triangles(&uniforms);
                player_vao.draw_triangles(&uniforms);
            }

            ClientState::GameFinished { ref reason } => {
                info!("ゲーム終了");
                match *reason {
                    GameFinishReason::Normal { winner } => {
                        if let Some(winner) = winner {
                            info!("勝者は{}", winner);
                        } else {
                            info!("引き分け");
                        }
                    }
                    GameFinishReason::Abnromal => info!("異常終了"),
                }
                client_state = ClientState::Quit; // TODO: request-after-gameイベントをemit
            }

            ClientState::Quit => {
                setting.save().expect_or_log("設定ファイルの保存に失敗");
                break 'main;
            }
        }

        engine.window().gl_swap_window();
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60)); // 60FPS
    }
}

/// 自機を見つける
/// `players`の要素数は2程度
fn find_own_player(players: &Vec<Player>, uid: Uuid) -> Option<&Player> {
    players.iter().find(|player| player.uid == uid)
}

#[derive(PartialEq)]
enum ClientState {
    /// タイトル画面
    TitleScreen,
    /// サーバーに接続してsetup-uidすべき状態
    SettingConnection,
    /// on-update-userイベントを待っている状態
    WaitingSettingUid,
    /// `wait_frames`フレーム後にjoin-roomすべき状態
    JoiningRoom { wait_frames: u32 },
    /// ルームに入っていて、ゲーム開始を待っている状態
    WaitingInRoom,
    /// ゲーム中
    Playing,
    /// ゲームが終了して、結果を表示している状態
    GameFinished { reason: GameFinishReason },
    /// アプリケーションを終了すべき状態
    Quit,
}

#[derive(PartialEq)]
pub enum GameFinishReason {
    Normal { winner: Option<Uuid> },
    Abnromal,
}
