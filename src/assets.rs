use super::*;

#[derive(geng::asset::Load)]
pub struct Player {
    pub head: ugli::Texture,
    pub leg: ugli::Texture,
    pub shadow: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Tutorial {
    pub keyboard: ugli::Texture,
    pub touch: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(listed_in = "_list.ron")]
    #[load(options(wrap_mode = "ugli::WrapMode::Repeat"))]
    pub walls: Vec<Rc<ugli::Texture>>,
    #[load(listed_in = "_list.ron")]
    pub obstacles: Vec<Rc<ugli::Texture>>,
    pub player: Player,
    pub tutorial: Tutorial,
}
