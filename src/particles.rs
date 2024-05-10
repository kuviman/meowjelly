use super::*;

#[derive(ugli::Vertex)]
struct Vertex {
    a_pos: vec2<f32>,
    a_uv: vec2<f32>,
}

#[derive(Debug, ugli::Vertex)]
pub struct Instance {
    pub i_pos: vec3<f32>,
    pub i_vel: vec3<f32>,
    pub i_start_time: f32,
    pub i_end_time: f32,
    pub i_color: Rgba<f32>,
    pub i_size: f32,
}

#[derive(geng::asset::Load)]
struct Shaders {
    particles: ugli::Program,
}

#[derive(geng::asset::Load)]
struct Assets {
    particle: ugli::Texture,
    shaders: Shaders,
}

#[derive(Deserialize)]
pub struct Config {
    pub movement: Rc<SpawnerConfig>,
    pub bounce: Rc<SpawnerConfig>,
    pub death: Rc<SpawnerConfig>,
}

#[derive(Deserialize)]
pub struct SpawnerConfig {
    freq: f32,
    life: f32,
    size: f32,
    color: Rgba<f32>,
    extra_vel: f32,
    extra_life: f32,
    extra_size: f32,
    extra_hue: f32,
    extra_saturation: f32,
    extra_lightness: f32,
}

pub struct ParticleSpawner {
    pub pos: vec3<f32>,
    pub vel: vec3<f32>,
    next: f32,
    config: Rc<SpawnerConfig>,
    inner: Rc<Inner>,
}

impl ParticleSpawner {
    pub fn update(&mut self, delta_time: f32) {
        self.next -= delta_time;
        while self.next < 0.0 {
            let mut instance = self.create();
            let time_fix = -self.next;
            instance.i_start_time -= time_fix;
            instance.i_end_time -= time_fix;
            self.next += 1.0 / self.config.freq;
            self.inner.instances.borrow_mut().push(instance);
        }
    }
    pub fn spawn(&mut self) {
        let instance = self.create();
        self.inner.instances.borrow_mut().push(instance);
    }
    pub fn create(&mut self) -> Instance {
        let mut rng = thread_rng();
        let lifetime = self.config.life + rng.gen_range(0.0..self.config.extra_life);
        let current_time = self.inner.timer.elapsed().as_secs_f64() as f32;
        Instance {
            i_pos: self.pos,
            i_vel: rng
                .gen_circle(self.vel.xy(), self.config.extra_vel)
                .extend(self.vel.z + rng.gen_range(-self.config.extra_vel..self.config.extra_vel)),
            i_start_time: current_time,
            i_end_time: current_time + lifetime,
            i_size: self.config.size + rng.gen_range(0.0..self.config.extra_size),
            i_color: {
                let mut color: Hsla<f32> = self.config.color.into();
                color.h += rng.gen_range(-self.config.extra_hue..self.config.extra_hue);
                color.s +=
                    rng.gen_range(-self.config.extra_saturation..self.config.extra_saturation);
                color.l += rng.gen_range(-self.config.extra_lightness..self.config.extra_lightness);
                color.into()
            },
        }
    }
}

pub struct Inner {
    timer: Timer,
    quad: ugli::VertexBuffer<Vertex>,
    assets: Assets,
    pub config: Config,
    instances: RefCell<ugli::VertexBuffer<Instance>>,
}

#[derive(Deref)]
pub struct Particles {
    inner: Rc<Inner>,
}

impl Particles {
    pub async fn init(geng: &Geng) -> Self {
        let assets = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let config = file::load_detect(run_dir().join("assets").join("particles.toml"))
            .await
            .unwrap();
        let quad = ugli::VertexBuffer::new_static(
            geng.ugli(),
            [(0, 0), (0, 1), (1, 1), (1, 0)]
                .into_iter()
                .map(|(x, y)| Vertex {
                    a_pos: vec2(x, y).map(|x| x as f32 * 2.0 - 1.0),
                    a_uv: vec2(x, y).map(|x| x as f32),
                })
                .collect(),
        );
        Self {
            inner: Rc::new(Inner {
                timer: Timer::new(),
                quad,
                assets,
                config,
                instances: RefCell::new(ugli::VertexBuffer::new_dynamic(geng.ugli(), vec![])),
            }),
        }
    }
    pub fn spawner(&self, config: &Rc<SpawnerConfig>) -> ParticleSpawner {
        ParticleSpawner {
            pos: vec3::ZERO,
            vel: vec3::ZERO,
            next: 0.0,
            config: config.clone(),
            inner: self.inner.clone(),
        }
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &dyn AbstractCamera3d) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let texture = &self.assets.particle;
        let time = self.timer.elapsed().as_secs_f64() as f32;
        let mut instances = self.instances.borrow_mut();
        instances.retain(|instance| instance.i_end_time > time);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.particles,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(&self.quad, &*instances),
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_texture_size: texture.size(),
                    u_time: time,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                // depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }
}
