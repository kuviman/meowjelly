use geng::prelude::*;

mod assets;
mod config;
mod ctx;
mod game_state;
mod render;

use ctx::Ctx;

async fn run(geng: Geng) {
    let ctx = Ctx::load(&geng).await;
    let ctx = &ctx;
    geng.run_state(game_state::GameState::new(ctx)).await;
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
