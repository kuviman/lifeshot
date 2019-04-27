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
    projectiles: Vec<Entity>,
    quad_geometry: ugli::VertexBuffer<QuadVertex>,
    particle_instances: ugli::VertexBuffer<ParticleInstance>,
    particle_program: ugli::Program,
    mouse_pos: Rc<Cell<Vec2<f32>>>,
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
    fn add_mass(&mut self, delta: f32) {
        let mass = self.size * self.size + delta;
        self.size = mass.max(0.0).sqrt();
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
    projectile: Option<Entity>,
    controller: Box<dyn Controller>,
}

impl Player {
    const INITIAL_SIZE: f32 = 1.0;
    const MAX_SPEED: f32 = 5.0;
    const ACCELERATION: f32 = 10.0;
    const PROJECTILE_SPEED: f32 = 15.0;

    fn new<T: Controller + 'static>(pos: Vec2<f32>, color: Color<f32>, controller: T) -> Self {
        Self {
            entity: Entity {
                color,
                pos,
                vel: vec2(0.0, 0.0),
                size: Self::INITIAL_SIZE,
            },
            projectile: None,
            controller: Box::new(controller),
        }
    }
    fn update(&mut self, delta_time: f32) -> Option<Entity> {
        let mut action = self.controller.act();
        action.target_vel = action.target_vel.clamp(1.0) * Self::MAX_SPEED;
        let delta_vel = action.target_vel - self.vel;
        self.vel += delta_vel.clamp(Self::ACCELERATION * delta_time);
        let delta_pos = self.vel * delta_time;
        self.pos += delta_pos;
        if let Some(target) = action.shoot {
            if self.projectile.is_none() {
                self.projectile = Some(Entity {
                    color: self.color,
                    size: 0.0,
                    pos: vec2(0.0, 0.0),
                    vel: vec2(0.0, 0.0),
                })
            }
            let projectile = self.projectile.as_mut().unwrap();
            let e = &mut self.entity;

            projectile.pos = e.pos + (target - e.pos).clamp(e.size);
            projectile.vel = (target - e.pos).normalize() * Self::PROJECTILE_SPEED;
            let delta_mass = delta_time / 3.0;
            projectile.add_mass(delta_mass);
            e.add_mass(-delta_mass);
            None
        } else {
            self.projectile.take()
        }
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
    mouse_pos: Rc<Cell<Vec2<f32>>>,
}

impl KeyboardController {
    fn new(context: &Rc<geng::Context>, mouse_pos: &Rc<Cell<Vec2<f32>>>) -> Self {
        Self {
            context: context.clone(),
            mouse_pos: mouse_pos.clone(),
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
            shoot: if self
                .context
                .window()
                .is_button_pressed(geng::MouseButton::Left)
            {
                Some(self.mouse_pos.get())
            } else {
                None
            },
        }
    }
}

impl Game {
    fn new(context: &Rc<geng::Context>) -> Self {
        let mouse_pos = Rc::new(Cell::new(vec2(0.0, 0.0)));
        let keyboard_controller = KeyboardController::new(context, &mouse_pos);
        Self {
            context: context.clone(),
            players: vec![Player::new(
                vec2(0.0, 0.0),
                Color::WHITE,
                keyboard_controller,
            )],
            projectiles: Vec::new(),
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
            mouse_pos,
        }
    }
}

impl geng::App for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        for player in &mut self.players {
            if let Some(e) = player.update(delta_time) {
                self.projectiles.push(e);
            }
        }
        self.players.retain(|e| e.size > 0.0);
        for e in &mut self.projectiles {
            e.pos += e.vel * delta_time;
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.get_size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let view_matrix = Mat4::scale(vec3(framebuffer_size.y / framebuffer_size.x, 1.0, 1.0))
            * Mat4::scale_uniform(1.0 / 10.0);
        self.mouse_pos.set({
            let mouse_pos = self.context.window().mouse_pos().map(|x| x as f32);
            let mouse_pos = vec2(
                mouse_pos.x / framebuffer_size.x * 2.0 - 1.0,
                mouse_pos.y / framebuffer_size.y * 2.0 - 1.0,
            );
            let mouse_pos = view_matrix.inverse() * vec4(mouse_pos.x, mouse_pos.y, 0.0, 1.0);
            let mouse_pos = vec2(mouse_pos.x, mouse_pos.y);
            mouse_pos
        });
        {
            let particles: &mut Vec<_> = &mut self.particle_instances;
            particles.clear();
            for player in &self.players {
                particles.push(player.draw());
            }
            for e in &self.projectiles {
                particles.push(e.draw());
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
