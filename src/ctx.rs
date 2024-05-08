use super::*;

#[derive(Clone, Deref)]
pub struct Ctx {
    #[deref]
    inner: Rc<CtxInner>,
}

pub struct CtxInner {
    pub geng: Geng,
    pub assets: assets::Assets,
    pub config: config::Config,
    pub render: render::Render,
    pub controls: controls::Controls,
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
        Self {
            inner: Rc::new(CtxInner {
                geng: geng.clone(),
                assets,
                config,
                controls,
                render,
            }),
        }
    }
}
