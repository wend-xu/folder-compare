//! Slint app for compare + detailed diff with Phase 10.5 usability improvements.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use slint::{ModelRc, SharedString, VecModel};
use std::sync::{Arc, Mutex};

slint::slint! {
    import { Button, LineEdit, ListView } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Folder Compare";
        width: 980px;
        height: 920px;

        in-out property <string> left_root;
        in-out property <string> right_root;
        in property <bool> running;
        in property <string> status_text;
        in property <string> summary_text;
        in property <string> warnings_text;
        in property <string> error_text;
        in-out property <string> entry_filter;
        in property <string> filter_stats_text;
        in property <[string]> row_statuses;
        in property <[string]> row_paths;
        in property <[string]> row_details;
        in property <[int]> row_source_indices;
        in property <[bool]> row_can_load_diff;
        in property <bool> diff_loading;
        in property <string> selected_relative_path;
        in property <string> diff_summary_text;
        in property <string> diff_warning_text;
        in property <string> diff_error_text;
        in property <bool> diff_truncated;
        in property <[string]> diff_row_kinds;
        in property <[string]> diff_old_line_nos;
        in property <[string]> diff_new_line_nos;
        in property <[string]> diff_markers;
        in property <[string]> diff_contents;
        in-out property <int> selected_row: -1;

        callback compare_clicked();
        callback filter_changed(string);
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
                text: "Compare Results";
                color: #444;
            }

            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: "Filter:";
                    width: 64px;
                    color: #444;
                }
                LineEdit {
                    text <=> root.entry_filter;
                    enabled: !root.running;
                    edited(value) => {
                        root.filter_changed(value);
                    }
                }
                Text {
                    text: root.filter_stats_text;
                    color: #666;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #d6d6d6;
                height: 260px;
                ListView {
                    for row_path[index] in root.row_paths: Rectangle {
                        height: 50px;
                        border-width: 1px;
                        border-color: root.row_source_indices[index] == root.selected_row ? rgb(110, 174, 232) : rgb(236, 236, 236);
                        background: root.row_source_indices[index] == root.selected_row ? rgb(234, 244, 255)
                            : (root.row_can_load_diff[index] ? rgb(255, 255, 255) : rgb(247, 247, 247));

                        VerticalLayout {
                            spacing: 2px;
                            HorizontalLayout {
                                spacing: 6px;
                                Rectangle {
                                    width: 92px;
                                    height: 20px;
                                    border-radius: 4px;
                                    background: root.row_statuses[index] == "different" ? rgb(255, 226, 226)
                                        : (root.row_statuses[index] == "equal" ? rgb(232, 248, 234)
                                        : (root.row_statuses[index] == "left-only" || root.row_statuses[index] == "right-only" ? rgb(255, 242, 218) : rgb(238, 242, 247)));
                                    Text {
                                        text: root.row_statuses[index];
                                        horizontal-alignment: center;
                                        vertical-alignment: center;
                                        color: #333;
                                    }
                                }
                                Text {
                                    text: row_path;
                                    color: #1b3a57;
                                    vertical-alignment: center;
                                }
                            }
                            Text {
                                text: root.row_can_load_diff[index] ? root.row_details[index] : root.row_details[index] + " | detailed diff unavailable";
                                color: root.row_can_load_diff[index] ? #555 : #777;
                                vertical-alignment: center;
                            }
                        }

                        TouchArea {
                            clicked => {
                                root.row_selected(root.row_source_indices[index]);
                            }
                        }
                    }
                }
            }

            Text {
                text: "Detailed Diff Panel:";
                color: #444;
            }

            Rectangle {
                border-width: 1px;
                border-color: #d6d6d6;
                height: 68px;
                VerticalLayout {
                    spacing: 2px;
                    Text {
                        text: root.selected_relative_path == "" ? "Path: (none selected)" : "Path: " + root.selected_relative_path;
                        color: #222;
                    }
                    Text {
                        text: root.diff_summary_text;
                        color: #222;
                    }
                    Text {
                        text: root.diff_loading ? "Loading detailed diff..." : (root.diff_truncated ? "Detailed diff is truncated." : "");
                        color: #7a4b00;
                    }
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #f0b64f;
                visible: root.diff_warning_text != "";
                height: root.diff_warning_text == "" ? 0px : 60px;
                Text {
                    text: root.diff_warning_text;
                    wrap: word-wrap;
                    color: #7a4b00;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #d36f6f;
                visible: root.diff_error_text != "";
                height: root.diff_error_text == "" ? 0px : 60px;
                Text {
                    text: root.diff_error_text;
                    wrap: word-wrap;
                    color: #8c1d1d;
                }
            }

            Rectangle {
                border-width: 1px;
                border-color: #d6d6d6;
                height: 320px;
                ListView {
                    for row_content[index] in root.diff_contents: Rectangle {
                        height: root.diff_row_kinds[index] == "hunk" ? 28px : 24px;
                        background: root.diff_row_kinds[index] == "hunk" ? rgb(238, 244, 251)
                            : (root.diff_row_kinds[index] == "added" ? rgb(236, 255, 241)
                            : (root.diff_row_kinds[index] == "removed" ? rgb(255, 241, 241) : rgb(255, 255, 255)));

                        Text {
                            visible: root.diff_row_kinds[index] == "hunk";
                            text: row_content;
                            color: #1c4365;
                            vertical-alignment: center;
                        }

                        HorizontalLayout {
                            visible: root.diff_row_kinds[index] != "hunk";
                            spacing: 8px;
                            Text {
                                text: root.diff_old_line_nos[index];
                                width: 56px;
                                horizontal-alignment: right;
                                vertical-alignment: center;
                                color: #6a6a6a;
                            }
                            Text {
                                text: root.diff_new_line_nos[index];
                                width: 56px;
                                horizontal-alignment: right;
                                vertical-alignment: center;
                                color: #6a6a6a;
                            }
                            Text {
                                text: root.diff_markers[index];
                                width: 20px;
                                horizontal-alignment: center;
                                vertical-alignment: center;
                                color: root.diff_row_kinds[index] == "added" ? #1f6d39
                                    : (root.diff_row_kinds[index] == "removed" ? #8a1b1b : #4a4a4a);
                            }
                            Text {
                                text: row_content;
                                color: #222;
                                vertical-alignment: center;
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
    window.set_entry_filter(state.entry_filter.clone().into());
    window.set_filter_stats_text(state.filter_stats_text().into());
    window.set_diff_loading(state.diff_loading);
    window.set_selected_relative_path(state.selected_relative_path_text().into());
    window.set_diff_summary_text(
        state
            .selected_diff
            .as_ref()
            .map(|vm| vm.summary_text.clone())
            .unwrap_or_default()
            .into(),
    );
    window.set_diff_warning_text(state.diff_warning_text().into());
    window.set_diff_error_text(state.diff_error_message.clone().unwrap_or_default().into());
    window.set_diff_truncated(state.diff_truncated);
    window.set_selected_row(state.selected_row.map(|value| value as i32).unwrap_or(-1));
    let filtered_rows = state.filtered_entry_rows_with_index();
    let row_statuses = filtered_rows
        .iter()
        .map(|(_, row)| SharedString::from(row.status.clone()))
        .collect::<Vec<_>>();
    window.set_row_statuses(ModelRc::new(VecModel::from(row_statuses)));
    let row_paths = filtered_rows
        .iter()
        .map(|(_, row)| SharedString::from(row.relative_path.clone()))
        .collect::<Vec<_>>();
    window.set_row_paths(ModelRc::new(VecModel::from(row_paths)));
    let row_details = filtered_rows
        .iter()
        .map(|(_, row)| SharedString::from(row.detail.clone()))
        .collect::<Vec<_>>();
    window.set_row_details(ModelRc::new(VecModel::from(row_details)));
    let row_source_indices = filtered_rows
        .iter()
        .map(|(index, _)| *index as i32)
        .collect::<Vec<_>>();
    window.set_row_source_indices(ModelRc::new(VecModel::from(row_source_indices)));
    let row_can_load_diff = filtered_rows
        .iter()
        .map(|(_, row)| row.can_load_diff)
        .collect::<Vec<_>>();
    window.set_row_can_load_diff(ModelRc::new(VecModel::from(row_can_load_diff)));

    let diff_rows = state.diff_viewer_rows();
    let diff_row_kinds = diff_rows
        .iter()
        .map(|row| SharedString::from(row.row_kind.clone()))
        .collect::<Vec<_>>();
    window.set_diff_row_kinds(ModelRc::new(VecModel::from(diff_row_kinds)));
    let diff_old_line_nos = diff_rows
        .iter()
        .map(|row| SharedString::from(row.old_line_no.clone()))
        .collect::<Vec<_>>();
    window.set_diff_old_line_nos(ModelRc::new(VecModel::from(diff_old_line_nos)));
    let diff_new_line_nos = diff_rows
        .iter()
        .map(|row| SharedString::from(row.new_line_no.clone()))
        .collect::<Vec<_>>();
    window.set_diff_new_line_nos(ModelRc::new(VecModel::from(diff_new_line_nos)));
    let diff_markers = diff_rows
        .iter()
        .map(|row| SharedString::from(row.marker.clone()))
        .collect::<Vec<_>>();
    window.set_diff_markers(ModelRc::new(VecModel::from(diff_markers)));
    let diff_contents = diff_rows
        .into_iter()
        .map(|row| SharedString::from(row.content))
        .collect::<Vec<_>>();
    window.set_diff_contents(ModelRc::new(VecModel::from(diff_contents)));
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
        row_bridge.dispatch(UiCommand::LoadSelectedDiff);
        let state = row_bridge.snapshot();
        sync_window_state(&window, &state);
    });

    let app_weak = app.as_weak();
    let filter_bridge = bridge.clone();
    app.on_filter_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        filter_bridge.dispatch(UiCommand::UpdateEntryFilter(value.to_string()));
        let state = filter_bridge.snapshot();
        sync_window_state(&window, &state);
    });

    app.run().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    Ok(())
}
