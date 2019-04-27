use geng::prelude::*;

mod bot;

use bot::BotController;

fn mix(a: Color<f32>, b: Color<f32>) -> Color<f32> {
    Color::rgba(
        (a.r + b.r) / 2.0,
        (a.g + b.g) / 2.0,
        (a.b + b.b) / 2.0,
        (a.a + b.a) / 2.0,
    )
}

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
    food: Vec<Entity>,
    next_food: f32,
    camera_pos: Vec2<f32>,
    quad_geometry: ugli::VertexBuffer<QuadVertex>,
    particle_instances: ugli::VertexBuffer<ParticleInstance>,
    particle_program: ugli::Program,
    mouse_pos: Rc<Cell<Vec2<f32>>>,
    next_wave_timer: f32,
    next_wave: usize,
}

struct Entity {
    owner_id: Option<usize>,
    color: Color<f32>,
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    size: f32,
}

impl Entity {
    fn draw(&self, buffer: &mut Vec<ParticleInstance>) {
        buffer.push(ParticleInstance {
            i_pos: self.pos,
            i_size: self.size,
            i_color: self.color,
        });
    }
    fn mass(&self) -> f32 {
        self.size * self.size
    }
    fn add_mass(&mut self, delta: f32) {
        let mass = self.size * self.size + delta;
        self.size = mass.max(0.0).sqrt();
    }
    fn update(&mut self, delta_time: f32) {
        self.pos += self.vel * delta_time;
        self.pos = Game::normalize(self.pos);
    }
    fn collide(a: &mut Self, b: &mut Self) {
        let penetration = (a.size + b.size) - Game::normalize(a.pos - b.pos).len();
        let penetration = penetration.min(min(a.size, b.size));
        let n = Game::normalize(b.pos - a.pos).normalize();
        if penetration > 0.0 {
            let ka = 1.0 / a.mass();
            let kb = 1.0 / b.mass();
            let sum_k = ka + kb;
            let ka = ka / sum_k;
            let kb = kb / sum_k;
            a.pos -= n * penetration * ka;
            b.pos += n * penetration * kb;
        }
    }
    fn hit(&mut self, target: &mut Self) {
        let penetration = (self.size + target.size) - Game::normalize(self.pos - target.pos).len();
        let penetration = penetration.min(min(self.size, target.size));
        if penetration > 0.0 {
            let prev_mass = self.mass();
            self.size = (self.size - penetration).max(0.0);
            let delta_mass = prev_mass - self.mass();
            target.add_mass(-delta_mass);
        }
    }
    fn consume(&mut self, target: &mut Self, k: f32) {
        let penetration = (self.size + target.size) - Game::normalize(self.pos - target.pos).len();
        let penetration = penetration.min(min(self.size, target.size));
        if penetration > 0.0 {
            let prev_mass = target.mass();
            target.size = (target.size - penetration).max(0.0);
            let delta_mass = prev_mass - target.mass();
            self.add_mass(delta_mass * k);
        }
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn act(&mut self, self_id: usize, game: &Game) -> Action;
}

struct Player {
    entity: Entity,
    team_id: usize,
    controller: RefCell<Box<dyn Controller>>,
    projectile: Option<Entity>,
    action: Cell<Action>,
}

impl Player {
    const INITIAL_SIZE: f32 = 1.0;
    const MAX_SPEED: f32 = 5.0;
    const MAX_AIMING_SPEED: f32 = 1.0;
    const ACCELERATION: f32 = 15.0;
    const PROJECTILE_SPEED: f32 = 15.0;
    const PROJECTILE_MASS_GAIN_SPEED: f32 = 0.3;
    const PROJECTILE_COST_SPEED: f32 = 0.1;

    fn new<T: Controller + 'static>(
        pos: Vec2<f32>,
        color: Color<f32>,
        controller: T,
        team_id: usize,
    ) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        Self {
            entity: Entity {
                owner_id: Some({ NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }),
                color,
                pos,
                vel: vec2(0.0, 0.0),
                size: Self::INITIAL_SIZE,
            },
            team_id,
            controller: RefCell::new(Box::new(controller)),
            projectile: None,
            action: Cell::new(default()),
        }
    }
    fn update(&mut self, delta_time: f32) -> Option<Entity> {
        let mut action = self.action.get();
        action.target_vel = action.target_vel.clamp(1.0) * Self::MAX_SPEED;
        if action.shoot.is_some() {
            action.target_vel = action.target_vel.clamp(Self::MAX_AIMING_SPEED);
        }
        let delta_vel = action.target_vel - self.vel;
        self.vel += delta_vel.clamp(Self::ACCELERATION * delta_time);
        self.entity.update(delta_time);
        if let Some(target) = action.shoot {
            if self.projectile.is_none() {
                self.projectile = Some(Entity {
                    owner_id: self.owner_id,
                    color: mix(self.color, Color::WHITE),
                    size: 0.0,
                    pos: vec2(0.0, 0.0),
                    vel: vec2(0.0, 0.0),
                })
            }
            let projectile = self.projectile.as_mut().unwrap();
            let e = &mut self.entity;

            projectile.pos = e.pos + Game::delta_pos(e.pos, target).clamp(e.size);
            projectile.vel = Game::delta_pos(e.pos, target).normalize() * Self::PROJECTILE_SPEED;
            projectile.add_mass(Self::PROJECTILE_MASS_GAIN_SPEED * delta_time);
            e.add_mass(-Self::PROJECTILE_COST_SPEED * delta_time);
            None
        } else {
            self.projectile.take()
        }
    }

    fn draw(&self, particles: &mut Vec<ParticleInstance>) {
        if let Some(e) = self.projectile.as_ref() {
            e.draw(particles);
        }
        particles.push(ParticleInstance {
            i_pos: self.pos,
            i_size: self.size,
            i_color: mix(self.color, Color::BLACK),
        });
        particles.push(ParticleInstance {
            i_pos: self.pos,
            i_size: self.size * 0.9,
            i_color: self.color,
        });
    }

    fn act(&self, game: &Game) {
        self.action.set(
            self.controller
                .borrow_mut()
                .act(self.owner_id.unwrap(), game),
        );
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
    fn act(&mut self, _: usize, _: &Game) -> Action {
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

struct EmptyController;

impl Controller for EmptyController {
    fn act(&mut self, _: usize, _: &Game) -> Action {
        default()
    }
}

impl Game {
    const CAMERA_FOV: f32 = 15.0;

    const MAX_FOOD: usize = 100;
    const FOOD_K: f32 = 3.0;
    const FOOD_SIZE: Range<f32> = 0.1..0.3;
    const FOOD_SPAWN: Range<f32> = 0.05..0.1;

    const TIME_BETWEEN_WAVES: f32 = 120.0;

    const WORLD_SIZE: f32 = 50.0;

    const PROJECTILE_DEATH_SPEED: f32 = 0.2;
    const PLAYER_DEATH_SPEED: f32 = 1.0 / 60.0;

    fn delta_pos(a: Vec2<f32>, b: Vec2<f32>) -> Vec2<f32> {
        let dv = b - a;
        Self::normalize(dv)
    }

    fn normalize(mut v: Vec2<f32>) -> Vec2<f32> {
        while v.x > Self::WORLD_SIZE {
            v.x -= 2.0 * Self::WORLD_SIZE;
        }
        while v.y > Self::WORLD_SIZE {
            v.y -= 2.0 * Self::WORLD_SIZE;
        }
        while v.x < -Self::WORLD_SIZE {
            v.x += 2.0 * Self::WORLD_SIZE;
        }
        while v.y < -Self::WORLD_SIZE {
            v.y += 2.0 * Self::WORLD_SIZE;
        }
        v
    }

    fn reset(&mut self) {
        self.players = vec![Player::new(
            vec2(0.0, 0.0),
            Color::BLUE,
            KeyboardController::new(&self.context, &self.mouse_pos),
            0,
        )];
        self.food = Vec::new();
        self.next_food = 0.0;
        self.projectiles = Vec::new();
        self.next_wave_timer = 0.0;
        self.next_wave = 1;
    }

    fn new(context: &Rc<geng::Context>) -> Self {
        let mut game = Self {
            context: context.clone(),
            players: Vec::new(),
            food: Vec::new(),
            next_food: 0.0,
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
            mouse_pos: Rc::new(Cell::new(vec2(0.0, 0.0))),
            camera_pos: vec2(0.0, 0.0),
            next_wave_timer: 0.0,
            next_wave: 1,
        };
        game.reset();
        game
    }

    fn spawn_enemy(&mut self) {
        self.players.push(Player::new(
            vec2(
                global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
            ),
            Color::RED,
            BotController,
            1,
        ));
    }
}

impl geng::App for Game {
    fn update(&mut self, delta_time: f64) {
        for player in &self.players {
            player.act(self);
        }
        let delta_time = delta_time as f32 * 1.5;
        for player in &mut self.players {
            player.size -= Self::PLAYER_DEATH_SPEED * delta_time;
            if let Some(e) = player.update(delta_time) {
                self.projectiles.push(e);
            }
            if player.size <= 0.0 {
                self.food.push(Entity {
                    owner_id: None,
                    color: Color::GREEN,
                    pos: player.pos,
                    vel: vec2(0.0, 0.0),
                    size: Player::INITIAL_SIZE / Self::FOOD_K.sqrt(),
                });
            }
        }
        self.players.retain(|e| e.size > 0.0);
        for e in &mut self.projectiles {
            e.size -= Self::PROJECTILE_DEATH_SPEED * delta_time;
            e.update(delta_time);
        }
        self.projectiles.retain(|e| e.size > 0.0);
        for i in 0..self.players.len() {
            let (head, tail) = self.players.split_at_mut(i);
            let cur = &mut tail[0];
            for prev in head {
                Entity::collide(prev, cur);
            }
        }
        for e in &mut self.projectiles {
            for player in &mut self.players {
                if e.owner_id != player.owner_id {
                    e.hit(player);
                }
            }
        }
        for i in 0..self.projectiles.len() {
            let (head, tail) = self.projectiles.split_at_mut(i);
            let cur = &mut tail[0];
            for prev in head {
                cur.hit(prev);
            }
        }
        self.next_food -= delta_time;
        while self.next_food < 0.0 {
            self.next_food += global_rng().gen_range(Self::FOOD_SPAWN.start, Self::FOOD_SPAWN.end);
            if self.food.len() < Self::MAX_FOOD {
                self.food.push(Entity {
                    owner_id: None,
                    color: Color::GREEN,
                    pos: vec2(
                        global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                        global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                    ),
                    vel: vec2(0.0, 0.0),
                    size: Self::FOOD_SIZE.start
                        + global_rng().gen_range::<f32>(0.0, 1.0).powf(4.0)
                            * (Self::FOOD_SIZE.end - Self::FOOD_SIZE.start),
                });
            }
        }
        for f in &mut self.food {
            f.update(delta_time);
        }
        for f in &mut self.food {
            for player in &mut self.players {
                player.consume(f, Self::FOOD_K);
            }
        }
        self.food.retain(|e| e.size > 0.0);

        if self
            .players
            .iter()
            .filter(|p| p.owner_id.unwrap() != 1)
            .count()
            == 0
        {
            self.next_wave_timer = self.next_wave_timer.min(10.0);
        }
        self.next_wave_timer -= delta_time;
        if self.next_wave_timer < 0.0 {
            self.next_wave_timer = Self::TIME_BETWEEN_WAVES;
            for _ in 0..self.next_wave {
                self.spawn_enemy();
            }
            self.next_wave += 1;
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        for player in &self.players {
            if player.team_id == 0 {
                self.camera_pos = player.pos;
            }
        }
        let framebuffer_size = framebuffer.get_size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        let view_matrix = Mat4::scale(vec3(framebuffer_size.y / framebuffer_size.x, 1.0, 1.0))
            * Mat4::scale_uniform(1.0 / Self::CAMERA_FOV)
            * Mat4::translate(-self.camera_pos.extend(0.0));
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
            for f in &self.food {
                f.draw(particles);
            }
            for player in &self.players {
                player.draw(particles);
            }
            for e in &self.projectiles {
                e.draw(particles);
            }
        }
        for i in -1..=1 {
            for j in -1..=1 {
                ugli::draw(
                    framebuffer,
                    &self.particle_program,
                    ugli::DrawMode::TriangleFan,
                    ugli::instanced(&self.quad_geometry, &self.particle_instances),
                    ugli::uniforms! {
                        u_view_matrix: view_matrix,
                        u_world_offset: vec2(i as f32 * Self::WORLD_SIZE, j as f32 * Self::WORLD_SIZE) * 2.0,
                    },
                    ugli::DrawParameters {
                        blend_mode: Some(default()),
                        ..default()
                    },
                );
            }
        }
        self.context.default_font().draw(
            framebuffer,
            &format!(
                "ENEMIES: {}, next wave in {} secs",
                self.players.iter().filter(|p| p.team_id != 0).count(),
                f32::floor(self.next_wave_timer / 2.0),
            ),
            vec2(0.0, 0.0),
            32.0,
            Color::WHITE,
        );
    }
    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.reset();
            }
            _ => {}
        }
    }
}

fn main() {
    let context = Rc::new(geng::Context::new(default()));
    let game = Game::new(&context);
    geng::run(context, game);
}
