use self::particles::ParticleSpawner;

use super::*;

struct Player {
    pos: vec3<f32>,
    radius: f32,
    vel: vec3<f32>,
    move_particles: ParticleSpawner,
    leg_rot: Angle<f32>,
}

#[derive(Debug)]
struct CameraShake {
    next: f32,
    offset: vec2<f32>,
    amount: f32,
}

#[derive(Debug)]
struct Camera {
    pos: vec3<f32>,
    shake: CameraShake,
    fov: Angle,
    vel: vec3<f32>,
    far: f32,
}

impl geng::AbstractCamera3d for Camera {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::translate(-self.pos + self.shake.offset.extend(0.0) * self.shake.amount)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        let near = 0.1;
        let far = self.far;
        let fov = self.fov.as_radians();
        let aspect = framebuffer_size.aspect();
        if aspect >= 1.0 {
            let ymax = near * (fov / 2.0).tan();
            let xmax = ymax * aspect;
            mat4::frustum(-xmax, xmax, -ymax, ymax, near, far)
        } else {
            let xmax = near * (fov / 2.0).tan();
            let ymax = xmax / aspect;
            mat4::frustum(-xmax, xmax, -ymax, ymax, near, far)
        }
    }
}

struct Wall {
    texture: Rc<ugli::Texture>,
    range: Range<f32>,
    texture_shift: f32,
}

struct TouchControl {
    move_delta: vec2<f32>,
    prev_pos: vec2<f64>,
}

struct Bounce {
    t: f32,
    axis: vec3<f32>,
}

struct Obstacle {
    z: f32,
    transform: mat4<f32>,
    texture: Rc<ugli::Texture>,
    data: Vec<Vec<Rgba<u8>>>,
}

impl Obstacle {
    /// Returns signed distance if vertical ray hits
    fn hittest(&self, pos: vec3<f32>) -> Option<f32> {
        let inv = (mat4::translate(vec3(0.0, 0.0, self.z)) * self.transform).inverse();
        let from = (inv * pos.extend(1.0)).xyz();
        let dir = (inv * vec4(0.0, 0.0, 1.0, 0.0)).xyz();
        // from + dir * t = 0
        let t = -from.z / dir.z;
        let vec2(x, y) = (from.xy() + dir.xy() * t).map(|x| x * 0.5 + 0.5);
        if x < 0.0 || y < 0.0 || x > 1.0 || y > 1.0 {
            return None;
        }
        let x = (x * self.data.len() as f32).floor() as usize;
        let y = (y * self.data[0].len() as f32).floor() as usize;
        let color = self.data.get(x)?.get(y)?;
        if color.a == 0 {
            return None;
        }
        Some(t)
    }
}

#[derive(Deref, DerefMut)]
struct SoundEffect {
    #[deref]
    #[deref_mut]
    inner: geng::SoundEffect,
    fade_time: time::Duration,
}

impl Drop for SoundEffect {
    fn drop(&mut self) {
        self.inner.fade_out(self.fade_time);
    }
}

impl Ctx {
    fn start_music(&self, sound: &geng::Sound) -> SoundEffect {
        let mut effect = sound.effect();
        let fade_time = time::Duration::from_secs_f64(self.config.music.fade_time);
        effect.fade_in(fade_time);
        effect.play();
        SoundEffect {
            inner: effect,
            fade_time,
        }
    }
    fn sound_effect(&self, sound: &geng::Sound, initial_volume: f32) -> SoundEffect {
        let mut effect = sound.effect();
        effect.set_volume(initial_volume);
        let fade_time = time::Duration::from_secs_f64(self.config.music.fade_time);
        effect.play();
        SoundEffect {
            inner: effect,
            fade_time,
        }
    }
}

pub struct GameState {
    key_input: bool,
    framebuffer_size: vec2<f32>,
    death_rotation: Angle<f32>,
    ctx: Ctx,
    time: f32,
    camera: Camera,
    player: Option<Player>,
    death_location: Option<vec3<f32>>,
    transition: Option<geng::state::Transition>,
    walls: Vec<Wall>,
    obstacles: Vec<Obstacle>,
    touch_control: Option<TouchControl>,
    bounce: Option<Bounce>,
    bounce_particles: ParticleSpawner,
    shake_time: f32,
    started: Option<f32>,
    finished: Option<f32>,
    music: SoundEffect,
    wind: SoundEffect,
}

impl GameState {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            finished: None,
            time: 0.0,
            obstacles: Vec::new(),
            framebuffer_size: vec2::splat(1.0),
            death_location: None,
            wind: ctx.sound_effect(&ctx.assets.sfx.wind, 0.0),
            music: ctx.start_music(&ctx.assets.music.piano),
            camera: Camera {
                pos: vec3::ZERO,
                fov: Angle::from_degrees(ctx.config.camera.start_fov),
                vel: vec3::ZERO,
                far: ctx.render.config.fog_distance,
                shake: CameraShake {
                    next: 0.0,
                    offset: vec2::ZERO,
                    amount: 0.0,
                },
            },
            key_input: false,
            death_rotation: Angle::ZERO,
            player: Some(Player {
                leg_rot: Angle::ZERO,
                pos: vec3::ZERO,
                vel: vec3::ZERO,
                radius: ctx.config.player.radius,
                move_particles: ctx.particles.spawner(&ctx.particles.config.movement),
            }),
            transition: None,
            walls: Vec::new(),
            touch_control: None,
            bounce: None,
            bounce_particles: ctx.particles.spawner(&ctx.particles.config.bounce),
            shake_time: 0.0,
            started: None,
        }
    }

    fn restart(&mut self) {
        *self = Self::new(&self.ctx);
    }

    fn key_press(&mut self, key: geng::Key) {
        self.key_input = true;
        if self.ctx.controls.quit.contains(&key) {
            self.transition = Some(geng::state::Transition::Pop);
        }
        if self.ctx.controls.restart.contains(&key) {
            self.restart();
        }
        if self.finished.unwrap_or(0.0) > 1.0 {
            self.restart();
        }
    }

    fn touch_start(&mut self, pos: vec2<f64>) {
        self.touch_control = Some(TouchControl {
            move_delta: vec2::ZERO,
            prev_pos: pos,
        });
        if self.finished.unwrap_or(0.0) > 1.0 {
            self.restart();
        }
    }

    fn raycast(&self, window_pos: vec2<f64>) -> vec2<f32> {
        let ray = self
            .camera
            .pixel_ray(self.framebuffer_size, window_pos.map(|x| x as f32));
        let z = self.camera.pos.z - self.ctx.config.camera.distance;
        let t = (z - ray.from.z) / ray.dir.z;
        let result = (ray.from + ray.dir * t).xy();
        log::trace!(
            "camera: {camera:#?} raycast from {window_pos} = {result}",
            camera = self.camera,
        );
        result
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
        self.ctx.render.player.set(
            self.player
                .as_ref()
                .map(|player| (player.pos, player.radius)),
        );

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
                &wall.texture,
                wall.texture_shift,
                wall.range.clone(),
                self.ctx.config.tube_radius,
            );
        }

        for obstacle in self.obstacles.iter().rev() {
            self.ctx.render.sprite(
                framebuffer,
                &self.camera,
                &obstacle.texture,
                mat4::translate(vec3(0.0, 0.0, obstacle.z)) * obstacle.transform,
            );
        }

        self.ctx.render.color_overlay(
            framebuffer,
            Rgba::new(0.0, 0.0, 0.0, 1.0 - self.started.unwrap_or(0.0).min(1.0)),
        );
        self.ctx.render.color_overlay(
            framebuffer,
            Rgba::new(0.0, 0.0, 0.0, self.finished.unwrap_or(0.0).min(1.0)),
        );

        if let Some(player) = &self.player {
            #[cfg(feature = "never")]
            {
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
            }

            let mut transform = mat4::translate(player.pos) * mat4::scale_uniform(player.radius);
            transform *= mat4::rotate_y(Angle::from_degrees(
                self.ctx.config.player.rotate_angle * player.vel.x
                    / self.ctx.config.player.max_speed,
            ));
            transform *= mat4::rotate_x(Angle::from_degrees(
                -self.ctx.config.player.rotate_angle * player.vel.y
                    / self.ctx.config.player.max_speed,
            ));
            transform *= mat4::rotate_z(Angle::from_degrees(
                (self.time * f32::PI * 2.0 * self.ctx.config.passive_rotation.frequency).sin()
                    * self.ctx.config.passive_rotation.amplitude
                    + self.time * self.ctx.config.passive_rotation.speed,
            ));
            if let Some(bounce) = &self.bounce {
                transform *= mat4::rotate(
                    bounce.axis,
                    Angle::from_degrees(360.0 * ease_out_elastic(bounce.t)),
                )
            }

            const LEGS: usize = 8;
            for leg in 0..8 {
                static PHASES: once_cell::sync::Lazy<[f32; LEGS]> =
                    once_cell::sync::Lazy::new(|| {
                        let mut rng = StdRng::seed_from_u64(123);
                        std::array::from_fn(|_| rng.gen())
                    });
                let texture = &self.ctx.assets.player.leg;
                let v = vec2(self.ctx.config.legs.length, 0.0)
                    .rotate(Angle::from_degrees(360.0 * leg as f32 / LEGS as f32) + player.leg_rot);
                let v = v + vec2(self.ctx.config.legs.wiggle, 0.0).rotate(Angle::from_degrees(
                    (self.time * self.ctx.config.legs.freq + PHASES[leg]) * 360.0,
                ));
                self.ctx.render.sprite(
                    framebuffer,
                    &self.camera,
                    texture,
                    transform
                        * mat4::from_orts(
                            v.extend(-self.ctx.config.legs.z),
                            v.rotate_90().extend(0.0),
                            vec3::UNIT_Z,
                        )
                        * mat4::scale(
                            vec3(texture.size().map(|x| x as f32).aspect(), 1.0, 1.0) / 2.0,
                        )
                        * mat4::translate(vec3(1.0, 0.0, 0.0)),
                );
            }

            // head
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

        self.ctx.particles.draw(framebuffer, &self.camera);

        if let Some(location) = self.death_location {
            let t = self.finished.unwrap_or(0.0).min(1.0);
            let t = ease_out_elastic(t);
            self.ctx.render.sprite_ext(
                framebuffer,
                &self.camera,
                &self.ctx.assets.player.death,
                mat4::translate(
                    location * (1.0 - t)
                        + self
                            .camera
                            .pos
                            .xy()
                            .extend(self.camera.pos.z - self.ctx.config.camera.distance)
                            * t,
                ) * mat4::scale_uniform(self.ctx.config.player.death_radius)
                    * mat4::rotate_z(self.death_rotation),
                Rgba::WHITE,
                false,
            );
        }

        // tutorial
        {
            let alpha = (1.0 - self.started.unwrap_or(0.0)).clamp(0.0, 1.0);
            for (texture, pos) in [
                (
                    &self.ctx.assets.tutorial.touch,
                    self.ctx.config.tutorial.touch_pos,
                ),
                (
                    &self.ctx.assets.tutorial.wasd,
                    self.ctx.config.tutorial.wasd_pos,
                ),
                (
                    &self.ctx.assets.tutorial.arrows,
                    self.ctx.config.tutorial.arrows_pos,
                ),
            ] {
                // let mut pos = pos;
                // if self.framebuffer_size.aspect() > 1.0 {
                //     pos = pos.xy().rotate_90().extend(pos.z);
                // }
                self.ctx.render.sprite_ext(
                    framebuffer,
                    &self.camera,
                    texture,
                    mat4::translate(pos.extend(0.0))
                        * mat4::scale(
                            texture.size().map(|x| x as f32).extend(1.0)
                                * self.ctx.config.tutorial.scale,
                        ),
                    Rgba::new(1.0, 1.0, 1.0, alpha),
                    false,
                );
            }
        }
        {
            let alpha = self.finished.unwrap_or(0.0);
            for (texture, pos) in [
                (&self.ctx.assets.tutorial.r, self.ctx.config.tutorial.r_pos),
                (
                    &self.ctx.assets.tutorial.touch_restart,
                    self.ctx.config.tutorial.touch_restart_pos,
                ),
            ] {
                // let mut pos = pos;
                // if self.framebuffer_size.aspect() > 1.0 {
                //     pos = pos.xy().rotate_90().extend(pos.z);
                // }
                self.ctx.render.sprite_ext(
                    framebuffer,
                    &self.camera,
                    texture,
                    mat4::translate(pos.extend(-self.ctx.config.camera.distance) + self.camera.pos)
                        * mat4::scale(
                            texture.size().map(|x| x as f32).extend(1.0)
                                * self.ctx.config.tutorial.final_scale,
                        ),
                    Rgba::new(1.0, 1.0, 1.0, alpha),
                    false,
                );
            }
        }
    }
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;

        self.time += delta_time;
        if let Some(time) = &mut self.started {
            *time += delta_time / self.ctx.config.start_time;
        }
        if self.finished.is_none() && self.player.is_none() {
            self.finished = Some(0.0);
            self.music = self.ctx.start_music(&self.ctx.assets.music.mallet);
        }
        if let Some(time) = &mut self.finished {
            *time += delta_time / self.ctx.config.finish_time;
        }
        {
            let t = partial_min(
                self.started.unwrap_or(0.0).clamp(0.0, 1.0),
                1.0,
                // (1.0 - self.finished.unwrap_or(0.0)).clamp(0.0, 1.0),
            );
            self.camera.fov = Angle::from_degrees(
                t * self.ctx.config.camera.fov + (1.0 - t) * self.ctx.config.camera.start_fov,
            );
        }

        self.shake_time -= delta_time;
        self.camera.shake.amount = self.ctx.config.shake.amount
            * (self.shake_time / self.ctx.config.shake.time).clamp(0.0, 1.0);
        self.camera.shake.next -= delta_time;
        if self.camera.shake.next < 0.0 {
            self.camera.shake.next = 1.0 / self.ctx.config.shake.freq;
            self.camera.shake.offset = thread_rng().gen_circle(vec2::ZERO, 1.0);
        }

        if let Some(bounce) = &mut self.bounce {
            bounce.t += delta_time / self.ctx.config.player.bounce_time;
            if bounce.t >= 1.0 {
                self.bounce = None;
            }
        }

        if let Some(player) = &mut self.player {
            self.wind.set_volume(
                player.vel.xy().len() / self.ctx.config.player.max_speed
                    * self.ctx.config.sfx.wind_move_volume
                    + player.vel.z.abs() / self.ctx.config.player.fall_speed
                        * self.ctx.config.sfx.wind_fall_volume,
            );

            player.move_particles.pos = player.pos;
            player.move_particles.vel = player.vel * self.ctx.config.player.particle_speed_ratio;
            player
                .move_particles
                .update(delta_time * player.vel.z.abs() / self.ctx.config.player.fall_speed);

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
                if self.key_input {
                    control(&self.ctx.controls.player.up, 0.0, 1.0);
                    control(&self.ctx.controls.player.left, -1.0, 0.0);
                    control(&self.ctx.controls.player.down, 0.0, -1.0);
                    control(&self.ctx.controls.player.right, 1.0, 0.0);
                }
                target_vel.clamp_len(..=1.0) * self.ctx.config.player.max_speed
            };
            if self.started.is_none() && (target_vel != vec2::ZERO || self.touch_control.is_some())
            {
                self.music = self.ctx.start_music(&self.ctx.assets.music.guitar);
                self.started = Some(0.0);
            }
            let target_vel = target_vel * self.started.unwrap_or(0.0).min(1.0);
            assert!(target_vel.x.is_finite());
            player.vel += (target_vel - player.vel.xy())
                .clamp_len(..=self.ctx.config.player.acceleration * delta_time)
                .extend(0.0);

            // gravity
            if self.started.is_some() {
                player.vel.z -= if player.vel.z.abs() > self.ctx.config.player.fall_speed {
                    self.ctx.config.player.fall_slow_acceleration
                } else {
                    self.ctx.config.player.fall_acceleration
                } * delta_time;
            }

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
                    self.bounce_particles.pos = (player.pos.xy().normalize()
                        * self.ctx.config.tube_radius)
                        .extend(player.pos.z);
                    self.bounce_particles.vel = (-player.pos.xy().normalize()
                        * self.ctx.config.bounce_particle_speed)
                        .extend(player.vel.z);
                    for _ in 0..self.ctx.config.bounce_particles {
                        self.bounce_particles.spawn();
                    }
                    self.shake_time = self.ctx.config.shake.time;
                }
            }

            let prev_pos = player.pos;
            player.pos += player.vel * delta_time;
            if let Some(touch) = &mut self.touch_control {
                if touch.move_delta.len() < 1e-3 {
                    touch.move_delta = vec2::ZERO;
                } else {
                    let performed = (player.pos - prev_pos).xy() * 0.5; // TODO: why 0.5???
                    touch.move_delta = touch
                        .move_delta
                        .clamp_len(..=touch.move_delta.len() - performed.len());
                    touch.move_delta = touch
                        .move_delta
                        .clamp_len(..=self.ctx.config.touch_control.big_radius);
                }
            }

            player.leg_rot += Angle::from_degrees(
                self.ctx.config.legs.rotate_speed * player.vel.xy().len()
                    / self.ctx.config.player.max_speed
                    * delta_time,
            );

            // camera
            self.camera.pos = (player.pos.xy() * self.ctx.config.camera.horizontal_movement)
                .extend(player.pos.z + self.ctx.config.camera.distance);
            self.camera.vel = player.vel;

            // collisions
            let hitted_at = |local: vec2<f32>| -> bool {
                let local = local.extend(0.0) * player.radius;
                for obstacle in &self.obstacles {
                    let Some(prev) = obstacle.hittest(prev_pos + local) else {
                        continue;
                    };
                    let Some(new) = obstacle.hittest(player.pos + local) else {
                        continue;
                    };
                    if prev * new <= 0.0 {
                        return true;
                    }
                }
                false
            };

            // death
            let died = 'died: {
                for obstacle in &self.obstacles {
                    let Some(prev) = obstacle.hittest(prev_pos) else {
                        continue;
                    };
                    let Some(new) = obstacle.hittest(player.pos) else {
                        continue;
                    };
                    if new * prev <= 0.0
                        || new.abs().min(prev.abs()) < self.ctx.config.death_distance
                    {
                        break 'died true;
                    }
                }
                false
            };
            if died {
                let mut spawner = self.ctx.particles.spawner(&self.ctx.particles.config.death);
                spawner.pos = player.pos;
                spawner.vel = vec3::ZERO;
                for _ in 0..self.ctx.config.bounce_particles {
                    spawner.spawn();
                }
                self.death_location = Some(player.pos);
                self.player = None;
                self.death_rotation = thread_rng().gen();
                self.shake_time = self.ctx.config.shake.time;
            } else {
                // bounce
                const CHECKS: usize = 10;
                for i in 0..CHECKS {
                    let angle = 2.0 * f32::PI * i as f32 / CHECKS as f32;
                    let (sin, cos) = angle.sin_cos();
                    let v = vec2(sin, cos);
                    if hitted_at(v) {
                        let normal = -v;
                        let normal_vel = vec2::dot(player.vel.xy(), normal);
                        let change = (self.ctx.config.player.bounce_speed - normal_vel) * normal;
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
                        self.bounce_particles.pos = player.pos;
                        self.bounce_particles.vel =
                            (v * self.ctx.config.bounce_particle_speed).extend(player.vel.z);
                        for _ in 0..self.ctx.config.bounce_particles {
                            self.bounce_particles.spawn();
                        }
                        self.shake_time = self.ctx.config.shake.time;
                        break;
                    }
                }
            }
        } else {
            self.wind.set_volume(0.0);
            self.camera.pos += self.camera.vel * delta_time;
            self.camera.pos = self
                .camera
                .pos
                .xy()
                .clamp_len(..=self.ctx.config.tube_radius - self.ctx.config.player.radius)
                .extend(self.camera.pos.z);
            self.camera.vel -= self
                .camera
                .vel
                .clamp_len(..=self.ctx.config.camera.acceleration * delta_time);
        }

        let far = self.camera.pos.z - self.camera.far;
        while self.walls.last().map_or(true, |last| last.range.end > far) {
            let start = self.walls.last().map_or(0.0, |last| last.range.end);
            let texture = self
                .ctx
                .assets
                .walls
                .choose(&mut thread_rng())
                .unwrap()
                .clone();
            let len = 2.0 * f32::PI * self.ctx.config.tube_radius
                / texture.size().map(|x| x as f32).aspect();
            self.walls.push(Wall {
                texture,
                range: start..start - len,
                texture_shift: thread_rng().gen(),
            });
        }
        self.walls
            .retain(|wall| wall.range.end < self.camera.pos.z + 10.0);
        while self.obstacles.last().map_or(true, |last| last.z > far) {
            let z = self.obstacles.last().map_or(0.0, |last| last.z)
                - thread_rng().gen_range(self.ctx.config.obstacles.distance.range());
            let texture = self
                .ctx
                .assets
                .obstacles
                .choose(&mut thread_rng())
                .unwrap()
                .clone();
            let mut aspect = texture.size().map(|x| x as f32).aspect();
            let mut transform = mat4::identity();
            if aspect >= 1.0 {
                // transform *= mat4::rotate_z(Angle::from_degrees(90.0));
                aspect = 1.0 / aspect;
            }
            transform =
                mat4::scale(vec3(1.0, 1.0 / aspect, 1.0) * self.ctx.config.tube_radius) * transform;
            transform = mat4::rotate_x(Angle::from_radians(
                aspect.acos() * if thread_rng().gen() { -1.0 } else { 1.0 },
            )) * transform;
            transform = mat4::rotate_z(thread_rng().gen()) * transform;

            let fb = ugli::FramebufferRead::new_color(
                self.ctx.geng.ugli(),
                ugli::ColorAttachmentRead::Texture(&texture),
            );
            let data = fb.read_color();
            let data = (0..texture.size().x)
                .map(|x| (0..texture.size().y).map(|y| data.get(x, y)).collect())
                .collect();

            self.obstacles.push(Obstacle {
                z,
                transform,
                texture,
                data,
            });
        }
        self.obstacles
            .retain(|obstacle| obstacle.z < self.camera.pos.z + 10.0);
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
