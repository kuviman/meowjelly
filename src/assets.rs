use super::*;

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub background_color: Rgba<f32>,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub config: Config,
}
