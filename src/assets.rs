use super::*;

#[derive(geng::asset::Load)]
pub struct Player {
    pub death: ugli::Texture,
    pub head: ugli::Texture,
    pub leg: ugli::Texture,
    pub shadow: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Tutorial {
    pub wasd: ugli::Texture,
    pub arrows: ugli::Texture,
    pub touch: ugli::Texture,
    pub r: ugli::Texture,
    pub touch_restart: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Music {
    #[load(ext = "mp3", options(looped = "true"))]
    pub mallet: geng::Sound,
    #[load(ext = "mp3", options(looped = "true"))]
    pub piano: geng::Sound,
    #[load(ext = "mp3", options(looped = "true"))]
    pub guitar: geng::Sound,
}

#[derive(geng::asset::Load)]
pub struct Sfx {
    #[load(options(looped = "true"))]
    pub wind: geng::Sound,
    #[load(options(looped = "true"))]
    pub swim: geng::Sound,
    pub death: geng::Sound,
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
    #[load(list = "0..=9", path = "digits/*.png")]
    pub digits: Vec<ugli::Texture>,
    pub music: Music,
    pub sfx: Sfx,
}
