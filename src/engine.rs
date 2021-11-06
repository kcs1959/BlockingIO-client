use imgui_sdl2::ImguiSdl2;
use sdl2::video::{GLContext, Window};
use sdl2::{EventPump, Sdl, TimerSubsystem, VideoSubsystem};

use re::gl::Gl;
use re::texture::image_manager::ImageManager;

use crate::types::*;

pub struct Engine {
    _sdl: Sdl,
    _video_subsystem: VideoSubsystem,
    _timer_subsystem: TimerSubsystem,
    window: Window,
    _gl_context: GLContext, /* GLContextを誰かが所有していないとOpenGLを使えない */
    gl: Gl,
    pub imgui: imgui::Context,
    pub imgui_sdl2: ImguiSdl2,
    _imgui_renderer: imgui_opengl_renderer::Renderer,
    pub event_pump: EventPump,
    pub image_manager: ImageManager,
}

impl Engine {
    #[tracing::instrument("init engine")]
    pub fn init() -> Engine {
        use tracing::info;

        let sdl = sdl2::init().unwrap_or_log();
        info!("init SDL2: {}", sdl2::version::version());
        let video_subsystem = sdl.video().unwrap_or_log();
        info!("init SDL2 Video Subsystem");
        let timer_subsystem = sdl.timer().unwrap_or_log();
        info!("init SDL2 Timer Subsystem");

        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            let (major, minor) = gl_attr.context_version();
            info!("init OpenGL: version {}.{}", major, minor);
        }

        let window = video_subsystem
            .window("SDL", 900, 480)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap_or_log();
        info!("init window '{}'", window.title());

        let _gl_context = window.gl_create_context().unwrap_or_log();
        let gl = Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
        info!("init GL context");

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
        let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsystem.gl_get_proc_address(s) as _
        });
        info!(
            "init ImGui (Platform: {}, Renderer: {})",
            imgui.platform_name().unwrap_or(imgui::im_str!("Unknown")),
            imgui.renderer_name().unwrap_or(imgui::im_str!("Unknown"))
        );

        let event_pump = sdl.event_pump().unwrap_or_log();
        info!("init event pump");

        let image_manager = ImageManager::new(gl.clone());
        info!("init ImageManager");

        Engine {
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

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn gl(&self) -> &Gl {
        &self.gl
    }

    pub fn imgui(&self) -> &imgui::Context {
        &self.imgui
    }

    pub fn imgui_sdl2(&self) -> &ImguiSdl2 {
        &self.imgui_sdl2
    }

    pub fn event_pump(&self) -> &EventPump {
        &self.event_pump
    }

    pub fn image_manager(&self) -> &ImageManager {
        &self.image_manager
    }
}
