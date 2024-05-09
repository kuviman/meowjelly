use super::*;

#[derive(Deserialize)]
pub struct Camera {
    pub acceleration: f32,
    pub horizontal_movement: f32,
    pub distance: f32,
}

#[derive(Deserialize)]
pub struct Player {
    pub fall_speed: f32,
    pub fall_acceleration: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub radius: f32,
    pub bounce_speed: f32,
    pub bounce_time: f32,
}

#[derive(Deserialize)]
pub struct Shadow {
    pub alpha: f32,
    pub distance: f32,
    pub scale: f32,
}

#[derive(Deserialize)]
pub struct TouchControl {
    pub max_speed: f32,
    pub small_radius: f32,
    pub big_radius: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub tube_radius: f32,
    pub camera: Camera,
    pub wall_section: f32,
    pub shadow: Shadow,
    pub player: Player,
    pub touch_control: TouchControl,
}
