//! Slint app for compare + detailed diff with non-blocking and safe UI sync behavior.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::folder_picker;
use crate::presenter::Presenter;
use crate::state::AppState;
use crate::toast_controller::{
    ToastPlacement, ToastQueueState, ToastRequest, ToastStrategy, ToastTone,
};
use copypasta::{ClipboardContext, ClipboardProvider};
use fc_ai::AiProviderKind;
use slint::{ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

slint::slint! {
    import { LineEdit, ListView, ScrollView, Spinner } from "std-widgets.slint";

    // Contract: shared visual primitives used across sidebar/workspace/modal.
    // They define reusable look-and-feel only; business state stays in MainWindow + Rust bridge.
    component SectionCard inherits Rectangle {
        in property <bool> clip_content: false;
        border-width: 1px;
        border-color: #e3e8ef;
        border-radius: 6px;
        background: #fcfdff;
        clip: self.clip_content;
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
        background: self.selected ? #e5edf7 : transparent;
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
            color: root.selected ? #234766 : #526377;
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

    component WorkspaceTabButton inherits Rectangle {
        in property <string> label;
        in property <bool> selected: false;
        in property <bool> enabled: true;
        in property <length> tab_width: 94px;
        in property <brush> selected_fill: #f9fbfe;
        in property <brush> selected_border: #d7e0ec;
        in property <length> connector_depth: 5px;
        callback tapped();

        width: self.tab_width;
        height: 36px;
        border-width: 1px;
        border-radius: 8px;
        border-color: self.selected ? self.selected_border : #d5dde8;
        background: self.selected ? self.selected_fill : #f1f5f9;
        opacity: self.enabled ? 1 : 0.62;
        clip: false;

        Rectangle {
            x: 0px;
            y: parent.height - root.connector_depth - 1px;
            width: parent.width;
            height: root.connector_depth + 1px;
            background: root.selected ? root.selected_fill : #f1f5f9;
        }

        Rectangle {
            visible: true;
            x: 0px;
            y: parent.height - root.connector_depth - 1px;
            width: 1px;
            height: root.connector_depth + 1px;
            background: root.selected ? root.selected_border : #d5dde8;
        }

        Rectangle {
            visible: true;
            x: parent.width - 1px;
            y: parent.height - root.connector_depth - 1px;
            width: 1px;
            height: root.connector_depth + 1px;
            background: root.selected ? root.selected_border : #d5dde8;
        }

        Rectangle {
            visible: !root.selected;
            x: 1px;
            y: parent.height - 1px;
            width: max(0px, parent.width - 2px);
            height: 1px;
            background: #d5dde8;
        }

        Text {
            text: root.label;
            width: parent.width;
            height: parent.height - (root.selected ? 2px : 0px);
            color: root.selected ? #2d4358 : #5d6c7d;
            font-size: 15px;
            font-weight: root.selected ? 600 : 400;
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
        border-color: root.tone == "different"
            ? #cc8d7a
            : (root.tone == "equal"
                ? #afc7b0
                : (root.tone == "left"
                    ? #c8b58f
                    : (root.tone == "right"
                        ? #acbdc3
                        : (root.tone == "warn"
                            ? #dac6ab
                            : (root.tone == "error"
                                ? #dfc1c1
                                : (root.tone == "info"
                                    ? #c6d7ec
                                    : #cfc9c2))))));
        background: root.tone == "different"
            ? #f8e4de
            : (root.tone == "equal"
                ? #edf5ee
                : (root.tone == "left"
                    ? #f4ede1
                    : (root.tone == "right"
                        ? #e9f0f2
                        : (root.tone == "warn"
                            ? #fbf5ea
                            : (root.tone == "error"
                                ? #fdf2f2
                                : (root.tone == "info"
                                    ? #eef4fb
                                    : #f1efec))))));

        Text {
            text: root.label;
            horizontal-alignment: center;
            vertical-alignment: center;
            color: root.tone == "different"
                ? #7a3f31
                : (root.tone == "equal"
                    ? #355a3f
                    : (root.tone == "left"
                        ? #6d5737
                        : (root.tone == "right"
                            ? #3f5e68
                            : (root.tone == "warn"
                                ? #7b5a2e
                                : (root.tone == "error"
                                    ? #8a2f2f
                                    : (root.tone == "info"
                                        ? #2e5579
                                        : #6b6660))))));
            font-size: 11px;
        }
    }

    component LoadingMask inherits Rectangle {
        in property <string> message;
        in property <length> corner_radius: 6px;

        background: rgba(17, 24, 34, 0.24);
        border-radius: root.corner_radius;
        clip: true;

        Rectangle {
            width: min(340px, max(200px, parent.width - 40px));
            height: 52px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            border-width: 1px;
            border-radius: 6px;
            border-color: #d9e2ee;
            background: #f9fbfe;

            HorizontalLayout {
                padding: 10px;
                spacing: 8px;
                Spinner {
                    width: 20px;
                    height: 20px;
                    indeterminate: true;
                }
                Text {
                    text: root.message == "" ? "Working..." : root.message;
                    color: #47596c;
                    font-size: 13px;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                    overflow: elide;
                }
            }
        }

        TouchArea {}
    }

    component DiffStateShell inherits Rectangle {
        in property <string> state_label;
        in property <string> title;
        in property <string> body;
        in property <string> note;
        in property <string> tone: "neutral";
        in property <bool> embedded: false;

        border-width: root.embedded ? 0px : 1px;
        border-radius: root.embedded ? 0px : 6px;
        border-color: root.panel_border;
        background: root.panel_background;
        clip: true;

        property <brush> panel_border: root.tone == "error"
            ? #dec8c8
            : (root.tone == "warn"
                ? #e0d1bc
                : (root.tone == "info"
                    ? #cad9ea
                    : (root.tone == "success"
                        ? #cad9cf
                        : #d7e1ed)));
        property <brush> panel_background: root.tone == "error"
            ? #fbf3f3
            : (root.tone == "warn"
                ? #fbf8f2
                : (root.tone == "info"
                    ? #f1f6fb
                    : (root.tone == "success"
                        ? #f3f8f4
                        : #f8fafc)));
        property <brush> accent_color: root.tone == "error"
            ? #ba7676
            : (root.tone == "warn"
                ? #b79868
                : (root.tone == "info"
                    ? #7fa2c9
                    : (root.tone == "success"
                        ? #7ca889
                        : #9aadbf)));

        property <brush> title_color: root.tone == "error"
            ? #7f3333
            : (root.tone == "warn"
                ? #735730
                : (root.tone == "info"
                    ? #2e5579
                    : (root.tone == "success"
                        ? #315d42
                        : #475c71)));

        Rectangle {
            x: 0px;
            y: 0px;
            width: 6px;
            height: parent.height;
            background: root.accent_color;
        }

        Rectangle {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: 42px;
            background: root.tone == "error"
                ? #f8e9e9
                : (root.tone == "warn"
                    ? #f7efdf
                    : (root.tone == "info"
                        ? #e9f1fb
                        : (root.tone == "success"
                            ? #e8f2eb
                            : #eef3f8)));

            Rectangle {
                x: 0px;
                y: parent.height - 1px;
                width: parent.width;
                height: 1px;
                background: #dce5ee;
            }

            StatusPill {
                x: 18px;
                y: 12px;
                label: root.state_label;
                tone: root.tone == "neutral" ? "info" : root.tone;
            }
        }

        VerticalLayout {
            x: 22px;
            y: 58px;
            width: max(0px, min(root.width - 44px, 700px));
            spacing: 10px;

            Text {
                text: root.title;
                color: root.title_color;
                font-size: 18px;
                font-weight: 600;
                wrap: word-wrap;
                horizontal-stretch: 1;
            }

            Text {
                visible: root.body != "";
                text: root.body;
                color: #55687b;
                font-size: 14px;
                wrap: word-wrap;
                horizontal-stretch: 1;
            }

            Rectangle {
                visible: root.note != "";
                height: 1px;
                background: #dde5ee;
                horizontal-stretch: 1;
            }

            Text {
                visible: root.note != "";
                text: root.note;
                color: #6d7c8d;
                font-size: 13px;
                wrap: word-wrap;
                horizontal-stretch: 1;
            }
        }
    }

    component SelectableDiffText inherits Rectangle {
        in property <string> value;
        in property <brush> foreground: #2f4357;
        in property <int> font_weight: 400;
        in property <length> content_padding: 6px;

        background: transparent;
        clip: true;

        TextInput {
            x: root.content_padding;
            y: 0px;
            width: max(0px, parent.width - 2 * root.content_padding);
            height: parent.height;
            text: root.value;
            read-only: true;
            single-line: true;
            wrap: no-wrap;
            color: root.foreground;
            font-size: 13px;
            font-weight: root.font_weight;
            horizontal-alignment: left;
            vertical-alignment: center;
            selection-background-color: #c9daec;
            selection-foreground-color: #23384d;
        }
    }

    component SelectableSectionText inherits Rectangle {
        in property <string> value;
        in property <brush> foreground: #4d6176;
        in property <length> font_size: 13px;
        in property <int> font_weight: 400;

        background: transparent;
        clip: true;
        height: sizing_text.preferred-height;

        // Keep wrapped-height measurement aligned with the legacy Text block.
        sizing_text := Text {
            x: 0px;
            y: 0px;
            width: root.width;
            text: root.value;
            color: transparent;
            font-size: root.font_size;
            font-weight: root.font_weight;
            wrap: word-wrap;
        }

        TextInput {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            text: root.value;
            read-only: true;
            single-line: false;
            wrap: word-wrap;
            color: root.foreground;
            font-size: root.font_size;
            font-weight: root.font_weight;
            horizontal-alignment: left;
            vertical-alignment: top;
            selection-background-color: #c9daec;
            selection-foreground-color: #23384d;
        }
    }

    component AnalysisSectionPanel inherits Rectangle {
        in property <string> section_label;
        in property <string> title;
        in property <string> body;
        in property <string> tone: "neutral";
        in property <string> copy_value;
        in property <string> copy_feedback_label: root.section_label;
        callback copy_requested(string, string);

        border-width: 1px;
        border-radius: 8px;
        border-color: root.tone == "error"
            ? #dfcccc
            : (root.tone == "warn"
                ? #dfd1bb
                : (root.tone == "success"
                    ? #d1dfd5
                    : #dbe4ef));
        background: root.tone == "error"
            ? #fdf6f6
            : (root.tone == "warn"
                ? #fcf8f1
                : (root.tone == "success"
                    ? #f5fbf6
                    : #ffffff));

        VerticalLayout {
            padding: 14px;
            spacing: 8px;

            HorizontalLayout {
                spacing: 8px;

                Text {
                    text: root.section_label;
                    color: #708193;
                    font-size: 11px;
                    font-weight: 600;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                }

                Rectangle {
                    visible: root.copy_value != "";
                    height: 20px;
                    background: transparent;

                    Text {
                        text: "Copy";
                        color: #6d7b8b;
                        vertical-alignment: center;
                    }

                    TouchArea {
                        clicked => {
                            root.copy_requested(root.copy_value, root.copy_feedback_label);
                        }
                    }
                }
            }

            SelectableSectionText {
                visible: root.title != "";
                value: root.title;
                foreground: #2f4a63;
                font-size: 18px;
                font-weight: 600;
                horizontal-stretch: 1;
            }

            SelectableSectionText {
                visible: root.body != "";
                value: root.body;
                foreground: #4d6176;
                font-size: 13px;
                horizontal-stretch: 1;
            }
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

    component DiffCopyHotspot inherits Rectangle {
        in property <string> label;
        in property <string> feedback_label;
        in property <string> copy_value;
        in property <bool> enabled: root.label != "";
        in property <bool> align_center: false;
        in property <color> text_color: #667789;
        callback activated();

        property <bool> hovered: hotspot.has_hover && root.enabled;

        border-radius: 4px;
        background: root.hovered ? #eaf1f9 : transparent;
        clip: true;

        Text {
            text: root.label;
            width: parent.width;
            height: parent.height;
            color: root.label == ""
                ? #a2aebb
                : (root.hovered ? #2f5a83 : root.text_color);
            horizontal-alignment: root.align_center ? center : right;
            vertical-alignment: center;
            font-size: 12px;
        }

        hotspot := TouchArea {
            enabled: root.enabled;
            double-clicked => {
                root.activated();
            }
        }
    }

    // Contract: top-level app window shell.
    // Owns layout + UI properties/callback surfaces, but does not execute compare/diff/analysis logic directly.
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
        in property <bool> diff_loaded;
        in property <bool> diff_has_rows;
        in property <[string]> diff_row_kinds;
        in property <[string]> diff_old_line_nos;
        in property <[string]> diff_new_line_nos;
        in property <[string]> diff_markers;
        in property <[string]> diff_contents;
        in property <bool> analysis_loading;
        in property <bool> analysis_available;
        in property <bool> analysis_has_result;
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
        in property <string> analysis_state_label;
        in property <string> analysis_state_token;
        in property <string> analysis_state_tone;
        in property <string> analysis_header_summary_text;
        in property <string> analysis_technical_context_text;
        in property <string> analysis_provider_status_label;
        in property <string> analysis_provider_status_tone;
        in property <string> analysis_state_title_text;
        in property <string> analysis_state_body_text;
        in property <string> analysis_state_note_text;
        in property <string> analysis_summary_text;
        in property <string> analysis_core_judgment_text;
        in property <string> analysis_risk_label_text;
        in property <string> analysis_risk_tone;
        in property <string> analysis_risk_guidance_text;
        in property <string> analysis_result_notes_text;
        in property <string> analysis_summary_copy_text;
        in property <string> analysis_risk_copy_text;
        in property <string> analysis_core_judgment_copy_text;
        in property <string> analysis_key_points_copy_text;
        in property <string> analysis_review_suggestions_copy_text;
        in property <string> analysis_notes_copy_text;
        in property <string> analysis_full_copy_text;
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
        in property <string> selected_row_status;
        in property <string> diff_mode_label;
        in property <string> diff_mode_tone;
        in property <string> diff_result_status_label;
        in property <string> diff_result_status_tone;
        in property <string> diff_shell_state_label;
        in property <string> diff_shell_state_tone;
        in property <string> diff_shell_state_token;
        in property <string> diff_context_summary_text;
        in property <string> diff_context_hint_text;
        in property <string> diff_left_column_label;
        in property <string> diff_right_column_label;
        in property <string> diff_shell_title_text;
        in property <string> diff_shell_body_text;
        in property <string> diff_shell_note_text;
        in property <int> diff_content_char_capacity;
        in-out property <string> toast_feedback_text: "";
        in-out property <string> toast_feedback_tone: "info";
        in property <bool> sidebar_loading_mask_visible: false;
        in property <bool> workspace_loading_mask_visible: false;
        in property <string> loading_mask_text: "";
        property <bool> has_selected_result: root.selected_row >= 0;
        property <bool> diff_shell_ready: root.diff_shell_state_token == "preview-ready"
            || root.diff_shell_state_token == "detailed-ready";
        property <bool> diff_show_shell: root.diff_shell_state_token == "no-selection"
            || root.diff_shell_state_token == "loading"
            || root.diff_shell_state_token == "unavailable"
            || root.diff_shell_state_token == "error"
            || (root.diff_shell_ready && !root.diff_has_rows);
        property <length> diff_number_column_width: 52px;
        property <length> diff_marker_column_width: 20px;
        property <length> diff_scrollbar_safe_inset: 18px;
        property <length> workbench_header_height: 66px;
        property <length> workbench_helper_strip_height: 32px;
        property <length> workbench_action_strip_height: 30px;
        callback compare_clicked();
        callback left_browse_clicked();
        callback right_browse_clicked();
        callback filter_changed(string);
        callback status_filter_changed(string);
        callback row_selected(int);
        callback copy_requested(string, string);
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

            // Contract: app bar shell (title + global provider settings entry).
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

                // Contract: sidebar shell.
                // Hosts compare setup/status/filter/navigation controls; detailed file view stays in workspace.
                Rectangle {
                    horizontal-stretch: 0;
                    min-width: 360px;
                    max-width: 360px;
                    VerticalLayout {
                        spacing: 8px;

                        // Contract: Compare Inputs.
                        // Collects left/right roots and compare trigger; does not render compare results.
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

                        sidebar_busy_scope := Rectangle {
                            vertical-stretch: 1;
                            background: transparent;
                            clip: true;
                            VerticalLayout {
                                spacing: 8px;

                                // Contract: Compare Status.
                                // Summarizes compare run status/metrics/warnings; no row selection or file-level content here.
                                SectionCard {
                            height: root.compare_warnings_expanded && (root.summary_text != "" || root.warnings_text != "" || root.error_text != "") ? 132px : 88px;
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
                                        color: #455d74;
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
                                    color: #566a7f;
                                    overflow: elide;
                                }
                                Rectangle {
                                    visible: root.compare_warnings_expanded && (root.summary_text != "" || root.warnings_text != "" || root.error_text != "");
                                    height: 42px;
                                    border-width: 1px;
                                    border-color: #dde5ef;
                                    border-radius: 4px;
                                    background: #f7fafd;
                                    clip: true;
                                    VerticalLayout {
                                        padding: 5px;
                                        spacing: 2px;
                                        Text {
                                            visible: root.summary_text != "";
                                            text: root.compact_summary_text;
                                            color: #637285;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.error_text != "";
                                            text: root.error_text;
                                            color: #8a2f2f;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                        Text {
                                            visible: root.warnings_text != "";
                                            text: root.warnings_text;
                                            color: #7a5a2f;
                                            overflow: elide;
                                            horizontal-stretch: 1;
                                        }
                                    }
                                }
                            }
                        }

                        // Contract: Filter / Scope.
                        // Applies text/status filters to navigator rows; does not mutate source compare data.
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

                        // Contract: Results / Navigator.
                        // Presents filtered rows and dispatches selection intent; diff/analysis rendering stays in workspace.
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
                                    for row_path[index] in root.row_paths: row_item := Rectangle {
                                        property <int> source_index: root.row_source_indices[index];
                                        property <string> row_status: root.row_statuses[index];
                                        property <bool> row_unavailable: !root.row_can_load_diff[index];
                                        property <bool> row_selected: source_index == root.selected_row;
                                        property <string> row_status_label: row_status == "different"
                                            ? "diff"
                                            : (row_status == "equal"
                                                ? "equal"
                                                : (row_status == "left-only"
                                                    ? "left"
                                                    : (row_status == "right-only"
                                                        ? "right"
                                                        : row_status)));
                                        property <string> row_status_tone: row_status == "different"
                                            ? "different"
                                            : (row_status == "equal"
                                                ? "equal"
                                                : (row_status == "left-only"
                                                    ? "left"
                                                    : (row_status == "right-only"
                                                        ? "right"
                                                        : "neutral")));
                                        property <color> item_border_color: row_selected
                                            ? #3f72b2
                                            : (row_unavailable
                                                ? #cfc9c2
                                                : (row_status == "different"
                                                    ? #c88f7b
                                                    : (row_status == "equal"
                                                        ? #b3cab4
                                                        : (row_status == "left-only"
                                                            ? #c7b28d
                                                            : (row_status == "right-only"
                                                                ? #aabdc3
                                                                : #dce3eb)))));
                                        property <color> item_background_color: row_selected
                                            ? #dbe9fb
                                            : (row_unavailable
                                                ? #f1efec
                                                : (row_status == "different"
                                                    ? #f8e7e1
                                                    : (row_status == "equal"
                                                        ? #eef5ef
                                                        : (row_status == "left-only"
                                                            ? #f5eee2
                                                            : (row_status == "right-only"
                                                                ? #eaf0f2
                                                                : #fbfcfe)))));
                                        property <color> path_text_color: row_selected
                                            ? #123e69
                                            : (row_unavailable ? #6f6962 : #2f3f50);
                                        property <color> detail_text_color: row_selected
                                            ? #3f5f7f
                                            : (row_unavailable ? #89837a : #647486);

                                        height: 44px;
                                        border-width: 1px;
                                        border-color: row_item.item_border_color;
                                        border-radius: 3px;
                                        background: row_item.item_background_color;

                                        VerticalLayout {
                                            padding: 4px;
                                            spacing: 2px;
                                            HorizontalLayout {
                                                spacing: 7px;
                                                StatusPill {
                                                    label: row_item.row_status_label;
                                                    tone: row_item.row_unavailable ? "neutral" : row_item.row_status_tone;
                                                }
                                                Text {
                                                    text: row_path;
                                                    color: row_item.path_text_color;
                                                    vertical-alignment: center;
                                                    horizontal-stretch: 1;
                                                    overflow: elide;
                                                }
                                            }
                                            Text {
                                                text: row_item.row_unavailable ? "detailed diff unavailable" : root.row_details[index];
                                                color: row_item.detail_text_color;
                                                vertical-alignment: center;
                                                horizontal-stretch: 1;
                                                overflow: elide;
                                            }
                                        }

                                        TouchArea {
                                            enabled: !root.diff_loading;
                                            clicked => {
                                                root.row_selected(row_item.source_index);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                            }
                            LoadingMask {
                                visible: root.sidebar_loading_mask_visible;
                                x: 0px;
                                y: 0px;
                                width: parent.width;
                                height: parent.height;
                                message: root.loading_mask_text;
                                corner_radius: 6px;
                            }
                        }
                    }
                }

                // Contract: workspace shell for file-level tabs (Diff / Analysis).
                SectionCard {
                    horizontal-stretch: 1;
                    min-width: 500px;
                    border-color: #d2dbe7;
                    background: #f8fbfe;
                    VerticalLayout {
                        padding: 0px;
                        spacing: 0px;

                        Rectangle {
                            vertical-stretch: 1;
                            background: #fbfcfe;
                            workbench_host := Rectangle {
                                x: 10px;
                                y: 8px;
                                width: max(0px, parent.width - 20px);
                                height: max(0px, parent.height - 18px);
                                background: transparent;

                                property <length> tab_row_height: 36px;
                                property <length> panel_overlap: 4px;
                                property <length> panel_top: self.tab_row_height - self.panel_overlap;

                                workbench_panel := Rectangle {
                                    x: 0px;
                                    y: workbench_host.panel_top;
                                    width: parent.width;
                                    height: max(0px, parent.height - workbench_host.panel_top);
                                    border-width: 1px;
                                    border-color: #d7e0ec;
                                    border-radius: 8px;
                                    background: #fcfdff;
                                    clip: true;

                                    // Contract: workspace content switch.
                                    // Exactly one main branch renders at a time: Diff tab or Analysis tab.
                                    if root.workspace_tab == 0 : Rectangle {
                                        width: parent.width;
                                        height: parent.height;
                                        background: #fcfdff;

                                        VerticalLayout {
                                            padding: 0px;
                                            spacing: 0px;

                                            Rectangle {
                                                height: root.workbench_header_height;
                                                background: #f9fbfe;
                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #dde5f0;
                                                }

                                                VerticalLayout {
                                                    padding: 10px;
                                                    spacing: 4px;

                                                    Text {
                                                        text: root.selected_relative_path == "" ? "No file selected" : root.selected_relative_path;
                                                        color: root.selected_relative_path == "" ? #607286 : #294b6b;
                                                        font-size: 16px;
                                                        font-weight: 600;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
                                                    }

                                                    HorizontalLayout {
                                                        spacing: 6px;
                                                        if root.has_selected_result : StatusPill {
                                                            label: root.diff_mode_label;
                                                            tone: root.diff_mode_tone;
                                                        }
                                                        if root.has_selected_result : StatusPill {
                                                            label: root.diff_result_status_label;
                                                            tone: root.diff_result_status_tone;
                                                        }
                                                        if root.diff_shell_state_token == "loading"
                                                            || root.diff_shell_state_token == "unavailable"
                                                            || root.diff_shell_state_token == "error" : StatusPill {
                                                            label: root.diff_shell_state_label;
                                                            tone: root.diff_shell_state_tone;
                                                        }
                                                        Text {
                                                            text: root.diff_context_summary_text
                                                                + (root.diff_context_hint_text != ""
                                                                    ? (root.diff_context_summary_text != "" ? "  ·  " : "") + root.diff_context_hint_text
                                                                    : "");
                                                            color: #617285;
                                                            font-size: 12px;
                                                            vertical-alignment: center;
                                                            horizontal-stretch: 1;
                                                            overflow: elide;
                                                        }
                                                    }
                                                }
                                            }

                                            if root.diff_show_shell : DiffStateShell {
                                                vertical-stretch: 1;
                                                embedded: true;
                                                state_label: root.diff_shell_state_label;
                                                tone: root.diff_shell_state_tone;
                                                title: root.diff_shell_title_text;
                                                body: root.diff_shell_body_text;
                                                note: root.diff_shell_note_text;
                                            }

                                            if root.diff_shell_ready && root.diff_has_rows : Rectangle {
                                                vertical-stretch: 1;
                                                background: #fcfdff;

                                                VerticalLayout {
                                                    padding: 0px;
                                                    spacing: 0px;

                                                    Rectangle {
                                                        height: root.workbench_helper_strip_height;
                                                        background: #f5f8fc;
                                                        Rectangle {
                                                            x: 0px;
                                                            y: parent.height - 1px;
                                                            width: parent.width;
                                                            height: 1px;
                                                            background: #dbe4ef;
                                                        }

                                                        HorizontalLayout {
                                                            padding: 7px;
                                                            spacing: 8px;
                                                            Text {
                                                                text: "Select text or double-click a line number to copy the full row."
                                                                    + (root.diff_content_char_capacity > 112
                                                                        ? " Long lines scroll horizontally."
                                                                        : "");
                                                                color: #617285;
                                                                font-size: 12px;
                                                                vertical-alignment: center;
                                                                horizontal-stretch: 1;
                                                                overflow: elide;
                                                            }
                                                        }
                                                    }

                                                    diff_ready_surface := Rectangle {
                                                        vertical-stretch: 1;
                                                        background: #ffffff;
                                                        clip: true;
                                                        property <length> table_width: max(
                                                            self.width,
                                                            root.diff_number_column_width * 2
                                                                + root.diff_marker_column_width
                                                                + 160px
                                                                + root.diff_content_char_capacity * 8px
                                                        );

                                                        Rectangle {
                                                            x: 0px;
                                                            y: 0px;
                                                            width: parent.width;
                                                            height: 30px;
                                                            background: #f7f9fc;
                                                            clip: true;

                                                            Rectangle {
                                                                x: 0px;
                                                                y: parent.height - 1px;
                                                                width: parent.width;
                                                                height: 1px;
                                                                background: #dbe4ef;
                                                            }

                                                            Rectangle {
                                                                x: diff_rows.viewport-x;
                                                                y: 0px;
                                                                width: diff_ready_surface.table_width;
                                                                height: parent.height;

                                                                HorizontalLayout {
                                                                    padding: 5px;
                                                                    spacing: 0px;
                                                                    Text {
                                                                        text: root.diff_left_column_label;
                                                                        width: root.diff_number_column_width;
                                                                        horizontal-alignment: right;
                                                                        vertical-alignment: center;
                                                                        font-size: 12px;
                                                                        color: #667789;
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height - 6px;
                                                                        background: #dce5f0;
                                                                    }
                                                                    Text {
                                                                        text: root.diff_right_column_label;
                                                                        width: root.diff_number_column_width;
                                                                        horizontal-alignment: right;
                                                                        vertical-alignment: center;
                                                                        font-size: 12px;
                                                                        color: #667789;
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height - 6px;
                                                                        background: #dce5f0;
                                                                    }
                                                                    Text {
                                                                        text: " ";
                                                                        width: root.diff_marker_column_width;
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height - 6px;
                                                                        background: #dce5f0;
                                                                    }
                                                                    Text {
                                                                        text: "content";
                                                                        color: #667789;
                                                                        font-size: 12px;
                                                                        vertical-alignment: center;
                                                                        horizontal-stretch: 1;
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        diff_rows := ListView {
                                                            x: 0px;
                                                            y: 30px;
                                                            width: parent.width;
                                                            height: max(0px, parent.height - 30px - root.diff_scrollbar_safe_inset);
                                                            viewport-width: diff_ready_surface.table_width;
                                                            for row_content[index] in root.diff_contents: row_line := Rectangle {
                                                                property <string> row_kind: root.diff_row_kinds[index];
                                                                property <bool> is_hunk: row_kind == "hunk";
                                                                property <bool> is_added: row_kind == "added";
                                                                property <bool> is_removed: row_kind == "removed";
                                                                property <string> marker_text: root.diff_markers[index];
                                                                property <string> copy_text: row_line.is_hunk || row_line.marker_text == "" || row_line.marker_text == " "
                                                                    ? row_content
                                                                    : row_line.marker_text + " " + row_content;
                                                                property <string> old_feedback_label: root.diff_old_line_nos[index] == ""
                                                                    ? ""
                                                                    : "Line " + root.diff_old_line_nos[index];
                                                                property <string> new_feedback_label: root.diff_new_line_nos[index] == ""
                                                                    ? ""
                                                                    : "Line " + root.diff_new_line_nos[index];
                                                                width: diff_ready_surface.table_width;
                                                                height: row_line.is_hunk ? 30px : 26px;
                                                                background: row_line.is_hunk
                                                                    ? #ebf2fa
                                                                    : (row_line.is_added
                                                                        ? #f1f8f3
                                                                        : (row_line.is_removed
                                                                            ? #fbf1f1
                                                                            : (Math.mod(index, 2) == 0 ? #fbfdff : #f8fbfe)));

                                                                HorizontalLayout {
                                                                    spacing: 0px;
                                                                    DiffCopyHotspot {
                                                                        width: root.diff_number_column_width;
                                                                        height: parent.height;
                                                                        label: row_line.is_hunk ? "" : root.diff_old_line_nos[index];
                                                                        feedback_label: row_line.old_feedback_label;
                                                                        copy_value: row_line.copy_text;
                                                                        activated => {
                                                                            root.copy_requested(self.copy_value, self.feedback_label);
                                                                        }
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    DiffCopyHotspot {
                                                                        width: root.diff_number_column_width;
                                                                        height: parent.height;
                                                                        label: row_line.is_hunk ? "" : root.diff_new_line_nos[index];
                                                                        feedback_label: row_line.new_feedback_label;
                                                                        copy_value: row_line.copy_text;
                                                                        activated => {
                                                                            root.copy_requested(self.copy_value, self.feedback_label);
                                                                        }
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    DiffCopyHotspot {
                                                                        width: root.diff_marker_column_width;
                                                                        height: parent.height;
                                                                        label: row_line.marker_text;
                                                                        enabled: row_line.is_hunk;
                                                                        align_center: true;
                                                                        feedback_label: "Hunk header";
                                                                        copy_value: row_line.copy_text;
                                                                        text_color: row_line.is_hunk ? #58789c : (row_line.is_added ? #346a4a
                                                                            : (row_line.is_removed ? #8a4242 : #5d6d7e));
                                                                        activated => {
                                                                            root.copy_requested(self.copy_value, self.feedback_label);
                                                                        }
                                                                    }
                                                                    Rectangle {
                                                                        width: 1px;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    SelectableDiffText {
                                                                        value: row_content;
                                                                        foreground: row_line.is_hunk
                                                                            ? #2f5376
                                                                            : #2f4357;
                                                                        font_weight: row_line.is_hunk ? 600 : 400;
                                                                        content_padding: 8px;
                                                                        horizontal-stretch: 1;
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        Rectangle {
                                                            x: 0px;
                                                            y: parent.height - root.diff_scrollbar_safe_inset;
                                                            width: parent.width;
                                                            height: root.diff_scrollbar_safe_inset;
                                                            background: #f7f9fc;
                                                            Rectangle {
                                                                x: 0px;
                                                                y: 0px;
                                                                width: parent.width;
                                                                height: 1px;
                                                                background: #dbe4ef;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if root.workspace_tab == 1 : Rectangle {
                                        width: parent.width;
                                        height: parent.height;
                                        background: #fcfdff;

                                        VerticalLayout {
                                            padding: 0px;
                                            spacing: 0px;

                                            Rectangle {
                                                height: root.workbench_header_height;
                                                background: #f9fbfe;
                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #dde5f0;
                                                }

                                                HorizontalLayout {
                                                    padding: 10px;
                                                    spacing: 10px;

                                                    VerticalLayout {
                                                        spacing: 4px;

                                                        Text {
                                                            text: root.selected_relative_path == "" ? "No file selected" : root.selected_relative_path;
                                                            color: root.selected_relative_path == "" ? #607286 : #294b6b;
                                                            font-size: 16px;
                                                            font-weight: 600;
                                                            horizontal-stretch: 1;
                                                            overflow: elide;
                                                        }

                                                        HorizontalLayout {
                                                            spacing: 6px;
                                                            StatusPill {
                                                                label: "Analysis";
                                                                tone: "neutral";
                                                            }
                                                            StatusPill {
                                                                label: root.analysis_state_label;
                                                                tone: root.analysis_state_tone;
                                                            }
                                                            StatusPill {
                                                                label: root.analysis_provider_status_label;
                                                                tone: root.analysis_provider_status_tone;
                                                            }
                                                            Text {
                                                                text: root.analysis_header_summary_text;
                                                                color: #5f7184;
                                                                font-size: 12px;
                                                                vertical-alignment: center;
                                                                horizontal-stretch: 1;
                                                                overflow: elide;
                                                            }
                                                        }
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
                                            }

                                            Rectangle {
                                                height: root.workbench_helper_strip_height;
                                                background: #f5f8fc;
                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #dbe4ef;
                                                }

                                                HorizontalLayout {
                                                    padding: 7px;
                                                    spacing: 8px;
                                                    Text {
                                                        text: root.analysis_technical_context_text;
                                                        color: #617285;
                                                        font-size: 12px;
                                                        vertical-alignment: center;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
                                                    }
                                                    Text {
                                                        text: "Use Provider Settings in App Bar to edit.";
                                                        color: #7b8a99;
                                                        font-size: 11px;
                                                        vertical-alignment: center;
                                                    }
                                                }
                                            }

                                            if root.analysis_state_token != "success" : DiffStateShell {
                                                vertical-stretch: 1;
                                                embedded: true;
                                                state_label: root.analysis_state_label;
                                                title: root.analysis_state_title_text;
                                                body: root.analysis_state_body_text;
                                                note: root.analysis_state_note_text;
                                                tone: root.analysis_state_tone;
                                            }

                                            if root.analysis_state_token == "success" : Rectangle {
                                                vertical-stretch: 1;
                                                background: #fcfdff;
                                                clip: true;

                                                VerticalLayout {
                                                    padding: 0px;
                                                    spacing: 0px;

                                                    Rectangle {
                                                        height: root.workbench_action_strip_height;
                                                        background: #f7f9fc;
                                                        Rectangle {
                                                            x: 0px;
                                                            y: parent.height - 1px;
                                                            width: parent.width;
                                                            height: 1px;
                                                            background: #dbe4ef;
                                                        }

                                                        HorizontalLayout {
                                                            padding: 6px;
                                                            spacing: 8px;
                                                            Text {
                                                                text: "Copy the structured review or switch back to Diff.";
                                                                color: #617285;
                                                                font-size: 12px;
                                                                vertical-alignment: center;
                                                                horizontal-stretch: 1;
                                                                overflow: elide;
                                                            }
                                                            TextAction {
                                                                label: "Open Diff";
                                                                enabled: root.has_selected_result;
                                                                tapped => {
                                                                    root.workspace_tab = 0;
                                                                }
                                                            }
                                                            TextAction {
                                                                label: "Copy All";
                                                                enabled: root.analysis_full_copy_text != "";
                                                                tapped => {
                                                                    root.copy_requested(root.analysis_full_copy_text, "Analysis");
                                                                }
                                                            }
                                                        }
                                                    }

                                                    analysis_success_surface := Rectangle {
                                                        vertical-stretch: 1;
                                                        background: #fcfdff;
                                                        clip: true;

                                                        analysis_success_scroll := ScrollView {
                                                            width: parent.width;
                                                            height: parent.height;
                                                            property <length> viewport_side_padding: 16px;
                                                            property <length> viewport_top_padding: 16px;
                                                            property <length> viewport_bottom_padding: 16px;
                                                            property <length> section_spacing: 12px;
                                                            property <length> content_width: max(
                                                                0px,
                                                                min(self.width - viewport_side_padding * 2, 780px)
                                                            );
                                                            // Use real stacked section geometry instead of guard-based estimation.
                                                            property <length> content_bottom: notes_section.visible
                                                                ? (notes_section.y + notes_section.height)
                                                                : (review_suggestions_section.visible
                                                                    ? (review_suggestions_section.y + review_suggestions_section.height)
                                                                    : (key_points_section.visible
                                                                        ? (key_points_section.y + key_points_section.height)
                                                                        : (core_section.y + core_section.height)));
                                                            property <length> target_viewport_height: max(
                                                                self.height,
                                                                content_bottom + viewport_bottom_padding
                                                            );
                                                            viewport-width: self.width;
                                                            viewport-height: target_viewport_height;
                                                            viewport := Rectangle {
                                                                width: analysis_success_scroll.viewport-width;
                                                                height: analysis_success_scroll.viewport-height;

                                                                summary_section := AnalysisSectionPanel {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: analysis_success_scroll.viewport_top_padding;
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: self.preferred-height;
                                                                    section_label: "Summary";
                                                                    title: root.analysis_title_text != "" ? root.analysis_title_text : "Analysis Summary";
                                                                    body: root.analysis_summary_text;
                                                                    copy_value: root.analysis_summary_copy_text;
                                                                    copy_requested(copy_value, feedback_label) => {
                                                                        root.copy_requested(copy_value, feedback_label);
                                                                    }
                                                                }

                                                                risk_section := Rectangle {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: summary_section.y + summary_section.height + analysis_success_scroll.section_spacing;
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: risk_layout.preferred-height;
                                                                    border-width: 1px;
                                                                    border-radius: 8px;
                                                                    border-color: root.analysis_risk_tone == "error"
                                                                        ? #dfcccc
                                                                        : (root.analysis_risk_tone == "warn"
                                                                            ? #dfd1bb
                                                                            : (root.analysis_risk_tone == "success"
                                                                                ? #d1dfd5
                                                                                : #dbe4ef));
                                                                    background: root.analysis_risk_tone == "error"
                                                                        ? #fdf6f6
                                                                        : (root.analysis_risk_tone == "warn"
                                                                            ? #fcf8f1
                                                                            : (root.analysis_risk_tone == "success"
                                                                                ? #f5fbf6
                                                                                : #ffffff));

                                                                    risk_layout := VerticalLayout {
                                                                        padding: 14px;
                                                                        spacing: 8px;

                                                                        HorizontalLayout {
                                                                            spacing: 8px;

                                                                            Text {
                                                                                text: "Risk Level";
                                                                                color: #708193;
                                                                                font-size: 11px;
                                                                                font-weight: 600;
                                                                                vertical-alignment: center;
                                                                                horizontal-stretch: 1;
                                                                            }

                                                                            TextAction {
                                                                                visible: root.analysis_risk_copy_text != "";
                                                                                label: "Copy";
                                                                                tapped => {
                                                                                    root.copy_requested(root.analysis_risk_copy_text, "Risk Level");
                                                                                }
                                                                            }
                                                                        }

                                                                        StatusPill {
                                                                            label: root.analysis_risk_label_text;
                                                                            tone: root.analysis_risk_tone;
                                                                        }

                                                                        Text {
                                                                            text: root.analysis_risk_guidance_text;
                                                                            color: #4d6176;
                                                                            font-size: 13px;
                                                                            wrap: word-wrap;
                                                                            horizontal-stretch: 1;
                                                                        }
                                                                    }
                                                                }

                                                                core_section := AnalysisSectionPanel {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: risk_section.y + risk_section.height + analysis_success_scroll.section_spacing;
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: self.preferred-height;
                                                                    section_label: "Core Judgment";
                                                                    body: root.analysis_core_judgment_text;
                                                                    copy_value: root.analysis_core_judgment_copy_text;
                                                                    copy_requested(copy_value, feedback_label) => {
                                                                        root.copy_requested(copy_value, feedback_label);
                                                                    }
                                                                }

                                                                key_points_section := AnalysisSectionPanel {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: core_section.y + core_section.height + analysis_success_scroll.section_spacing;
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: self.preferred-height;
                                                                    visible: root.analysis_key_points_text != "";
                                                                    section_label: "Key Points";
                                                                    body: root.analysis_key_points_text;
                                                                    copy_value: root.analysis_key_points_copy_text;
                                                                    copy_requested(copy_value, feedback_label) => {
                                                                        root.copy_requested(copy_value, feedback_label);
                                                                    }
                                                                }

                                                                review_suggestions_section := AnalysisSectionPanel {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: key_points_section.y
                                                                        + (key_points_section.visible
                                                                            ? (key_points_section.height + analysis_success_scroll.section_spacing)
                                                                            : 0px);
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: self.preferred-height;
                                                                    visible: root.analysis_review_suggestions_text != "";
                                                                    section_label: "Review Suggestions";
                                                                    body: root.analysis_review_suggestions_text;
                                                                    copy_value: root.analysis_review_suggestions_copy_text;
                                                                    copy_requested(copy_value, feedback_label) => {
                                                                        root.copy_requested(copy_value, feedback_label);
                                                                    }
                                                                }

                                                                notes_section := AnalysisSectionPanel {
                                                                    x: analysis_success_scroll.viewport_side_padding;
                                                                    y: review_suggestions_section.y
                                                                        + (review_suggestions_section.visible
                                                                            ? (review_suggestions_section.height + analysis_success_scroll.section_spacing)
                                                                            : 0px);
                                                                    width: analysis_success_scroll.content_width;
                                                                    height: self.preferred-height;
                                                                    visible: root.analysis_result_notes_text != "";
                                                                    section_label: "Notes";
                                                                    body: root.analysis_result_notes_text;
                                                                    tone: "warn";
                                                                    copy_value: root.analysis_notes_copy_text;
                                                                    copy_requested(copy_value, feedback_label) => {
                                                                        root.copy_requested(copy_value, feedback_label);
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

                                Rectangle {
                                    x: 0px;
                                    y: workbench_host.panel_top;
                                    width: parent.width;
                                    height: 1px;
                                    background: #d7e0ec;
                                }

                                HorizontalLayout {
                                    x: 0px;
                                    y: 0px;
                                    width: parent.width;
                                    height: workbench_host.tab_row_height;
                                    spacing: 0px;

                                    WorkspaceTabButton {
                                        label: "Diff";
                                        selected: root.workspace_tab == 0;
                                        selected_fill: #f9fbfe;
                                        selected_border: #d7e0ec;
                                        connector_depth: 5px;
                                        tapped => {
                                            root.workspace_tab = 0;
                                        }
                                    }

                                    WorkspaceTabButton {
                                        label: "Analysis";
                                        selected: root.workspace_tab == 1;
                                        selected_fill: #f9fbfe;
                                        selected_border: #d7e0ec;
                                        connector_depth: 5px;
                                        tapped => {
                                            root.workspace_tab = 1;
                                        }
                                    }

                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }

                                    Text {
                                        text: root.workspace_tab == 0 ? "File View" : "Analysis";
                                        color: #6a7b8d;
                                        font-size: 12px;
                                        vertical-alignment: center;
                                        overflow: elide;
                                    }
                                }
                            }
                        }
                    }
                    LoadingMask {
                        visible: root.workspace_loading_mask_visible;
                        x: 0px;
                        y: 0px;
                        width: parent.width;
                        height: parent.height;
                        message: root.loading_mask_text;
                        corner_radius: 6px;
                    }
                }
            }
        }

        if root.toast_feedback_text != "" : Rectangle {
            property <length> viewport_width: max(220px, root.width - 24px);
            property <length> bubble_width: min(viewport_width, 420px);
            x: (root.width - self.bubble_width) / 2;
            y: 14px;
            width: self.bubble_width;
            height: 34px;
            opacity: 0.5;
            border-width: 1px;
            border-radius: 6px;
            border-color: root.toast_feedback_tone == "error"
                ? #d8b2b2
                : (root.toast_feedback_tone == "warn"
                    ? #dcc9a9
                    : (root.toast_feedback_tone == "success"
                        ? #b9d5c1
                        : #b8ccdf));
            background: root.toast_feedback_tone == "error"
                ? #f8eeee
                : (root.toast_feedback_tone == "warn"
                    ? #fbf5e9
                    : (root.toast_feedback_tone == "success"
                        ? #edf7ef
                        : #edf4fb));

            toast_message := Text {
                text: root.toast_feedback_text;
                x: 14px;
                y: 0px;
                width: max(0px, parent.width - 28px);
                height: parent.height;
                color: root.toast_feedback_tone == "error"
                    ? #7f3333
                    : (root.toast_feedback_tone == "warn"
                        ? #735730
                        : (root.toast_feedback_tone == "success"
                            ? #315d42
                            : #2e5579));
                horizontal-alignment: center;
                vertical-alignment: center;
                overflow: elide;
            }
        }

        // Contract: Provider Settings modal.
        // Edits global provider config and validation errors; compare/diff workflow remains in main shell.
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

const LOADING_MASK_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadingMaskPhase {
    Idle,
    Comparing,
    DiffLoading,
    AnalysisLoading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LoadingMaskProjection {
    sidebar_visible: bool,
    workspace_visible: bool,
    message: &'static str,
}

impl Default for LoadingMaskProjection {
    fn default() -> Self {
        Self {
            sidebar_visible: false,
            workspace_visible: false,
            message: "",
        }
    }
}

#[derive(Debug, Clone)]
struct LoadingMaskWatchdog {
    phase: LoadingMaskPhase,
    phase_started_at: Option<Instant>,
    last_projection: LoadingMaskProjection,
}

impl Default for LoadingMaskWatchdog {
    fn default() -> Self {
        Self {
            phase: LoadingMaskPhase::Idle,
            phase_started_at: None,
            last_projection: LoadingMaskProjection::default(),
        }
    }
}

impl LoadingMaskWatchdog {
    fn tick(
        &mut self,
        running: bool,
        diff_loading: bool,
        analysis_loading: bool,
        now: Instant,
    ) -> Option<LoadingMaskProjection> {
        let phase = derive_loading_mask_phase(running, diff_loading, analysis_loading);
        if phase == LoadingMaskPhase::Idle {
            self.phase = LoadingMaskPhase::Idle;
            self.phase_started_at = None;
        } else if self.phase != phase {
            self.phase = phase;
            self.phase_started_at = Some(now);
        } else if self.phase_started_at.is_none() {
            self.phase_started_at = Some(now);
        }

        let timeout_reached = phase != LoadingMaskPhase::Idle
            && self
                .phase_started_at
                .map(|start| now.duration_since(start) >= LOADING_MASK_TIMEOUT)
                .unwrap_or(false);
        let projection = derive_loading_mask_projection(
            running,
            diff_loading,
            analysis_loading,
            timeout_reached,
        );
        if projection == self.last_projection {
            return None;
        }
        self.last_projection = projection;
        Some(projection)
    }
}

fn derive_loading_mask_phase(
    running: bool,
    diff_loading: bool,
    analysis_loading: bool,
) -> LoadingMaskPhase {
    if running {
        LoadingMaskPhase::Comparing
    } else if diff_loading {
        LoadingMaskPhase::DiffLoading
    } else if analysis_loading {
        LoadingMaskPhase::AnalysisLoading
    } else {
        LoadingMaskPhase::Idle
    }
}

fn derive_loading_mask_projection(
    running: bool,
    diff_loading: bool,
    analysis_loading: bool,
    timeout_reached: bool,
) -> LoadingMaskProjection {
    let phase = derive_loading_mask_phase(running, diff_loading, analysis_loading);
    let (sidebar_visible, workspace_visible) = match phase {
        LoadingMaskPhase::Idle => (false, false),
        LoadingMaskPhase::Comparing => (true, true),
        LoadingMaskPhase::DiffLoading | LoadingMaskPhase::AnalysisLoading => (false, true),
    };
    let message = if timeout_reached {
        "Taking longer than expected..."
    } else {
        match phase {
            LoadingMaskPhase::Idle => "",
            LoadingMaskPhase::Comparing => "Comparing folders...",
            LoadingMaskPhase::DiffLoading => "Loading diff...",
            LoadingMaskPhase::AnalysisLoading => "Running AI analysis...",
        }
    };
    LoadingMaskProjection {
        sidebar_visible,
        workspace_visible,
        message,
    }
}

fn apply_loading_mask_projection(window: &MainWindow, projection: LoadingMaskProjection) {
    window.set_sidebar_loading_mask_visible(projection.sidebar_visible);
    window.set_workspace_loading_mask_visible(projection.workspace_visible);
    window.set_loading_mask_text(projection.message.into());
}

// Contract: sync mode gate for editable UI fields.
// Full mode pulls editable inputs from state; Passive mode preserves in-flight user typing.
fn should_sync_editable_inputs(mode: SyncMode) -> bool {
    matches!(mode, SyncMode::Full)
}

// Contract: state cache guard.
// Prevents redundant property/model writes when the presenter state snapshot is unchanged.
fn should_skip_sync(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    last_state == Some(next_state)
}

// Contract: navigator model refresh boundary.
// Rebuild list models only when row/filter/status inputs changed.
fn should_refresh_result_models(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.entry_rows != next_state.entry_rows
                || last.entry_filter != next_state.entry_filter
                || last.entry_status_filter != next_state.entry_status_filter
        }
    }
}

// Contract: diff model refresh boundary.
// Rebuild diff row models only when selected diff payload changes.
fn should_refresh_diff_models(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => true,
        Some(last) => last.selected_diff != next_state.selected_diff,
    }
}

// Contract: state -> window projection.
// Centralized one-way sync from AppState snapshot into Slint properties/models.
fn sync_window_state(
    window: &MainWindow,
    state: &AppState,
    mode: SyncMode,
    last_state: Option<&AppState>,
) {
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
    window.set_diff_loaded(state.selected_diff.is_some());
    window.set_diff_has_rows(state.diff_has_rows());
    window.set_analysis_loading(state.analysis_loading);
    window.set_analysis_available(state.analysis_available);
    window.set_analysis_has_result(state.analysis_result.is_some());
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
    window.set_analysis_state_label(state.analysis_state_label().into());
    window.set_analysis_state_token(state.analysis_state_token().into());
    window.set_analysis_state_tone(state.analysis_state_tone().into());
    window.set_analysis_header_summary_text(state.analysis_header_summary_text().into());
    window.set_analysis_technical_context_text(state.analysis_technical_context_text().into());
    window.set_analysis_provider_status_label(state.analysis_provider_status_label().into());
    window.set_analysis_provider_status_tone(state.analysis_provider_status_tone().into());
    window.set_analysis_state_title_text(state.analysis_state_title_text().into());
    window.set_analysis_state_body_text(state.analysis_state_body_text().into());
    window.set_analysis_state_note_text(state.analysis_state_note_text().into());
    window.set_analysis_summary_text(state.analysis_summary_text().into());
    window.set_analysis_core_judgment_text(state.analysis_core_judgment_text().into());
    window.set_analysis_risk_label_text(state.analysis_risk_label_text().into());
    window.set_analysis_risk_tone(state.analysis_risk_tone().into());
    window.set_analysis_risk_guidance_text(state.analysis_risk_guidance_text().into());
    window.set_analysis_result_notes_text(state.analysis_result_notes_text().into());
    window.set_analysis_summary_copy_text(state.analysis_summary_copy_text().into());
    window.set_analysis_risk_copy_text(state.analysis_risk_copy_text().into());
    window.set_analysis_core_judgment_copy_text(state.analysis_core_judgment_copy_text().into());
    window.set_analysis_key_points_copy_text(state.analysis_key_points_copy_text().into());
    window.set_analysis_review_suggestions_copy_text(
        state.analysis_review_suggestions_copy_text().into(),
    );
    window.set_analysis_notes_copy_text(state.analysis_notes_copy_text().into());
    window.set_analysis_full_copy_text(state.analysis_full_copy_text().into());
    window.set_provider_settings_error_text(state.provider_settings_error_text().into());
    window.set_selected_row(state.selected_row.map(|value| value as i32).unwrap_or(-1));
    window.set_selected_row_status(state.selected_row_status_text().into());
    window.set_diff_mode_label(state.diff_mode_label().into());
    window.set_diff_mode_tone(state.diff_mode_tone().into());
    window.set_diff_result_status_label(state.diff_result_status_label().into());
    window.set_diff_result_status_tone(state.diff_result_status_tone().into());
    window.set_diff_shell_state_label(state.diff_shell_state_label().into());
    window.set_diff_shell_state_tone(state.diff_shell_state_tone().into());
    window.set_diff_shell_state_token(state.diff_shell_state_token().into());
    window.set_diff_context_summary_text(state.diff_context_summary_text().into());
    window.set_diff_context_hint_text(state.diff_context_hint_text().into());
    window.set_diff_left_column_label(state.diff_left_column_label().into());
    window.set_diff_right_column_label(state.diff_right_column_label().into());
    window.set_diff_shell_title_text(state.diff_shell_title_text().into());
    window.set_diff_shell_body_text(state.diff_shell_body_text().into());
    window.set_diff_shell_note_text(state.diff_shell_note_text().into());
    window.set_diff_content_char_capacity(state.diff_content_char_capacity());
    if should_refresh_result_models(last_state, state) {
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
    }

    if should_refresh_diff_models(last_state, state) {
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
}

// Contract: cache-aware sync wrapper used by timer + callbacks.
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
    sync_window_state(window, &state, mode, cache_guard.as_ref());
    // Keep loading-mask projection aligned with the latest synced busy flags,
    // so short-lived diff loading started from row selection can still render immediately.
    let immediate_projection = derive_loading_mask_projection(
        state.running,
        state.diff_loading,
        state.analysis_loading,
        false,
    );
    apply_loading_mask_projection(window, immediate_projection);
    *cache_guard = Some(state);
}

fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        ClipboardContext::new().map_err(|err| format!("clipboard unavailable: {err}"))?;
    clipboard
        .set_contents(text.to_string())
        .map_err(|err| format!("clipboard write failed: {err}"))
}

#[derive(Clone)]
struct ToastController {
    inner: Rc<RefCell<ToastControllerInner>>,
}

struct ToastControllerInner {
    window: slint::Weak<MainWindow>,
    queue: ToastQueueState,
    generation: u64,
}

impl ToastControllerInner {
    fn render_active(&self, request: &ToastRequest) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let tone = toast_tone_token(request.tone);
        window.set_toast_feedback_tone(tone.into());
        window.set_toast_feedback_text(request.message.clone().into());
    }

    fn clear_toast(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        window.set_toast_feedback_text("".into());
    }
}

impl ToastController {
    fn new(window: &MainWindow) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ToastControllerInner {
                window: window.as_weak(),
                queue: ToastQueueState::default(),
                generation: 0,
            })),
        }
    }

    fn dispatch(&self, request: ToastRequest) {
        let next_timer = {
            let mut inner = self.inner.borrow_mut();
            let dispatch = inner.queue.enqueue(request);
            if let Some(active) = dispatch.active.as_ref() {
                inner.render_active(active);
            }
            if dispatch.reset_timer {
                dispatch.active.map(|active| {
                    inner.generation = inner.generation.wrapping_add(1);
                    (inner.generation, active.duration)
                })
            } else {
                None
            }
        };

        if let Some((generation, duration)) = next_timer {
            self.schedule_timeout(generation, duration);
        }
    }

    fn schedule_timeout(&self, generation: u64, duration: Duration) {
        let controller = self.clone();
        Timer::single_shot(duration, move || {
            controller.on_timeout(generation);
        });
    }

    fn on_timeout(&self, generation: u64) {
        let next_timer = {
            let mut inner = self.inner.borrow_mut();
            if inner.generation != generation {
                return;
            }

            match inner.queue.advance_after_timeout() {
                Some(active) => {
                    inner.render_active(&active);
                    inner.generation = inner.generation.wrapping_add(1);
                    Some((inner.generation, active.duration))
                }
                None => {
                    inner.clear_toast();
                    None
                }
            }
        };

        if let Some((next_generation, duration)) = next_timer {
            self.schedule_timeout(next_generation, duration);
        }
    }
}

fn toast_tone_token(tone: ToastTone) -> &'static str {
    match tone {
        ToastTone::Success => "success",
        ToastTone::Warn => "warn",
        ToastTone::Error => "error",
        ToastTone::Info => "info",
    }
}

fn copy_text_with_feedback(toast_controller: &ToastController, text: &str, feedback_label: &str) {
    let (message, tone) = if copy_text_to_clipboard(text).is_ok() {
        let label = feedback_label.trim();
        (
            if label.is_empty() {
                "Copied".to_string()
            } else {
                format!("{label} copied")
            },
            ToastTone::Info,
        )
    } else {
        ("Copy failed".to_string(), ToastTone::Error)
    };

    toast_controller.dispatch(
        ToastRequest::new(message, tone, ToastPlacement::Toast)
            .with_duration(Duration::from_millis(1600))
            .with_strategy(ToastStrategy::Replace),
    );
}

/// Runs the UI application.
pub fn run() -> anyhow::Result<()> {
    let app = MainWindow::new().map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let state = Arc::new(Mutex::new(AppState::default()));
    let presenter = Presenter::new(state);
    let bridge = UiBridge::new(presenter);
    bridge.dispatch(UiCommand::Initialize);
    let initial_state = bridge.snapshot();
    sync_window_state(&app, &initial_state, SyncMode::Full, None);
    let sync_cache = Arc::new(Mutex::new(Some(initial_state)));
    let toast_controller = ToastController::new(&app);

    // Contract: background UI polling loop.
    // Polls presenter busy flags and performs passive sync only when runtime busy-state diverges from window state.
    let ui_refresh_timer = Timer::default();
    let loading_mask_watchdog = Rc::new(RefCell::new(LoadingMaskWatchdog::default()));
    let app_weak = app.as_weak();
    let refresh_bridge = bridge.clone();
    let refresh_cache = Arc::clone(&sync_cache);
    let refresh_loading_mask_watchdog = Rc::clone(&loading_mask_watchdog);
    ui_refresh_timer.start(TimerMode::Repeated, Duration::from_millis(50), move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        let bridge_busy = refresh_bridge.busy_flags();
        let window_busy = (
            window.get_running(),
            window.get_diff_loading(),
            window.get_analysis_loading(),
        );
        if bridge_busy != window_busy {
            sync_window_state_if_changed(
                &window,
                &refresh_bridge,
                &refresh_cache,
                SyncMode::Passive,
            );
        }
        let mut watchdog = refresh_loading_mask_watchdog.borrow_mut();
        if let Some(projection) = watchdog.tick(
            window.get_running(),
            window.get_diff_loading(),
            window.get_analysis_loading(),
            Instant::now(),
        ) {
            apply_loading_mask_projection(&window, projection);
        }
    });

    // Contract: UI event dispatch and bridge binding.
    // Each callback converts UI intent into UiCommand(s), then triggers passive sync.

    // Compare flow callbacks.
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

    // Local folder picker callbacks (UI-only input capture, no direct presenter mutation except via compare click).
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

    // Navigator selection + filters callbacks.
    let app_weak = app.as_weak();
    let row_bridge = bridge.clone();
    let row_cache = Arc::clone(&sync_cache);
    app.on_row_selected(move |index| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_diff_loading() {
            return;
        }
        window.set_workspace_tab(0);
        if window.get_selected_row() == index
            && (window.get_diff_loaded() || window.get_diff_loading())
        {
            return;
        }

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
    let copy_toast_controller = toast_controller.clone();
    app.on_copy_requested(move |value, feedback_label| {
        if app_weak.upgrade().is_none() {
            return;
        }
        copy_text_with_feedback(
            &copy_toast_controller,
            value.as_str(),
            feedback_label.as_str(),
        );
    });

    // Provider settings lifecycle callbacks (open/cancel/save).
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
    let provider_settings_toast_controller = toast_controller.clone();
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
            provider_settings_toast_controller.dispatch(ToastRequest::new(
                "Provider settings saved",
                ToastTone::Success,
                ToastPlacement::Toast,
            ));
        }
    });

    // Analysis action callbacks.
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

    // Analysis provider mode callbacks.
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

    // Analysis remote config field callbacks.
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
    use std::time::Duration;

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

    #[test]
    fn loading_mask_projection_follows_scope_boundary() {
        let idle = derive_loading_mask_projection(false, false, false, false);
        assert_eq!(idle, LoadingMaskProjection::default());

        let running = derive_loading_mask_projection(true, false, false, false);
        assert_eq!(
            running,
            LoadingMaskProjection {
                sidebar_visible: true,
                workspace_visible: true,
                message: "Comparing folders...",
            }
        );

        let diff = derive_loading_mask_projection(false, true, false, false);
        assert_eq!(
            diff,
            LoadingMaskProjection {
                sidebar_visible: false,
                workspace_visible: true,
                message: "Loading diff...",
            }
        );

        let analysis = derive_loading_mask_projection(false, false, true, false);
        assert_eq!(
            analysis,
            LoadingMaskProjection {
                sidebar_visible: false,
                workspace_visible: true,
                message: "Running AI analysis...",
            }
        );
    }

    #[test]
    fn loading_mask_projection_uses_timeout_copy() {
        let projection = derive_loading_mask_projection(false, true, false, true);
        assert_eq!(
            projection,
            LoadingMaskProjection {
                sidebar_visible: false,
                workspace_visible: true,
                message: "Taking longer than expected...",
            }
        );
    }

    #[test]
    fn loading_mask_watchdog_resets_on_phase_change() {
        let mut watchdog = LoadingMaskWatchdog::default();
        let now = Instant::now();
        let started = watchdog
            .tick(false, true, false, now)
            .expect("first diff-loading projection should be emitted");
        assert_eq!(started.message, "Loading diff...");

        let timed_out = watchdog
            .tick(
                false,
                true,
                false,
                now + LOADING_MASK_TIMEOUT + Duration::from_millis(1),
            )
            .expect("timeout transition should emit degraded copy");
        assert_eq!(timed_out.message, "Taking longer than expected...");

        let analysis_reset = watchdog
            .tick(
                false,
                false,
                true,
                now + LOADING_MASK_TIMEOUT + Duration::from_millis(2),
            )
            .expect("phase switch should reset timeout copy");
        assert_eq!(analysis_reset.message, "Running AI analysis...");
    }
}
