use std::path::Path;

use imgui_sdl2::ImguiSdl2;
use sdl2::keyboard::KeyboardState;
use sdl2::keyboard::Scancode;
use sdl2::video::GLContext;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;

use sn::gl;
use sn::gl::Gl;
use sn::shader::Program;
use sn::shader::Shader;
use sn::shader::Uniform;
use sn::shader::UniformVariables;
use sn::texture::image_manager::ImageManager;
use sn::texture::texture_atlas::TextureUV;
use sn::vao::vao_builder::CuboidTextures;
use sn::vao::vao_builder::VaoBuilder;
use starry_night as sn;

mod player;

use crate::player::Player;

type Vector3 = nalgebra::Vector3<f32>;
type Matrix4 = nalgebra::Matrix4<f32>;
type Point3 = nalgebra::Point3<f32>;

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

fn add_block(vao_builder: &mut VaoBuilder, x: i32, y: i32, z: i32, textures: &CuboidTextures) {
    let x = x as f32 * 0.5;
    let y = y as f32 * 0.5;
    let z = z as f32 * 0.5;
    vao_builder.add_cuboid(
        &Point3::new(x, y, z),
        &Point3::new(x + 0.5, y + 0.5, z + 0.5),
        &textures,
    );
}

fn main() {
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

    let cuboid_textures = CuboidTextures {
        top: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
        bottom: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
        south: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
        north: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
        west: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
        east: &TextureUV::of_atlas(0, 0, 64, 64, main_texture.width, main_texture.height),
    };
    let mut vao_builder = VaoBuilder::new();
    for x in 0..16 {
        for z in 0..16 {
            add_block(&mut vao_builder, x, 0, z, &cuboid_textures);
            if (x + 1) % (z + 1) == 0 {
                add_block(&mut vao_builder, x, 1, z, &cuboid_textures);
            }
        }
    }
    vao_builder.attatch_program(shader);
    let vao = vao_builder.build(gl);

    let mut player = Player {
        pos: Point3::new(0.25, 5.0, 0.25),
    };

    /* デバッグ用 */
    let depth_test = true;
    let blend = true;
    let wireframe = false;
    let culling = true;
    let alpha: f32 = 1.0;
    /* ベクトルではなく色 */
    let material_specular = Vector3::new(0.2, 0.2, 0.2);
    let material_shininess: f32 = 0.1;
    let light_direction = Vector3::new(1.0, 1.0, 0.0);
    /* ambient, diffuse, specular はベクトルではなく色 */
    let ambient = Vector3::new(0.3, 0.3, 0.3);
    let diffuse = Vector3::new(0.5, 0.5, 0.5);
    let specular = Vector3::new(0.2, 0.2, 0.2);

    let mut frames: u64 = 0;
    'main: loop {
        frames += 1;

        // イベントを処理
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

        if frames % 30 == 0 {
            const SPEED: f32 = 0.5;
            let key_state = KeyboardState::new(&game.event_pump);
            if key_state.is_scancode_pressed(Scancode::W) {
                player.pos.x += SPEED;
            }
            if key_state.is_scancode_pressed(Scancode::S) {
                player.pos.x -= SPEED;
            }
            if key_state.is_scancode_pressed(Scancode::D) {
                player.pos.z += SPEED;
            }
            if key_state.is_scancode_pressed(Scancode::A) {
                player.pos.z -= SPEED;
            }
        }

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

        let model_matrix = Matrix4::identity();
        const DOWN: Vector3 = Vector3::new(0.0, -1.0, 0.0);
        const X_POSITIVE: Vector3 = Vector3::new(1.0, 0.0, 0.0);
        let view_matrix = Matrix4::look_at_rh(&player.pos, &(player.pos + DOWN), &X_POSITIVE);
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
                TripleFloat(player.pos.x, player.pos.y, player.pos.z),
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
            vao.draw_triangles(&uniforms);
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        game.window.gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60)); // 60FPS
    }
}
