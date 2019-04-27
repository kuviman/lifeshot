use geng::prelude::*;

struct Game {
    context: Rc<geng::Context>,
}

impl Game {
    fn new(context: &Rc<geng::Context>) -> Self {
        Self {
            context: context.clone(),
        }
    }
}

impl geng::App for Game {
    fn update(&mut self, delta_time: f64) {}
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
    }
}

fn main() {
    let context = Rc::new(geng::Context::new(default()));
    let game = Game::new(&context);
    geng::run(context, game);
}
