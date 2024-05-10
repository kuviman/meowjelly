use super::*;

#[derive(geng::asset::Load)]
pub struct Walls {
    #[load(options(wrap_mode = "ugli::WrapMode::Repeat"))]
    pub brick: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Player {
    pub head: ugli::Texture,
    pub leg: ugli::Texture,
    pub shadow: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub walls: Walls,
    #[load(listed_in = "_list.ron")]
    pub obstacles: Vec<Rc<ugli::Texture>>,
    pub player: Player,
}
