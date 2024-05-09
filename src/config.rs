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
    pub rotate_angle: f32,
    pub particle_speed_ratio: f32,
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
pub struct PassiveRotation {
    pub speed: f32,
    pub frequency: f32,
    pub amplitude: f32,
}

#[derive(Deserialize)]
pub struct Shake {
    pub time: f32,
    pub amount: f32,
    pub freq: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub shake: Shake,
    pub bounce_particles: usize,
    pub bounce_particle_speed: f32,
    pub tube_radius: f32,
    pub camera: Camera,
    pub wall_section: f32,
    pub shadow: Shadow,
    pub player: Player,
    pub passive_rotation: PassiveRotation,
    pub touch_control: TouchControl,
}
