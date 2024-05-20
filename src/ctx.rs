use super::*;

#[derive(Clone, Deref)]
pub struct Ctx {
    #[deref]
    inner: Rc<CtxInner>,
}

#[cfg(feature = "yandex")]
pub struct Yandex {
    pub sdk: ysdk::Ysdk,
    pub player: ysdk::Player,
}

pub struct CtxInner {
    pub geng: Geng,
    pub assets: assets::Assets,
    pub config: config::Config,
    pub render: render::Render,
    pub particles: particles::Particles,
    pub controls: controls::Controls,
    #[cfg(feature = "yandex")]
    pub yandex: Yandex,
}

impl Ctx {
    pub async fn load(geng: &Geng) -> Self {
        let config: config::Config =
            file::load_detect(run_dir().join("assets").join("config.toml"))
                .await
                .unwrap();
        let controls = file::load_detect(run_dir().join("assets").join("controls.toml"))
            .await
            .unwrap();
        let assets: assets::Assets = geng
            .asset_manager()
            .load(run_dir().join("assets"))
            .await
            .unwrap();
        let render = render::Render::init(geng).await;
        let particles = particles::Particles::init(geng).await;
        Self {
            inner: Rc::new(CtxInner {
                geng: geng.clone(),
                assets,
                config,
                controls,
                render,
                particles,
                #[cfg(feature = "yandex")]
                yandex: {
                    let sdk = ysdk::Ysdk::init().await.expect("Failed to initialize ysdk");
                    let player = sdk.player(false).await.unwrap();
                    Yandex { sdk, player }
                },
            }),
        }
    }
}
