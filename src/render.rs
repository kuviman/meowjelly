use super::*;

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_uv: vec2<f32>,
}

pub struct Render {
    cylinder: ugli::VertexBuffer<Vertex>,
    assets: Assets,
    pub config: Config,
}

#[derive(geng::asset::Load)]
struct Shaders {
    texture: ugli::Program,
}

#[derive(geng::asset::Load)]
struct Assets {
    shaders: Shaders,
}

#[derive(Deserialize)]
pub struct Config {
    pub cylinder_segments: usize,
    pub fog_distance: f32,
    pub fog_color: Rgba<f32>,
    pub fov: f32,
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
        Self {
            cylinder,
            assets,
            config,
        }
    }

    pub fn cylinder(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn AbstractCamera3d,
        texture: &ugli::Texture,
        z: Range<f32>,
        radius: f32,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let model_matrix = mat4::translate(vec3(0.0, 0.0, z.start))
            * mat4::scale(vec2::splat(radius).extend(z.end - z.start));
        let uv_matrix = mat3::scale(vec2(
            1.0,
            texture.size().map(|x| x as f32).aspect() / (2.0 * f32::PI * radius),
        )) * mat3::translate(vec2(0.0, z.start))
            * mat3::scale(vec2(1.0, z.end - z.start));
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
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );
    }
}
