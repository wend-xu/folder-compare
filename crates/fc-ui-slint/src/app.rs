//! Minimal Slint app shell for Phase 1.

slint::slint! {
    export component MainWindow inherits Window {
        in-out property <string> title_text: "Folder Compare (Phase 1)";

        width: 420px;
        height: 220px;

        VerticalLayout {
            padding: 16px;
            spacing: 8px;

            Text {
                text: root.title_text;
                font-size: 20px;
            }

            Text {
                text: "Workspace skeleton is ready.";
            }
        }
    }
}

/// Runs the UI application.
pub fn run() -> anyhow::Result<()> {
    let app = MainWindow::new().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    app.run().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    Ok(())
}
