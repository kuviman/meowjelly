use super::*;

struct Player {
    pos: vec3<f32>,
    radius: f32,
    vel: vec3<f32>,
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
                vel: vec3::ZERO,
                radius: ctx.config.player.radius,
            }),
            transition: None,
            walls: Vec::new(),
        }
    }

    fn key_press(&mut self, key: geng::Key) {
        if self.ctx.controls.quit.contains(&key) {
            self.transition = Some(geng::state::Transition::Pop);
        }
        if self.ctx.controls.restart.contains(&key) {
            *self = Self::new(&self.ctx);
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
        if let Some(player) = &self.player {
            self.ctx.render.sprite(
                framebuffer,
                &self.camera,
                &self.ctx.assets.player.head,
                mat4::translate(player.pos) * mat4::scale_uniform(player.radius),
            );
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        if let Some(player) = &mut self.player {
            // controls
            let mut target_vel = vec2::ZERO;
            let mut control = |keys: &[geng::Key], x: f32, y: f32| {
                if keys
                    .iter()
                    .any(|key| self.ctx.geng.window().pressed_keys().contains(key))
                {
                    target_vel += vec2(x, y);
                }
            };
            control(&self.ctx.controls.player.up, 0.0, 1.0);
            control(&self.ctx.controls.player.left, -1.0, 0.0);
            control(&self.ctx.controls.player.down, 0.0, -1.0);
            control(&self.ctx.controls.player.right, 1.0, 0.0);
            let target_vel = target_vel.clamp_len(..=1.0) * self.ctx.config.player.speed;
            player.vel += (target_vel - player.vel.xy())
                .clamp_len(..=self.ctx.config.player.acceleration * delta_time)
                .extend(0.0);

            // gravity
            player.vel.z = (player.vel.z - self.ctx.config.player.fall_acceleration * delta_time)
                .clamp_abs(self.ctx.config.player.fall_speed);

            // collision with the tube
            let tube_normal = -player.pos.xy().normalize_or_zero();
            let tube_penetration = -vec2::dot(player.pos.xy(), tube_normal) + player.radius
                - self.ctx.config.tube_radius;
            if tube_penetration > 0.0 {
                player.pos += tube_normal.extend(0.0) * tube_penetration;
                let normal_vel = vec2::dot(tube_normal, player.vel.xy());
                if normal_vel < 0.0 {
                    let change = (self.ctx.config.player.bounce_speed - normal_vel) * tube_normal;
                    player.vel += change.extend(0.0);
                }
            }

            player.pos += player.vel * delta_time;

            // camera
            self.camera.pos = (player.pos.xy() * self.ctx.config.camera.horizontal_movement)
                .extend(player.pos.z + self.ctx.config.camera.distance);
            self.camera.vel = player.vel;
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
