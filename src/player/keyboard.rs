use crate::*;

pub struct KeyboardController {
    context: Rc<geng::Context>,
    mouse_pos: Rc<Cell<Vec2<f32>>>,
}

impl KeyboardController {
    pub fn new(context: &Rc<geng::Context>, mouse_pos: &Rc<Cell<Vec2<f32>>>) -> Self {
        Self {
            context: context.clone(),
            mouse_pos: mouse_pos.clone(),
        }
    }
}

impl Controller for KeyboardController {
    fn act(&mut self, _: usize, _: &Game) -> Action {
        let mut target_vel = vec2(0.0, 0.0);
        if self.context.window().is_key_pressed(geng::Key::W) {
            target_vel.y += 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::A) {
            target_vel.x -= 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::S) {
            target_vel.y -= 1.0;
        }
        if self.context.window().is_key_pressed(geng::Key::D) {
            target_vel.x += 1.0;
        }
        Action {
            target_vel,
            shoot: if self
                .context
                .window()
                .is_button_pressed(geng::MouseButton::Left)
            {
                Some(self.mouse_pos.get())
            } else {
                None
            },
        }
    }
}