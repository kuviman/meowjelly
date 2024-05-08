use super::*;

struct Player {
    pos: vec3<f32>,
    fall_speed: f32,
}

struct Camera {
    pos: vec3<f32>,
    fov: Angle,
    vel: vec3<f32>,
    far: f32,
}

impl geng::AbstractCamera3d for Camera {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        mat4::perspective(
            self.fov.as_radians(),
            framebuffer_size.aspect(),
            0.1,
            self.far,
        )
    }
}

struct Wall {
    range: Range<f32>,
}

pub struct GameState {
    ctx: Ctx,
    camera: Camera,
    player: Option<Player>,
    transition: Option<geng::state::Transition>,
    walls: Vec<Wall>,
}

impl GameState {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            camera: Camera {
                pos: vec3::ZERO,
                fov: Angle::from_degrees(ctx.render.config.fov),
                vel: vec3::ZERO,
                far: ctx.render.config.fog_distance,
            },
            player: Some(Player {
                pos: vec3::ZERO,
                fall_speed: 0.0,
            }),
            transition: None,
            walls: Vec::new(),
        }
    }

    fn key_press(&mut self, key: geng::Key) {
        match key {
            geng::Key::Escape => {
                self.transition = Some(geng::state::Transition::Pop);
            }
            geng::Key::R => {
                *self = Self::new(&self.ctx);
            }
            _ => {}
        }
    }
}

impl geng::State for GameState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.ctx.render.config.fog_color),
            Some(1.0),
            None,
        );
        for wall in self.walls.iter().rev() {
            if wall.range.end > self.camera.pos.z {
                break;
            }
            self.ctx.render.cylinder(
                framebuffer,
                &self.camera,
                &self.ctx.assets.walls.brick,
                wall.range.clone(),
                self.ctx.config.tube_radius,
            );
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        if let Some(player) = &mut self.player {
            player.fall_speed = (player.fall_speed + self.ctx.config.acceleration * delta_time)
                .min(self.ctx.config.fall_speed);
            player.pos.z -= player.fall_speed * delta_time;

            self.camera.pos = player.pos;
            self.camera.pos.z += self.ctx.config.camera.distance;
            self.camera.vel = vec2::ZERO.extend(-player.fall_speed);
        } else {
            self.camera.pos += self.camera.vel * delta_time;
            self.camera.vel -= self
                .camera
                .vel
                .clamp_len(..=self.ctx.config.camera.acceleration * delta_time);
        }

        let far = self.camera.pos.z - self.camera.far;
        while self.walls.last().map_or(true, |last| last.range.end > far) {
            let start = self.walls.last().map_or(0.0, |last| last.range.end);
            let len = self.ctx.config.wall_section;
            self.walls.push(Wall {
                range: start..start - len,
            });
        }
    }
    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyPress { key } = event {
            self.key_press(key)
        }
    }
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
}
