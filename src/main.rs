use std::collections::VecDeque;
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use uuid::Uuid;

use re::gl;
use re::shader::Program;
use re::shader::Shader;
use re::shader::Uniform;
use re::shader::UniformVariables;
use re::vao::Vao;
use re::vao::VaoConfigBuilder;

mod api;
mod camera;
mod engine;
mod gui_renderer;
mod mock_server;
mod player;
mod setting_storage;
mod socketio_encoding;
mod types;
mod world;

use crate::api::json::DirectionJson;
use crate::camera::Camera;
use crate::engine::Engine;
use crate::gui_renderer::GuiRenderer;
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
    info!("done Engine init");

    let gl = engine.gl().clone();

    let vert_shader = Shader::from_vert_code(
        gl.clone(),
        &CString::new(include_str!("../rsc_included/shader/world.vert")).unwrap_or_log(),
    )
    .unwrap_or_log();
    let frag_shader = Shader::from_frag_code(
        gl.clone(),
        &CString::new(include_str!("../rsc_included/shader/world.frag")).unwrap_or_log(),
    )
    .unwrap_or_log();
    let shader = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap_or_log();
    info!("world shader program");

    let vert_shader = Shader::from_vert_code(
        gl.clone(),
        &CString::new(include_str!("../rsc_included/shader/gui.vert")).unwrap_or_log(),
    )
    .unwrap_or_log();
    let frag_shader = Shader::from_frag_code(
        gl.clone(),
        &CString::new(include_str!("../rsc_included/shader/gui.frag")).unwrap_or_log(),
    )
    .unwrap_or_log();
    let shader_gui = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap_or_log();
    info!("gui shader program");

    let main_texture = engine
        .image_manager
        .load_from_file(Path::new("rsc/textures/atlas/main.png"), "atlas/main", true)
        .expect_or_log("テクスチャの読み込みに失敗");
    debug_assert_eq!(main_texture.width, TEX_ATLAS_W,);
    debug_assert_eq!(main_texture.height, TEX_ATLAS_H,);
    info!("load texture 1/2");

    let gui_texture = engine
        .image_manager
        .load_from_file(Path::new("rsc/textures/title.png"), "gui", false)
        .unwrap_or_log();
    info!("load texture 2/2");

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
    let mut field_vao = world.render_field().build(&gl, &vao_config);
    let mut player_vao = world.render_players().build(&gl, &vao_config);
    info!("world VAO buffers");

    let gui_vao_config = VaoConfigBuilder::new(&shader_gui).texture(&gui_texture).build();
    let (width, height) = engine.window().drawable_size();
    let mut gui_renderer = GuiRenderer::new(width, height, &gui_texture);
    info!("GUI Renderer");

    let mut own_player_pos: Point3 = Point3::new(16.0, 0.0, 16.0);
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
            if let Some(event) = lock.pop_front() {
                match event {
                    ApiEvent::UpdateUser { uid, name } => {
                        if client_state == ClientState::WaitingSettingUid {
                            user_id = uid;
                            user_name = name;
                            client_state = ClientState::JoiningRoom { wait_frames: 0 };
                        } else {
                            warn!(
                                "unexpected event ApiEvent::UpdateUser uid:{}. state: {:?}",
                                uid, client_state
                            );
                        }
                    }
                    ApiEvent::RoomStateOpening { room_id, room_name } => {
                        info!("room opening id: {}, name: {}", room_id, room_name);
                    }
                    ApiEvent::RoomStateFulfilled { room_id, room_name } => {
                        if client_state == ClientState::WaitingInRoom {
                            info!("room fulfilled id: {}, name: {}", room_id, room_name);
                            client_state = ClientState::Playing;
                        } else {
                            warn!(
                                "unexpected event ApiEvent::RoomStateFulfilled room_id: {}, room_name: {}. state: {:?}",
                                 room_id, room_name ,
                                client_state
                            );
                        }
                    }
                    ApiEvent::RoomStateEmpty { .. } => {
                        warn!("room-stateイベントによるとルームはEmptyです");
                    }
                    ApiEvent::RoomStateNotJoined => {
                        info!("ルームから切断されました。");
                        client_state = ClientState::TitleScreen;
                    }
                    ApiEvent::UpdateField { players, tagger, field } => {
                        if client_state == ClientState::Playing {
                            if let Some(own_player) = find_own_player(&players, user_id) {
                                own_player_pos = world.player_world_pos(own_player);
                            } else {
                                // TODO: 自機がいないときのカメラの場所
                            }
                            world.update(field);
                            world.set_players(players);
                            world.set_tagger(tagger);
                            world_updated = true;
                        } else {
                            warn!(
                                "unexpected event ApiEvent::UpdateField. state: {:?}",
                                client_state
                            );
                        }
                    }
                    ApiEvent::GameFinished { reason } => {
                        if client_state == ClientState::Playing {
                            client_state = ClientState::GameFinished { reason };
                        }else {
                            warn!("unexpected event ApiEvent::GameFinished reason: {:?}. state: {:?}", reason, client_state);
                        }
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
                gui_renderer.clear();
                gui_renderer.change_window_size(width, height);
                gui_renderer.draw_title();
                gui_renderer.draw_スペースキーでスタート();
                gui_renderer.render(&gl, &gui_vao_config);

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

            ClientState::WaitingSettingUid => {
                gui_renderer.clear();
                gui_renderer.change_window_size(width, height);
                gui_renderer.draw_接続中();
                gui_renderer.render(&gl, &gui_vao_config);
            }

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
                gui_renderer.clear();
                gui_renderer.change_window_size(width, height);
                gui_renderer.draw_接続中();
                gui_renderer.render(&gl, &gui_vao_config);
            }

            ClientState::WaitingInRoom => {
                gui_renderer.clear();
                gui_renderer.change_window_size(width, height);
                gui_renderer.draw_待機中();
                gui_renderer.render(&gl, &gui_vao_config);
            }

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

                if world_updated {
                    field_vao = world.render_field().build(&gl, &vao_config);
                    player_vao = world.render_players().build(&gl, &vao_config);
                }
                render_field_and_player_vao(&field_vao, &player_vao, &camera, 0.5, width, height);
            }

            ClientState::GameFinished { ref reason } => {
                gui_renderer.clear();
                gui_renderer.change_window_size(width, height);
                match *reason {
                    GameFinishReason::Normal { winner } => {
                        if let Some(winner) = winner {
                            if winner == user_id {
                                gui_renderer.draw_勝ち();
                            } else {
                                gui_renderer.draw_負け();
                            }
                        } else {
                            gui_renderer.draw_引き分け();
                        }
                    }
                    GameFinishReason::Abnromal => {
                        gui_renderer.draw_引き分け();
                    }
                };
                gui_renderer.draw_スペースキーでスタート();

                render_field_and_player_vao(&field_vao, &player_vao, &camera, 0.2, width, height);
                gui_renderer.render(&gl, &gui_vao_config);

                let key_state = KeyboardState::new(&engine.event_pump);
                if key_state.is_scancode_pressed(Scancode::Space) {
                    client_state = ClientState::TitleScreen; // TODO: request-after-gameイベントをemit
                }
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

fn render_field_and_player_vao(
    field_vao: &Vao,
    player_vao: &Vao,
    camera: &Camera,
    scale: f32,
    window_width: u32,
    window_height: u32,
) {
    let model_matrix: Matrix4 = Matrix4::identity().scale(scale);
    let view_matrix: Matrix4 = camera.view_matrix(scale);
    let projection_matrix: Matrix4 = Matrix4::new_perspective(
        window_width as f32 / window_height as f32,
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

    field_vao.draw_triangles(&uniforms);
    player_vao.draw_triangles(&uniforms);
}

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq, Debug)]
pub enum GameFinishReason {
    Normal { winner: Option<Uuid> },
    Abnromal,
}
