use super::*;

#[derive(geng::asset::Load)]
pub struct Walls {
    #[load(options(wrap_mode = "ugli::WrapMode::Repeat"))]
    pub brick: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Player {
    pub head: ugli::Texture,
    pub shadow: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub walls: Walls,
    pub player: Player,
}
