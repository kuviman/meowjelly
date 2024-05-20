use geng::prelude::*;

mod assets;
mod config;
mod controls;
mod ctx;
mod easings;
mod game_state;
mod loading;
mod particles;
mod render;

use easings::*;

use ctx::Ctx;

async fn run(geng: Geng) {
    let ctx = match future::select(
        Ctx::load(&geng).boxed_local(),
        loading::run(&geng).boxed_local(),
    )
    .await
    {
        future::Either::Left(ctx) => ctx.0,
        future::Either::Right(_) => return,
    };
    let ctx = &ctx;
    #[cfg(feature = "yandex")]
    ctx.ysdk.ready();
    game_state::GameState::new(ctx).run().await;
}

#[derive(clap::Parser)]
struct CliArgs {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

pub fn main() {
    logger::init();
    geng::setup_panic_handler();

    let args: CliArgs = if cfg!(feature = "yandex") {
        <CliArgs as clap::Parser>::parse_from::<[&str; 0], _>([])
    } else {
        cli::parse()
    };
    let mut options = geng::ContextOptions::default();
    options.window.title = format!(
        "{name} v{version}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
    );
    options.with_cli(&args.geng);

    Geng::run_with(&options, run);
}
