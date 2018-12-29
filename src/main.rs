use glium::*;
use glium::glutin::{Api, ContextBuilder, Event, EventsLoop, GlRequest, WindowBuilder, WindowEvent};
use std::time;

use std::error::Error;

mod particles;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

const SCREEN_RATIO: f32 = SCREEN_WIDTH as f32/SCREEN_HEIGHT as f32;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
    tex_coords: (f32, f32),
}

implement_vertex!(Vertex, position, color, tex_coords);

fn main() -> Result<(), Box<Error>> {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("Particle Engine")
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT);
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &events_loop)?;

    let mut window_open = true;
    let mut update = true;

    let mut start_time;

    let mut particles = particles::ParticleEngine::new(&display);
    let mut particle_center: (f32, f32, f32) = (0.0, 0.0, 0.0);
    let mut w_pressed = false;
    let mut s_pressed = false;
    let mut a_pressed = false;
    let mut d_pressed = false;
    while window_open {

        let mut frame = display.draw();

        // Clear the screen with blue and reset the depth buffer
        frame.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
        start_time = time::Instant::now();
        if update {
            // Create 10 particles per frame
            for _ in 0..10 {
                particles.create_particle(particle_center);
            }
            particles.update();
        }
        particles.render(&mut frame);
        let frame_time = time::Instant::now() - start_time;
        println!("{}s {}ms {}ns", frame_time.as_secs(), frame_time.subsec_millis(), frame_time.subsec_nanos());

        frame.finish().expect("FRAME FINISH FAILED");

        // This is hacky and in a real game input handling would be more elegant
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                    window_open = false;
                }
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
                }
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
                }

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
    }
    Ok(())
}
