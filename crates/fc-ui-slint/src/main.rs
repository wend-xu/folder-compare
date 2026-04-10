//! Desktop entry for the Slint UI shell.

mod app;
mod bridge;
mod commands;
mod compare_file;
mod compare_foundation;
mod compare_tree;
mod context_menu;
mod folder_picker;
mod macos_font_bootstrap;
mod navigator_tree;
mod presenter;
mod settings;
mod state;
#[cfg(test)]
mod tests;
mod toast_controller;
mod view_models;
mod window_chrome;

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("warn,fc_ui_slint=info"))
        .unwrap();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .try_init();
}

fn main() -> anyhow::Result<()> {
    init_tracing();
    macos_font_bootstrap::configure_slint_font_collection();
    app::run()
}
