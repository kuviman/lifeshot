use crate::*;

mod bot;
mod keyboard;

pub use bot::*;
pub use keyboard::*;

pub struct Player {
    entity: Entity,
    pub team_id: usize,
    pub controller: RefCell<Box<dyn Controller>>,
    pub projectile: Option<Projectile>,
    pub action: Cell<Action>,
}

pub trait Controller {
    fn act(&mut self, self_id: usize, game: &Game) -> Action;
}

#[derive(Copy, Clone, Debug)]
pub struct Action {
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

impl Player {
    pub const INITIAL_SIZE: f32 = 1.0;
    const MAX_SPEED: f32 = 8.0;
    const MAX_AIMING_SPEED: f32 = 4.0;
    const ACCELERATION: f32 = 15.0;
    const PROJECTILE_SPEED: f32 = 25.0;
    const PROJECTILE_MASS_GAIN_SPEED: f32 = 0.3;
    const PROJECTILE_COST_SPEED: f32 = 0.1;

    pub fn new<T: Controller + 'static>(
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
    pub fn update(&mut self, delta_time: f32) -> Option<Projectile> {
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
                self.projectile = Some(Projectile::new(
                    self.owner_id,
                    mix(self.color, Color::WHITE),
                ));
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

    pub fn draw(&self, particles: &mut Vec<ParticleInstance>) {
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

    pub fn act(&self, game: &Game) {
        self.action.set(
            self.controller
                .borrow_mut()
                .act(self.owner_id.unwrap(), game),
        );
    }
}
