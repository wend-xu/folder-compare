//! Desktop entry for the Slint UI shell.

mod app;
mod bridge;
mod commands;
mod presenter;
mod state;
mod view_models;

fn main() -> anyhow::Result<()> {
    app::run()
}
