#![allow(unused)]

mod app;
mod domain;
mod errors;
mod stack;
mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre.install()?;
    init_logging()?;
    tracing::info!("starting");

    ratatui::run(app::run)?;

    tracing::info("clean exit");
    Ok(())
}

fn init_logging() -> color_eyre::Result<()> {
    let file = std::fs::File::create("app.log")?;
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .init();
    Ok(())
}
