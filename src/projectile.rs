use crate::*;

pub struct Projectile {
    entity: Entity,
    sparks: Vec<(f32, Entity)>,
    next_spark: f32,
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
    const SPARK_FREQ: f32 = 500.0;
    const SPARK_MAX_SPEED: f32 = 5.0;
    const SPARK_LIFE: f32 = 0.3;

    pub fn new(owner_id: Option<usize>, color: Color<f32>) -> Self {
        Self {
            entity: Entity {
                owner_id,
                color,
                pos: vec2(0.0, 0.0),
                vel: vec2(0.0, 0.0),
                size: 0.0,
            },
            sparks: Vec::new(),
            next_spark: 0.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.size > 0.0 || !self.sparks.is_empty()
    }

    pub fn update(&mut self, delta_time: f32) {
        self.entity.update(delta_time);
        self.next_spark -= delta_time * self.mass();
        while self.next_spark < 0.0 {
            self.next_spark += 1.0 / Self::SPARK_FREQ;
            self.sparks.push((
                0.0,
                Entity {
                    owner_id: None,
                    color: mix(Color::WHITE, self.entity.color),
                    pos: self.entity.pos,
                    vel: random_circle_point() * Self::SPARK_MAX_SPEED,
                    size: self.entity.size / 2.0,
                },
            ))
        }
        for &mut (ref mut t, ref mut e) in &mut self.sparks {
            *t += delta_time;
            e.update(delta_time);
            e.color.a = 1.0 - *t / Self::SPARK_LIFE;
        }
        self.sparks.retain(|&(t, _)| t < Self::SPARK_LIFE);
    }

    pub fn draw(&self, particles: &mut Vec<ParticleInstance>) {
        self.entity.draw(particles);
        for &(_, ref e) in &self.sparks {
            e.draw(particles);
        }
    }
}
