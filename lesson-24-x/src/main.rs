extern crate gl;
extern crate sdl2;
extern crate ui;
extern crate nalgebra as na;
extern crate failure;
extern crate lesson_24_x_render as render;
extern crate resources;
extern crate lesson_24_x_render_gl as render_gl;
#[macro_use]
extern crate lesson_24_x_render_gl_derive as render_gl_derive;
extern crate floating_duration;

pub mod profiling;
pub mod debug;
pub mod system;

use failure::err_msg;
use std::time::{Instant, Duration};
use floating_duration::TimeAsFloat;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = resources::Resources::new()
        .loaded_from(
            "core", 0,
            resources::backend::FileSystem::from_rel_path(env!("CARGO_MANIFEST_DIR"), "core")
                .with_write()
                .with_watch(),
        );

    let config_resource = resources.resource("Config.toml");
    let config = config_resource.get().unwrap();

    let schema = ui::schema::Root::new(ui::Size::new(800.0.into(), 500.0.into()))
        .with_container(
            ui::schema::Container::PaneLeft(
                ui::schema::PaneLeft::new(40.0.into())
                    .with_bg_color(ui::Color::new(0.5, 0.1, 0.1))
            )
        );

    let mut mutator = ui::mutator::Mutator::from_schema(&schema);

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);
    gl_attr.set_accelerated_visual(true);
    gl_attr.set_double_buffer(true);

    let mut window_size = render::WindowSize {
        width: 960,
        height: 600,
        highdpi_width: 960,
        highdpi_height: 600
    };

    let window = video_subsystem
        .window("Game", window_size.width as u32, window_size.height as u32)
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()?;

    let drawable_size = window.drawable_size();
    window_size.highdpi_width = drawable_size.0 as i32;
    window_size.highdpi_height = drawable_size.1 as i32;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // 0 for immediate updates,
    // 1 for updates synchronized with the vertical retrace,
    // -1 for late swap tearing

    let vsync = false;
    video_subsystem.gl_set_swap_interval(if vsync { 1 } else { 0 });

    let mut viewport = render_gl::Viewport::for_window(window_size.highdpi_width, window_size.highdpi_height);
    let color_buffer = render_gl::ColorBuffer::new();

    // set up shared state for window

    viewport.set_used(&gl);
    color_buffer.set_clear_color(&gl, na::Vector3::new(0.3, 0.3, 0.5));

    // main loop

    let mut time = Instant::now();

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {

        for event in event_pump.poll_iter() {
            if system::input::window::handle_default_window_events(&event, &gl, &window, &mut window_size, &mut viewport) == system::input::window::HandleResult::Quit {
                break 'main;
            }

//            match event {
//                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::C), .. } => {
//                    side_cam = !side_cam;
//                },
//                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::I), .. } => {
//                    debug_lines.toggle();
//                },
//                sdl2::event::Event::KeyDown { scancode: Some(sdl2::keyboard::Scancode::P), .. } => {
//                    frame_profiler.toggle();
//                    allocation_profiler.toggle();
//                    gl_call_profiler.toggle();
//                },
//                _ => (),
//            }
        }

        let delta = time.elapsed().as_fractional_secs() as f32;
        time = Instant::now();

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }

        color_buffer.clear(&gl);

        while time.elapsed() < Duration::from_millis(12) {
            ::std::thread::yield_now()
        }

        window.gl_swap_window();
    }

    println!("mutator: {:#?}", mutator);

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
