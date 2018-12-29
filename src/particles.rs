use glium::*;
use glium::index::PrimitiveType;

use rand::Rng;

use crate::SCREEN_WIDTH;
use crate::SCREEN_HEIGHT;
use crate::SCREEN_RATIO;

const MAX_PARTICLES: usize = 100000;

#[derive(Default, Copy, Clone)]
pub struct ParticleData {
    center: (f32, f32, f32),
    col: (f32, f32, f32),
    size: f32,
    life: u32,
    vel: (f32, f32, f32),
}

// This is the struct we pass into VRAM, it is a copy of ParticleData but leaves out life and
// velocity as the GPU doesn't need to know about those.
#[derive(Default, Copy, Clone)]
pub struct ParticleDataGL {
    center: (f32, f32, f32),
    col: (f32, f32, f32),
    size: f32,
}

implement_vertex!(ParticleDataGL, center, col, size);

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);

pub struct ParticleEngine<'a> {
    particles: Vec<ParticleData>,
    particle_vb: VertexBuffer<ParticleDataGL>,
    mesh_vertices: VertexBuffer<Vertex>,
    mesh_indices: index::IndexBuffer<u32>,
    params: glium::DrawParameters<'a>,
    shader: glium::program::Program,
    gravity: (f32, f32, f32),
    next_particle: usize,
    rng: rand::rngs::ThreadRng,
    total_particles: usize,
    first_live: Option<usize>,
}

impl<'a> ParticleEngine<'a> {
    pub fn new(display: &glium::Display) -> Self {
        let shader =
            Program::from_source(display,
                                 &include_str!("shader.vert"),
                                 &include_str!("shader.frag"),
                                 None).expect("FAILED TO LOAD SHADERS");

        let vertices: Vec<Vertex> = vec![
            Vertex{ position: (0.5, 0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (0.5, -0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (-0.5, -0.5 * SCREEN_RATIO, 0.0) },
            Vertex{ position: (-0.5, 0.5 * SCREEN_RATIO, 0.0) },
        ];
        let indices: Vec<u32> = vec![
            0, 1, 3,
            1, 2, 3
        ];

        let mesh_vertices = VertexBuffer::new(display, &vertices).expect("FAILED TO CREATE MESH VERTEX BUFFER");
        let mesh_indices = index::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices).expect("FAILED TO CREATE MESH INDEX BUFFER");
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess, write: true, .. Default::default()
            },
            backface_culling: draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            blend: glium::draw_parameters::Blend::alpha_blending(),
            multisampling: false,
            .. Default::default()
        };
        let particles = vec![ParticleData::default(); MAX_PARTICLES];

        ParticleEngine {
            particles,
            particle_vb: glium::VertexBuffer::empty_dynamic(display, MAX_PARTICLES).expect("FAILED TO CREATE PARTICLE VERTEX BUFFER"),
            mesh_vertices,
            mesh_indices,
            params,
            shader,
            gravity: (0.0, -0.001 * SCREEN_RATIO, 0.0),
            next_particle: 0,
            rng: rand::thread_rng(),
            total_particles: 0,
            first_live: None,
        }
    }

    // Generates a new particle with a random velocity in a range and a red color
    pub fn create_particle(&mut self, center: (f32, f32, f32)) {
        let center: (f32, f32, f32) = center;
        let vel: (f32, f32, f32) = (self.rng.gen_range(-0.01, 0.01), self.rng.gen_range(0.01, 0.03) * SCREEN_RATIO, 0.0);
        self.particles[self.next_particle].center = center;
        self.particles[self.next_particle].vel = vel;
        self.particles[self.next_particle].col = (0.6, 0.8, 0.0);
        self.particles[self.next_particle].life = 100;
        self.particles[self.next_particle].size = self.rng.gen_range(0.01, 0.02);

        if self.first_live == None {
            self.first_live = Some(self.next_particle);
        }
        self.next_particle += 1;
        if self.next_particle == MAX_PARTICLES {
            self.next_particle = 0;
        }
        self.total_particles += 1;
    }

    // Updates the particles and sends the new data to the gpu
    pub fn update(&mut self) {
        // If we don't have any live particles, dont bother trying to update them
        if self.total_particles == 0 {
            return;
        }
        let starting_point = if self.first_live != None {
            self.first_live.unwrap()
        } else {
            0
        };
        self.first_live = None;
        let mut particles_updated = 0;
        // There should be a better way to chain these iterators somehow
        //
        // Start updating at the last particle we knew was alive, this is so we traverse the
        // particle array as little as possible
        for (i, (p, vb)) in self.particles.iter_mut().zip(self.particle_vb.map().iter_mut()).enumerate().skip(starting_point) {
            if p.life > 0 {
                p.life -= 1;
                if p.life > 0 {
                    if self.first_live == None {
                        self.first_live = Some(i);
                    }
                    if p.size > 0.0 {
                        p.size -= 0.0001;
                    }
                    p.vel = (p.vel.0 + self.gravity.0, p.vel.1 + self.gravity.1, p.vel.2 + self.gravity.2);
                    p.center = (p.center.0 + p.vel.0, p.center.1 + p.vel.1, p.center.2 + p.vel.2);
                    vb.col = p.col;
                    vb.center = p.center;
                    vb.size = p.size;
                    particles_updated += 1;
                    // If we have updated as many particles as there are particles alive we are
                    // done.
                    if particles_updated == self.total_particles {
                        return;
                    }
                } else {
                    p.size = 0.0;
                    vb.size = 0.0;
                    self.total_particles -= 1;
                    // If we have updated as many particles as there are particles alive we are
                    // done.
                    if particles_updated == self.total_particles {
                        return;
                    }
                }
            }
        }
        // if we get to the end of the array from the starting poing, update [0, starting_point)
        // particles
        for (i, (p, vb)) in self.particles.iter_mut().zip(self.particle_vb.map().iter_mut()).enumerate().take(starting_point) {
            if p.life > 0 {
                p.life -= 1;
                if p.life > 0 {
                    if self.first_live == None {
                        self.first_live = Some(i);
                    }
                    if p.size > 0.0 {
                        p.size -= 0.0001;
                    }
                    p.vel = (p.vel.0 + self.gravity.0, p.vel.1 + self.gravity.1, p.vel.2 + self.gravity.2);
                    p.center = (p.center.0 + p.vel.0, p.center.1 + p.vel.1, p.center.2 + p.vel.2);
                    vb.col = p.col;
                    vb.center = p.center;
                    vb.size = p.size;
                    particles_updated += 1;
                    // If we have updated as many particles as there are particles alive we are
                    // done.
                    if particles_updated == self.total_particles {
                        return;
                    }
                } else {
                    p.size = 0.0;
                    vb.size = 0.0;
                    self.total_particles -= 1;
                    // If we have updated as many particles as there are particles alive we are
                    // done.
                    if particles_updated == self.total_particles {
                        return;
                    }
                }
            }
        }
    }

    // Renders the particles using instancing with one mesh for better performance
    pub fn render(&mut self, frame: &mut glium::Frame) {
        frame.draw((&self.mesh_vertices, self.particle_vb.per_instance().unwrap()), &self.mesh_indices, &self.shader, &uniform! {}, &self.params).unwrap();
    }
}
