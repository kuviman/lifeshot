#![windows_subsystem = "windows"]

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

use geng::prelude::*;

mod entity;
mod food;
mod player;
mod projectile;

use entity::*;
use food::*;
use player::*;
use projectile::*;

static mut CAMERA_POS: Vec2<f32> = Vec2 { x: 0.0, y: 0.0 };

struct Sound {
    #[cfg(target_arch = "wasm32")]
    inner: stdweb::Reference,
}

impl Sound {
    fn set_pos(&self, pos: Vec2<f32>) {
        let volume = clamp(
            1.0 - (Game::delta_pos(pos, unsafe { CAMERA_POS }).len() / Game::CAMERA_FOV / 2.0)
                .powf(2.0),
            0.0..=1.0,
        );
        #[cfg(target_arch = "wasm32")]
        js! {
            @(no_return)
            @{&self.inner}.volume = @{volume} * 0.2;
        }
    }
    fn stop(&self) {
        #[cfg(target_arch = "wasm32")]
        js! {
            @(no_return)
            @{&self.inner}.pause();
        }
    }
}

fn play_sound(name: &str, pos: Vec2<f32>) -> Sound {
    #[cfg(target_arch = "wasm32")]
    {
        use stdweb::unstable::TryInto;
        let inner = js! {
            var audio = new Audio(@{name});
            audio.play();
            return audio;
        }
        .try_into()
        .unwrap();
        let result = Sound { inner };
        result.set_pos(pos);
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    Sound {}
}

fn mix(a: Color<f32>, b: Color<f32>) -> Color<f32> {
    Color::rgba(
        (a.r + b.r) / 2.0,
        (a.g + b.g) / 2.0,
        (a.b + b.b) / 2.0,
        (a.a + b.a) / 2.0,
    )
}

fn random_circle_point() -> Vec2<f32> {
    let mut rng = global_rng();
    loop {
        let result = vec2(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));
        if result.len() < 1.0 {
            return result;
        }
    }
}

#[derive(ugli::Vertex)]
struct QuadVertex {
    a_pos: Vec2<f32>,
}

#[derive(ugli::Vertex, Debug)]
pub struct ParticleInstance {
    i_pos: Vec2<f32>,
    i_size: f32,
    i_color: Color<f32>,
}

pub struct Game {
    context: Rc<Geng>,
    font: geng::Font,
    players: Vec<Player>,
    projectiles: Vec<Projectile>,
    food: Vec<Food>,
    next_food: f32,
    camera_pos: Vec2<f32>,
    quad_geometry: ugli::VertexBuffer<QuadVertex>,
    particle_instances: ugli::VertexBuffer<ParticleInstance>,
    particle_program: ugli::Program,
    background_particles: Vec<Entity>,
    mouse_pos: Rc<Cell<Vec2<f32>>>,
    next_wave_timer: f32,
    next_wave: usize,
    time_played: f32,
    kills: usize,
    waves_finished: usize,
    start: f32,
}

impl Game {
    const CAMERA_FOV: f32 = 15.0;
    const START: f32 = 5.0;

    const MAX_FOOD: usize = 100;
    const FOOD_K: f32 = 3.0;
    const FOOD_SIZE: Range<f32> = 0.1..0.5;
    const FOOD_SPAWN: Range<f32> = 0.05..0.1;

    const TIME_BETWEEN_WAVES: f32 = 120.0;

    const WORLD_SIZE: f32 = 50.0;

    const WAVE_PAUSE: f32 = 10.0;

    const PROJECTILE_DEATH_SPEED: f32 = 0.1;
    const PROJECTILE_STRENGTH: f32 = 0.5;
    const PLAYER_DEATH_SPEED: f32 = 1.0 / 20.0;

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
            Color::rgb(0.5, 0.5, 1.0),
            KeyboardController::new(&self.context, &self.mouse_pos),
            0,
        )];
        self.food = Vec::new();
        self.next_food = 0.0;
        self.projectiles = Vec::new();
        self.next_wave_timer = Self::WAVE_PAUSE;
        self.next_wave = 1;
        self.start = 0.0;
        self.kills = 0;
        self.waves_finished = 0;
        self.time_played = 0.0;
    }

    fn new(context: &Rc<Geng>) -> Self {
        let mut game = Self {
            context: context.clone(),
            players: Vec::new(),
            food: Vec::new(),
            next_food: 0.0,
            projectiles: Vec::new(),
            quad_geometry: ugli::VertexBuffer::new_static(
                context.ugli(),
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
            background_particles: {
                let mut ps = Vec::new();
                for _ in 0..10 {
                    ps.push(Entity {
                        owner_id: None,
                        color: Color::rgba(
                            global_rng().gen_range(0.0, 1.0),
                            global_rng().gen_range(0.0, 1.0),
                            global_rng().gen_range(0.0, 1.0),
                            0.02,
                        ),
                        pos: vec2(
                            global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                            global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                        ),
                        vel: vec2(
                            global_rng().gen_range(-1.0, 1.0),
                            global_rng().gen_range(-1.0, 1.0),
                        ),
                        size: global_rng().gen_range(Self::CAMERA_FOV, Self::CAMERA_FOV * 2.0),
                    })
                }
                ps
            },
            particle_instances: ugli::VertexBuffer::new_dynamic(context.ugli(), Vec::new()),
            particle_program: context
                .shader_lib()
                .compile(include_str!("particle.glsl"))
                .unwrap(),
            mouse_pos: Rc::new(Cell::new(vec2(0.0, 0.0))),
            camera_pos: vec2(0.0, 0.0),
            next_wave_timer: 0.0,
            next_wave: 1,
            start: 0.0,
            font: geng::Font::new(
                context,
                include_bytes!("../font/Simply Rounded Bold.ttf").to_vec(),
            )
            .unwrap(),
            waves_finished: 0,
            kills: 0,
            time_played: 0.0,
        };
        game.reset();
        game
    }

    fn spawn_enemy(&mut self) {
        loop {
            let enemy = Player::new(
                vec2(
                    global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                    global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                ),
                Color::RED,
                BotController,
                1,
            );
            if Self::delta_pos(self.camera_pos, enemy.pos).len() > Self::CAMERA_FOV * 2.0 {
                self.players.push(enemy);
                break;
            }
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let mut player_alive = false;
        for player in &self.players {
            player.act(self);
            if player.team_id == 0 {
                player_alive = true;
            }
        }
        let delta_time = delta_time as f32;

        for p in &mut self.background_particles {
            p.update(delta_time);
        }

        for player in &mut self.players {
            if self.start > Self::START {
                player.size -= Self::PLAYER_DEATH_SPEED * delta_time;
            }
            if let Some(e) = player.update(delta_time) {
                self.projectiles.push(e);
            }
            if player.size <= 0.0 {
                self.food.push(Food::new(
                    player.pos,
                    Player::INITIAL_SIZE / Self::FOOD_K.sqrt(),
                ));
                play_sound("death.wav", player.pos);
                if player.team_id != 0 && player_alive {
                    self.kills += 1;
                }
            }
        }
        self.players.retain(|e| e.size > 0.0);
        for e in &mut self.projectiles {
            e.size -= Self::PROJECTILE_DEATH_SPEED * delta_time;
            e.update(delta_time);
        }
        self.projectiles.retain(|e| e.alive());
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
                    if e.hit(player, Self::PROJECTILE_STRENGTH) {
                        e.actually_hit = true;
                    }
                }
            }
        }
        for i in 0..self.projectiles.len() {
            let (head, tail) = self.projectiles.split_at_mut(i);
            let cur = &mut tail[0];
            for prev in head {
                cur.hit(prev, 1.0);
            }
        }
        self.next_food -= delta_time;
        while self.next_food < 0.0 {
            self.next_food += global_rng().gen_range(Self::FOOD_SPAWN.start, Self::FOOD_SPAWN.end);
            if self.food.len() < Self::MAX_FOOD {
                self.food.push(Food::new(
                    vec2(
                        global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                        global_rng().gen_range(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                    ),
                    Self::FOOD_SIZE.start
                        + global_rng().gen_range(0.0f32, 1.0f32).powf(4.0)
                            * (Self::FOOD_SIZE.end - Self::FOOD_SIZE.start),
                ));
            }
        }
        for f in &mut self.food {
            f.update(delta_time);
        }
        for f in &mut self.food {
            for player in &mut self.players {
                player.consume(f, Self::FOOD_K);
            }
            if f.size <= 0.0 {
                play_sound("heal.wav", f.pos);
            }
        }
        self.food.retain(|e| e.size > 0.0);

        if self.players.iter().filter(|p| p.team_id != 0).count() == 0 {
            self.next_wave_timer = self.next_wave_timer.min(Self::WAVE_PAUSE);
        }
        if self.start > Self::START {
            self.next_wave_timer -= delta_time;
            if self.next_wave_timer < 0.0 {
                self.next_wave_timer = Self::TIME_BETWEEN_WAVES;
                for _ in 0..self.next_wave {
                    self.spawn_enemy();
                }
                if player_alive {
                    self.waves_finished = self.next_wave.max(1) - 1;
                }
                self.next_wave += 1;
            }
            if player_alive {
                self.time_played += delta_time;
            }
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let mut player_alive = false;
        for player in &self.players {
            if player.team_id == 0 {
                player_alive = true;
                self.camera_pos = player.pos;
                unsafe {
                    CAMERA_POS = self.camera_pos;
                }
                self.start = self.start.max(player.pos.len());
            }
        }
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
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

            for p in &self.background_particles {
                p.draw(particles);
            }

            if player_alive {
                let dv =
                    (self.mouse_pos.get() - self.camera_pos).normalize() * Self::CAMERA_FOV * 2.0;
                const N: usize = 40;
                for i in 1..=N {
                    particles.push(ParticleInstance {
                        i_pos: self.camera_pos + dv * i as f32 / N as f32,
                        i_color: Color::rgba(0.5, 0.5, 1.0, 0.4),
                        i_size: 0.1,
                    });
                }
            }

            for f in &self.food {
                f.draw(particles);
            }
            for player in &self.players {
                player.draw(particles);
            }
            for e in &self.projectiles {
                e.draw(particles);
            }
            for player in &self.players {
                if player.team_id != 0 {
                    let dv = Self::delta_pos(self.camera_pos, player.pos);
                    let max_y = Self::CAMERA_FOV;
                    let max_x = max_y * framebuffer_size.x / framebuffer_size.y;
                    if dv.x.abs() > max_x || dv.y.abs() > max_y {
                        let mut color = player.color;
                        color.a = 0.5;
                        particles.push(ParticleInstance {
                            i_pos: self.camera_pos
                                + vec2(clamp_abs(dv.x, max_x), clamp_abs(dv.y, max_y)),
                            i_color: color,
                            i_size: player.size,
                        });
                    }
                }
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

        if !player_alive {
            let font = &self.font;
            let scale = framebuffer_size.y / 20.0;
            let mid = framebuffer_size / 2.0;
            font.draw_aligned(
                framebuffer,
                "YOU DED",
                vec2(0.0, 4.0 * scale) + mid,
                0.5,
                scale * 2.0,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            );
            font.draw_aligned(
                framebuffer,
                &format!("time played: {} secs", self.time_played as i64),
                vec2(0.0, 3.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            );
            font.draw_aligned(
                framebuffer,
                &format!("waves finished: {}", self.waves_finished),
                vec2(0.0, 2.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            );
            font.draw_aligned(
                framebuffer,
                &format!("enemies killed: {}", self.kills),
                vec2(0.0, 1.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            );
            font.draw_aligned(
                framebuffer,
                "press R to restart",
                vec2(0.0, -4.0 * scale) + mid,
                0.5,
                scale * 2.0,
                Color::rgba(0.5, 0.5, 0.5, 1.0),
            );
        } else if self.start < Self::START {
            let alpha = 1.0 - self.start / Self::START;
            let font = &self.font;
            let scale = framebuffer_size.y / 20.0;
            let mid = framebuffer_size / 2.0;
            font.draw_aligned(
                framebuffer,
                "WASD to move",
                vec2(0.0, 5.0 * scale) + mid,
                0.5,
                scale * 2.0,
                Color::rgba(0.5, 0.5, 0.5, alpha),
            );
            font.draw_aligned(
                framebuffer,
                "LMB to shoot",
                vec2(0.0, 3.0 * scale) + mid,
                0.5,
                scale * 2.0,
                Color::rgba(0.5, 0.5, 0.5, alpha),
            );
            font.draw_aligned(
                framebuffer,
                "F to toggle fullscreen",
                vec2(0.0, 2.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(0.5, 0.5, 0.5, alpha),
            );
            font.draw_aligned(
                framebuffer,
                "Survive as long as possible",
                vec2(0.0, -3.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(0.5, 0.5, 0.5, alpha),
            );
        } else {
            let font = &self.font;
            let scale = framebuffer_size.y / 20.0;
            let mid = framebuffer_size / 2.0;
            font.draw_aligned(
                framebuffer,
                &format!(
                    "wave #{} in {} secs",
                    self.next_wave,
                    f32::floor(self.next_wave_timer),
                ),
                vec2(0.0, 8.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(1.0, 1.0, 1.0, 0.5),
            );
            font.draw_aligned(
                framebuffer,
                &format!(
                    "{} enemies",
                    self.players.iter().filter(|p| p.team_id != 0).count(),
                ),
                vec2(0.0, -9.0 * scale) + mid,
                0.5,
                scale,
                Color::rgba(1.0, 1.0, 1.0, 0.5),
            );
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyDown { .. } | geng::Event::MouseDown { .. } = event {
            check_music_start();
        }
        match event {
            geng::Event::KeyDown { key } => match key {
                geng::Key::R => self.reset(),
                geng::Key::F => self.context.window().toggle_fullscreen(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn check_music_start() {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !STARTED.fetch_or(true, std::sync::atomic::Ordering::Relaxed) {
        #[cfg(target_arch = "wasm32")]
        js! {
            @(no_return)
            var music = new Audio("music.ogg");
            music.volume = 0.2;
            music.addEventListener("ended", function () {
                this.currentTime = 0;
                this.play();
            }, false);
            music.play();
        }
    }
}

fn main() {
    let context = Rc::new(Geng::new(geng::ContextOptions {
        title: "LifeShot".to_owned(),
        ..default()
    }));
    let game = Game::new(&context);
    geng::run(context, game);
}
