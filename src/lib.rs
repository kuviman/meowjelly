use geng::prelude::*;

mod assets;
mod config;
mod controls;
mod ctx;
mod easings;
mod fancy_number;
mod game_state;
mod loading;
mod particles;
mod render;

use easings::*;

use ctx::Ctx;

async fn run(args: CliArgs, geng: Geng) {
    let ctx = match future::select(
        Ctx::load(args, &geng).boxed_local(),
        loading::run(&geng).boxed_local(),
    )
    .await
    {
        future::Either::Left(ctx) => ctx.0,
        future::Either::Right(_) => return,
    };
    let ctx = &ctx;
    let game = game_state::GameState::new(ctx).await;
    #[cfg(feature = "yandex")]
    ctx.yandex.sdk.ready();
    game.run().await;
}

#[derive(clap::Parser)]
struct CliArgs {
    #[clap(long)]
    mobile: Option<bool>,
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

    Geng::run_with(&options, |geng| run(args, geng));
}
