use super::*;

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_uv: vec2<f32>,
}

pub struct Render {
    cylinder: ugli::VertexBuffer<Vertex>,
    quad: ugli::VertexBuffer<Vertex>,
    assets: Assets,
    pub player: Cell<Option<(vec3<f32>, f32)>>,
    pub config: Config,
    pub white_texture: ugli::Texture,
}

#[derive(geng::asset::Load)]
struct Shaders {
    texture: ugli::Program,
    color_overlay: ugli::Program,
}

#[derive(geng::asset::Load)]
struct Assets {
    shaders: Shaders,
    #[load(list = "0..=9", path = "digits/*.png")]
    pub digits: Vec<ugli::Texture>,
}

#[derive(Deserialize)]
pub struct Config {
    pub cylinder_segments: usize,
    pub fog_distance: f32,
    pub fog_color: Rgba<f32>,
}

impl Render {
    pub async fn init(geng: &Geng) -> Self {
        let config: Config = file::load_detect(run_dir().join("assets").join("render.toml"))
            .await
            .unwrap();
        let assets: Assets = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let cylinder = ugli::VertexBuffer::new_static(
            geng.ugli(),
            (0..=config.cylinder_segments)
                .flat_map(|i| {
                    let x = i as f32 / config.cylinder_segments as f32;
                    let angle = Angle::from_degrees(360.0 * x);
                    let (sin, cos) = angle.sin_cos();
                    let pos = vec2(sin, cos);
                    let at_z = |z: f32| Vertex {
                        a_pos: pos.extend(z),
                        a_uv: vec2(x, z),
                    };
                    [at_z(0.0), at_z(1.0)]
                })
                .collect(),
        );
        let quad = ugli::VertexBuffer::new_static(
            geng.ugli(),
            [(0, 0), (0, 1), (1, 1), (1, 0)]
                .into_iter()
                .map(|(x, y)| Vertex {
                    a_pos: vec2(x, y).map(|x| x as f32 * 2.0 - 1.0).extend(0.0),
                    a_uv: vec2(x, y).map(|x| x as f32),
                })
                .collect(),
        );
        Self {
            player: Cell::new(None),
            cylinder,
            quad,
            assets,
            config,
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2::splat(1), |_| Rgba::WHITE),
        }
    }

    pub fn digit(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        value: f32,
        color: Rgba<f32>,
        matrix: mat4<f32>,
    ) {
        let digit = (value.round() as i32).rem_euclid(10) as usize;
        self.sprite_ext(
            framebuffer,
            camera,
            &self.assets.digits[digit],
            matrix * mat4::rotate_x(Angle::from_degrees(value.fract() * 360.0)),
            color,
            false,
        )
    }

    pub fn sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        texture: &ugli::Texture,
        matrix: mat4<f32>,
    ) {
        self.sprite_ext(framebuffer, camera, texture, matrix, Rgba::WHITE, true)
    }

    pub fn color_overlay(&self, framebuffer: &mut ugli::Framebuffer, color: Rgba<f32>) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.color_overlay,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            ugli::uniforms! {
                u_color: color,
            },
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                write_depth: false,
                ..default()
            },
        );
    }

    pub fn sprite_ext(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        texture: &ugli::Texture,
        matrix: mat4<f32>,
        color: Rgba<f32>,
        depth_test: bool,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let (player_pos, player_radius) = self.player.get().unwrap_or((vec3::ZERO, 0.0));
        ugli::draw(
            framebuffer,
            &self.assets.shaders.texture,
            ugli::DrawMode::TriangleFan,
            &self.quad,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_texture_size: texture.size(),
                    u_model_matrix: matrix,
                    u_uv_matrix: mat3::identity(),
                    u_fog_color: self.config.fog_color,
                    u_fog_distance: self.config.fog_distance,
                    u_color: color,
                    u_player_pos: player_pos,
                    u_player_radius: player_radius,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: depth_test.then_some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    pub fn cylinder(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        texture: &ugli::Texture,
        texture_shift: f32,
        z: Range<f32>,
        radius: f32,
    ) {
        let model_matrix = mat4::translate(vec3(0.0, 0.0, z.start))
            * mat4::scale(vec2::splat(radius).extend(z.end - z.start));
        let uv_matrix = mat3::scale(vec2(
            1.0,
            texture.size().map(|x| x as f32).aspect() / (2.0 * f32::PI * radius),
        )) * mat3::translate(vec2(texture_shift, 0.0))
            * mat3::scale(vec2(1.0, z.end - z.start));
        self.cylinder_ext(framebuffer, camera, texture, model_matrix, uv_matrix);
    }

    pub fn cylinder_ext(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        texture: &ugli::Texture,
        model_matrix: mat4<f32>,
        uv_matrix: mat3<f32>,
    ) {
        let (player_pos, player_radius) = self.player.get().unwrap_or((vec3::ZERO, 0.0));
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.texture,
            ugli::DrawMode::TriangleStrip,
            &self.cylinder,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_texture_size: texture.size(),
                    u_model_matrix: model_matrix,
                    u_uv_matrix: uv_matrix,
                    u_fog_color: self.config.fog_color,
                    u_fog_distance: self.config.fog_distance,
                    u_color: Rgba::WHITE,
                    u_player_pos: player_pos,
                    u_player_radius: player_radius,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }
}
