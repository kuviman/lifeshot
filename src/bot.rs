use crate::*;

pub struct BotController;

impl BotController {
    const SHOT_HIT_SIZE: f32 = 0.3;
    const MIN_SIZE: f32 = 0.7;
}

impl Controller for BotController {
    fn act(&mut self, self_id: usize, game: &Game) -> Action {
        let me = game
            .players
            .iter()
            .find(|player| player.owner_id.unwrap() == self_id)
            .unwrap();
        let closest_food = game.food.iter().min_by(|a, b| {
            Game::delta_pos(me.pos, a.pos)
                .len()
                .partial_cmp(&Game::delta_pos(me.pos, b.pos).len())
                .unwrap()
        });
        let closest_enemy = game
            .players
            .iter()
            .filter(|player| player.owner_id.unwrap() != self_id)
            .min_by(|a, b| {
                Game::delta_pos(me.pos, a.pos)
                    .len()
                    .partial_cmp(&Game::delta_pos(me.pos, b.pos).len())
                    .unwrap()
            });
        Action {
            target_vel: closest_food.map(|f| f.pos).unwrap_or(vec2(0.0, 0.0)) - me.pos,
            shoot: closest_enemy.and_then(|e| match me.projectile {
                Some(ref p) => {
                    let hit_time = Game::delta_pos(p.pos, e.pos).len() / Player::PROJECTILE_SPEED;
                    if p.size - Game::PROJECTILE_DEATH_SPEED * hit_time > Self::SHOT_HIT_SIZE {
                        None
                    } else {
                        Some(e.pos + e.vel * hit_time)
                    }
                }
                _ => {
                    if me.size < Self::MIN_SIZE {
                        None
                    } else {
                        Some(e.pos)
                    }
                }
            }),
        }
    }
}
