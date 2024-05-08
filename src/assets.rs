use super::*;

#[derive(geng::asset::Load)]
pub struct Walls {
    #[load(options(wrap_mode = "ugli::WrapMode::Repeat"))]
    pub brick: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub walls: Walls,
}
