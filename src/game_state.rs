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

struct TouchControl {
    move_delta: vec2<f32>,
    prev_pos: vec2<f64>,
}

struct Bounce {
    t: f32,
    axis: vec3<f32>,
}

pub struct GameState {
    framebuffer_size: vec2<f32>,
    ctx: Ctx,
    camera: Camera,
    player: Option<Player>,
    transition: Option<geng::state::Transition>,
    walls: Vec<Wall>,
    touch_control: Option<TouchControl>,
    bounce: Option<Bounce>,
}

impl GameState {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            framebuffer_size: vec2::splat(1.0),
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
            touch_control: None,
            bounce: None,
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

    fn touch_start(&mut self, pos: vec2<f64>) {
        self.touch_control = Some(TouchControl {
            move_delta: vec2::ZERO,
            prev_pos: pos,
        });
    }

    fn raycast(&self, window_pos: vec2<f64>) -> vec2<f32> {
        let ray = self
            .camera
            .pixel_ray(self.framebuffer_size, window_pos.map(|x| x as f32));
        let z = self.camera.pos.z - self.ctx.config.camera.distance;
        let t = (z - ray.from.z) / ray.dir.z;
        (ray.from + ray.dir * t).xy()
    }

    fn touch_move(&mut self, pos: vec2<f64>) {
        let mut touch_control = self.touch_control.take();
        if let Some(touch) = &mut touch_control {
            touch.move_delta += self.raycast(pos) - self.raycast(touch.prev_pos);
            touch.prev_pos = pos;
        }
        self.touch_control = touch_control;
    }

    fn touch_end(&mut self) {
        self.touch_control = None;
    }
}

impl geng::State for GameState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
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
            // shadow
            let distance_to_tube =
                self.ctx.config.tube_radius - player.pos.xy().len() - player.radius;
            let shadow_k =
                (1.0 - distance_to_tube / self.ctx.config.shadow.distance).clamp(0.0, 1.0);
            self.ctx.render.sprite_ext(
                framebuffer,
                &self.camera,
                &self.ctx.assets.player.shadow,
                mat4::translate(
                    (player.pos.xy().normalize_or_zero() * self.ctx.config.tube_radius)
                        .extend(player.pos.z),
                ) * mat4::rotate_z(player.pos.xy().arg())
                    * mat4::rotate_y(Angle::from_degrees(-90.0))
                    * mat4::scale_uniform(
                        (1.0 - shadow_k) * self.ctx.config.shadow.scale + shadow_k,
                    ),
                Rgba::new(1.0, 1.0, 1.0, self.ctx.config.shadow.alpha * shadow_k),
                false,
            );

            // head
            let mut transform = mat4::translate(player.pos) * mat4::scale_uniform(player.radius);
            if let Some(bounce) = &self.bounce {
                /// https://easings.net/#easeOutElastic
                fn ease_out_elastic(x: f32) -> f32 {
                    if x == 0.0 {
                        return 0.0;
                    }
                    if x == 1.0 {
                        return 1.0;
                    }
                    let c4 = 2.0 * f32::PI / 3.0;
                    2.0.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0
                }
                transform *= mat4::rotate(
                    bounce.axis,
                    Angle::from_degrees(360.0 * ease_out_elastic(bounce.t)),
                )
            }
            self.ctx.render.sprite(
                framebuffer,
                &self.camera,
                &self.ctx.assets.player.head,
                transform,
            );

            if let Some(touch) = &self.touch_control {
                self.ctx.render.sprite_ext(
                    framebuffer,
                    &self.camera,
                    &self.ctx.render.white_texture,
                    mat4::translate(self.raycast(touch.prev_pos).extend(player.pos.z))
                        * mat4::from_orts(
                            -touch.move_delta.extend(0.0),
                            touch.move_delta.normalize_or_zero().rotate_90().extend(0.0) * 0.1,
                            vec3::UNIT_Z,
                        )
                        * mat4::scale_uniform(0.5)
                        * mat4::translate(vec3(1.0, 1.0, 0.0)),
                    Rgba::WHITE,
                    false,
                );
            }
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        if let Some(bounce) = &mut self.bounce {
            bounce.t += delta_time / self.ctx.config.player.bounce_time;
            if bounce.t >= 1.0 {
                self.bounce = None;
            }
        }

        if let Some(player) = &mut self.player {
            // controls
            let target_vel = if let Some(touch) = &self.touch_control {
                (touch.move_delta / self.ctx.config.touch_control.small_radius).clamp_len(..=1.0)
                    * self.ctx.config.touch_control.max_speed
            } else {
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
                target_vel.clamp_len(..=1.0) * self.ctx.config.player.max_speed
            };
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
                    let mut rng = thread_rng();
                    self.bounce = Some(Bounce {
                        t: 0.0,
                        axis: vec3(
                            rng.gen_range(-1.0..1.0),
                            rng.gen_range(-1.0..1.0),
                            rng.gen_range(-1.0..1.0),
                        )
                        .normalize_or_zero(),
                    });
                }
            }

            let prev_pos = player.pos;
            player.pos += player.vel * delta_time;
            if let Some(touch) = &mut self.touch_control {
                let performed = (player.pos - prev_pos).xy() * 0.5;
                touch.move_delta = touch
                    .move_delta
                    .clamp_len(..=touch.move_delta.len() - performed.len());
                touch.move_delta = touch
                    .move_delta
                    .clamp_len(..=self.ctx.config.touch_control.big_radius);
            }

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
        match event {
            geng::Event::KeyPress { key } => {
                self.key_press(key);
            }
            geng::Event::MousePress { .. } => {
                if let Some(pos) = self.ctx.geng.window().cursor_position() {
                    self.touch_start(pos);
                }
            }
            geng::Event::CursorMove { position } => {
                self.touch_move(position);
            }
            geng::Event::MouseRelease { .. } => {
                self.touch_end();
            }
            geng::Event::TouchStart(touch) => {
                self.touch_start(touch.position);
            }
            geng::Event::TouchMove(touch) => {
                self.touch_move(touch.position);
            }
            geng::Event::TouchEnd(..) => {
                self.touch_end();
            }
            _ => {}
        }
    }
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }
}
