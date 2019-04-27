use geng::prelude::*;

#[derive(ugli::Vertex)]
struct QuadVertex {
    a_pos: Vec2<f32>,
}

#[derive(ugli::Vertex, Debug)]
struct ParticleInstance {
    i_pos: Vec2<f32>,
    i_size: f32,
    i_color: Color<f32>,
}

struct Game {
    context: Rc<geng::Context>,
    players: Vec<Player>,
    quad_geometry: ugli::VertexBuffer<QuadVertex>,
    particle_instances: ugli::VertexBuffer<ParticleInstance>,
    particle_program: ugli::Program,
}

struct Entity {
    color: Color<f32>,
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    size: f32,
}

impl Entity {
    fn draw(&self) -> ParticleInstance {
        ParticleInstance {
            i_pos: self.pos,
            i_size: self.size,
            i_color: self.color,
        }
    }
}

struct Action {
    target_vel: Vec2<f32>,
    shoot: Option<Vec2<f32>>,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            target_vel: vec2(0.0, 0.0),
            shoot: None,
        }
    }
}

trait Controller {
    fn act(&mut self) -> Action;
}

struct Player {
    entity: Entity,
    controller: Box<dyn Controller>,
}

impl Player {
    const INITIAL_SIZE: f32 = 1.0;
    const MAX_SPEED: f32 = 5.0;
    const ACCELERATION: f32 = 10.0;

    fn new<T: Controller + 'static>(pos: Vec2<f32>, color: Color<f32>, controller: T) -> Self {
        Self {
            entity: Entity {
                color,
                pos,
                vel: vec2(0.0, 0.0),
                size: Self::INITIAL_SIZE,
            },
            controller: Box::new(controller),
        }
    }
    fn update(&mut self, delta_time: f32) {
        let mut action = self.controller.act();
        action.target_vel = action.target_vel.clamp(1.0) * Self::MAX_SPEED;
        let delta_vel = action.target_vel - self.vel;
        self.vel += delta_vel.clamp(Self::ACCELERATION * delta_time);
        self.pos += self.vel * delta_time;
    }
}

impl Deref for Player {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

struct KeyboardController {
    context: Rc<geng::Context>,
}

impl KeyboardController {
    fn new(context: &Rc<geng::Context>) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

impl Controller for KeyboardController {
    fn act(&mut self) -> Action {
        let mut target_vel = vec2(0.0, 0.0);
        if self.context.window().is_key_pressed(geng::Key::W) {
            target_vel.y += 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::A) {
            target_vel.x -= 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::S) {
            target_vel.y -= 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::D) {
            target_vel.x += 1.0;
        }
        Action {
            target_vel,
            shoot: None,
        }
    }
}

impl Game {
    fn new(context: &Rc<geng::Context>) -> Self {
        let keyboard_controller = KeyboardController::new(context);
        Self {
            context: context.clone(),
            players: vec![Player::new(
                vec2(0.0, 0.0),
                Color::WHITE,
                keyboard_controller,
            )],
            quad_geometry: ugli::VertexBuffer::new_static(
                context.ugli_context(),
                vec![
                    QuadVertex {
                        a_pos: vec2(-1.0, -1.0),
                    },
                    QuadVertex {
                        a_pos: vec2(1.0, -1.0),
                    },
                    QuadVertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    QuadVertex {
                        a_pos: vec2(-1.0, 1.0),
                    },
                ],
            ),
            particle_instances: ugli::VertexBuffer::new_dynamic(context.ugli_context(), Vec::new()),
            particle_program: context
                .shader_lib()
                .compile(include_str!("particle.glsl"))
                .unwrap(),
        }
    }
}

impl geng::App for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        for player in &mut self.players {
            player.update(delta_time);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.get_size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let view_matrix = Mat4::scale(vec3(framebuffer_size.y / framebuffer_size.x, 1.0, 1.0))
            * Mat4::scale_uniform(1.0 / 10.0);
        {
            let particles: &mut Vec<_> = &mut self.particle_instances;
            particles.clear();
            for player in &self.players {
                particles.push(player.draw());
            }
        }
        ugli::draw(
            framebuffer,
            &self.particle_program,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(&self.quad_geometry, &self.particle_instances),
            ugli::uniforms! {
                u_view_matrix: view_matrix,
            },
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}

fn main() {
    let context = Rc::new(geng::Context::new(default()));
    let game = Game::new(&context);
    geng::run(context, game);
}
