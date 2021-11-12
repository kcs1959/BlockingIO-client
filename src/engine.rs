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
            .window(&format!("Blocking.io v{}", env!("CARGO_PKG_VERSION")), 900, 480)
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap_or_log();
        info!("init window '{}'", window.title());

        let _gl_context = window.gl_create_context().unwrap_or_log();
        let gl = Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
        info!("init GL context");

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

    pub fn event_pump(&self) -> &EventPump {
        &self.event_pump
    }
}
