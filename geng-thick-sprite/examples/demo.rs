use geng::prelude::*;
use geng_thick_sprite::ThickSprite;

fn main() {
    Geng::run("thick sprite", |geng| async move {
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: vec3<f32>,
            a_uv: vec2<f32>,
            a_color: Rgba<f32>,
        }

        impl From<geng_thick_sprite::Vertex> for Vertex {
            fn from(value: geng_thick_sprite::Vertex) -> Self {
                Self {
                    a_pos: value.a_pos,
                    a_uv: value.a_uv,
                    a_color: Hsla::new(thread_rng().gen(), 0.5, 0.5, 1.0).into(),
                }
            }
        }

        let sprite: ThickSprite<Vertex> = geng
            .asset_manager()
            .load_with(
                "assets/obstacles/shovel.png",
                &geng_thick_sprite::Options {
                    cell_size: 20,
                    ..default()
                },
            )
            .await
            .unwrap();
        let program = geng
            .shader_lib()
            .compile(include_str!("shader.glsl"))
            .unwrap();
        while let Some(event) = geng.window().events().next().await {
            if let geng::Event::Draw = event {
                geng.window().with_framebuffer(|framebuffer| {
                    ugli::clear(framebuffer, Some(Rgba::WHITE), Some(1.0), None);
                    ugli::draw(
                        framebuffer,
                        &program,
                        ugli::DrawMode::Triangles,
                        &sprite.mesh,
                        ugli::uniforms! {
                            u_texture: &sprite.texture,
                        },
                        ugli::DrawParameters {
                            depth_func: Some(ugli::DepthFunc::Less),
                            ..default()
                        },
                    );
                });
            }
        }
    });
}
