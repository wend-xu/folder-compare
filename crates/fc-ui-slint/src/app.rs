//! Slint app for compare + detailed diff with non-blocking and safe UI sync behavior.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::presenter::Presenter;
use crate::state::AppState;
use slint::{ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

slint::slint! {
    import { Button, LineEdit, ListView, ScrollView } from "std-widgets.slint";

    component SectionCard inherits Rectangle {
        border-width: 1px;
        border-color: #e2e7ed;
        border-radius: 6px;
        background: #ffffff;
        clip: true;
    }

    export component MainWindow inherits Window {
        title: "Folder Compare";
        preferred-width: 1200px;
        preferred-height: 860px;
        min-width: 900px;
        min-height: 620px;

        in-out property <string> left_root;
        in-out property <string> right_root;
        in property <bool> running;
        in property <string> status_text;
        in property <string> summary_text;
        in property <string> warnings_text;
        in property <string> error_text;
        in property <bool> compare_truncated;
        in-out property <string> entry_filter;
        in-out property <string> entry_status_filter;
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
        in property <bool> analysis_loading;
        in property <bool> analysis_available;
        in property <string> analysis_hint_text;
        in property <string> analysis_error_text;
        in property <string> analysis_title_text;
        in property <string> analysis_risk_level_text;
        in property <string> analysis_rationale_text;
        in property <string> analysis_key_points_text;
        in property <string> analysis_review_suggestions_text;
        in property <string> analysis_provider_mode_text;
        in property <bool> analysis_remote_mode;
        in property <bool> analysis_remote_config_ready;
        in-out property <string> analysis_endpoint;
        in-out property <string> analysis_api_key;
        in-out property <string> analysis_model;
        in-out property <int> workspace_tab: 0;
        in-out property <bool> compare_warnings_expanded: false;
        in-out property <int> selected_row: -1;

        callback compare_clicked();
        callback filter_changed(string);
        callback status_filter_changed(string);
        callback row_selected(int);
        callback analyze_clicked();
        callback analysis_provider_mock_selected();
        callback analysis_provider_openai_selected();
        callback analysis_endpoint_changed(string);
        callback analysis_api_key_changed(string);
        callback analysis_model_changed(string);
        callback provider_settings_clicked();

        VerticalLayout {
            padding: 10px;
            spacing: 8px;

            SectionCard {
                height: 40px;
                border-color: #e6ebf1;
                background: #f8fafc;
                HorizontalLayout {
                    padding: 8px;
                    spacing: 8px;
                    Text {
                        text: "Folder Compare";
                        font-size: 18px;
                        color: #1f2d3d;
                        vertical-alignment: center;
                    }
                    Rectangle {
                        horizontal-stretch: 1;
                    }
                    Button {
                        text: "Provider Settings";
                        clicked => {
                            root.workspace_tab = 1;
                            root.provider_settings_clicked();
                        }
                    }
                }
            }

            HorizontalLayout {
                vertical-stretch: 1;
                spacing: 10px;

                Rectangle {
                    horizontal-stretch: 3;
                    min-width: 320px;
                    VerticalLayout {
                        spacing: 8px;

                        SectionCard {
                            height: 142px;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 6px;
                                Text {
                                    text: "Compare Inputs";
                                    color: #4c5b6b;
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Left";
                                        width: 42px;
                                        color: #5f6d7c;
                                    }
                                    LineEdit {
                                        text <=> root.left_root;
                                        enabled: !root.running;
                                    }
                                    Button {
                                        text: "Browse";
                                        enabled: false;
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Right";
                                        width: 42px;
                                        color: #5f6d7c;
                                    }
                                    LineEdit {
                                        text <=> root.right_root;
                                        enabled: !root.running;
                                    }
                                    Button {
                                        text: "Browse";
                                        enabled: false;
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
                                        text: root.running ? "Running compare..." : "Ready";
                                        color: #6a7581;
                                    }
                                }
                            }
                        }

                        SectionCard {
                            height: root.compare_warnings_expanded && root.warnings_text != "" ? 136px : 88px;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 3px;
                                Text {
                                    text: "Compare Status";
                                    color: #4c5b6b;
                                }
                                Text {
                                    text: root.status_text;
                                    color: #2f4f70;
                                    overflow: elide;
                                }
                                Text {
                                    text: root.summary_text == "" ? "No compare summary yet." : root.summary_text;
                                    color: #2f3b46;
                                    wrap: word-wrap;
                                    height: 30px;
                                    horizontal-stretch: 1;
                                    overflow: elide;
                                }
                                Text {
                                    visible: root.compare_truncated;
                                    text: "Compare result is truncated.";
                                    color: #7a4b00;
                                    overflow: elide;
                                }
                                Text {
                                    visible: root.error_text != "";
                                    text: root.error_text;
                                    color: #8c1d1d;
                                    wrap: word-wrap;
                                    height: 20px;
                                    overflow: elide;
                                }
                                HorizontalLayout {
                                    visible: root.warnings_text != "";
                                    spacing: 6px;
                                    Text {
                                        text: "Warnings";
                                        color: #7a4b00;
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    Button {
                                        text: root.compare_warnings_expanded ? "Hide" : "Show";
                                        clicked => {
                                            root.compare_warnings_expanded = !root.compare_warnings_expanded;
                                        }
                                    }
                                }
                                Rectangle {
                                    visible: root.compare_warnings_expanded && root.warnings_text != "";
                                    height: 44px;
                                    border-width: 1px;
                                    border-color: #f0d8ac;
                                    clip: true;
                                    ScrollView {
                                        Text {
                                            text: root.warnings_text;
                                            wrap: word-wrap;
                                            color: #7a4b00;
                                            horizontal-stretch: 1;
                                        }
                                    }
                                }
                            }
                        }

                        SectionCard {
                            height: 102px;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 6px;
                                Text {
                                    text: "Filter / Scope";
                                    color: #4c5b6b;
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Search";
                                        width: 48px;
                                        color: #5f6d7c;
                                    }
                                    LineEdit {
                                        text <=> root.entry_filter;
                                        enabled: !root.running;
                                        edited(value) => {
                                            root.filter_changed(value);
                                        }
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Status";
                                        width: 48px;
                                        color: #5f6d7c;
                                    }
                                    Button {
                                        text: "All";
                                        enabled: root.entry_status_filter != "all";
                                        clicked => {
                                            root.status_filter_changed("all");
                                        }
                                    }
                                    Button {
                                        text: "Different";
                                        enabled: root.entry_status_filter != "different";
                                        clicked => {
                                            root.status_filter_changed("different");
                                        }
                                    }
                                    Button {
                                        text: "Equal";
                                        enabled: root.entry_status_filter != "equal";
                                        clicked => {
                                            root.status_filter_changed("equal");
                                        }
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Rectangle {
                                        width: 48px;
                                    }
                                    Button {
                                        text: "Left-only";
                                        enabled: root.entry_status_filter != "left-only";
                                        clicked => {
                                            root.status_filter_changed("left-only");
                                        }
                                    }
                                    Button {
                                        text: "Right-only";
                                        enabled: root.entry_status_filter != "right-only";
                                        clicked => {
                                            root.status_filter_changed("right-only");
                                        }
                                    }
                                }
                            }
                        }

                        SectionCard {
                            vertical-stretch: 1;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 6px;
                                Text {
                                    text: "Results / Navigator";
                                    color: #4c5b6b;
                                }
                                Text {
                                    text: root.filter_stats_text;
                                    color: #6a7581;
                                    overflow: elide;
                                }
                                ListView {
                                    vertical-stretch: 1;
                                    for row_path[index] in root.row_paths: Rectangle {
                                        height: 50px;
                                        border-width: 1px;
                                        border-color: root.row_source_indices[index] == root.selected_row ? rgb(129, 176, 222) : rgb(234, 238, 243);
                                        background: root.row_source_indices[index] == root.selected_row ? rgb(236, 245, 255)
                                            : (root.row_can_load_diff[index] ? rgb(255, 255, 255) : rgb(248, 249, 251));

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
                                                    overflow: elide;
                                                }
                                            }
                                            Text {
                                                text: root.row_can_load_diff[index] ? root.row_details[index] : root.row_details[index] + " | detailed diff unavailable";
                                                color: root.row_can_load_diff[index] ? #555 : #777;
                                                vertical-alignment: center;
                                                overflow: elide;
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
                        }
                    }
                }

                SectionCard {
                    horizontal-stretch: 7;
                    VerticalLayout {
                        padding: 8px;
                        spacing: 8px;

                        SectionCard {
                            height: 40px;
                            background: #fbfcfd;
                            HorizontalLayout {
                                padding: 6px;
                                spacing: 6px;
                                Text {
                                    text: "Tabs";
                                    width: 36px;
                                    color: #6a7581;
                                    vertical-alignment: center;
                                }
                                Button {
                                    text: "Diff";
                                    enabled: root.workspace_tab != 0;
                                    clicked => {
                                        root.workspace_tab = 0;
                                    }
                                }
                                Button {
                                    text: "Analysis";
                                    enabled: root.workspace_tab != 1;
                                    clicked => {
                                        root.workspace_tab = 1;
                                    }
                                }
                                Rectangle {
                                    horizontal-stretch: 1;
                                }
                            }
                        }

                        SectionCard {
                            height: 60px;
                            VerticalLayout {
                                padding: 8px;
                                spacing: 2px;
                                Text {
                                    text: root.workspace_tab == 0 ? "Diff Mode" : "Analysis Mode";
                                    color: #4c5b6b;
                                }
                                Text {
                                    visible: root.workspace_tab == 0;
                                    text: root.selected_relative_path == "" ? "(none selected)" : root.selected_relative_path;
                                    color: #1f3e58;
                                    wrap: word-wrap;
                                    horizontal-stretch: 1;
                                    overflow: elide;
                                }
                                Text {
                                    visible: root.workspace_tab == 1;
                                    text: "Provider: " + root.analysis_provider_mode_text
                                        + (root.analysis_remote_mode ? (root.analysis_remote_config_ready ? " (ready)" : " (config incomplete)") : " (local)");
                                    color: #1f3e58;
                                    wrap: word-wrap;
                                    horizontal-stretch: 1;
                                    overflow: elide;
                                }
                            }
                        }

                        SectionCard {
                            vertical-stretch: 1;

                            VerticalLayout {
                                visible: root.workspace_tab == 0;
                                padding: 8px;
                                spacing: 6px;

                                Text {
                                    text: root.diff_summary_text == "" ? "Select one result row to load detailed diff." : root.diff_summary_text;
                                    color: #2f3b46;
                                    wrap: word-wrap;
                                    horizontal-stretch: 1;
                                }
                                HorizontalLayout {
                                    spacing: 10px;
                                    Text {
                                        visible: root.diff_loading;
                                        text: "Loading detailed diff...";
                                        color: #2f4f70;
                                    }
                                    Text {
                                        visible: root.diff_warning_text != "";
                                        text: root.diff_warning_text;
                                        color: #7a4b00;
                                        overflow: elide;
                                    }
                                    Text {
                                        visible: root.diff_truncated;
                                        text: "Detailed diff is truncated.";
                                        color: #7a4b00;
                                    }
                                    Text {
                                        visible: root.diff_error_text != "";
                                        text: root.diff_error_text;
                                        color: #8c1d1d;
                                        overflow: elide;
                                    }
                                }

                                Rectangle {
                                    border-width: 1px;
                                    border-color: #e5e9ef;
                                    height: 26px;
                                    HorizontalLayout {
                                        spacing: 8px;
                                        Text {
                                            text: "old";
                                            width: 56px;
                                            horizontal-alignment: right;
                                            color: #666;
                                        }
                                        Text {
                                            text: "new";
                                            width: 56px;
                                            horizontal-alignment: right;
                                            color: #666;
                                        }
                                        Text {
                                            text: " ";
                                            width: 20px;
                                        }
                                        Text {
                                            text: "content";
                                            color: #666;
                                        }
                                    }
                                }

                                ListView {
                                    vertical-stretch: 1;
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

                            VerticalLayout {
                                visible: root.workspace_tab == 1;
                                padding: 8px;
                                spacing: 6px;

                                HorizontalLayout {
                                    spacing: 8px;
                                    Text {
                                        text: "AI Analysis";
                                        color: #4c5b6b;
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    Button {
                                        text: root.analysis_loading ? "Analyzing..." : "Analyze";
                                        enabled: !root.analysis_loading && root.analysis_available && !root.diff_loading && !root.running
                                            && (!root.analysis_remote_mode || root.analysis_remote_config_ready);
                                        clicked => {
                                            root.analyze_clicked();
                                        }
                                    }
                                }

                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Provider:";
                                        color: #5f6d7c;
                                    }
                                    Button {
                                        text: "Mock";
                                        enabled: root.analysis_provider_mode_text != "Mock";
                                        clicked => {
                                            root.analysis_provider_mock_selected();
                                        }
                                    }
                                    Button {
                                        text: "OpenAI-compatible";
                                        enabled: root.analysis_provider_mode_text != "OpenAI-compatible";
                                        clicked => {
                                            root.analysis_provider_openai_selected();
                                        }
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    Text {
                                        text: "Global settings entry is in App Bar.";
                                        color: #7a4b00;
                                    }
                                }

                                Rectangle {
                                    visible: root.analysis_remote_mode;
                                    border-width: 1px;
                                    border-color: #e2e7ed;
                                    height: 112px;
                                    clip: true;
                                    VerticalLayout {
                                        padding: 6px;
                                        spacing: 4px;
                                        Text {
                                            text: "Provider Config (temporary in-content editor)";
                                            color: #6a7581;
                                        }
                                        HorizontalLayout {
                                            spacing: 6px;
                                            Text {
                                                text: "Endpoint";
                                                width: 62px;
                                                color: #5f6d7c;
                                            }
                                            LineEdit {
                                                text <=> root.analysis_endpoint;
                                                enabled: !root.analysis_loading;
                                                edited(value) => {
                                                    root.analysis_endpoint_changed(value);
                                                }
                                            }
                                        }
                                        HorizontalLayout {
                                            spacing: 6px;
                                            Text {
                                                text: "API Key";
                                                width: 62px;
                                                color: #5f6d7c;
                                            }
                                            LineEdit {
                                                text <=> root.analysis_api_key;
                                                enabled: !root.analysis_loading;
                                                edited(value) => {
                                                    root.analysis_api_key_changed(value);
                                                }
                                            }
                                        }
                                        HorizontalLayout {
                                            spacing: 6px;
                                            Text {
                                                text: "Model";
                                                width: 62px;
                                                color: #5f6d7c;
                                            }
                                            LineEdit {
                                                text <=> root.analysis_model;
                                                enabled: !root.analysis_loading;
                                                edited(value) => {
                                                    root.analysis_model_changed(value);
                                                }
                                            }
                                        }
                                    }
                                }

                                ScrollView {
                                    vertical-stretch: 1;
                                    VerticalLayout {
                                        spacing: 4px;
                                        Text {
                                            visible: root.analysis_remote_mode;
                                            text: "Remote mode sends diff excerpts to the configured endpoint.";
                                            color: #7a4b00;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_remote_mode && !root.analysis_remote_config_ready;
                                            text: "Remote config incomplete: endpoint, API key and model are required.";
                                            color: #8c1d1d;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_hint_text != "";
                                            text: root.analysis_hint_text;
                                            color: #777;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_loading;
                                            text: "Running AI analysis...";
                                            color: #2f4f70;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_error_text != "";
                                            text: root.analysis_error_text;
                                            color: #8c1d1d;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_title_text != "";
                                            text: root.analysis_title_text;
                                            color: #1f3e58;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_risk_level_text != "";
                                            text: "Risk Level: " + root.analysis_risk_level_text;
                                            color: #445a6a;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_rationale_text != "";
                                            text: root.analysis_rationale_text;
                                            color: #444;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_key_points_text != "";
                                            text: "Key Points:\n" + root.analysis_key_points_text;
                                            color: #444;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.analysis_review_suggestions_text != "";
                                            text: "Review Suggestions:\n" + root.analysis_review_suggestions_text;
                                            color: #444;
                                            wrap: word-wrap;
                                            horizontal-stretch: 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncMode {
    Full,
    Passive,
}

fn should_sync_editable_inputs(mode: SyncMode) -> bool {
    matches!(mode, SyncMode::Full)
}

fn should_skip_sync(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    last_state == Some(next_state)
}

fn sync_window_state(window: &MainWindow, state: &AppState, mode: SyncMode) {
    if should_sync_editable_inputs(mode) {
        window.set_left_root(state.left_root.clone().into());
        window.set_right_root(state.right_root.clone().into());
        window.set_entry_filter(state.entry_filter.clone().into());
        window.set_entry_status_filter(state.entry_status_filter.clone().into());
        window.set_analysis_endpoint(state.analysis_openai_endpoint.clone().into());
        window.set_analysis_api_key(state.analysis_openai_api_key.clone().into());
        window.set_analysis_model(state.analysis_openai_model.clone().into());
    }

    window.set_running(state.running);
    window.set_status_text(state.status_text.clone().into());
    window.set_summary_text(state.summary_text.clone().into());
    window.set_warnings_text(state.warnings_text().into());
    window.set_error_text(state.error_message.clone().unwrap_or_default().into());
    window.set_compare_truncated(state.truncated);
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
    window.set_analysis_loading(state.analysis_loading);
    window.set_analysis_available(state.analysis_available);
    window.set_analysis_hint_text(state.analysis_hint_text().into());
    window.set_analysis_error_text(
        state
            .analysis_error_message
            .clone()
            .unwrap_or_default()
            .into(),
    );
    window.set_analysis_title_text(state.analysis_title_text().into());
    window.set_analysis_risk_level_text(state.analysis_risk_level_text().into());
    window.set_analysis_rationale_text(state.analysis_rationale_text().into());
    window.set_analysis_key_points_text(state.analysis_key_points_text().into());
    window.set_analysis_review_suggestions_text(state.analysis_review_suggestions_text().into());
    window.set_analysis_provider_mode_text(state.analysis_provider_mode_text().into());
    window.set_analysis_remote_mode(state.analysis_remote_mode());
    window.set_analysis_remote_config_ready(state.analysis_remote_config_ready());
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

fn sync_window_state_if_changed(
    window: &MainWindow,
    bridge: &UiBridge,
    cache: &Arc<Mutex<Option<AppState>>>,
    mode: SyncMode,
) {
    let state = bridge.snapshot();
    let mut cache_guard = cache.lock().expect("sync cache mutex poisoned");
    if should_skip_sync(cache_guard.as_ref(), &state) {
        return;
    }
    sync_window_state(window, &state, mode);
    *cache_guard = Some(state);
}

/// Runs the UI application.
pub fn run() -> anyhow::Result<()> {
    let app = MainWindow::new().map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let state = Arc::new(Mutex::new(AppState::default()));
    let presenter = Presenter::new(state);
    let bridge = UiBridge::new(presenter);
    bridge.dispatch(UiCommand::Initialize);
    let initial_state = bridge.snapshot();
    sync_window_state(&app, &initial_state, SyncMode::Full);
    let sync_cache = Arc::new(Mutex::new(Some(initial_state)));

    let ui_refresh_timer = Timer::default();
    let app_weak = app.as_weak();
    let refresh_bridge = bridge.clone();
    let refresh_cache = Arc::clone(&sync_cache);
    ui_refresh_timer.start(TimerMode::Repeated, Duration::from_millis(33), move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        sync_window_state_if_changed(&window, &refresh_bridge, &refresh_cache, SyncMode::Passive);
    });

    let app_weak = app.as_weak();
    let compare_bridge = bridge.clone();
    let compare_cache = Arc::clone(&sync_cache);
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
        sync_window_state_if_changed(&window, &compare_bridge, &compare_cache, SyncMode::Passive);
    });

    let app_weak = app.as_weak();
    let row_bridge = bridge.clone();
    let row_cache = Arc::clone(&sync_cache);
    app.on_row_selected(move |index| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        row_bridge.dispatch(UiCommand::SelectRow(index));
        row_bridge.dispatch(UiCommand::LoadSelectedDiff);
        sync_window_state_if_changed(&window, &row_bridge, &row_cache, SyncMode::Passive);
    });

    let app_weak = app.as_weak();
    let filter_bridge = bridge.clone();
    let filter_cache = Arc::clone(&sync_cache);
    app.on_filter_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        filter_bridge.dispatch(UiCommand::UpdateEntryFilter(value.to_string()));
        sync_window_state_if_changed(&window, &filter_bridge, &filter_cache, SyncMode::Passive);
    });

    let app_weak = app.as_weak();
    let status_filter_bridge = bridge.clone();
    let status_filter_cache = Arc::clone(&sync_cache);
    app.on_status_filter_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        status_filter_bridge.dispatch(UiCommand::UpdateEntryStatusFilter(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &status_filter_bridge,
            &status_filter_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let analysis_bridge = bridge.clone();
    let analysis_cache = Arc::clone(&sync_cache);
    app.on_analyze_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        analysis_bridge.dispatch(UiCommand::UpdateAiEndpoint(
            window.get_analysis_endpoint().to_string(),
        ));
        analysis_bridge.dispatch(UiCommand::UpdateAiApiKey(
            window.get_analysis_api_key().to_string(),
        ));
        analysis_bridge.dispatch(UiCommand::UpdateAiModel(
            window.get_analysis_model().to_string(),
        ));
        analysis_bridge.dispatch(UiCommand::LoadAiAnalysis);
        sync_window_state_if_changed(
            &window,
            &analysis_bridge,
            &analysis_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let provider_bridge = bridge.clone();
    let provider_cache = Arc::clone(&sync_cache);
    app.on_analysis_provider_mock_selected(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_bridge.dispatch(UiCommand::SetAiProviderModeMock);
        sync_window_state_if_changed(
            &window,
            &provider_bridge,
            &provider_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let provider_bridge = bridge.clone();
    let provider_cache = Arc::clone(&sync_cache);
    app.on_analysis_provider_openai_selected(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_bridge.dispatch(UiCommand::SetAiProviderModeOpenAiCompatible);
        sync_window_state_if_changed(
            &window,
            &provider_bridge,
            &provider_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let endpoint_bridge = bridge.clone();
    let endpoint_cache = Arc::clone(&sync_cache);
    app.on_analysis_endpoint_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        endpoint_bridge.dispatch(UiCommand::UpdateAiEndpoint(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &endpoint_bridge,
            &endpoint_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let api_key_bridge = bridge.clone();
    let api_key_cache = Arc::clone(&sync_cache);
    app.on_analysis_api_key_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        api_key_bridge.dispatch(UiCommand::UpdateAiApiKey(value.to_string()));
        sync_window_state_if_changed(&window, &api_key_bridge, &api_key_cache, SyncMode::Passive);
    });

    let app_weak = app.as_weak();
    let model_bridge = bridge.clone();
    let model_cache = Arc::clone(&sync_cache);
    app.on_analysis_model_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        model_bridge.dispatch(UiCommand::UpdateAiModel(value.to_string()));
        sync_window_state_if_changed(&window, &model_bridge, &model_cache, SyncMode::Passive);
    });

    app.run().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passive_mode_does_not_sync_editable_inputs() {
        assert!(!should_sync_editable_inputs(SyncMode::Passive));
    }

    #[test]
    fn full_mode_syncs_editable_inputs() {
        assert!(should_sync_editable_inputs(SyncMode::Full));
    }

    #[test]
    fn unchanged_state_should_skip_sync() {
        let state = AppState::default();
        assert!(should_skip_sync(Some(&state), &state));
    }

    #[test]
    fn changed_state_should_not_skip_sync() {
        let previous = AppState::default();
        let next = AppState {
            running: true,
            ..AppState::default()
        };
        assert!(!should_skip_sync(Some(&previous), &next));
    }
}
