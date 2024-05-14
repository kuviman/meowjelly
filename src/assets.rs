use geng_thick_sprite::ThickSprite;

use super::*;

#[derive(geng::asset::Load)]
pub struct Player {
    pub death: ugli::Texture,
    pub head: ugli::Texture,
    pub leg: ugli::Texture,
    pub shadow: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Tutorial {
    pub wasd: ugli::Texture,
    pub arrows: ugli::Texture,
    pub touch: ugli::Texture,
    pub r: ugli::Texture,
    pub touch_restart: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Music {
    #[load(ext = "mp3", options(looped = "true"))]
    pub mallet: geng::Sound,
    #[load(ext = "mp3", options(looped = "true"))]
    pub piano: geng::Sound,
    #[load(ext = "mp3", options(looped = "true"))]
    pub guitar: geng::Sound,
}

#[derive(geng::asset::Load)]
pub struct Sfx {
    #[load(options(looped = "true"))]
    pub wind: geng::Sound,
    #[load(options(looped = "true"))]
    pub swim: geng::Sound,
    pub death: geng::Sound,
    pub obstacle_pass: geng::Sound,
    pub hit: geng::Sound,
    pub start: geng::Sound,
    pub coin: geng::Sound,
}

#[derive(Deserialize)]
pub struct ObstacleConfig {
    pub thickness: f32,
}

pub struct Obstacle {
    pub config: ObstacleConfig,
    pub sprite: ThickSprite<render::Vertex>,
    pub data: Vec<Vec<Rgba<u8>>>,
}

impl geng::asset::Load for Obstacle {
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();

        async move {
            let sprite = manager
                .load_with::<ThickSprite<render::Vertex>>(
                    path.with_extension("png"),
                    &geng_thick_sprite::Options {
                        // cell_size: 1,
                        // iso: 0.5,
                        normal_uv_offset: 3.0,
                        ..default()
                    },
                )
                .await?;
            let config: ObstacleConfig = file::load_detect(path.with_extension("toml")).await?;
            let fb = ugli::FramebufferRead::new_color(
                manager.ugli(),
                ugli::ColorAttachmentRead::Texture(&sprite.texture),
            );
            let data = fb.read_color();
            let data = (0..sprite.texture.size().x)
                .map(|x| {
                    (0..sprite.texture.size().y)
                        .map(|y| data.get(x, y))
                        .collect()
                })
                .collect();
            Ok(Self {
                config,
                sprite,
                data,
            })
        }
        .boxed_local()
    }
    type Options = ();
    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(listed_in = "_list.ron")]
    #[load(options(wrap_mode = "ugli::WrapMode::Repeat"))]
    pub walls: Vec<Rc<ugli::Texture>>,
    #[load(listed_in = "_list.ron")]
    pub obstacles: Vec<Rc<Obstacle>>,
    pub player: Player,
    pub tutorial: Tutorial,
    pub music: Music,
    pub sfx: Sfx,
    pub top1: ugli::Texture,
    pub coin: ThickSprite<render::Vertex>,
}

impl From<geng_thick_sprite::Vertex> for render::Vertex {
    fn from(value: geng_thick_sprite::Vertex) -> Self {
        Self {
            a_normal: value.a_normal,
            a_pos: value.a_pos,
            a_uv: value.a_uv,
        }
    }
}
