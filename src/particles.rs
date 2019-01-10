use gl;
use gl::types::*;

use rand::Rng;

use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::time;
use std::error::Error;

use crate::SCREEN_WIDTH;
use crate::SCREEN_HEIGHT;
use crate::SCREEN_RATIO;
use crate::shader::*;

const MAX_PARTICLES: usize = 100000;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    position: (f32, f32, f32),
}

#[derive(Copy, Clone, Default)]
pub struct Particle {
    pos: (f32, f32, f32, f32),
    color: (f32, f32, f32, f32),
    vel: (f32, f32, f32, f32),
    size: f32,
    life: u32,
    pad: [f32; 2],
}

pub struct ParticleEngine {
    particles_vao: GLuint,
    particles_vbo: GLuint,
    mesh_vbo: GLuint,
    shader: Program,
    compute_shader: ComputeProgram,
    gravity: (f32, f32, f32),
    next_particle: usize,
    rng: rand::rngs::ThreadRng,
    total_particles: usize,
}

impl ParticleEngine {
    pub fn new() -> Self {
        let mut shader = Program::new(&include_str!("shader.vert"), &include_str!("shader.frag"));
        shader.enable();

        let mut compute_shader = ComputeProgram::new(&include_str!("shader.compute"));

        let vertices: [Vertex; 6] = [
            Vertex{ position: (0.5, 0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (0.5, -0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (-0.5, -0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (-0.5, -0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (-0.5, 0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (0.5, 0.5 * SCREEN_RATIO, 0.0) },
        ];
        let particles_data: Vec<Particle> = vec![Particle::default(); MAX_PARTICLES];

        let mut particles_vao = 0;
        let mut mesh_vbo = 0;
        let mut particles_vbo = 0;

        unsafe {
            // Enable backface culling
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CW);
            gl::GenVertexArrays(1, &mut particles_vao);
            gl::GenBuffers(1, &mut mesh_vbo);
            gl::GenBuffers(1, &mut particles_vbo);

            gl::BindVertexArray(particles_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, mesh_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
                mem::transmute(&vertices[0]),
                gl::STATIC_DRAW,
                );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0 as i32, 0 as *const GLvoid);
            gl::EnableVertexAttribArray(0);

            gl::BindBuffer(gl::ARRAY_BUFFER, particles_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (particles_data.len() * mem::size_of::<Particle>()) as GLsizeiptr,
                mem::transmute(&particles_data[0]),
                gl::STREAM_DRAW,
                );

            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 16 * mem::size_of::<f32>() as i32, (0 * mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 16 * mem::size_of::<f32>() as i32, (4 * mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, 16 * mem::size_of::<f32>() as i32, (8 * mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(4, 1, gl::FLOAT, gl::FALSE, 16 * mem::size_of::<f32>() as i32, (12 * mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(5, 1, gl::UNSIGNED_INT, gl::FALSE, 16 * mem::size_of::<f32>() as i32, (13 * mem::size_of::<f32>()) as *const GLvoid);
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribDivisor(0, 0);
            gl::VertexAttribDivisor(1, 1);
            gl::VertexAttribDivisor(2, 1);
            gl::VertexAttribDivisor(3, 1);
            gl::VertexAttribDivisor(4, 1);
            gl::VertexAttribDivisor(5, 1);
            gl::BindVertexArray(0);
        }

        ParticleEngine {
            particles_vao,
            particles_vbo,
            mesh_vbo,
            shader,
            compute_shader,
            gravity: (0.0, -0.001 * SCREEN_RATIO, 0.0),
            next_particle: 0,
            rng: rand::thread_rng(),
            total_particles: 0,
        }
    }

    // Generates a new particle with a random velocity in a range and a red color
    pub fn create_particle(&mut self, mut center: (f32, f32, f32, f32)) {
        let vel: (f32, f32, f32, f32) = (self.rng.gen_range(-0.01, 0.01), self.rng.gen_range(0.01, 0.03) * SCREEN_RATIO, 0.0, 0.0);
        let tmp_particle = Particle {
            pos: center,
            color: (0.5, 1.0, 0.0, 1.0),
            vel,
            size: 0.004,
            life: 120,
            pad: [0.0; 2],
        };
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.particles_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (self.next_particle * mem::size_of::<Particle>()) as isize,
                mem::size_of::<Particle>() as GLsizeiptr,
                mem::transmute(&tmp_particle),
                );
        }
        self.next_particle += 1;
        if self.next_particle == MAX_PARTICLES {
            self.next_particle = 0;
        }
    }

    // Updates the particles using the compute shader
    pub fn update(&mut self) {
        self.compute_shader.enable();
        unsafe {
            //gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
            gl::MemoryBarrier(gl::BUFFER_UPDATE_BARRIER_BIT);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.particles_vbo);
            gl::DispatchCompute(MAX_PARTICLES as u32/256 + 1, 1, 1);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0);
        }
    }

    // Renders the particles using instancing with one mesh for better performance
    pub fn render(&mut self) {
        self.shader.enable();
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT | gl::VERTEX_ATTRIB_ARRAY_BARRIER_BIT);

            gl::BindVertexArray(self.particles_vao);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, MAX_PARTICLES as i32);
            gl::BindVertexArray(0);
        }
    }
}
