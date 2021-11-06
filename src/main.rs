use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use player::Player;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;

use re::gl;
use re::shader::Program;
use re::shader::Shader;
use re::shader::Uniform;
use re::shader::UniformVariables;
use re::texture::texture_atlas::TextureAtlasPos;
use re::vao::vao_config::VaoConfigBuilder;
use uuid::Uuid;

mod api;
mod camera;
mod engine;
mod field;
mod mock_server;
mod player;
mod setting_storage;
mod socketio_encoding;
mod types;
mod vao_ex;

use crate::api::json::DirectionJson;
use crate::camera::Camera;
use crate::engine::Engine;
use crate::field::Field;
use crate::mock_server::Api;
use crate::mock_server::ApiEvent;
use crate::setting_storage::Setting;
use crate::types::*;
use crate::vao_ex::VaoBuilderEx;

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
    use tracing::info;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let socketio_thread = tokio::runtime::Runtime::new().unwrap_or_log();
    let mut engine = Engine::init();
    let gl = engine.gl().clone();
    let vert_shader = Shader::from_vert_file(gl.clone(), "rsc/shader/shader.vs").unwrap_or_log();
    let frag_shader = Shader::from_frag_file(gl.clone(), "rsc/shader/shader.fs").unwrap_or_log();
    let shader = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap_or_log();
    info!("shader program");

    let setting = Setting::load().expect_or_log("設定ファイルの読み込みに失敗");

    let main_texture = engine
        .image_manager
        .load_image(Path::new("rsc/textures/atlas/main.png"), "atlas/main", true)
        .expect_or_log("テクスチャの読み込みに失敗");
    assert_eq!(
        main_texture.width, TEX_ATLAS_W,
        "テクスチャのサイズが想定と違います"
    );
    assert_eq!(
        main_texture.height, TEX_ATLAS_H,
        "テクスチャのサイズが想定と違います"
    );
    info!("load texture");

    let vao_config = VaoConfigBuilder::new(&shader)
        .texture(&main_texture)
        .material_specular(Vector3::new(0.2, 0.2, 0.2))
        .material_shininess(0.1_f32)
        .light_direction(Vector3::new(0.2, 1.0, 0.2))
        .ambient(Vector3::new(0.3, 0.3, 0.3))
        .diffuse(Vector3::new(0.5, 0.5, 0.5))
        .specular(Vector3::new(0.2, 0.2, 0.2))
        .build();

    let mut field = Field::<FIELD_SIZE, FIELD_SIZE>::new();

    let mut stage_vao_builder = VaoBuilder::new();
    let mut stage_vao = stage_vao_builder.build(&gl, &vao_config);

    const URL: &str = "http://localhost:3000";
    // const URL: &str = "http://13.114.119.94:3000";
    let mut api = Api::new(URL);
    let unhandled_events = Arc::new(Mutex::new(VecDeque::new()));
    api.connect(&unhandled_events)
        .expect_or_log("サーバーに接続できません");
    api.setup_uid(setting.uuid).expect_or_log("uidを設定できません");
    api.join_room("foo").expect_or_log("ルームに入れません");
    let mut own_player;
    let mut camera = Camera::new(Point3::new(0.0, 0.0, 0.0));

    let mut user_id = setting.uuid;
    let mut user_name: String = "".to_string();

    // ゲーム開始から現在までのフレーム数。約60フレームで1秒
    let mut frames: u64 = 0;

    let mut players = Vec::new();

    'main: loop {
        frames += 1;
        api.update();

        // OSのイベントを処理
        for event in engine.event_pump.poll_iter() {
            engine.imgui_sdl2.handle_event(&mut engine.imgui, &event);
            if engine.imgui_sdl2.ignore_event(&event) {
                continue;
            }

            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    setting.save().expect_or_log("設定ファイルの保存に失敗");
                    break 'main;
                }
                _ => {}
            }
        }

        let mut field_updated = false;
        // Socket.ioのイベントを処理
        socketio_thread.block_on(async {
            let mut lock = unhandled_events.lock().unwrap_or_log();
            while let Some(event) = lock.pop_front() {
                match event {
                    ApiEvent::UpdateField {
                        players: players_,
                        field: field_,
                    } => {
                        players = players_;

                        field.update(field_);
                        field_updated = true;
                    }
                    ApiEvent::JoinRoom => todo!(),
                    ApiEvent::UpdateRoomState => todo!(),
                    ApiEvent::UpdateUser { uid, name } => {
                        user_id = uid;
                        user_name = name;
                    }
                }
            }
        });

        let key_state = KeyboardState::new(&engine.event_pump());
        if key_state.is_scancode_pressed(Scancode::W) || key_state.is_scancode_pressed(Scancode::Up) {
            api.try_move(&DirectionJson::Up);
        }
        if key_state.is_scancode_pressed(Scancode::S) || key_state.is_scancode_pressed(Scancode::Down) {
            api.try_move(&DirectionJson::Down);
        }
        if key_state.is_scancode_pressed(Scancode::D) || key_state.is_scancode_pressed(Scancode::Right) {
            api.try_move(&DirectionJson::Right);
        }
        if key_state.is_scancode_pressed(Scancode::A) || key_state.is_scancode_pressed(Scancode::Left) {
            api.try_move(&DirectionJson::Left);
        }

        if field_updated {
            own_player = find_own_player(&players, user_id);
            if let Some(own_player) = own_player {
                camera.shade_to_new_position(own_player.pos, frames, 12);
            } else {
                // TODO: 自機がいないときのカメラの場所
            }
        }

        camera.update_position(frames);

        if field_updated {
            stage_vao_builder = VaoBuilder::new();
            stage_vao_builder.add_floor(FIELD_SIZE, FIELD_SIZE);
            field.add_to(&mut stage_vao_builder);
            stage_vao = stage_vao_builder.build(&gl, &vao_config);
        }

        let mut player_vao_builder = VaoBuilder::new();
        for player in &players {
            player_vao_builder.add_octahedron(&player.pos, 0.5, &TextureUV::of_atlas(&TEX_PLAYER_TMP));
        }
        let player_vao = player_vao_builder.build(&gl, &vao_config);

        let (width, height) = engine.window().drawable_size();

        unsafe {
            gl.Viewport(0, 0, width as i32, height as i32);

            gl.ClearColor(1.0, 1.0, 1.0, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        const SCALE: f32 = 0.5;
        let model_matrix = Matrix4::identity().scale(SCALE);
        let view_matrix = camera.view_matrix(SCALE);
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

        stage_vao.draw_triangles(&uniforms);
        player_vao.draw_triangles(&uniforms);

        engine.window().gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60)); // 60FPS
    }
}

/// 自機を見つける
/// `players`の要素数は2程度
fn find_own_player(players: &Vec<Player>, uid: Uuid) -> Option<&Player> {
    players.iter().find(|player| player.uid == uid)
}
