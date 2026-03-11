//! Desktop entry for the Slint UI shell.

mod app;
mod bridge;
mod commands;
mod folder_picker;
mod presenter;
mod settings;
mod state;
mod view_models;

fn main() -> anyhow::Result<()> {
    app::run()
}
