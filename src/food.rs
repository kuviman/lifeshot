use crate::*;

pub struct Food {
    entity: Entity,
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
    pub fn new(pos: Vec2<f32>, size: f32) -> Self {
        Self {
            entity: Entity {
                owner_id: None,
                color: Color::GREEN,
                pos,
                vel: vec2(0.0, 0.0),
                size,
            },
        }
    }
}
