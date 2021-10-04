use imgui_sdl2::ImguiSdl2;
use sdl2::video::GLContext;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;

use sn::gl::Gl;
use sn::texture::image_manager::ImageManager;
use starry_night as sn;

use crate::shader::Program;
use crate::shader::Shader;

pub mod shader;

struct Game {
    sdl: Sdl,
    _video_subsystem: VideoSubsystem,
    timer_subsystem: TimerSubsystem,
    window: Window,
    _gl_context: GLContext, /* GLContextを誰かが所有していないとOpenGLを使えない */
    gl: Gl,
    shader: Program,
    imgui: imgui::Context,
    imgui_sdl2: ImguiSdl2,
    imgui_renderer: imgui_opengl_renderer::Renderer,
    event_pump: EventPump,
    _image_manager: ImageManager,
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

        let vert_shader = Shader::from_vert_file(gl.clone(), "rsc/shader/shader.vs").unwrap();
        let frag_shader = Shader::from_frag_file(gl.clone(), "rsc/shader/shader.fs").unwrap();
        let shader = Program::from_shaders(gl.clone(), &[vert_shader, frag_shader]).unwrap();
        println!("OK: shader program");

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

        let mut image_manager = ImageManager::new(gl.clone());
        println!("OK: init ImageManager");

        Game {
            sdl,
            _video_subsystem: video_subsystem,
            timer_subsystem,
            window,
            _gl_context,
            gl,
            shader,
            imgui,
            imgui_sdl2,
            imgui_renderer,
            event_pump,
            _image_manager: image_manager,
        }
    }
}

fn main() {
    let mut game = Game::init();

    'main: loop {
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

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60)); // 60FPS
    }
}
