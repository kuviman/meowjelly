use super::*;

#[derive(Deserialize)]
pub struct Camera {
    pub acceleration: f32,
    pub horizontal_movement: f32,
    pub distance: f32,
    pub start_fov: f32,
    pub fov: f32,
}

#[derive(Deserialize)]
pub struct Player {
    pub fall_speed: f32,
    pub fall_acceleration: f32,
    pub fall_slow_acceleration: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub radius: f32,
    pub bounce_speed: f32,
    pub bounce_time: f32,
    pub rotate_angle: f32,
    pub particle_speed_ratio: f32,
    pub death_radius: f32,
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
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}

impl<T: Copy> MinMax<T> {
    pub fn range(&self) -> RangeInclusive<T> {
        self.min..=self.max
    }
}

#[derive(Deserialize)]
pub struct Obstacles {
    pub distance: MinMax<f32>,
}

#[derive(Deserialize)]
pub struct Legs {
    pub length: f32,
    pub wiggle: f32,
    pub freq: f32,
    pub z: f32,
    pub rotate_speed: f32,
}

#[derive(Deserialize)]
pub struct Tutorial {
    pub scale: f32,
    pub final_scale: f32,
    pub touch_pos: vec2<f32>,
    pub wasd_pos: vec2<f32>,
    pub arrows_pos: vec2<f32>,
    pub touch_restart_pos: vec2<f32>,
    pub r_pos: vec2<f32>,
}

#[derive(Deserialize)]
pub struct Music {
    pub fade_time: f64,
}

#[derive(Deserialize)]
pub struct Sfx {
    pub swim_volume: f32,
    pub wind_move_volume: f32,
    pub wind_fall_volume: f32,
    pub obstacle_pass_volume: f32,
    pub obstacle_pass_speed_range: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub sfx: Sfx,
    pub music: Music,
    pub tutorial: Tutorial,
    pub start_time: f32,
    pub finish_time: f32,
    pub death_distance: f32,
    pub legs: Legs,
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
    pub obstacles: Obstacles,
}
