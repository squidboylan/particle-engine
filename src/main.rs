use glutin::dpi::*;
use glutin::{Api, ContextBuilder, Event, EventsLoop, GlContext, GlRequest, WindowBuilder, WindowEvent, ControlFlow};

use gl::types::*;

use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::time;

use std::error::Error;

mod particles;
mod shader;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

const SCREEN_RATIO: f32 = SCREEN_WIDTH as f32/SCREEN_HEIGHT as f32;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
}


fn main() -> Result<(), Box<Error>> {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("Particle Engine")
        .with_dimensions(LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64));
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 3)))
        .with_multisampling(0);

    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current() }.unwrap();

    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let mut window_open = true;
    let mut update = true;

    let mut particles = particles::ParticleEngine::new();
    let mut particle_center: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);
    let mut w_pressed = false;
    let mut s_pressed = false;
    let mut a_pressed = false;
    let mut d_pressed = false;


    while window_open {
        for _ in 0..50 {
            particles.create_particle(particle_center);
        }

        // This is hacky and in a real game input handling would be more elegant
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    window_open = false;
                },
                Event::WindowEvent { event: WindowEvent::Resized(logical_size), .. } => {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    let physical_size = logical_size.to_physical(dpi_factor);
                    unsafe {
                        gl::Viewport(0, 0, physical_size.width as GLsizei, physical_size.height as GLsizei);
                    }
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                    ()
                },
                Event::WindowEvent { event: WindowEvent::KeyboardInput {input: glutin::KeyboardInput { state: glutin::ElementState::Pressed, virtual_keycode: Some(v), .. }, .. }, .. } => {
                    if v == glutin::VirtualKeyCode::Space {
                        update = !update;
                    } else if v == glutin::VirtualKeyCode::W {
                        w_pressed = true;
                    } else if v == glutin::VirtualKeyCode::S {
                        s_pressed = true;
                    } else if v == glutin::VirtualKeyCode::D {
                        d_pressed = true;
                    } else if v == glutin::VirtualKeyCode::A {
                        a_pressed = true;
                    }
                },
                Event::WindowEvent { event: WindowEvent::KeyboardInput {input: glutin::KeyboardInput { state: glutin::ElementState::Released, virtual_keycode: Some(v), .. }, .. }, .. } => {
                    if v == glutin::VirtualKeyCode::W {
                        w_pressed = false;
                    } else if v == glutin::VirtualKeyCode::S {
                        s_pressed = false;
                    } else if v == glutin::VirtualKeyCode::D {
                        d_pressed = false;
                    } else if v == glutin::VirtualKeyCode::A {
                        a_pressed = false;
                    }
                },

                _ => (),
            }
        });

        if w_pressed {
            particle_center.1 += 0.05 * SCREEN_RATIO;
        }

        if s_pressed {
            particle_center.1 -= 0.05 * SCREEN_RATIO;
        }

        if a_pressed {
            particle_center.0 -= 0.05;
        }

        if d_pressed {
            particle_center.0 += 0.05;
        }

        let start = time::Instant::now();
        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        particles.update();
        particles.render();
        let diff = time::Instant::now() - start;
        println!("particle time: {}ms {}ns", diff.subsec_millis(), diff.subsec_nanos());
        gl_window.swap_buffers().unwrap();

    }
    Ok(())
}
