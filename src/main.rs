use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use imgui_sdl2::ImguiSdl2;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use sdl2::video::GLContext;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;

type Vector3 = nalgebra::Vector3<f32>;
type Matrix4 = nalgebra::Matrix4<f32>;
type Point3 = nalgebra::Point3<f32>;

use re::gl;
use re::gl::Gl;
use re::shader::Program;
use re::shader::Shader;
use re::shader::Uniform;
use re::shader::UniformVariables;
use re::texture::image_manager::ImageManager;
use re::texture::texture_atlas::TextureAtlasPos;
use reverie_engine as re;

type TextureUV = re::texture::texture_atlas::TextureUV<TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
type CuboidTextures<'a> =
    re::vao::vao_builder::CuboidTextures<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;
type VaoBuilder<'a> = re::vao::vao_builder::VaoBuilder<'a, TEX_W, TEX_H, TEX_ATLAS_W, TEX_ATLAS_H>;

mod api;
mod camera;
mod field;
mod mock_server;
mod player;
mod socketio_encoding;
mod vao_ex;

use crate::api::json::DirectionJson;
use crate::camera::Camera;
use crate::field::Field;
use crate::mock_server::Api;
use crate::mock_server::ApiEvent;
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

struct Game {
    _sdl: Sdl,
    _video_subsystem: VideoSubsystem,
    _timer_subsystem: TimerSubsystem,
    window: Window,
    _gl_context: GLContext, /* GLContextを誰かが所有していないとOpenGLを使えない */
    gl: Gl,
    imgui: imgui::Context,
    imgui_sdl2: ImguiSdl2,
    _imgui_renderer: imgui_opengl_renderer::Renderer,
    event_pump: EventPump,
    image_manager: ImageManager,
}

impl Game {
    fn init() -> Game {
        let sdl = sdl2::init().unwrap();
        println!("OK: init SDL2: {}", sdl2::version::version());
        let video_subsystem = sdl.video().unwrap();
        println!("OK: init SDL2 Video Subsystem");
        let timer_subsystem = sdl.timer().unwrap();
        println!("OK: init SDL2 Timer Subsystem");

        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            let (major, minor) = gl_attr.context_version();
            println!("OK: init OpenGL: version {}.{}", major, minor);
        }

        let window = video_subsystem
            .window("SDL", 900, 480)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        println!("OK: init window '{}'", window.title());

        let _gl_context = window.gl_create_context().unwrap();
        let gl = Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
        println!("OK: init GL context");

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });
        {
            use imgui::im_str;
            println!(
                "OK: init ImGui (Platform: {}, Renderer: {})",
                imgui.platform_name().unwrap_or(im_str!("Unknown")),
                imgui.renderer_name().unwrap_or(im_str!("Unknown"))
            );
        }

        let event_pump = sdl.event_pump().unwrap();
        println!("OK: init event pump");

        let image_manager = ImageManager::new(gl.clone());
        println!("OK: init ImageManager");

        Game {
            _sdl: sdl,
            _video_subsystem: video_subsystem,
            _timer_subsystem: timer_subsystem,
            window,
            _gl_context,
            gl,
            imgui,
            imgui_sdl2,
            _imgui_renderer: imgui_renderer,
            event_pump,
            image_manager,
        }
    }
}

fn main() {
    let socketio_thread = tokio::runtime::Runtime::new().unwrap();
    let mut game = Game::init();
    let gl = &game.gl;
    let vert_shader = Shader::from_vert_file(gl.clone(), "rsc/shader/shader.vs").unwrap();
    let frag_shader = Shader::from_frag_file(gl.clone(), "rsc/shader/shader.fs").unwrap();
    let shader = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap();
    println!("OK: shader program");

    let main_texture = game
        .image_manager
        .load_image(Path::new("rsc/textures/atlas/main.png"), "atlas/main", true)
        .unwrap();
    assert_eq!(
        main_texture.width, TEX_ATLAS_W,
        "テクスチャのサイズが想定と違います"
    );
    assert_eq!(
        main_texture.height, TEX_ATLAS_H,
        "テクスチャのサイズが想定と違います"
    );

    let mut field = Field::<FIELD_SIZE, FIELD_SIZE>::new();

    let mut stage_vao_builder = VaoBuilder::new();
    stage_vao_builder.attatch_program(&shader);
    let mut stage_vao = stage_vao_builder.build(gl);

    let mut api = Api::new();
    let unhandled_events = Arc::new(Mutex::new(VecDeque::new()));
    api.connect(Arc::clone(&unhandled_events))
        .expect("cannot connect");
    let player = api.join_room("foo").expect("cannot join room");
    let mut camera = Camera::new(player.pos);

    /* デバッグ用 */
    let depth_test = true;
    let blend = true;
    let wireframe = false;
    let culling = true;
    let alpha: f32 = 1.0;
    /* ベクトルではなく色 */
    let material_specular = Vector3::new(0.2, 0.2, 0.2);
    let material_shininess: f32 = 0.1;
    let light_direction = Vector3::new(0.2, 1.0, 0.2);
    /* ambient, diffuse, specular はベクトルではなく色 */
    let ambient = Vector3::new(0.3, 0.3, 0.3);
    let diffuse = Vector3::new(0.5, 0.5, 0.5);
    let specular = Vector3::new(0.2, 0.2, 0.2);

    // ゲーム開始から現在までのフレーム数。約60フレームで1秒
    let mut frames: u64 = 0;

    let mut players = vec![player];

    'main: loop {
        frames += 1;
        api.update();

        // OSのイベントを処理
        for event in game.event_pump.poll_iter() {
            game.imgui_sdl2.handle_event(&mut game.imgui, &event);
            if game.imgui_sdl2.ignore_event(&event) {
                continue;
            }

            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let mut moved = false;
        let mut field_updated = false;
        // Socket.ioのイベントを処理
        socketio_thread.block_on(async {
            let mut lock = unhandled_events.lock().unwrap();
            while let Some(event) = lock.pop_front() {
                match event {
                    ApiEvent::UpdateField {
                        players: players_,
                        field: field_,
                    } => {
                        players = players_;
                        moved = true;

                        field.update(field_);
                        field_updated = true;
                    }
                    ApiEvent::JoinRoom => todo!(),
                    ApiEvent::UpdateRoomState => todo!(),
                }
            }
        });

        let key_state = KeyboardState::new(&game.event_pump);
        if key_state.is_scancode_pressed(Scancode::W) {
            if let Some(new_pos) = api.try_move(&DirectionJson::Up, &players[0], frames) {
                players[0].pos = new_pos;
                moved = true;
            }
        }
        if key_state.is_scancode_pressed(Scancode::S) {
            if let Some(new_pos) = api.try_move(&DirectionJson::Down, &players[0], frames) {
                players[0].pos = new_pos;
                moved = true;
            }
        }
        if key_state.is_scancode_pressed(Scancode::D) {
            if let Some(new_pos) = api.try_move(&DirectionJson::Right, &players[0], frames) {
                players[0].pos = new_pos;
                moved = true;
            }
        }
        if key_state.is_scancode_pressed(Scancode::A) {
            if let Some(new_pos) = api.try_move(&DirectionJson::Left, &players[0], frames) {
                players[0].pos = new_pos;
                moved = true;
            }
        }
        if moved {
            camera.shade_to_new_position(players[0].pos, frames, 12);
        }

        camera.update_position(frames);

        if field_updated {
            stage_vao_builder = VaoBuilder::new();
            stage_vao_builder.attatch_program(&shader);
            stage_vao_builder.add_floor(FIELD_SIZE, FIELD_SIZE);
            field.add_to(&mut stage_vao_builder);
            stage_vao = stage_vao_builder.build(gl);
        }

        let mut player_vao_builder = VaoBuilder::new();
        player_vao_builder.attatch_program(&shader);
        for player in &players {
            player_vao_builder.add_octahedron(
                &player.pos,
                0.5,
                &TextureUV::of_atlas(&TEX_PLAYER_TMP),
            );
        }
        let player_vao = player_vao_builder.build(gl);

        let (width, height) = game.window.drawable_size();

        unsafe {
            if depth_test {
                gl.Enable(gl::DEPTH_TEST);
            } else {
                gl.Disable(gl::DEPTH_TEST);
            }

            if blend {
                gl.Enable(gl::BLEND);
                gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            } else {
                gl.Disable(gl::BLEND);
            }

            if wireframe {
                gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            } else {
                gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }

            if culling {
                gl.Enable(gl::CULL_FACE);
            } else {
                gl.Disable(gl::CULL_FACE);
            }
        }

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
            uniforms.add(c_str!("uAlpha"), Float(alpha));
            uniforms.add(
                c_str!("uViewPosition"),
                TripleFloat(players[0].pos.x, players[0].pos.y, players[0].pos.z),
            );
            uniforms.add(c_str!("uMaterial.specular"), Vector3(&material_specular));
            uniforms.add(c_str!("uMaterial.shininess"), Float(material_shininess));
            uniforms.add(c_str!("uLight.direction"), Vector3(&light_direction));
            uniforms.add(c_str!("uLight.ambient"), Vector3(&ambient));
            uniforms.add(c_str!("uLight.diffuse"), Vector3(&diffuse));
            uniforms.add(c_str!("uLight.specular"), Vector3(&specular));
            uniforms
        };

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, main_texture.gl_id);
            stage_vao.draw_triangles(&uniforms);
            player_vao.draw_triangles(&uniforms);
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        game.window.gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60)); // 60FPS
    }
}
