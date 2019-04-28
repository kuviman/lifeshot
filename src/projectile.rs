use crate::*;

pub struct Projectile {
    entity: Entity,
}

impl Deref for Projectile {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl DerefMut for Projectile {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl Projectile {
    pub fn new(owner_id: Option<usize>, color: Color<f32>) -> Self {
        Self {
            entity: Entity {
                owner_id,
                color,
                pos: vec2(0.0, 0.0),
                vel: vec2(0.0, 0.0),
                size: 0.0,
            },
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.entity.update(delta_time);
    }

    pub fn draw(&self, particles: &mut Vec<ParticleInstance>) {
        self.entity.draw(particles);
    }
}
