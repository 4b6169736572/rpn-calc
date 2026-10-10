#![allow(unused)]
mod app;
mod domain;
mod errors;
mod stack;
mod ui;

fn main() {
    ratatui::run(app::run).unwrap();
}
