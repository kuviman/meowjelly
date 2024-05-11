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
}

pub struct Obstacle {
    pub texture: ugli::Texture,
    pub data: Vec<Vec<Rgba<u8>>>,
}

impl geng::asset::Load for Obstacle {
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let texture = manager.load(path);
        async move {
            let texture = texture.await?;
            let fb = ugli::FramebufferRead::new_color(
                manager.ugli(),
                ugli::ColorAttachmentRead::Texture(&texture),
            );
            let data = fb.read_color();
            let data = (0..texture.size().x)
                .map(|x| (0..texture.size().y).map(|y| data.get(x, y)).collect())
                .collect();
            Ok(Self { texture, data })
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
}
