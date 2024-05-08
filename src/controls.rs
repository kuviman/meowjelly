use super::*;

#[derive(Deserialize)]
pub struct Player {
    pub up: Vec<geng::Key>,
    pub left: Vec<geng::Key>,
    pub down: Vec<geng::Key>,
    pub right: Vec<geng::Key>,
}

#[derive(Deserialize)]
pub struct Controls {
    pub restart: Vec<geng::Key>,
    pub quit: Vec<geng::Key>,
    pub player: Player,
}
