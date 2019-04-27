use crate::*;

pub struct BotController;

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
        Action {
            target_vel: closest_food.map(|f| f.pos).unwrap_or(vec2(0.0, 0.0)) - me.pos,
            shoot: None,
        }
    }
}
