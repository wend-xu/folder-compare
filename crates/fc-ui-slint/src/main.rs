//! Desktop entry for the Slint UI shell.

mod app;
mod bridge;
mod commands;
mod context_menu;
mod folder_picker;
mod navigator_tree;
mod presenter;
mod settings;
mod state;
mod toast_controller;
mod view_models;
mod window_chrome;

fn main() -> anyhow::Result<()> {
    app::run()
}
