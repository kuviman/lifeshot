use crate::*;

pub struct Entity {
    pub owner_id: Option<usize>,
    pub color: Color<f32>,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub size: f32,
}

impl Entity {
    pub fn draw(&self, buffer: &mut Vec<ParticleInstance>) {
        buffer.push(ParticleInstance {
            i_pos: self.pos,
            i_size: self.size,
            i_color: self.color,
        });
    }
    pub fn mass(&self) -> f32 {
        self.size * self.size
    }
    pub fn add_mass(&mut self, delta: f32) {
        let mass = self.size * self.size + delta;
        self.size = mass.max(0.0).sqrt();
    }
    pub fn update(&mut self, delta_time: f32) {
        self.pos += self.vel * delta_time;
        self.pos = Game::normalize(self.pos);
    }
    pub fn collide(a: &mut Self, b: &mut Self) {
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
    pub fn hit(&mut self, target: &mut Self, k: f32) -> bool {
        let penetration = (self.size + target.size) - Game::normalize(self.pos - target.pos).len();
        let penetration = penetration.min(min(self.size, target.size));
        if penetration > 0.0 {
            let prev_mass = target.mass();
            target.size = (target.size - penetration).max(0.0);
            let delta_mass = prev_mass - target.mass();
            self.add_mass(-delta_mass / k);
            true
        } else {
            false
        }
    }
    pub fn consume(&mut self, target: &mut Self, k: f32) {
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
