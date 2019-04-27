use crate::*;

pub struct BotController;

impl BotController {
    const SHOT_SIZE: f32 = 0.2;
    const MIN_SIZE: f32 = 0.4;
}

impl Controller for BotController {
    fn act(&mut self, self_id: usize, game: &Game) -> Action {
        let me = game
            .players
            .iter()
            .find(|player| player.owner_id.unwrap() == self_id)
            .unwrap();
        let closest_food = game.food.iter().min_by(|a, b| {
            (a.pos - me.pos)
                .len()
                .partial_cmp(&(b.pos - me.pos).len())
                .unwrap()
        });
        let closest_enemy = game
            .players
            .iter()
            .filter(|player| player.owner_id.unwrap() != self_id)
            .min_by(|a, b| {
                (a.pos - me.pos)
                    .len()
                    .partial_cmp(&(b.pos - me.pos).len())
                    .unwrap()
            });
        Action {
            target_vel: closest_food.map(|f| f.pos).unwrap_or(vec2(0.0, 0.0)) - me.pos,
            shoot: if me.size < Self::MIN_SIZE {
                None
            } else {
                match me.projectile {
                    Some(ref p) if p.size > Self::SHOT_SIZE => None,
                    _ => closest_enemy.map(|e| e.pos),
                }
            },
        }
    }
}
