//! Platform window chrome configuration for the main desktop shell.

pub fn immersive_titlebar_enabled() -> bool {
    cfg!(target_os = "macos")
}

pub fn titlebar_visual_height() -> f32 {
    if immersive_titlebar_enabled() {
        52.0
    } else {
        36.0
    }
}

pub fn titlebar_leading_inset() -> f32 {
    if immersive_titlebar_enabled() {
        86.0
    } else {
        0.0
    }
}

#[cfg(target_os = "macos")]
pub fn install_platform_windowing() -> anyhow::Result<()> {
    use slint::winit_030::winit::platform::macos::WindowAttributesExtMacOS;
    use std::sync::OnceLock;

    static INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    match INIT_RESULT.get_or_init(|| {
        slint::BackendSelector::new()
            .backend_name("winit".into())
            .with_winit_window_attributes_hook(|attributes| {
                attributes
                    .with_titlebar_transparent(true)
                    .with_fullsize_content_view(true)
                    .with_title_hidden(true)
                    .with_movable_by_window_background(true)
            })
            .select()
            .map_err(|err| err.to_string())
    }) {
        Ok(()) => Ok(()),
        Err(message) => Err(anyhow::anyhow!(message.clone())),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn install_platform_windowing() -> anyhow::Result<()> {
    Ok(())
}
