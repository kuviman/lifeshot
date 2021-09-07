use crate::*;

pub struct Food {
    entity: Entity,
    parts: Vec<(Vec2<f32>, Entity)>,
    time: f32,
}

impl Deref for Food {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl DerefMut for Food {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl Food {
    const PREFERRED_MASS: f32 = 0.02;
    const COLOR_OFF: f32 = 0.3;
    pub fn new(pos: Vec2<f32>, size: f32) -> Self {
        let part_count = f32::ceil(size * size / Self::PREFERRED_MASS) as usize;
        Self {
            entity: Entity {
                owner_id: None,
                color: Color::GREEN,
                pos,
                vel: vec2(0.0, 0.0),
                size,
            },
            parts: (0..part_count)
                .map(|_| {
                    (
                        random_circle_point(),
                        Entity {
                            owner_id: None,
                            color: Color::rgb(
                                global_rng().gen_range(0.0..=Self::COLOR_OFF),
                                global_rng().gen_range(1.0 - Self::COLOR_OFF..=1.0),
                                global_rng().gen_range(0.0..=Self::COLOR_OFF),
                            ),
                            pos: pos,
                            vel: vec2(0.0, 0.0),
                            size: size / (part_count as f32).sqrt(),
                        },
                    )
                })
                .collect(),
            time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time = (self.time + delta_time * 3.0).min(1.0);
        for &mut (pos, ref mut part) in &mut self.parts {
            part.pos =
                self.entity.pos + self.entity.size * pos * (1.0 - (1.0 - self.time).powf(2.0));
        }
    }

    pub fn draw(&self, particles: &mut Vec<ParticleInstance>) {
        for &(_, ref part) in &self.parts {
            particles.push(ParticleInstance {
                i_pos: part.pos,
                i_color: mix(Color::BLACK, part.color),
                i_size: part.size * 1.1,
            });
            part.draw(particles);
        }
    }
}
