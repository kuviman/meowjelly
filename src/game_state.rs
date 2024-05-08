use super::*;

pub struct GameState {
    ctx: Ctx,
}

impl GameState {
    pub fn new(ctx: &Ctx) -> Self {
        Self { ctx: ctx.clone() }
    }
}

impl geng::State for GameState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.ctx.assets.config.background_color),
            Some(1.0),
            None,
        );
    }
}
