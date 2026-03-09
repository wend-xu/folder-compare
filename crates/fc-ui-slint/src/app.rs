//! Minimal Slint app for Phase 9 compare MVP.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use slint::{ModelRc, SharedString, VecModel};
use std::sync::{Arc, Mutex};

slint::slint! {
    import { Button, LineEdit } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Folder Compare";
        width: 980px;
        height: 700px;

        in-out property <string> left_root;
        in-out property <string> right_root;
        in property <bool> running;
        in property <string> status_text;
        in property <string> summary_text;
        in property <string> warnings_text;
        in property <string> error_text;
        in property <[string]> entry_rows;
        in-out property <int> selected_row: -1;

        callback compare_clicked();
        callback row_selected(int);

        VerticalLayout {
            padding: 12px;
            spacing: 10px;

            Text {
                text: "Folder Compare MVP";
                font-size: 24px;
            }

            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: "Left:";
                    width: 64px;
                }
                LineEdit {
                    text <=> root.left_root;
                    enabled: !root.running;
                }
            }

            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: "Right:";
                    width: 64px;
                }
                LineEdit {
                    text <=> root.right_root;
                    enabled: !root.running;
                }
            }

            HorizontalLayout {
                spacing: 8px;
                Button {
                    text: root.running ? "Comparing..." : "Compare";
                    enabled: !root.running;
                    clicked => {
                        root.compare_clicked();
                    }
                }

                Text {
                    text: root.status_text;
                    color: #444;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #d6d6d6;
                height: 70px;
                Text {
                    text: root.summary_text;
                    wrap: word-wrap;
                    color: #222;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #f0b64f;
                visible: root.warnings_text != "";
                height: root.warnings_text == "" ? 0px : 90px;
                Text {
                    text: root.warnings_text;
                    wrap: word-wrap;
                    color: #7a4b00;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #d36f6f;
                visible: root.error_text != "";
                height: root.error_text == "" ? 0px : 50px;
                Text {
                    text: root.error_text;
                    wrap: word-wrap;
                    color: #8c1d1d;
                }
            }

            Text {
                text: "Results (path / status / detail):";
                color: #444;
            }

            Rectangle {
                border-width: 1px;
                border-color: #d6d6d6;
                height: 360px;
                clip: true;
                VerticalLayout {
                    spacing: 2px;
                    for row[index] in root.entry_rows: Rectangle {
                        height: 26px;
                        background: index == root.selected_row ? #eaf4ff : transparent;
                        Text {
                            text: row;
                            vertical-alignment: center;
                            color: #222;
                        }
                        TouchArea {
                            clicked => {
                                root.row_selected(index);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn sync_window_state(window: &MainWindow, state: &AppState) {
    window.set_left_root(state.left_root.clone().into());
    window.set_right_root(state.right_root.clone().into());
    window.set_running(state.running);
    window.set_status_text(state.status_text.clone().into());
    window.set_summary_text(state.summary_text.clone().into());
    window.set_warnings_text(state.warnings_text().into());
    window.set_error_text(state.error_message.clone().unwrap_or_default().into());
    window.set_selected_row(state.selected_row.map(|value| value as i32).unwrap_or(-1));
    let rows = state
        .entry_display_lines()
        .into_iter()
        .map(SharedString::from)
        .collect::<Vec<_>>();
    window.set_entry_rows(ModelRc::new(VecModel::from(rows)));
}

/// Runs the UI application.
pub fn run() -> anyhow::Result<()> {
    let app = MainWindow::new().map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let state = Arc::new(Mutex::new(AppState::default()));
    let presenter = Presenter::new(state);
    let bridge = UiBridge::new(presenter);
    bridge.dispatch(UiCommand::Initialize);
    sync_window_state(&app, &bridge.snapshot());

    let app_weak = app.as_weak();
    let compare_bridge = bridge.clone();
    app.on_compare_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_bridge.dispatch(UiCommand::UpdateLeftRoot(
            window.get_left_root().to_string(),
        ));
        compare_bridge.dispatch(UiCommand::UpdateRightRoot(
            window.get_right_root().to_string(),
        ));
        compare_bridge.dispatch(UiCommand::RunCompare);
        let state = compare_bridge.snapshot();
        sync_window_state(&window, &state);
    });

    let app_weak = app.as_weak();
    let row_bridge = bridge.clone();
    app.on_row_selected(move |index| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        row_bridge.dispatch(UiCommand::SelectRow(index));
        let state = row_bridge.snapshot();
        sync_window_state(&window, &state);
    });

    app.run().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    Ok(())
}
