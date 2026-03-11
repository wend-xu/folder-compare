//! Slint app for compare + detailed diff with non-blocking and safe UI sync behavior.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::folder_picker;
use crate::presenter::Presenter;
use crate::state::AppState;
use fc_ai::AiProviderKind;
use slint::{ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

slint::slint! {
    import { LineEdit, ListView, ScrollView } from "std-widgets.slint";

    component SectionCard inherits Rectangle {
        border-width: 1px;
        border-color: #e3e8ef;
        border-radius: 6px;
        background: #fcfdff;
        clip: true;
    }

    component ToolButton inherits Rectangle {
        in property <string> label;
        in property <bool> primary: false;
        in property <bool> active: false;
        in property <bool> enabled: true;
        in property <length> button_min_width: 72px;
        in property <length> control_height: 30px;
        callback tapped();

        min-width: self.button_min_width;
        height: self.control_height;
        border-width: 1px;
        border-radius: 4px;
        opacity: self.enabled ? 1 : 0.58;
        border-color: self.primary
            ? #2f69ad
            : (self.active ? #c7d3e4 : #d4dbe5);
        background: self.primary
            ? #3a74ba
            : (self.active ? #edf2f8 : #f8f9fb);

        Text {
            text: root.label;
            color: root.primary ? #ffffff : (root.active ? #27476b : #384555);
            horizontal-alignment: center;
            vertical-alignment: center;
            font-size: 14px;
        }

        TouchArea {
            enabled: root.enabled;
            clicked => {
                root.tapped();
            }
        }
    }

    component SegmentedRail inherits Rectangle {
        border-width: 1px;
        border-color: #d7dde7;
        border-radius: 5px;
        background: #f6f8fb;
        clip: true;
    }

    component SegmentItem inherits Rectangle {
        in property <string> label;
        in property <bool> selected: false;
        in property <bool> enabled: true;
        in property <bool> show_divider: false;
        callback tapped();

        horizontal-stretch: 1;
        background: self.selected ? #ebf0f7 : transparent;
        opacity: self.enabled ? 1 : 0.6;

        Rectangle {
            visible: root.show_divider;
            x: 0px;
            y: 5px;
            width: 1px;
            height: parent.height - 10px;
            background: #dde3ec;
        }

        Text {
            text: root.label;
            color: root.selected ? #294866 : #506176;
            horizontal-alignment: center;
            vertical-alignment: center;
        }

        TouchArea {
            enabled: root.enabled;
            clicked => {
                root.tapped();
            }
        }
    }

    component StatusPill inherits Rectangle {
        in property <string> label;
        in property <string> tone: "neutral";

        height: 17px;
        min-width: 48px;
        border-radius: 6px;
        border-width: 1px;
        border-color: root.tone == "ok"
            ? #b6cab9
            : (root.tone == "warn"
                ? #d4c3a7
                : (root.tone == "error"
                    ? #d8b2b2
                    : #d6dce5));
        background: root.tone == "ok"
            ? #f3f9f4
            : (root.tone == "warn"
                ? #fdf7ed
                : (root.tone == "error"
                    ? #fdf1f1
                    : #f4f6f9));

        Text {
            text: root.label;
            horizontal-alignment: center;
            vertical-alignment: center;
            color: root.tone == "error" ? #7f2d2d : #516274;
            font-size: 11px;
        }
    }

    component TextAction inherits Rectangle {
        in property <string> label;
        in property <bool> enabled: true;
        callback tapped();

        height: 20px;
        background: transparent;
        opacity: root.enabled ? 1 : 0.55;

        Text {
            text: root.label;
            color: #6d7b8b;
            vertical-alignment: center;
        }

        TouchArea {
            enabled: root.enabled;
            clicked => {
                root.tapped();
            }
        }
    }

    export component MainWindow inherits Window {
        title: "Folder Compare";
        preferred-width: 1200px;
        preferred-height: 860px;
        min-width: 900px;
        min-height: 620px;
        background: #f2f4f7;
        in property <length> sidebar_form_label_width: 52px;
        in property <length> sidebar_action_button_width: 72px;

        in-out property <string> left_root;
        in-out property <string> right_root;
        in property <bool> running;
        in property <string> status_text;
        in property <string> summary_text;
        in property <string> compact_summary_text;
        in property <string> compare_metrics_text;
        in property <string> warnings_text;
        in property <string> error_text;
        in property <bool> compare_truncated;
        in property <bool> compare_has_deferred;
        in property <bool> compare_has_oversized;
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
        in property <string> analysis_timeout_text;
        in property <string> provider_settings_error_text;
        in-out property <int> workspace_tab: 0;
        in-out property <bool> compare_warnings_expanded: false;
        in-out property <bool> provider_settings_open: false;
        in-out property <int> provider_settings_mode: 0;
        in-out property <string> provider_settings_endpoint;
        in-out property <string> provider_settings_api_key;
        in-out property <string> provider_settings_model;
        in-out property <string> provider_settings_timeout;
        in-out property <bool> provider_settings_show_api_key: false;
        in-out property <int> selected_row: -1;

        callback compare_clicked();
        callback left_browse_clicked();
        callback right_browse_clicked();
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
        callback provider_settings_save_clicked();
        callback provider_settings_cancel_clicked();

        VerticalLayout {
            padding: 10px;
            spacing: 8px;

            SectionCard {
                height: 36px;
                border-color: #e5e9ef;
                background: #f7f9fc;
                HorizontalLayout {
                    padding: 5px;
                    spacing: 8px;
                    Text {
                        text: "Folder Compare";
                        font-size: 15px;
                        color: #334252;
                        vertical-alignment: center;
                    }
                    Rectangle {
                        horizontal-stretch: 1;
                    }
                    ToolButton {
                        label: "Provider Settings";
                        button_min_width: 132px;
                        control_height: 26px;
                        tapped => {
                            root.provider_settings_mode = root.analysis_remote_mode ? 1 : 0;
                            root.provider_settings_endpoint = root.analysis_endpoint;
                            root.provider_settings_api_key = root.analysis_api_key;
                            root.provider_settings_model = root.analysis_model;
                            root.provider_settings_timeout = root.analysis_timeout_text;
                            root.provider_settings_show_api_key = false;
                            root.provider_settings_open = true;
                            root.provider_settings_clicked();
                        }
                    }
                }
            }

            HorizontalLayout {
                vertical-stretch: 1;
                spacing: 10px;

                Rectangle {
                    horizontal-stretch: 0;
                    min-width: 360px;
                    max-width: 360px;
                    VerticalLayout {
                        spacing: 8px;

                        SectionCard {
                            height: 142px;
                            VerticalLayout {
                                padding: 9px;
                                spacing: 6px;
                                Text {
                                    text: "Compare Inputs";
                                    color: #3b4a5b;
                                    font-size: 15px;
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Left";
                                        width: root.sidebar_form_label_width;
                                        color: #5d6d7e;
                                        vertical-alignment: center;
                                    }
                                    LineEdit {
                                        text <=> root.left_root;
                                        enabled: !root.running;
                                        horizontal-stretch: 1;
                                    }
                                    ToolButton {
                                        label: "Browse";
                                        button_min_width: root.sidebar_action_button_width;
                                        control_height: 28px;
                                        enabled: !root.running;
                                        tapped => {
                                            root.left_browse_clicked();
                                        }
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Right";
                                        width: root.sidebar_form_label_width;
                                        color: #5d6d7e;
                                        vertical-alignment: center;
                                    }
                                    LineEdit {
                                        text <=> root.right_root;
                                        enabled: !root.running;
                                        horizontal-stretch: 1;
                                    }
                                    ToolButton {
                                        label: "Browse";
                                        button_min_width: root.sidebar_action_button_width;
                                        control_height: 28px;
                                        enabled: !root.running;
                                        tapped => {
                                            root.right_browse_clicked();
                                        }
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 8px;
                                    ToolButton {
                                        label: root.running ? "Comparing..." : "Compare";
                                        primary: true;
                                        button_min_width: 120px;
                                        control_height: 31px;
                                        enabled: !root.running && root.left_root != "" && root.right_root != "";
                                        tapped => {
                                            root.compare_clicked();
                                        }
                                    }
                                    Text {
                                        text: root.running
                                            ? "Running compare..."
                                            : (root.left_root == "" || root.right_root == ""
                                                ? "Select left and right folders."
                                                : "Ready");
                                        color: #6c7a89;
                                        overflow: elide;
                                        vertical-alignment: center;
                                    }
                                }
                            }
                        }

                        SectionCard {
                            height: root.compare_warnings_expanded && (root.summary_text != "" || root.warnings_text != "" || root.error_text != "") ? 126px : 86px;
                            VerticalLayout {
                                padding: 9px;
                                spacing: 4px;
                                Text {
                                    text: "Compare Status";
                                    color: #3b4a5b;
                                    font-size: 15px;
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: root.status_text;
                                        color: #4a5f74;
                                        overflow: elide;
                                        vertical-alignment: center;
                                    }
                                    StatusPill {
                                        visible: root.compare_truncated;
                                        label: "truncated";
                                        tone: "warn";
                                    }
                                    StatusPill {
                                        visible: root.warnings_text != "";
                                        label: "warning";
                                        tone: "warn";
                                    }
                                    StatusPill {
                                        visible: root.error_text != "";
                                        label: "error";
                                        tone: "error";
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    TextAction {
                                        visible: root.summary_text != "" || root.warnings_text != "" || root.error_text != "";
                                        label: root.compare_warnings_expanded ? "Hide details" : "Details";
                                        tapped => {
                                            root.compare_warnings_expanded = !root.compare_warnings_expanded;
                                        }
                                    }
                                }
                                Text {
                                    text: root.compare_metrics_text
                                        + (root.compare_has_deferred ? " | deferred" : "")
                                        + (root.compare_has_oversized ? " | oversized" : "");
                                    color: #5a6a7b;
                                    overflow: elide;
                                }
                                Rectangle {
                                    visible: root.compare_warnings_expanded && (root.summary_text != "" || root.warnings_text != "" || root.error_text != "");
                                    height: 36px;
                                    border-width: 0px;
                                    background: #f6f8fb;
                                    clip: true;
                                    VerticalLayout {
                                        padding: 5px;
                                        spacing: 2px;
                                        Text {
                                            visible: root.summary_text != "";
                                            text: root.compact_summary_text;
                                            color: #6b7888;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.error_text != "";
                                            text: root.error_text;
                                            color: #8b3a3a;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.warnings_text != "";
                                            text: root.warnings_text;
                                            color: #86633a;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                    }
                                }
                            }
                        }

                        SectionCard {
                            height: 108px;
                            VerticalLayout {
                                padding: 10px;
                                spacing: 6px;
                                Text {
                                    text: "Filter / Scope";
                                    color: #374656;
                                    font-size: 15px;
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Search";
                                        width: root.sidebar_form_label_width;
                                        color: #5d6d7e;
                                        vertical-alignment: center;
                                    }
                                    LineEdit {
                                        text <=> root.entry_filter;
                                        horizontal-stretch: 1;
                                        enabled: !root.running;
                                        placeholder-text: "path or detail";
                                        edited(value) => {
                                            root.filter_changed(value);
                                        }
                                    }
                                    ToolButton {
                                        label: "Clear";
                                        button_min_width: root.sidebar_action_button_width;
                                        control_height: 28px;
                                        enabled: root.entry_filter != "";
                                        tapped => {
                                            root.entry_filter = "";
                                            root.filter_changed("");
                                        }
                                    }
                                }
                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Status";
                                        width: root.sidebar_form_label_width;
                                        color: #5d6d7e;
                                        vertical-alignment: center;
                                    }
                                    SegmentedRail {
                                        height: 28px;
                                        horizontal-stretch: 1;
                                        HorizontalLayout {
                                            spacing: 0px;
                                            SegmentItem {
                                                label: "All";
                                                selected: root.entry_status_filter == "all";
                                                show_divider: false;
                                                enabled: root.entry_status_filter != "all";
                                                tapped => {
                                                    root.entry_status_filter = "all";
                                                    root.status_filter_changed("all");
                                                }
                                            }
                                            SegmentItem {
                                                label: "Diff";
                                                selected: root.entry_status_filter == "different";
                                                show_divider: true;
                                                enabled: root.entry_status_filter != "different";
                                                tapped => {
                                                    root.entry_status_filter = "different";
                                                    root.status_filter_changed("different");
                                                }
                                            }
                                            SegmentItem {
                                                label: "Equal";
                                                selected: root.entry_status_filter == "equal";
                                                show_divider: true;
                                                enabled: root.entry_status_filter != "equal";
                                                tapped => {
                                                    root.entry_status_filter = "equal";
                                                    root.status_filter_changed("equal");
                                                }
                                            }
                                            SegmentItem {
                                                label: "Left";
                                                selected: root.entry_status_filter == "left-only";
                                                show_divider: true;
                                                enabled: root.entry_status_filter != "left-only";
                                                tapped => {
                                                    root.entry_status_filter = "left-only";
                                                    root.status_filter_changed("left-only");
                                                }
                                            }
                                            SegmentItem {
                                                label: "Right";
                                                selected: root.entry_status_filter == "right-only";
                                                show_divider: true;
                                                enabled: root.entry_status_filter != "right-only";
                                                tapped => {
                                                    root.entry_status_filter = "right-only";
                                                    root.status_filter_changed("right-only");
                                                }
                                            }
                                        }
                                    }
                                    Text {
                                        text: "scope: "
                                            + (root.entry_status_filter == "all"
                                                ? "All"
                                                : (root.entry_status_filter == "different"
                                                    ? "Diff"
                                                    : (root.entry_status_filter == "equal"
                                                        ? "Equal"
                                                        : (root.entry_status_filter == "left-only"
                                                            ? "Left"
                                                            : "Right"))));
                                        width: 84px;
                                        color: #6f7e8d;
                                        vertical-alignment: center;
                                        horizontal-alignment: right;
                                    }
                                }
                            }
                        }

                        SectionCard {
                            vertical-stretch: 1;
                            VerticalLayout {
                                padding: 10px;
                                spacing: 6px;
                                Text {
                                    text: "Results / Navigator";
                                    color: #374656;
                                    font-size: 15px;
                                }
                                Text {
                                    text: root.filter_stats_text;
                                    color: #6f7e8d;
                                    overflow: elide;
                                }
                                ListView {
                                    vertical-stretch: 1;
                                    for row_path[index] in root.row_paths: Rectangle {
                                        height: 46px;
                                        border-width: 1px;
                                        border-color: root.row_source_indices[index] == root.selected_row ? #9ab1cd : #e1e6ee;
                                        border-radius: 4px;
                                        background: root.row_source_indices[index] == root.selected_row ? #eaf1fb
                                            : (root.row_can_load_diff[index] ? #fbfcfe : #f3f5f8);

                                        VerticalLayout {
                                            padding: 4px;
                                            spacing: 2px;
                                            HorizontalLayout {
                                                spacing: 7px;
                                                StatusPill {
                                                    label: root.row_statuses[index] == "different"
                                                        ? "diff"
                                                        : (root.row_statuses[index] == "equal"
                                                            ? "equal"
                                                            : (root.row_statuses[index] == "left-only"
                                                                ? "left"
                                                                : (root.row_statuses[index] == "right-only"
                                                                    ? "right"
                                                                    : root.row_statuses[index])));
                                                    tone: root.row_statuses[index] == "equal"
                                                        ? "ok"
                                                        : ((root.row_statuses[index] == "left-only" || root.row_statuses[index] == "right-only")
                                                            ? "warn"
                                                            : "neutral");
                                                }
                                                Text {
                                                    text: row_path;
                                                    color: root.row_source_indices[index] == root.selected_row ? #1e466d : #2f3f50;
                                                    vertical-alignment: center;
                                                    horizontal-stretch: 1;
                                                    overflow: elide;
                                                }
                                            }
                                            Text {
                                                text: root.row_can_load_diff[index] ? root.row_details[index] : "detailed diff unavailable";
                                                color: root.row_can_load_diff[index] ? #6d7986 : #7a8692;
                                                vertical-alignment: center;
                                                horizontal-stretch: 1;
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
                    horizontal-stretch: 1;
                    min-width: 500px;
                    VerticalLayout {
                        padding: 10px;
                        spacing: 8px;

                        SectionCard {
                            height: 36px;
                            background: #f8fafd;
                            HorizontalLayout {
                                padding: 4px;
                                spacing: 8px;
                                SegmentedRail {
                                    width: 214px;
                                    height: 28px;
                                    HorizontalLayout {
                                        spacing: 0px;
                                        SegmentItem {
                                            label: "Diff";
                                            selected: root.workspace_tab == 0;
                                            show_divider: false;
                                            tapped => {
                                                root.workspace_tab = 0;
                                            }
                                        }
                                        SegmentItem {
                                            label: "Analysis";
                                            selected: root.workspace_tab == 1;
                                            show_divider: true;
                                            tapped => {
                                                root.workspace_tab = 1;
                                            }
                                        }
                                    }
                                }
                                Rectangle {
                                    horizontal-stretch: 1;
                                }
                            }
                        }

                        SectionCard {
                            height: 58px;
                            VerticalLayout {
                                padding: 9px;
                                spacing: 2px;
                                Text {
                                    text: root.workspace_tab == 0 ? "Diff Mode" : "Analysis Mode";
                                    color: #425161;
                                }
                                Text {
                                    visible: root.workspace_tab == 0;
                                    text: root.selected_relative_path == "" ? "(none selected)" : root.selected_relative_path;
                                    color: #294562;
                                    wrap: word-wrap;
                                    horizontal-stretch: 1;
                                    overflow: elide;
                                }
                                Text {
                                    visible: root.workspace_tab == 1;
                                    text: "Provider: " + root.analysis_provider_mode_text
                                        + (root.analysis_remote_mode ? (root.analysis_remote_config_ready ? " (ready)" : " (config incomplete)") : " (local)");
                                    color: #294562;
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
                                padding: 10px;
                                spacing: 6px;

                                Text {
                                    text: root.diff_summary_text == "" ? "Select one result row to load detailed diff." : root.diff_summary_text;
                                    color: #425364;
                                    wrap: word-wrap;
                                    horizontal-stretch: 1;
                                }
                                HorizontalLayout {
                                    spacing: 10px;
                                    Text {
                                        visible: root.diff_loading;
                                        text: "Loading detailed diff...";
                                        color: #36516f;
                                    }
                                    Text {
                                        visible: root.diff_warning_text != "";
                                        text: root.diff_warning_text;
                                        color: #805520;
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
                                        color: #8a2424;
                                        overflow: elide;
                                    }
                                }

                                Rectangle {
                                    border-width: 1px;
                                    border-color: #dce3ec;
                                    border-radius: 4px;
                                    background: #f8fafd;
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
                                            : (root.diff_row_kinds[index] == "added" ? rgb(239, 251, 242)
                                            : (root.diff_row_kinds[index] == "removed" ? rgb(252, 240, 240) : rgb(251, 252, 253)));

                                        Text {
                                            visible: root.diff_row_kinds[index] == "hunk";
                                            text: row_content;
                                            color: #2f4f70;
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
                                                color: #273544;
                                                vertical-alignment: center;
                                            }
                                        }
                                    }
                                }
                            }

                            VerticalLayout {
                                visible: root.workspace_tab == 1;
                                padding: 10px;
                                spacing: 6px;

                                HorizontalLayout {
                                    spacing: 8px;
                                    Text {
                                        text: "AI Analysis";
                                        color: #425161;
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    ToolButton {
                                        label: root.analysis_loading ? "Analyzing..." : "Analyze";
                                        primary: true;
                                        button_min_width: 108px;
                                        control_height: 30px;
                                        enabled: !root.analysis_loading && root.analysis_available && !root.diff_loading && !root.running
                                            && (!root.analysis_remote_mode || root.analysis_remote_config_ready);
                                        tapped => {
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
                                    Text {
                                        text: root.analysis_provider_mode_text;
                                        color: #1f3e58;
                                    }
                                    Text {
                                        text: "Timeout: " + root.analysis_timeout_text + "s";
                                        color: #5f6d7c;
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    Text {
                                        text: "Use Provider Settings in App Bar to edit.";
                                        color: #7a4b00;
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

        Rectangle {
            visible: root.provider_settings_open;
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            background: rgba(17, 24, 34, 0.24);

            TouchArea {}

            SectionCard {
                width: 700px;
                height: root.provider_settings_mode == 1 ? 430px : 338px;
                x: (parent.width - self.width) / 2;
                y: 70px;
                border-color: #dfe5ed;
                background: #fcfdff;

                VerticalLayout {
                    padding: 14px;
                    spacing: 8px;

                    Text {
                        text: "Provider Settings";
                        color: #2f4966;
                        font-size: 18px;
                    }
                    Text {
                        text: "Global configuration for AI analysis provider.";
                        color: #6a7888;
                    }

                    Rectangle {
                        height: 1px;
                        background: #e7ecf3;
                    }

                    HorizontalLayout {
                        spacing: 6px;
                        Text {
                            text: "Mode";
                            width: 104px;
                            color: #4f6074;
                            vertical-alignment: center;
                        }
                        SegmentedRail {
                            height: 30px;
                            HorizontalLayout {
                                spacing: 0px;
                                SegmentItem {
                                    label: "Mock";
                                    selected: root.provider_settings_mode == 0;
                                    show_divider: false;
                                    tapped => {
                                        root.provider_settings_mode = 0;
                                    }
                                }
                                SegmentItem {
                                    label: "OpenAI-compatible";
                                    selected: root.provider_settings_mode == 1;
                                    show_divider: true;
                                    tapped => {
                                        root.provider_settings_mode = 1;
                                    }
                                }
                            }
                        }
                    }

                    HorizontalLayout {
                        spacing: 6px;
                        Text {
                            text: "Timeout";
                            width: 104px;
                            color: #4f6074;
                            vertical-alignment: center;
                        }
                        LineEdit {
                            text <=> root.provider_settings_timeout;
                            width: 140px;
                            height: 28px;
                        }
                        Text {
                            text: "seconds";
                            color: #778595;
                            vertical-alignment: center;
                        }
                        Rectangle {
                            horizontal-stretch: 1;
                        }
                    }

                    VerticalLayout {
                        visible: root.provider_settings_mode == 1;
                        spacing: 6px;
                        HorizontalLayout {
                            spacing: 6px;
                            Text {
                                text: "Endpoint";
                                width: 104px;
                                color: #4f6074;
                                vertical-alignment: center;
                            }
                            LineEdit {
                                text <=> root.provider_settings_endpoint;
                                horizontal-stretch: 1;
                                height: 28px;
                            }
                        }
                        HorizontalLayout {
                            spacing: 6px;
                            Text {
                                text: "API Key";
                                width: 104px;
                                color: #4f6074;
                                vertical-alignment: center;
                            }
                            LineEdit {
                                text <=> root.provider_settings_api_key;
                                input-type: root.provider_settings_show_api_key ? InputType.text : InputType.password;
                                horizontal-stretch: 1;
                                height: 28px;
                            }
                            ToolButton {
                                label: root.provider_settings_show_api_key ? "Hide" : "Show";
                                button_min_width: 62px;
                                control_height: 27px;
                                tapped => {
                                    root.provider_settings_show_api_key = !root.provider_settings_show_api_key;
                                }
                            }
                        }
                        HorizontalLayout {
                            spacing: 6px;
                            Text {
                                text: "Model";
                                width: 104px;
                                color: #4f6074;
                                vertical-alignment: center;
                            }
                            LineEdit {
                                text <=> root.provider_settings_model;
                                horizontal-stretch: 1;
                                height: 28px;
                            }
                        }
                    }

                    Text {
                        visible: root.provider_settings_error_text != "";
                        text: root.provider_settings_error_text;
                        color: #8c1d1d;
                        wrap: word-wrap;
                        horizontal-stretch: 1;
                    }

                    Rectangle {
                        height: 1px;
                        background: #e7ecf3;
                    }

                    HorizontalLayout {
                        spacing: 8px;
                        Rectangle {
                            horizontal-stretch: 1;
                        }
                        ToolButton {
                            label: "Cancel";
                            button_min_width: 108px;
                            control_height: 30px;
                            tapped => {
                                root.provider_settings_open = false;
                                root.provider_settings_cancel_clicked();
                            }
                        }
                        ToolButton {
                            label: "Save";
                            primary: true;
                            button_min_width: 108px;
                            control_height: 30px;
                            tapped => {
                                root.provider_settings_save_clicked();
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
    }

    window.set_running(state.running);
    window.set_status_text(state.status_text.clone().into());
    window.set_summary_text(state.summary_text.clone().into());
    window.set_compact_summary_text(state.compact_summary_text().into());
    window.set_compare_metrics_text(state.compare_metrics_text().into());
    window.set_warnings_text(state.warnings_text().into());
    window.set_error_text(state.error_message.clone().unwrap_or_default().into());
    window.set_compare_truncated(state.truncated);
    window.set_compare_has_deferred(state.compare_has_deferred());
    window.set_compare_has_oversized(state.compare_has_oversized());
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
    window.set_analysis_endpoint(state.analysis_openai_endpoint.clone().into());
    window.set_analysis_api_key(state.analysis_openai_api_key.clone().into());
    window.set_analysis_model(state.analysis_openai_model.clone().into());
    window.set_analysis_timeout_text(state.analysis_timeout_text().into());
    window.set_provider_settings_error_text(state.provider_settings_error_text().into());
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
    app.on_left_browse_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_running() {
            return;
        }
        let Some(path) = folder_picker::pick_folder() else {
            return;
        };
        window.set_left_root(path.to_string_lossy().to_string().into());
    });

    let app_weak = app.as_weak();
    app.on_right_browse_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_running() {
            return;
        }
        let Some(path) = folder_picker::pick_folder() else {
            return;
        };
        window.set_right_root(path.to_string_lossy().to_string().into());
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
    let provider_settings_bridge = bridge.clone();
    let provider_settings_cache = Arc::clone(&sync_cache);
    app.on_provider_settings_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_settings_bridge.dispatch(UiCommand::ClearProviderSettingsError);
        sync_window_state_if_changed(
            &window,
            &provider_settings_bridge,
            &provider_settings_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let provider_settings_cancel_bridge = bridge.clone();
    let provider_settings_cancel_cache = Arc::clone(&sync_cache);
    app.on_provider_settings_cancel_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_settings_cancel_bridge.dispatch(UiCommand::ClearProviderSettingsError);
        sync_window_state_if_changed(
            &window,
            &provider_settings_cancel_bridge,
            &provider_settings_cancel_cache,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let provider_settings_save_bridge = bridge.clone();
    let provider_settings_save_cache = Arc::clone(&sync_cache);
    app.on_provider_settings_save_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        let provider_kind = if window.get_provider_settings_mode() == 1 {
            AiProviderKind::OpenAiCompatible
        } else {
            AiProviderKind::Mock
        };
        provider_settings_save_bridge.dispatch(UiCommand::SaveProviderSettings {
            provider_kind,
            endpoint: window.get_provider_settings_endpoint().to_string(),
            api_key: window.get_provider_settings_api_key().to_string(),
            model: window.get_provider_settings_model().to_string(),
            timeout_secs_text: window.get_provider_settings_timeout().to_string(),
        });
        sync_window_state_if_changed(
            &window,
            &provider_settings_save_bridge,
            &provider_settings_save_cache,
            SyncMode::Passive,
        );
        if window.get_provider_settings_error_text().is_empty() {
            window.set_provider_settings_open(false);
        }
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
