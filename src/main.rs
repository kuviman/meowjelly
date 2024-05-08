use geng::prelude::*;

mod assets;
mod game_state;

use assets::Assets;
use game_state::GameState;

#[derive(Clone, Deref)]
struct Ctx {
    #[deref]
    inner: Rc<CtxInner>,
}

struct CtxInner {
    geng: Geng,
    assets: Assets,
}

async fn run(geng: Geng) {
    let geng = &geng;
    let assets: Assets = geng
        .asset_manager()
        .load(run_dir().join("assets"))
        .await
        .expect("Failed to load assets");
    let ctx = Ctx {
        inner: Rc::new(CtxInner {
            geng: geng.clone(),
            assets,
        }),
    };
    let ctx = &ctx;
    geng.run_state(GameState::new(ctx)).await;
}

#[derive(clap::Parser)]
struct CliArgs {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let args: CliArgs = cli::parse();
    let mut options = geng::ContextOptions::default();
    options.window.title = format!(
        "{name} v{version}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
    );
    options.with_cli(&args.geng);

    Geng::run_with(&options, run);
}
