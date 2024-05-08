use super::*;

#[derive(Deserialize)]
pub struct Camera {
    pub acceleration: f32,
    pub distance: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub tube_radius: f32,
    pub fall_speed: f32,
    pub acceleration: f32,
    pub camera: Camera,
    pub wall_section: f32,
}
