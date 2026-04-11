//! Slint app for compare + detailed diff with non-blocking and safe UI sync behavior.

use crate::bridge::UiBridge;
use crate::commands::UiCommand;
use crate::context_menu::{
    CONTEXT_MENU_COPY_ACTION_ID, CONTEXT_MENU_COPY_SUMMARY_ACTION_ID, ContextMenuBuildResult,
    ContextMenuCustomAction, ContextMenuCustomActionDescriptor, ContextMenuInvocation,
    ContextMenuSyncState, ContextMenuTextPayload, build_action_specs,
    build_analysis_section_payload, build_compare_status_payload, build_results_row_payload,
    build_workspace_header_payload, should_close_for_sync_transition,
};
use crate::folder_picker;
use crate::presenter::Presenter;
use crate::state::{AppState, CompareViewRowAction, CompareViewRowProjection, NavigatorViewMode};
use crate::toast_controller::{
    ToastPlacement, ToastQueueState, ToastRequest, ToastStrategy, ToastTone,
};
use crate::window_chrome;
use copypasta::{ClipboardContext, ClipboardProvider};
use fc_ai::AiProviderKind;
use slint::{Model, ModelRc, SharedString, Timer, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

const RESULTS_LOCATE_AND_OPEN_ACTION_ID: &str = "results-locate-and-open";
const RESULTS_OPEN_IN_COMPARE_VIEW_ACTION_ID: &str = "results-open-in-compare-view";

slint::slint! {
    import { LineEdit, ListView, ScrollView, Spinner } from "std-widgets.slint";
    import { CompareFileView } from "src/compare_file_view.slint";
    import { CompareView } from "src/compare_view.slint";
    import { NavigatorTree } from "src/navigator_tree.slint";
    import { UiPalette } from "src/ui_palette.slint";

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
        in property <length> label_font_size: 14px;
        in property <bool> label_align_left: false;
        in property <length> label_left_padding: 0px;
        in property <length> label_right_padding: 0px;
        in property <string> tooltip_text: "";
        out property <bool> hovered: button_touch_area.has_hover;
        callback tapped();
        callback tooltip_requested(string, length, length, length);
        callback tooltip_closed();

        horizontal-stretch: 0;
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
            x: root.label_left_padding;
            width: max(0px, parent.width - root.label_left_padding - root.label_right_padding);
            text: root.label;
            color: root.primary ? #ffffff : (root.active ? #27476b : #384555);
            horizontal-alignment: root.label_align_left ? left : center;
            vertical-alignment: center;
            font-size: root.label_font_size;
        }

        button_touch_area := TouchArea {
            enabled: root.enabled || root.tooltip_text != "";
            clicked => {
                if root.enabled {
                    root.tapped();
                }
            }

            changed has-hover => {
                if self.has-hover && root.tooltip_text != "" {
                    root.tooltip_requested(
                        root.tooltip_text,
                        self.absolute-position.x + 10px,
                        self.absolute-position.y,
                        self.absolute-position.y + self.height,
                    );
                } else {
                    root.tooltip_closed();
                }
            }

            pointer-event(event) => {
                if event.kind == PointerEventKind.cancel {
                    root.tooltip_closed();
                }
            }
        }
    }

    component SidebarChromeButton inherits Rectangle {
        in property <bool> sidebar_visible: true;
        in property <string> tooltip_text: "";
        out property <bool> hovered: button_touch_area.has_hover;
        callback tapped();
        callback tooltip_requested(string, length, length, length);
        callback tooltip_closed();

        property <brush> border_brush: !root.sidebar_visible
            ? #c8d4e2
            : (root.hovered ? #d7e0ea : transparent);
        property <brush> background_brush: !root.sidebar_visible
            ? rgba(234, 241, 249, 0.96)
            : (root.hovered ? rgba(244, 247, 251, 0.96) : transparent);
        property <brush> icon_brush: !root.sidebar_visible ? #315c86 : #617487;

        width: 28px;
        height: 24px;
        border-width: 1px;
        border-radius: 6px;
        border-color: root.border_brush;
        background: root.background_brush;

        Rectangle {
            x: 6px;
            y: 5px;
            width: 13px;
            height: 14px;
            border-width: 1px;
            border-radius: 3px;
            border-color: root.icon_brush;
            background: transparent;
        }

        Rectangle {
            x: 7px;
            y: 6px;
            width: 3px;
            height: 12px;
            border-radius: 1px;
            border-width: root.sidebar_visible ? 0px : 1px;
            border-color: root.icon_brush;
            background: root.sidebar_visible ? root.icon_brush : transparent;
            opacity: root.sidebar_visible ? 1 : 0.76;
        }

        Path {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            viewbox-width: 24;
            viewbox-height: 24;
            fill: transparent;
            stroke: root.icon_brush;
            stroke-width: 1.4px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo {
                x: root.sidebar_visible ? 16.5 : 13.0;
                y: 9.0;
            }

            LineTo {
                x: root.sidebar_visible ? 13.0 : 16.5;
                y: 12.0;
            }

            LineTo {
                x: root.sidebar_visible ? 16.5 : 13.0;
                y: 15.0;
            }
        }

        button_touch_area := TouchArea {
            enabled: true;
            clicked => {
                root.tapped();
            }

            changed has-hover => {
                if self.has-hover && root.tooltip_text != "" {
                    root.tooltip_requested(
                        root.tooltip_text,
                        self.absolute-position.x + 8px,
                        self.absolute-position.y,
                        self.absolute-position.y + self.height,
                    );
                } else {
                    root.tooltip_closed();
                }
            }

            pointer-event(event) => {
                if event.kind == PointerEventKind.cancel || event.kind == PointerEventKind.down {
                    root.tooltip_closed();
                }
            }
        }
    }

    component CompareHeaderGhostButton inherits Rectangle {
        in property <string> label;
        in property <bool> enabled: true;
        in property <length> button_width: 152px;
        in property <string> tooltip_text: "";
        in property <string> icon_kind: "back";
        callback tapped();
        callback tooltip_requested(string, length, length, length);
        callback tooltip_closed();

        property <bool> hovered: button_touch_area.has_hover && root.enabled;
        property <length> horizontal_padding: 8px;
        property <length> icon_width: root.icon_kind == "reset" ? 10px : 9px;
        property <length> icon_height: root.icon_kind == "lock" || root.icon_kind == "unlock"
            ? 10px
            : (root.icon_kind == "recenter" ? 11px : 9px);
        property <length> icon_gap: root.icon_kind == "none" ? 0px : 6px;
        property <length> text_x: root.icon_kind == "none"
            ? root.horizontal_padding
            : root.horizontal_padding + root.icon_width + root.icon_gap;

        width: root.button_width;
        height: 20px;
        border-width: 0px;
        border-radius: 6px;
        background: transparent;
        clip: true;
        opacity: root.enabled ? 1 : 0.52;

        Rectangle {
            width: parent.width;
            height: parent.height;
            border-radius: 6px;
            border-width: root.hovered ? 1px : 0px;
            border-color: #d7e4f0;
            background: rgba(237, 244, 252, 0.72);
            opacity: root.hovered ? 1 : 0;

            animate opacity, border-width {
                duration: 140ms;
            }
        }

        if root.icon_kind == "back" : Path {
            x: root.horizontal_padding;
            y: (parent.height - root.icon_height) / 2;
            width: root.icon_width;
            height: root.icon_height;
            viewbox-width: 9;
            viewbox-height: 9;
            fill: transparent;
            stroke: root.hovered ? #4f6b84 : #6f8397;
            stroke-width: 1.1px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo { x: 7.0; y: 1.0; }
            LineTo { x: 2.5; y: 4.5; }
            LineTo { x: 7.0; y: 8.0; }
        }

        if root.icon_kind == "up" : Path {
            x: root.horizontal_padding;
            y: (parent.height - root.icon_height) / 2;
            width: root.icon_width;
            height: root.icon_height;
            viewbox-width: 9;
            viewbox-height: 9;
            fill: transparent;
            stroke: root.hovered ? #4f6b84 : #6f8397;
            stroke-width: 1.1px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo { x: 4.5; y: 1.2; }
            LineTo { x: 1.5; y: 4.4; }
            LineTo { x: 4.0; y: 4.4; }
            LineTo { x: 4.0; y: 8.0; }
            MoveTo { x: 4.5; y: 1.2; }
            LineTo { x: 7.5; y: 4.4; }
        }

        if root.icon_kind == "lock" || root.icon_kind == "unlock" : Rectangle {
            x: root.horizontal_padding + 1px;
            y: (parent.height - root.icon_height) / 2 + 3px;
            width: root.icon_width - 1px;
            height: root.icon_height - 4px;
            border-width: 1px;
            border-radius: 2px;
            border-color: root.hovered ? #4f6b84 : #6f8397;
            background: transparent;

            if root.icon_kind == "lock" : Path {
                x: 1px;
                y: -4px;
                width: 6px;
                height: 6px;
                viewbox-width: 6;
                viewbox-height: 6;
                fill: transparent;
                stroke: root.hovered ? #4f6b84 : #6f8397;
                stroke-width: 1px;
                stroke-line-cap: round;
                stroke-line-join: round;
                MoveTo { x: 1; y: 3.5; }
                LineTo { x: 1; y: 2.5; }
                CubicTo { x: 1; y: 1.0; control-1-x: 1; control-1-y: 1.4; control-2-x: 1.6; control-2-y: 1.0; }
                CubicTo { x: 5; y: 2.5; control-1-x: 2.4; control-1-y: 1.0; control-2-x: 5; control-2-y: 1.4; }
                LineTo { x: 5; y: 3.5; }
            }

            if root.icon_kind == "unlock" : Path {
                x: 1px;
                y: -4px;
                width: 6px;
                height: 6px;
                viewbox-width: 6;
                viewbox-height: 6;
                fill: transparent;
                stroke: root.hovered ? #4f6b84 : #6f8397;
                stroke-width: 1px;
                stroke-line-cap: round;
                stroke-line-join: round;
                MoveTo { x: 2; y: 3.5; }
                LineTo { x: 2; y: 2.4; }
                CubicTo { x: 5; y: 2.4; control-1-x: 2; control-1-y: 1.1; control-2-x: 5; control-2-y: 1.2; }
                LineTo { x: 5; y: 3.5; }
            }
        }

        if root.icon_kind == "reset" : Path {
            x: root.horizontal_padding;
            y: (parent.height - root.icon_height) / 2;
            width: root.icon_width;
            height: root.icon_height;
            viewbox-width: 10;
            viewbox-height: 10;
            fill: transparent;
            stroke: root.hovered ? #4f6b84 : #6f8397;
            stroke-width: 1px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo { x: 2.0; y: 3.0; }
            LineTo { x: 2.0; y: 1.5; }
            LineTo { x: 4.0; y: 1.5; }
            MoveTo { x: 2.1; y: 3.0; }
            CubicTo { x: 8.0; y: 4.8; control-1-x: 2.1; control-1-y: 1.8; control-2-x: 7.2; control-2-y: 2.0; }
            CubicTo { x: 3.8; y: 8.3; control-1-x: 8.6; control-1-y: 7.0; control-2-x: 6.2; control-2-y: 8.6; }
        }

        if root.icon_kind == "recenter" : Rectangle {
            x: root.horizontal_padding;
            y: (parent.height - root.icon_height) / 2;
            width: root.icon_width;
            height: root.icon_height;
            background: transparent;

            Rectangle {
                x: 3px;
                y: 3px;
                width: 3px;
                height: 3px;
                border-radius: 2px;
                background: root.hovered ? #4f6b84 : #6f8397;
            }

            Rectangle { x: 0px; y: 4px; width: 2px; height: 1px; background: root.hovered ? #4f6b84 : #6f8397; }
            Rectangle { x: 7px; y: 4px; width: 2px; height: 1px; background: root.hovered ? #4f6b84 : #6f8397; }
            Rectangle { x: 4px; y: 0px; width: 1px; height: 2px; background: root.hovered ? #4f6b84 : #6f8397; }
            Rectangle { x: 4px; y: 7px; width: 1px; height: 2px; background: root.hovered ? #4f6b84 : #6f8397; }
        }

        Text {
            x: root.text_x;
            width: max(0px, parent.width - root.text_x - root.horizontal_padding);
            height: parent.height;
            text: root.label;
            color: root.hovered ? #4f6b84 : #607489;
            font-size: 11px;
            font-weight: 500;
            horizontal-alignment: left;
            vertical-alignment: center;
            overflow: elide;
        }

        button_touch_area := TouchArea {
            width: parent.width;
            height: parent.height;
            enabled: root.enabled || root.tooltip_text != "";
            clicked => {
                if root.enabled {
                    root.tapped();
                }
            }

            changed has-hover => {
                if self.has-hover && root.tooltip_text != "" {
                    root.tooltip_requested(
                        root.tooltip_text,
                        self.absolute-position.x + 6px,
                        self.absolute-position.y,
                        self.absolute-position.y + self.height,
                    );
                } else {
                    root.tooltip_closed();
                }
            }

            pointer-event(event) => {
                if event.kind == PointerEventKind.cancel {
                    root.tooltip_closed();
                }
            }
        }
    }

    component CompareBreadcrumbChevron inherits Rectangle {
        width: 10px;
        height: 20px;
        background: transparent;

        Text {
            width: parent.width;
            height: parent.height;
            text: ">";
            color: #97a8b8;
            font-size: 12px;
            font-weight: 500;
            horizontal-alignment: center;
            vertical-alignment: center;
        }
    }

    component CompareBreadcrumbSegment inherits Rectangle {
        in property <string> label;
        in property <bool> active: false;
        in property <bool> enabled: true;
        callback tapped();

        property <bool> interactive: root.enabled && !root.active;
        property <bool> hovered: segment_touch_area.has_hover && root.interactive;
        property <length> segment_width: min(
            max(24px, measure_text.preferred-width + 8px),
            180px,
        );

        width: root.segment_width;
        height: 20px;
        border-width: 0px;
        border-radius: 5px;
        background: transparent;
        clip: true;
        opacity: root.active || root.interactive ? 1 : 0.78;

        Rectangle {
            width: parent.width;
            height: parent.height;
            border-radius: 5px;
            background: rgba(232, 239, 247, 0.62);
            opacity: root.hovered ? 1 : 0;

            animate opacity {
                duration: 120ms;
            }
        }

        measure_text := Text {
            visible: false;
            width: 0px;
            height: 0px;
            text: root.label;
            font-size: 12px;
            font-weight: root.active ? 650 : 500;
        }

        Text {
            x: 4px;
            width: max(0px, parent.width - 8px);
            height: parent.height;
            text: root.label;
            color: root.active
                ? #2d5b86
                : (root.hovered ? #476885 : #63788d);
            font-size: 12px;
            font-weight: root.active ? 650 : 500;
            vertical-alignment: center;
            overflow: elide;
        }

        segment_touch_area := TouchArea {
            enabled: root.interactive;
            clicked => {
                root.tapped();
            }
        }
    }

    component CompareBreadcrumbNavButton inherits Rectangle {
        in property <string> direction: "left";
        in property <bool> enabled: true;
        callback tapped();
        callback double_tapped();

        property <bool> hovered: nav_touch_area.has_hover && root.enabled;

        width: 22px;
        height: 22px;
        border-width: root.hovered ? 1px : 0px;
        border-radius: 11px;
        border-color: rgba(180, 194, 208, 0.76);
        background: root.hovered ? rgba(248, 251, 255, 0.96) : rgba(243, 248, 252, 0.72);
        opacity: root.enabled ? 1 : 0.36;
        clip: true;

        animate opacity, border-width {
            duration: 120ms;
        }

        if root.direction == "left" : Path {
            x: (parent.width - 6px) / 2;
            y: (parent.height - 10px) / 2;
            width: 6px;
            height: 10px;
            viewbox-width: 6;
            viewbox-height: 10;
            fill: transparent;
            stroke: root.hovered ? #486988 : #6d8297;
            stroke-width: 1.1px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo { x: 5; y: 1; }
            LineTo { x: 1; y: 5; }
            LineTo { x: 5; y: 9; }
        }

        if root.direction == "right" : Path {
            x: (parent.width - 6px) / 2;
            y: (parent.height - 10px) / 2;
            width: 6px;
            height: 10px;
            viewbox-width: 6;
            viewbox-height: 10;
            fill: transparent;
            stroke: root.hovered ? #486988 : #6d8297;
            stroke-width: 1.1px;
            stroke-line-cap: round;
            stroke-line-join: round;

            MoveTo { x: 1; y: 1; }
            LineTo { x: 5; y: 5; }
            LineTo { x: 1; y: 9; }
        }

        nav_touch_area := TouchArea {
            width: parent.width;
            height: parent.height;
            enabled: true;

            clicked => {
                if root.enabled {
                    root.tapped();
                }
            }

            double-clicked => {
                if root.enabled {
                    root.double_tapped();
                }
            }
        }
    }

    component CompareBreadcrumbViewport inherits Rectangle {
        in property <[string]> labels;
        in property <[string]> paths;
        callback segment_requested(string);

        property <length> scroll_step: 80px;
        property <duration> motion_duration: 150ms;
        property <length> scroll_offset: 0px;
        property <bool> follow_active_tail: true;
        property <length> content_width: breadcrumb_row.preferred-width;
        property <length> max_scroll: max(0px, root.content_width - root.width);
        property <length> effective_scroll_offset: root.follow_active_tail
            ? root.max_scroll
            : min(root.scroll_offset, root.max_scroll);
        property <bool> has_overflow: root.max_scroll > 1px;
        property <bool> can_scroll_left: root.effective_scroll_offset > 1px;
        property <bool> can_scroll_right: root.max_scroll > root.effective_scroll_offset + 1px;

        function clamp_scroll(target: length) -> length {
            return min(max(0px, target), root.max_scroll);
        }

        function set_scroll(target: length, animated: bool) {
            root.follow_active_tail = false;
            root.motion_duration = animated ? 150ms : 0ms;
            root.scroll_offset = root.clamp_scroll(target);
        }

        function jump_to_start() {
            root.follow_active_tail = false;
            root.motion_duration = 0ms;
            root.scroll_offset = 0px;
        }

        function jump_to_end() {
            root.follow_active_tail = true;
            root.motion_duration = 0ms;
            root.scroll_offset = root.max_scroll;
        }

        changed paths => {
            root.jump_to_end();
        }

        changed max_scroll => {
            if root.follow_active_tail {
                root.scroll_offset = root.max_scroll;
            } else if root.scroll_offset > root.max_scroll {
                root.scroll_offset = root.max_scroll;
            }
        }

        background: transparent;
        clip: true;

        viewport_touch_area := TouchArea {
            width: parent.width;
            height: parent.height;
            enabled: root.has_overflow;

            scroll-event(event) => {
                if !root.has_overflow {
                    return reject;
                }
                if event.delta-x != 0px {
                    root.set_scroll(root.effective_scroll_offset - event.delta-x, true);
                    return accept;
                }
                if event.delta-y != 0px {
                    root.set_scroll(root.effective_scroll_offset - event.delta-y, true);
                    return accept;
                }
                reject
            }
        }

        Rectangle {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            background: transparent;
            clip: true;

            breadcrumb_track := Rectangle {
                x: -root.effective_scroll_offset;
                y: 0px;
                width: max(parent.width, root.content_width);
                height: parent.height;
                background: transparent;

                animate x {
                    duration: root.motion_duration;
                    easing: ease;
                }

                breadcrumb_row := HorizontalLayout {
                    padding-left: 2px;
                    padding-right: 2px;
                    spacing: 6px;

                    for segment_path[index] in root.paths : HorizontalLayout {
                        spacing: 6px;

                        if index > 0 : CompareBreadcrumbChevron {}

                        CompareBreadcrumbSegment {
                            label: root.labels[index];
                            active: index + 1 == root.paths.length;
                            enabled: index + 1 < root.paths.length;
                            tapped => {
                                root.segment_requested(segment_path);
                            }
                        }
                    }
                }
            }
        }

        if root.has_overflow : Rectangle {
            width: parent.width;
            height: parent.height;
            background: transparent;

            CompareBreadcrumbNavButton {
                x: 2px;
                y: (parent.height - self.height) / 2;
                direction: "left";
                enabled: root.can_scroll_left;
                tapped => {
                    root.set_scroll(root.effective_scroll_offset - root.scroll_step, true);
                }
                double_tapped => {
                    root.jump_to_start();
                }
            }

            CompareBreadcrumbNavButton {
                x: parent.width - self.width - 2px;
                y: (parent.height - self.height) / 2;
                direction: "right";
                enabled: root.can_scroll_right;
                tapped => {
                    root.set_scroll(root.effective_scroll_offset + root.scroll_step, true);
                }
                double_tapped => {
                    root.jump_to_end();
                }
            }
        }
    }

    component TitleBarSurface inherits Rectangle {
        in property <length> leading_inset: 0px;
        in property <string> title_text: "Folder Compare";
        in property <string> sidebar_label: "Hide Sidebar";
        in property <bool> sidebar_active: true;
        callback sidebar_tapped();
        callback settings_tapped();
        callback drag_requested();
        callback sidebar_tooltip_requested(string, length, length, length);
        callback tooltip_closed();

        background: transparent;
        clip: true;

        TouchArea {
            width: parent.width;
            height: parent.height;

            pointer-event(event) => {
                if event.button == PointerEventButton.left && event.kind == PointerEventKind.down {
                    root.drag_requested();
                }
            }
        }

        Rectangle {
            x: 0px;
            y: parent.height - 1px;
            width: parent.width;
            height: 1px;
            background: #e6ebf2;
        }

        HorizontalLayout {
            padding-left: 4px;
            padding-right: 10px;
            padding-top: 5px;
            padding-bottom: 4px;
            spacing: 6px;

            Rectangle {
                width: root.leading_inset;
                background: transparent;
            }

            SidebarChromeButton {
                sidebar_visible: root.sidebar_active;
                tooltip_text: root.sidebar_label;
                tapped => {
                    root.sidebar_tapped();
                }
                tooltip_requested(text, anchor_x, anchor_top, anchor_bottom) => {
                    root.sidebar_tooltip_requested(text, anchor_x, anchor_top, anchor_bottom);
                }
                tooltip_closed => {
                    root.tooltip_closed();
                }
            }

            Text {
                text: root.title_text;
                font-size: 14px;
                color: #40505f;
                vertical-alignment: center;
            }

            Rectangle {
                horizontal-stretch: 1;
            }

            ToolButton {
                label: "Settings";
                width: 112px;
                button_min_width: 84px;
                control_height: 24px;
                label_font_size: 13px;
                tapped => {
                    root.settings_tapped();
                }
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

    component WorkspaceSessionTabButton inherits Rectangle {
        in property <string> label;
        in property <string> kind: "file";
        in property <bool> active: false;
        in property <bool> closable: true;
        callback tapped();
        callback close_requested();

        property <bool> close_hovered: false;
        property <bool> hovered: tab_touch_area.has_hover || root.close_hovered;
        property <length> close_slot_width: root.closable ? 20px : 0px;
        property <brush> active_fill: root.kind == "compare-tree" ? #eef4fb : #f8fbfe;
        property <brush> inactive_fill: root.kind == "compare-tree" ? #f2f6fa : #f4f7fb;
        property <brush> active_border: root.kind == "compare-tree" ? #c4d4e6 : #d5dee9;
        property <brush> inactive_border: root.kind == "compare-tree" ? #d3dee9 : #d8e0e9;

        measure_text := Text {
            visible: false;
            text: root.label;
            font-size: 13px;
            font-weight: root.active ? 600 : 500;
        }

        width: min(
            188px,
            max(104px, measure_text.preferred-width + root.close_slot_width + 26px),
        );
        height: 28px;
        border-width: 1px;
        border-radius: 6px;
        border-color: root.active ? root.active_border : root.inactive_border;
        background: root.active ? root.active_fill : root.inactive_fill;

        // Keep the whole-tab hit area behind the dedicated close target so the
        // close button is not shadowed by the selection hit region.
        tab_touch_area := TouchArea {
            width: parent.width;
            height: parent.height;
            clicked => {
                root.tapped();
            }
        }

        HorizontalLayout {
            padding-left: 10px;
            padding-right: 6px;
            spacing: 6px;

            Text {
                text: root.label;
                color: root.active
                    ? (root.kind == "compare-tree" ? #284c6f : #32485d)
                    : #5b6d80;
                font-size: 13px;
                font-weight: root.active ? 600 : 500;
                horizontal-stretch: 1;
                vertical-alignment: center;
                overflow: elide;
            }

            if root.closable : Rectangle {
                width: root.close_slot_width;
                height: parent.height;
                background: transparent;

                Text {
                    width: parent.width;
                    height: parent.height;
                    text: "×";
                    color: root.hovered
                        ? (root.kind == "compare-tree" ? #244a6c : #4a6077)
                        : #8393a3;
                    font-size: 15px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }

                close_touch_area := TouchArea {
                    width: parent.width;
                    height: parent.height;
                    enabled: root.closable;
                    changed has-hover => {
                        root.close_hovered = self.has_hover;
                    }
                    clicked => {
                        root.close_requested();
                    }
                }
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
            ? UiPalette.status_pill_tone_different_border
            : (root.tone == "equal"
                ? UiPalette.status_pill_tone_equal_border
                : (root.tone == "left"
                    ? UiPalette.status_pill_tone_left_border
                    : (root.tone == "right"
                        ? UiPalette.status_pill_tone_right_border
                        : (root.tone == "warn"
                            ? UiPalette.status_pill_tone_warn_border
                            : (root.tone == "error"
                                ? UiPalette.status_pill_tone_error_border
                                : (root.tone == "info"
                                    ? UiPalette.status_pill_tone_info_border
                                    : UiPalette.status_pill_tone_neutral_border))))));
        background: root.tone == "different"
            ? UiPalette.status_pill_tone_different_background
            : (root.tone == "equal"
                ? UiPalette.status_pill_tone_equal_background
                : (root.tone == "left"
                    ? UiPalette.status_pill_tone_left_background
                    : (root.tone == "right"
                        ? UiPalette.status_pill_tone_right_background
                        : (root.tone == "warn"
                            ? UiPalette.status_pill_tone_warn_background
                            : (root.tone == "error"
                                ? UiPalette.status_pill_tone_error_background
                                : (root.tone == "info"
                                    ? UiPalette.status_pill_tone_info_background
                                    : UiPalette.status_pill_tone_neutral_background))))));
        HorizontalLayout {
            width: parent.width;
            height: parent.height;
            padding-left: 4px;
            padding-right: 4px;
            spacing: 6px;
            Text {
                text: root.label;
                horizontal-alignment: center;
                vertical-alignment: center;
                color: root.tone == "different"
                    ? UiPalette.status_pill_tone_different_text
                    : (root.tone == "equal"
                        ? UiPalette.status_pill_tone_equal_text
                        : (root.tone == "left"
                            ? UiPalette.status_pill_tone_left_text
                            : (root.tone == "right"
                                ? UiPalette.status_pill_tone_right_text
                                : (root.tone == "warn"
                                    ? UiPalette.status_pill_tone_warn_text
                                    : (root.tone == "error"
                                        ? UiPalette.status_pill_tone_error_text
                                        : (root.tone == "info"
                                            ? UiPalette.status_pill_tone_info_text
                                            : UiPalette.status_pill_tone_neutral_text))))));
                font-size: 11px;
            }
        }
    }

    component HighlightTextLabel inherits Rectangle {
        in property <string> text;
        in property <bool> highlight: false;
        in property <brush> text_color: #2f3f50;
        in property <brush> highlight_fill: rgba(231, 221, 176, 0.45);
        in property <length> font_size: 12px;
        in property <int> font_weight: 400;
        out property <bool> is_truncated: label_text.preferred-width > self.width + 1px;

        clip: true;
        height: max(16px, label_text.preferred-height + 2px);

        Rectangle {
            visible: root.highlight && root.text != "";
            x: 0px;
            y: 1px;
            width: min(parent.width, label_text.preferred-width);
            height: max(0px, parent.height - 2px);
            border-radius: 4px;
            background: root.highlight_fill;
        }

        label_text := Text {
            width: parent.width;
            height: parent.height;
            text: root.text;
            color: root.text_color;
            font-size: root.font_size;
            font-weight: root.font_weight;
            vertical-alignment: center;
            overflow: elide;
        }
    }

    component TooltipBubble inherits Rectangle {
        in property <string> text;
        in property <length> max_panel_width: 520px;
        property <length> horizontal_padding: 10px;
        property <length> vertical_padding: 7px;

        border-width: 1px;
        border-radius: 6px;
        border-color: UiPalette.tooltip_border;
        background: UiPalette.tooltip_background;
        clip: true;

        width: min(
            root.max_panel_width,
            max(96px, bubble_text.preferred-width + root.horizontal_padding * 2)
        );
        height: max(28px, bubble_text.preferred-height + root.vertical_padding * 2);

        Rectangle {
            x: 1px;
            y: 1px;
            width: max(0px, parent.width - 2px);
            height: min(18px, max(0px, parent.height - 2px));
            border-radius: 5px;
            background: UiPalette.tooltip_inner_highlight;
        }

        bubble_text := Text {
            x: root.horizontal_padding;
            y: root.vertical_padding;
            width: max(0px, parent.width - root.horizontal_padding * 2);
            text: root.text;
            color: UiPalette.tooltip_text;
            font-size: 12px;
            wrap: word-wrap;
        }
    }

    component LoadingMask inherits Rectangle {
        in property <string> message;
        in property <length> corner_radius: 6px;

        background: UiPalette.loading_mask_overlay;
        border-radius: root.corner_radius;
        clip: true;

        Rectangle {
            width: min(340px, max(200px, parent.width - 40px));
            height: 52px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            border-width: 1px;
            border-radius: 6px;
            border-color: UiPalette.loading_mask_panel_border;
            background: UiPalette.loading_mask_panel_background;

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
                    color: UiPalette.loading_mask_message_text;
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

        border-width: 0px;
        border-radius: 0px;
        border-color: transparent;
        background: transparent;
        clip: true;

        property <brush> panel_border: root.tone == "error"
            ? UiPalette.state_shell_tone_error_border
            : (root.tone == "warn"
                ? UiPalette.state_shell_tone_warn_border
                : (root.tone == "info"
                    ? UiPalette.state_shell_tone_info_border
                    : (root.tone == "success"
                        ? UiPalette.state_shell_tone_success_border
                        : UiPalette.state_shell_tone_neutral_border)));
        property <brush> panel_background: root.tone == "error"
            ? UiPalette.state_shell_tone_error_background
            : (root.tone == "warn"
                ? UiPalette.state_shell_tone_warn_background
                : (root.tone == "info"
                    ? UiPalette.state_shell_tone_info_background
                    : (root.tone == "success"
                        ? UiPalette.state_shell_tone_success_background
                        : UiPalette.state_shell_tone_neutral_background)));
        property <brush> accent_color: root.tone == "error"
            ? UiPalette.state_shell_tone_error_accent
            : (root.tone == "warn"
                ? UiPalette.state_shell_tone_warn_accent
                : (root.tone == "info"
                    ? UiPalette.state_shell_tone_info_accent
                    : (root.tone == "success"
                        ? UiPalette.state_shell_tone_success_accent
                        : UiPalette.state_shell_tone_neutral_accent)));

        property <brush> title_color: root.tone == "error"
            ? UiPalette.state_shell_tone_error_title_text
            : (root.tone == "warn"
                ? UiPalette.state_shell_tone_warn_title_text
                : (root.tone == "info"
                    ? UiPalette.state_shell_tone_info_title_text
                    : (root.tone == "success"
                        ? UiPalette.state_shell_tone_success_title_text
                        : UiPalette.state_shell_tone_neutral_title_text)));
        property <length> embedded_accent_width: root.tone == "neutral" ? 0px : 1px;
        property <brush> embedded_accent_color: root.panel_border;
        property <brush> embedded_title_color: root.tone == "error"
            ? UiPalette.state_shell_tone_error_title_text
            : UiPalette.state_shell_tone_neutral_title_text;
        property <brush> embedded_background: root.tone == "neutral"
            ? #fcfdff
            : root.panel_background;
        property <length> embedded_horizontal_padding: 12px;
        property <length> embedded_top_padding: 12px;
        property <length> embedded_bottom_padding: 14px;
        property <length> embedded_content_max_width: 720px;

        if root.embedded : Rectangle {
            width: parent.width;
            height: parent.height;
            background: root.embedded_background;
            clip: true;

            Rectangle {
                visible: root.embedded_accent_width != 0px;
                x: 0px;
                y: 0px;
                width: root.embedded_accent_width;
                height: parent.height;
                background: root.embedded_accent_color;
            }

            HorizontalLayout {
                width: parent.width;
                height: parent.height;
                padding-left: root.embedded_horizontal_padding;
                padding-right: root.embedded_horizontal_padding;
                padding-top: root.embedded_top_padding;
                padding-bottom: root.embedded_bottom_padding;
                spacing: 0px;

                Rectangle {
                    width: min(max(0px, parent.width), root.embedded_content_max_width);
                    height: parent.height;
                    background: transparent;

                    VerticalLayout {
                        width: parent.width;
                        height: parent.height;
                        spacing: 8px;

                        HorizontalLayout {
                            height: 18px;
                            spacing: 0px;

                            StatusPill {
                                label: root.state_label;
                                tone: root.tone;
                            }

                            Rectangle {
                                horizontal-stretch: 1;
                                background: transparent;
                            }
                        }

                        Text {
                            text: root.title;
                            color: root.embedded_title_color;
                            font-size: 16px;
                            font-weight: 600;
                            wrap: word-wrap;
                            horizontal-stretch: 1;
                        }

                        Text {
                            visible: root.body != "";
                            text: root.body;
                            color: UiPalette.state_shell_body_text;
                            font-size: 13px;
                            wrap: word-wrap;
                            horizontal-stretch: 1;
                        }

                        Rectangle {
                            visible: root.note != "";
                            height: 1px;
                            background: UiPalette.state_shell_note_separator;
                            horizontal-stretch: 1;
                        }

                        Text {
                            visible: root.note != "";
                            text: root.note;
                            color: UiPalette.state_shell_note_text;
                            font-size: 12px;
                            wrap: word-wrap;
                            horizontal-stretch: 1;
                        }

                        Rectangle {
                            vertical-stretch: 1;
                            background: transparent;
                        }
                    }
                }

                Rectangle {
                    horizontal-stretch: 1;
                    background: transparent;
                }
            }
        }

        if !root.embedded : Rectangle {
            width: parent.width;
            height: parent.height;
            border-width: 1px;
            border-radius: 6px;
            border-color: root.panel_border;
            background: root.panel_background;
            clip: true;

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
                    ? UiPalette.state_shell_tone_error_header_background
                    : (root.tone == "warn"
                        ? UiPalette.state_shell_tone_warn_header_background
                        : (root.tone == "info"
                            ? UiPalette.state_shell_tone_info_header_background
                            : (root.tone == "success"
                                ? UiPalette.state_shell_tone_success_header_background
                                : UiPalette.state_shell_tone_neutral_header_background)));

                Rectangle {
                    x: 0px;
                    y: parent.height - 1px;
                    width: parent.width;
                    height: 1px;
                    background: UiPalette.state_shell_header_separator;
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
                    color: UiPalette.state_shell_body_text;
                    font-size: 14px;
                    wrap: word-wrap;
                    horizontal-stretch: 1;
                }

                Rectangle {
                    visible: root.note != "";
                    height: 1px;
                    background: UiPalette.state_shell_note_separator;
                    horizontal-stretch: 1;
                }

                Text {
                    visible: root.note != "";
                    text: root.note;
                    color: UiPalette.state_shell_note_text;
                    font-size: 13px;
                    wrap: word-wrap;
                    horizontal-stretch: 1;
                }
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

        ContextMenuArea {
            enabled: !root.value.is-empty;

            Menu {
                MenuItem {
                    title: @tr("Copy");
                    enabled: !root.value.is-empty;
                    activated => {
                        text_input.copy();
                    }
                }

                MenuItem {
                    title: @tr("Select All");
                    enabled: !root.value.is-empty;
                    activated => {
                        text_input.focus();
                        text_input.select-all();
                    }
                }
            }

            text_input := TextInput {
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
    }

    component ApiKeyLineEdit inherits Rectangle {
        in-out property <string> text;
        in-out property <bool> revealed: false;
        in property <bool> enabled: true;
        in property <bool> read_only: false;
        out property <bool> has_focus: text_input.has-focus;
        property <bool> toggle_hovered: toggle_touch_area.has-hover && root.enabled;
        property <length> horizontal_padding: 10px;
        property <length> toggle_slot_width: 40px;
        property <length> toggle_gap: 8px;
        property <length> text_lane_width: max(1px, root.width - 2 * root.horizontal_padding - root.toggle_gap - root.toggle_slot_width);

        border-width: 1px;
        border-radius: 4px;
        border-color: root.has_focus ? #9bb7d3 : #d7dde7;
        background: root.enabled ? #ffffff : #f3f6fa;
        opacity: root.enabled ? 1 : 0.6;
        clip: true;
        forward-focus: text_input;
        accessible-role: AccessibleRole.text-input;
        accessible-enabled: root.enabled;
        accessible-read-only: root.read_only;
        accessible-value <=> text;
        accessible-action-set-value(v) => {
            text = v;
        }

        // Contract: API key input keeps native TextInput editing behavior,
        // but narrows hidden-state secret actions to paste-only.
        ContextMenuArea {
            enabled: root.enabled;

            Menu {
                if root.revealed : MenuItem {
                    title: @tr("Cut");
                    enabled: !root.read_only && root.enabled && !root.text.is-empty;
                    activated => {
                        text_input.cut();
                    }
                }

                if root.revealed : MenuItem {
                    title: @tr("Copy");
                    enabled: !root.text.is-empty;
                    activated => {
                        text_input.copy();
                    }
                }

                MenuItem {
                    title: @tr("Paste");
                    enabled: !root.read_only && root.enabled;
                    activated => {
                        text_input.paste();
                    }
                }

                if root.revealed : MenuItem {
                    title: @tr("Select All");
                    enabled: !root.text.is-empty;
                    activated => {
                        text_input.select-all();
                    }
                }
            }

            text_input := TextInput {
                property <length> computed_x;

                x: root.horizontal_padding + min(
                    0px,
                    max(root.text_lane_width - self.width - self.text-cursor-width, self.computed_x)
                );
                y: 0px;
                width: max(root.text_lane_width - self.text-cursor-width, self.preferred-width);
                height: parent.height;
                text <=> root.text;
                enabled: root.enabled;
                read-only: root.read_only;
                single-line: true;
                vertical-alignment: center;
                input-type: root.revealed ? InputType.text : InputType.password;
                color: #33485d;
                font-size: 13px;
                selection-background-color: #c9daec;
                selection-foreground-color: #23384d;
                accessible-role: none;

                cursor-position-changed(cursor-position) => {
                    if cursor-position.x + self.computed_x < 0px {
                        self.computed_x = -cursor-position.x;
                    } else if cursor-position.x + self.computed_x > root.text_lane_width - self.text-cursor-width {
                        self.computed_x = root.text_lane_width - cursor-position.x - self.text-cursor-width;
                    }
                }

                key-pressed(event) => {
                    if !root.revealed && event.modifiers.control
                        && (event.text == "a" || event.text == "A"
                            || event.text == "c" || event.text == "C"
                            || event.text == "x" || event.text == "X") {
                        return accept;
                    }
                    reject
                }
            }

            toggle_label := Text {
                x: parent.width - root.horizontal_padding - root.toggle_slot_width;
                y: 0px;
                width: root.toggle_slot_width;
                height: parent.height;
                text: root.revealed ? "Hide" : "Show";
                color: root.toggle_hovered ? #2f5a83 : #5d6f82;
                horizontal-alignment: center;
                vertical-alignment: center;
                font-size: 12px;
                font-weight: 600;
            }

            toggle_touch_area := TouchArea {
                x: toggle_label.x;
                y: 0px;
                width: toggle_label.width;
                height: parent.height;
                enabled: root.enabled;
                clicked => {
                    root.revealed = !root.revealed;
                    text_input.focus();
                }
            }
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
        callback context_menu_requested(length, length, string, string, string, string);

        border-width: 1px;
        border-radius: 8px;
        border-color: root.tone == "error"
            ? UiPalette.result_section_tone_error_border
            : (root.tone == "warn"
                ? UiPalette.result_section_tone_warn_border
                : (root.tone == "success"
                    ? UiPalette.result_section_tone_success_border
                    : UiPalette.result_section_tone_neutral_border));
        background: root.tone == "error"
            ? UiPalette.result_section_tone_error_background
            : (root.tone == "warn"
                ? UiPalette.result_section_tone_warn_background
                : (root.tone == "success"
                    ? UiPalette.result_section_tone_success_background
                    : UiPalette.result_section_tone_neutral_background));

        VerticalLayout {
            padding: 14px;
            spacing: 8px;

            header_surface := Rectangle {
                height: 20px;
                background: transparent;

                HorizontalLayout {
                    spacing: 8px;

                    header_context_lane := Rectangle {
                        background: transparent;
                        horizontal-stretch: 1;

                        Text {
                            x: 0px;
                            y: 0px;
                            width: parent.width;
                            height: parent.height;
                            text: root.section_label;
                            color: #708193;
                            font-size: 11px;
                            font-weight: 600;
                            horizontal-alignment: left;
                            vertical-alignment: center;
                        }

                        TouchArea {
                            pointer-event(event) => {
                                if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                    root.context_menu_requested(
                                        self.absolute-position.x + self.mouse-x,
                                        self.absolute-position.y + self.mouse-y,
                                        root.section_label,
                                        root.title,
                                        root.body,
                                        root.copy_value,
                                    );
                                }
                            }
                        }
                    }

                    Rectangle {
                        visible: root.copy_value != "";
                        width: copy_text.preferred-width;
                        height: 20px;
                        background: transparent;

                        copy_text := Text {
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
            }

            if root.title != "" : SelectableSectionText {
                value: root.title;
                foreground: #2f4a63;
                font-size: 18px;
                font-weight: 600;
                horizontal-stretch: 1;
            }

            if root.body != "" : SelectableSectionText {
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
            width: parent.width;
            height: parent.height;
            enabled: root.enabled;
            clicked => {
                root.tapped();
            }
        }
    }

    component ContextMenuActionItem inherits Rectangle {
        in property <string> label;
        in property <string> action_id;
        in property <bool> enabled: true;
        callback activated(string);

        property <bool> hovered: action_touch_area.has_hover && root.enabled;
        property <length> item_inset: 2px;

        height: 36px;
        background: transparent;
        opacity: root.enabled ? 1 : 0.94;

        item_surface := Rectangle {
            x: root.item_inset;
            y: 2px;
            width: max(0px, parent.width - 2 * root.item_inset);
            height: max(0px, parent.height - 4px);
            border-width: root.hovered ? 1px : 0px;
            border-radius: 8px;
            border-color: UiPalette.context_menu_core_item_hover_border;
            background: root.hovered ? UiPalette.context_menu_core_item_hover : transparent;
            clip: true;

            Rectangle {
                visible: root.hovered;
                x: 0px;
                y: 7px;
                width: 3px;
                height: max(0px, parent.height - 14px);
                border-radius: 2px;
                background: UiPalette.context_menu_core_item_active_accent;
            }

            Text {
                text: root.label;
                x: 15px;
                y: 0px;
                width: max(0px, parent.width - 30px);
                height: parent.height;
                color: root.enabled
                    ? UiPalette.context_menu_core_text
                    : UiPalette.context_menu_core_disabled_text;
                vertical-alignment: center;
                overflow: elide;
                font-size: 13px;
                font-weight: root.hovered ? 600 : 500;
            }
        }

        action_touch_area := TouchArea {
            clicked => {
                if root.enabled {
                    root.activated(root.action_id);
                }
            }
        }
    }

    component AppLineEdit inherits LineEdit {
    }

    component TooltipLineEdit inherits Rectangle {
        in-out property <string> text;
        in property <string> placeholder_text;
        in property <bool> enabled: true;
        callback edited(string);
        callback tooltip_requested(string, length, length, length);
        callback tooltip_closed();

        out property <bool> has_focus: line_edit.has-focus;
        property <length> text_lane_padding: 18px;
        property <length> tooltip_x_inset: 8px;
        property <bool> can_show_tooltip: root.enabled
            && !root.has_focus
            && root.text != ""
            && text_probe.preferred-width > max(1px, line_edit.width - root.text_lane_padding);

        changed text => {
            if !root.can_show_tooltip {
                root.tooltip_closed();
            }
        }

        changed has_focus => {
            if root.has_focus {
                root.tooltip_closed();
            }
        }

        min-height: line_edit.min-height;
        max-height: line_edit.max-height;
        preferred-height: line_edit.preferred-height;
        forward-focus: line_edit;

        hover_sensor := TouchArea {
            width: parent.width;
            height: parent.height;
            enabled: root.enabled && root.text != "";

            changed has-hover => {
                if self.has-hover && root.can_show_tooltip {
                    root.tooltip_requested(
                        root.text,
                        self.absolute-position.x + root.tooltip_x_inset,
                        self.absolute-position.y,
                        self.absolute-position.y + self.height,
                    );
                } else {
                    root.tooltip_closed();
                }
            }

            pointer-event(event) => {
                if event.kind == PointerEventKind.move {
                    if self.has-hover && root.can_show_tooltip {
                        root.tooltip_requested(
                            root.text,
                            self.absolute-position.x + root.tooltip_x_inset,
                            self.absolute-position.y,
                            self.absolute-position.y + self.height,
                        );
                    } else {
                        root.tooltip_closed();
                    }
                } else if event.kind == PointerEventKind.down {
                    root.tooltip_closed();
                    line_edit.focus();
                }
            }

            line_edit := AppLineEdit {
                width: parent.width;
                height: parent.height;
                text <=> root.text;
                enabled: root.enabled;
                placeholder-text: root.placeholder_text;
                edited(value) => {
                    root.edited(value);
                }
            }
        }

        text_probe := Text {
            x: 0px;
            y: 0px;
            width: 0px;
            height: 0px;
            visible: false;
            text: root.text;
            font-size: line_edit.font-size;
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
        background: #f4f7fa;
        in property <bool> immersive_titlebar_enabled: false;
        in property <length> titlebar_visual_height: 34px;
        in property <length> titlebar_leading_inset: 0px;
        in property <length> sidebar_form_label_width: 52px;
        in property <length> sidebar_action_button_width: 72px;

        in-out property <string> left_root;
        in-out property <string> right_root;
        in property <bool> running;
        in property <string> status_text;
        in property <string> summary_text;
        in property <string> compact_summary_text;
        in property <string> compare_metrics_text;
        in property <string> compare_status_note_text;
        in property <bool> compare_status_has_detail;
        in property <string> compare_summary_copy_text;
        in property <string> compare_detail_copy_text;
        in property <string> warnings_text;
        in property <string> error_text;
        in property <bool> compare_truncated;
        in property <bool> compare_has_deferred;
        in property <bool> compare_has_oversized;
        in-out property <string> entry_filter;
        in-out property <string> entry_status_filter;
        in property <string> results_collection_text;
        in property <string> navigator_runtime_view_mode;
        in property <string> navigator_effective_view_mode;
        in property <bool> navigator_search_forces_flat_mode;
        in property <[string]> row_statuses;
        in property <[string]> row_paths;
        in property <[string]> row_details;
        in property <[string]> row_display_names;
        in property <[string]> row_parent_paths;
        in property <[string]> row_tooltip_texts;
        in property <[string]> row_secondary_texts;
        in property <[int]> row_source_indices;
        in property <[bool]> row_can_load_diff;
        in property <[bool]> row_display_name_matches;
        in property <[bool]> row_parent_path_matches;
        in property <[string]> tree_row_keys;
        in property <[string]> tree_row_display_names;
        in property <[string]> tree_row_statuses;
        in property <[string]> tree_row_tooltip_texts;
        in property <[int]> tree_row_depths;
        in property <[bool]> tree_row_is_directories;
        in property <[bool]> tree_row_is_expandable;
        in property <[bool]> tree_row_is_expanded;
        in property <[bool]> tree_row_is_selectable;
        in property <[int]> tree_row_source_indices;
        in property <[string]> compare_row_paths;
        in property <[int]> compare_row_depths;
        in property <[string]> compare_row_left_icons;
        in property <[string]> compare_row_left_names;
        in property <[bool]> compare_row_left_present;
        in property <[string]> compare_row_status_labels;
        in property <[string]> compare_row_status_tones;
        in property <[string]> compare_row_right_icons;
        in property <[string]> compare_row_right_names;
        in property <[bool]> compare_row_right_present;
        in property <[bool]> compare_row_is_directories;
        in property <[bool]> compare_row_is_expandable;
        in property <[bool]> compare_row_is_expanded;
        in property <int> compare_row_focused_index: -1;
        in property <bool> compare_file_view_active: false;
        in property <string> compare_file_summary_text;
        in property <string> compare_file_warning_text;
        in property <bool> compare_file_truncated: false;
        in property <bool> compare_file_has_rows: false;
        in property <string> compare_file_helper_text;
        in property <string> compare_file_shell_state_label;
        in property <string> compare_file_shell_state_tone;
        in property <string> compare_file_shell_title_text;
        in property <string> compare_file_shell_body_text;
        in property <string> compare_file_shell_note_text;
        in property <[string]> compare_file_row_kinds;
        in property <[string]> compare_file_row_relation_labels;
        in property <[string]> compare_file_row_relation_tones;
        in property <[string]> compare_file_row_left_line_nos;
        in property <[string]> compare_file_row_right_line_nos;
        in property <[string]> compare_file_row_left_prefixes;
        in property <[string]> compare_file_row_left_emphasis;
        in property <[string]> compare_file_row_left_suffixes;
        in property <[string]> compare_file_row_right_prefixes;
        in property <[string]> compare_file_row_right_emphasis;
        in property <[string]> compare_file_row_right_suffixes;
        in property <[bool]> compare_file_row_left_padding;
        in property <[bool]> compare_file_row_right_padding;
        in property <int> compare_file_left_content_width_px: 0;
        in property <int> compare_file_right_content_width_px: 0;
        in property <string> workspace_mode;
        in property <[string]> workspace_session_ids;
        in property <[string]> workspace_session_labels;
        in property <[string]> workspace_session_tooltips;
        in property <[string]> workspace_session_kinds;
        in property <[bool]> workspace_session_closable;
        in property <int> active_workspace_session_index: -1;
        in property <bool> workspace_sessions_visible: false;
        in property <string> compare_focus_path_raw;
        in property <string> compare_root_pair_text;
        in property <string> compare_view_current_path_text;
        in property <[string]> compare_view_breadcrumb_labels;
        in property <[string]> compare_view_breadcrumb_paths;
        in property <string> compare_view_target_status_label;
        in property <string> compare_view_target_status_tone;
        in property <string> compare_view_empty_title_text;
        in property <string> compare_view_empty_body_text;
        in property <bool> compare_view_has_targets: false;
        in property <bool> compare_view_can_go_up;
        in property <bool> compare_view_horizontal_scroll_locked: true;
        in property <int> compare_view_left_content_width_px: 0;
        in property <int> compare_view_right_content_width_px: 0;
        in property <bool> can_return_to_compare_view;
        in property <bool> diff_loading;
        in property <string> selected_relative_path;
        in property <string> selected_relative_path_raw;
        in property <string> file_view_title_text;
        in property <string> file_view_compare_status_label;
        in property <string> file_view_compare_status_tone;
        in property <string> file_view_path_context_text;
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
        in property <bool> show_hidden_files;
        in property <string> default_navigator_view_mode;
        in property <bool> sidebar_visible: true;
        in property <string> settings_error_text;
        in-out property <int> workspace_tab: 0;
        in-out property <bool> compare_status_details_expanded: false;
        in-out property <bool> settings_open: false;
        in-out property <int> settings_section: 0;
        in-out property <int> settings_provider_mode: 0;
        in-out property <string> settings_provider_endpoint;
        in-out property <string> settings_provider_api_key;
        in-out property <string> settings_provider_model;
        in-out property <string> settings_provider_timeout;
        in-out property <bool> settings_show_hidden_files: true;
        in-out property <int> settings_default_result_view: 0;
        in-out property <bool> settings_provider_show_api_key: false;
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
        in-out property <bool> context_menu_open: false;
        in-out property <length> context_menu_anchor_x: 0px;
        in-out property <length> context_menu_anchor_y: 0px;
        in property <string> context_menu_target_token: "";
        in property <[string]> context_menu_action_labels;
        in property <[string]> context_menu_action_ids;
        in property <[bool]> context_menu_action_enabled;
        in-out property <bool> tooltip_visible: false;
        in-out property <string> tooltip_text: "";
        in-out property <length> tooltip_anchor_x: 0px;
        in-out property <length> tooltip_anchor_top: 0px;
        in-out property <length> tooltip_anchor_bottom: 0px;
        in property <bool> workspace_session_confirm_open: false;
        in property <string> workspace_session_confirm_title;
        in property <string> workspace_session_confirm_body;
        in property <string> workspace_session_confirm_action_label;
        property <length> tooltip_gap: 6px;
        property <bool> compare_ready: !root.running && root.left_root != "" && root.right_root != "";
        property <string> compare_button_tooltip_text: root.running
            ? "Comparing folders..."
            : (root.left_root == "" && root.right_root == ""
                ? "Choose left and right folders to enable Compare."
                : (root.left_root == ""
                    ? "Choose a left folder to enable Compare."
                    : (root.right_root == ""
                        ? "Choose a right folder to enable Compare."
                        : "")));
        property <bool> has_selected_result: root.selected_row >= 0;
        property <bool> diff_shell_ready: root.diff_shell_state_token == "preview-ready"
            || root.diff_shell_state_token == "detailed-ready";
        property <bool> diff_show_shell: root.diff_shell_state_token == "no-selection"
            || root.diff_shell_state_token == "stale-selection"
            || root.diff_shell_state_token == "loading"
            || root.diff_shell_state_token == "unavailable"
            || root.diff_shell_state_token == "error"
            || (root.diff_shell_ready && !root.diff_has_rows);
        property <length> diff_number_column_width: 52px;
        property <length> diff_marker_column_width: 20px;
        property <length> diff_separator_width: 1px;
        property <length> diff_old_column_x: 0px;
        property <length> diff_first_separator_x: root.diff_old_column_x + root.diff_number_column_width;
        property <length> diff_new_column_x: root.diff_first_separator_x + root.diff_separator_width;
        property <length> diff_second_separator_x: root.diff_new_column_x + root.diff_number_column_width;
        property <length> diff_marker_column_x: root.diff_second_separator_x + root.diff_separator_width;
        property <length> diff_third_separator_x: root.diff_marker_column_x + root.diff_marker_column_width;
        property <length> diff_content_column_x: root.diff_third_separator_x + root.diff_separator_width;
        property <length> diff_header_separator_inset: 4px;
        property <length> diff_scrollbar_safe_inset: 18px;
        property <length> workbench_header_height: 66px;
        property <length> workbench_compare_context_header_height: 78px;
        property <length> compare_file_header_height: 82px;
        property <length> compare_workspace_header_height: 82px;
        property <length> workbench_helper_strip_height: 32px;
        property <length> workbench_action_strip_height: 30px;
        property <length> compare_navigation_lane_width: 124px;
        property <length> compare_navigation_button_width: 120px;
        property <length> compare_view_column_inset: 8px;
        property <length> compare_view_column_divider_width: 0px;
        property <length> compare_view_relation_column_width: 92px;
        property <length> workspace_session_strip_height: root.workspace_sessions_visible ? 34px : 0px;
        property <string> sidebar_toggle_label: root.sidebar_visible ? "Hide Sidebar" : "Show Sidebar";
        property <string> compare_view_empty_note_text: root.sidebar_visible
            ? "Use Compare Status -> Open root or Results / Navigator -> Open in Compare View to change targets."
            : "Show Sidebar, then use Compare Status or Results / Navigator to change targets.";
        property <string> diff_helper_strip_text: root.diff_shell_ready && root.diff_has_rows
            ? ("Select text or double-click a line number to copy the full row."
                + (root.diff_content_char_capacity > 112
                    ? " Long lines scroll horizontally."
                    : ""))
            : (root.diff_shell_note_text != ""
                ? root.diff_shell_note_text
                : root.diff_context_summary_text);
        function show_tooltip(
            text: string,
            anchor_x: length,
            anchor_top: length,
            anchor_bottom: length,
        ) {
            root.tooltip_text = text;
            root.tooltip_anchor_x = anchor_x;
            root.tooltip_anchor_top = anchor_top;
            root.tooltip_anchor_bottom = anchor_bottom;
            root.tooltip_visible = text != "";
        }
        function hide_tooltip() {
            root.tooltip_visible = false;
            root.tooltip_text = "";
        }
        public function ensure_flat_row_visible(target_row: int) {
            if root.navigator_effective_view_mode != "flat" {
                return;
            }
            flat_results_list.ensure_row_visible(target_row);
        }
        public function ensure_tree_row_visible(target_row: int) {
            if root.navigator_effective_view_mode != "tree" {
                return;
            }
            navigator_tree_view.ensure_visible_row(target_row);
        }
        public function ensure_compare_row_visible(target_row: int) {
            compare_workspace_view.ensure_row_visible(target_row);
        }
        public function focus_compare_rows() {
            compare_workspace_view.focus_rows();
        }
        function open_settings() {
            root.settings_provider_mode = root.analysis_remote_mode ? 1 : 0;
            root.settings_provider_endpoint = root.analysis_endpoint;
            root.settings_provider_api_key = root.analysis_api_key;
            root.settings_provider_model = root.analysis_model;
            root.settings_provider_timeout = root.analysis_timeout_text;
            root.settings_show_hidden_files = root.show_hidden_files;
            root.settings_default_result_view = root.default_navigator_view_mode == "flat" ? 1 : 0;
            root.settings_provider_show_api_key = false;
            root.settings_open = true;
            root.settings_clicked();
        }
        callback compare_clicked();
        callback left_browse_clicked();
        callback right_browse_clicked();
        callback filter_changed(string);
        callback status_filter_changed(string);
        callback navigator_view_mode_tree_requested();
        callback navigator_view_mode_flat_requested();
        callback navigator_tree_directory_toggled(string);
        callback navigator_tree_file_selected(int);
        callback navigator_tree_context_menu_requested(string, string, bool, int);
        callback row_selected(int);
        callback sidebar_toggle_requested();
        callback workspace_session_selected(string);
        callback workspace_session_close_requested(string);
        callback compare_root_view_requested();
        callback compare_view_up_requested();
        callback compare_view_breadcrumb_requested(string);
        callback compare_view_scroll_lock_toggled();
        callback compare_view_row_focused(string);
        callback compare_view_row_toggle_requested(string);
        callback compare_view_row_activated(string);
        callback compare_view_row_context_menu_requested(string);
        callback compare_file_back_requested();
        callback file_view_diff_requested();
        callback file_view_analysis_requested();
        callback workspace_session_confirmed();
        callback workspace_session_cancelled();
        callback copy_requested(string, string);
        callback analyze_clicked();
        callback analysis_provider_mock_selected();
        callback analysis_provider_openai_selected();
        callback analysis_endpoint_changed(string);
        callback analysis_api_key_changed(string);
        callback analysis_model_changed(string);
        callback settings_clicked();
        callback settings_save_clicked();
        callback settings_cancel_clicked();
        callback compare_status_context_menu_requested(string, string);
        callback results_context_menu_requested(int, string, string, string, bool);
        callback workspace_header_context_menu_requested(string, string, string, string, string);
        callback analysis_section_context_menu_requested(string, string, string, string);
        callback context_menu_close_requested();
        callback context_menu_action_triggered(string);
        callback titlebar_drag_requested();

        VerticalLayout {
            padding-top: root.immersive_titlebar_enabled ? 0px : 6px;
            padding-right: 6px;
            padding-bottom: 6px;
            padding-left: 6px;
            spacing: 4px;

            if root.immersive_titlebar_enabled : Rectangle {
                height: root.titlebar_visual_height;
                background: transparent;
                clip: false;

                TitleBarSurface {
                    x: -10px;
                    y: 0px;
                    width: parent.width + 20px;
                    height: parent.height;
                    leading_inset: root.titlebar_leading_inset;
                    sidebar_label: root.sidebar_toggle_label;
                    sidebar_active: root.sidebar_visible;
                    title_text: "Folder Compare";
                    drag_requested => {
                        root.titlebar_drag_requested();
                    }
                    sidebar_tapped => {
                        root.sidebar_toggle_requested();
                    }
                    sidebar_tooltip_requested(text, anchor_x, anchor_top, anchor_bottom) => {
                        root.show_tooltip(
                            text,
                            anchor_x - root.absolute-position.x,
                            anchor_top - root.absolute-position.y,
                            anchor_bottom - root.absolute-position.y,
                        );
                    }
                    tooltip_closed => {
                        root.hide_tooltip();
                    }
                    settings_tapped => {
                        root.open_settings();
                    }
                }
            }

            // Contract: app bar shell (title + global settings entry).
            if !root.immersive_titlebar_enabled : SectionCard {
                height: 32px;
                border-width: 0px;
                border-color: transparent;
                background: transparent;

                Rectangle {
                    x: 0px;
                    y: parent.height - 1px;
                    width: parent.width;
                    height: 1px;
                    background: #e6ebf2;
                }

                HorizontalLayout {
                    padding-left: 4px;
                    padding-right: 4px;
                    padding-top: 4px;
                    padding-bottom: 4px;
                    spacing: 6px;
                    SidebarChromeButton {
                        sidebar_visible: root.sidebar_visible;
                        tooltip_text: root.sidebar_toggle_label;
                        tapped => {
                            root.sidebar_toggle_requested();
                        }
                        tooltip_requested(text, anchor_x, anchor_top, anchor_bottom) => {
                            root.show_tooltip(
                                text,
                                anchor_x - root.absolute-position.x,
                                anchor_top - root.absolute-position.y,
                                anchor_bottom - root.absolute-position.y,
                            );
                        }
                        tooltip_closed => {
                            root.hide_tooltip();
                        }
                    }
                    Text {
                        text: "Folder Compare";
                        font-size: 14px;
                        color: #40505f;
                        vertical-alignment: center;
                    }
                    Rectangle {
                        horizontal-stretch: 1;
                    }
                    ToolButton {
                        label: "Settings";
                        width: 112px;
                        button_min_width: 84px;
                        control_height: 24px;
                        label_font_size: 13px;
                        tapped => {
                            root.open_settings();
                        }
                    }
                }
            }

            HorizontalLayout {
                vertical-stretch: 1;
                spacing: root.sidebar_visible ? 8px : 0px;

                        // Contract: sidebar shell.
                        // Hosts compare setup/status/filter/navigation controls; detailed file view stays in workspace.
                        sidebar_shell := Rectangle {
                            property <length> shell_width: root.sidebar_visible ? 360px : 0px;
                            horizontal-stretch: 0;
                            min-width: self.shell_width;
                            max-width: self.shell_width;
                            clip: true;
                            VerticalLayout {
                                spacing: 8px;

                                // Contract: Compare Inputs.
                                // Collects left/right roots and compare trigger; does not render compare results.
                                SectionCard {
                                    height: 146px;
                                    VerticalLayout {
                                        padding: 10px;
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
                                            TooltipLineEdit {
                                                text <=> root.left_root;
                                                enabled: !root.running;
                                                horizontal-stretch: 1;
                                                placeholder_text: "Choose left folder";
                                                tooltip_requested(value, anchor_x, anchor_y, anchor_bottom) => {
                                                    root.show_tooltip(
                                                        value,
                                                        anchor_x - root.absolute-position.x,
                                                        anchor_y - root.absolute-position.y,
                                                        anchor_bottom - root.absolute-position.y,
                                                    );
                                                }
                                                tooltip_closed => {
                                                    root.hide_tooltip();
                                                }
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
                                            TooltipLineEdit {
                                                text <=> root.right_root;
                                                enabled: !root.running;
                                                horizontal-stretch: 1;
                                                placeholder_text: "Choose right folder";
                                                tooltip_requested(value, anchor_x, anchor_y, anchor_bottom) => {
                                                    root.show_tooltip(
                                                        value,
                                                        anchor_x - root.absolute-position.x,
                                                        anchor_y - root.absolute-position.y,
                                                        anchor_bottom - root.absolute-position.y,
                                                    );
                                                }
                                                tooltip_closed => {
                                                    root.hide_tooltip();
                                                }
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
                                            spacing: 0px;
                                            compare_button := ToolButton {
                                                label: root.running ? "Comparing..." : "Compare";
                                                primary: true;
                                                button_min_width: 0px;
                                                control_height: 32px;
                                                horizontal-stretch: 1;
                                                enabled: root.compare_ready;
                                                tooltip_text: root.compare_button_tooltip_text;
                                                tooltip_requested(value, anchor_x, anchor_y, anchor_bottom) => {
                                                    root.show_tooltip(
                                                        value,
                                                        anchor_x - root.absolute-position.x,
                                                        anchor_y - root.absolute-position.y,
                                                        anchor_bottom - root.absolute-position.y,
                                                    );
                                                }
                                                tooltip_closed => {
                                                    root.hide_tooltip();
                                                }
                                                tapped => {
                                                    root.compare_clicked();
                                                }
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
                                    height: root.compare_status_details_expanded && root.compare_status_has_detail
                                        ? 206px
                                        : (root.compare_status_note_text != "" ? 106px : 92px);
                                    VerticalLayout {
                                        padding: 10px;
                                        spacing: 6px;

                                        compare_status_header := Rectangle {
                                            height: 20px;
                                            background: transparent;

                                            HorizontalLayout {
                                                spacing: 8px;

                                                compare_status_title_lane := Rectangle {
                                                    background: transparent;
                                                    horizontal-stretch: 1;

                                                    Text {
                                                        x: 0px;
                                                        y: 0px;
                                                        width: parent.width;
                                                        height: parent.height;
                                                        text: "Compare Status";
                                                        color: #3b4a5b;
                                                        font-size: 15px;
                                                        vertical-alignment: center;
                                                    }

                                                    TouchArea {
                                                        pointer-event(event) => {
                                                            if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                root.compare_status_context_menu_requested(
                                                                    root.compare_summary_copy_text,
                                                                    root.compare_detail_copy_text,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }

                                                TextAction {
                                                    visible: root.compare_view_has_targets;
                                                    label: "Open root";
                                                    tapped => {
                                                        root.compare_root_view_requested();
                                                    }
                                                }

                                                TextAction {
                                                    visible: root.compare_status_has_detail;
                                                    label: root.compare_status_details_expanded ? "Hide details" : "Show details";
                                                    tapped => {
                                                        root.compare_status_details_expanded = !root.compare_status_details_expanded;
                                                    }
                                                }
                                            }
                                        }

                                        compare_status_summary_surface := Rectangle {
                                            height: root.compare_status_note_text != "" ? 48px : 34px;
                                            background: transparent;

                                            VerticalLayout {
                                                spacing: 4px;

                                                HorizontalLayout {
                                                    spacing: 6px;
                                                    Text {
                                                        text: root.status_text;
                                                        color: #455d74;
                                                        overflow: elide;
                                                        vertical-alignment: center;
                                                        horizontal-stretch: 1;
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
                                                }

                                                Text {
                                                    visible: root.compare_metrics_text != "";
                                                    text: root.compare_metrics_text;
                                                    color: #566a7f;
                                                    overflow: elide;
                                                    horizontal-stretch: 1;
                                                }

                                                Text {
                                                    visible: root.compare_status_note_text != "";
                                                    text: root.compare_status_note_text;
                                                    color: #6c7b8a;
                                                    overflow: elide;
                                                    horizontal-stretch: 1;
                                                }
                                            }

                                            TouchArea {
                                                pointer-event(event) => {
                                                    if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                        root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                        root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                        root.compare_status_context_menu_requested(
                                                            root.compare_summary_copy_text,
                                                            root.compare_detail_copy_text,
                                                        );
                                                    }
                                                }
                                            }
                                        }

                                        compare_status_detail_tray := Rectangle {
                                            visible: root.compare_status_details_expanded && root.compare_status_has_detail;
                                            height: 104px;
                                            border-width: 1px;
                                            border-color: #dde5ef;
                                            border-radius: 5px;
                                            background: #f7fafd;
                                            clip: true;

                                            compare_status_detail_scroll := ScrollView {
                                                width: parent.width;
                                                height: parent.height;
                                                scrolled => {
                                                    root.context_menu_close_requested();
                                                }
                                                viewport-width: self.width;
                                                viewport-height: max(
                                                    self.height,
                                                    compare_status_detail_content.y + compare_status_detail_content.preferred-height + 8px
                                                );
                                                compare_status_detail_viewport := Rectangle {
                                                    width: compare_status_detail_scroll.viewport-width;
                                                    height: compare_status_detail_scroll.viewport-height;

                                                    compare_status_detail_content := VerticalLayout {
                                                        x: 8px;
                                                        y: 8px;
                                                        width: max(0px, parent.width - 16px);
                                                        spacing: 6px;

                                                        if root.compact_summary_text != "" : Rectangle {
                                                            height: compare_status_summary_detail.preferred-height;
                                                            background: transparent;

                                                            compare_status_summary_detail := VerticalLayout {
                                                                spacing: 2px;
                                                                Text {
                                                                    text: "Summary";
                                                                    color: #708193;
                                                                    font-size: 11px;
                                                                    font-weight: 600;
                                                                }
                                                                Text {
                                                                    text: root.compact_summary_text;
                                                                    color: #586c81;
                                                                    font-size: 12px;
                                                                    wrap: word-wrap;
                                                                    horizontal-stretch: 1;
                                                                }
                                                            }

                                                            TouchArea {
                                                                pointer-event(event) => {
                                                                    if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                        root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                        root.compare_status_context_menu_requested(
                                                                            root.compare_summary_copy_text,
                                                                            root.compare_detail_copy_text,
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if root.warnings_text != "" : Rectangle {
                                                            height: compare_status_warning_detail.preferred-height;
                                                            background: transparent;

                                                            compare_status_warning_detail := VerticalLayout {
                                                                spacing: 2px;
                                                                Text {
                                                                    text: "Warnings";
                                                                    color: #826136;
                                                                    font-size: 11px;
                                                                    font-weight: 600;
                                                                }
                                                                Text {
                                                                    text: root.warnings_text;
                                                                    color: #7a5a2f;
                                                                    font-size: 12px;
                                                                    wrap: word-wrap;
                                                                    horizontal-stretch: 1;
                                                                }
                                                            }

                                                            TouchArea {
                                                                pointer-event(event) => {
                                                                    if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                        root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                        root.compare_status_context_menu_requested(
                                                                            root.compare_summary_copy_text,
                                                                            root.compare_detail_copy_text,
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if root.error_text != "" : Rectangle {
                                                            height: compare_status_error_detail.preferred-height;
                                                            background: transparent;

                                                            compare_status_error_detail := VerticalLayout {
                                                                spacing: 2px;
                                                                Text {
                                                                    text: "Error";
                                                                    color: #8a2f2f;
                                                                    font-size: 11px;
                                                                    font-weight: 600;
                                                                }
                                                                Text {
                                                                    text: root.error_text;
                                                                    color: #8a2f2f;
                                                                    font-size: 12px;
                                                                    wrap: word-wrap;
                                                                    horizontal-stretch: 1;
                                                                }
                                                            }

                                                            TouchArea {
                                                                pointer-event(event) => {
                                                                    if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                        root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                        root.compare_status_context_menu_requested(
                                                                            root.compare_summary_copy_text,
                                                                            root.compare_detail_copy_text,
                                                                        );
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

                        // Contract: Filter / Scope.
                        // Applies text/status filters to navigator rows; does not mutate source compare data.
                        SectionCard {
                            height: 104px;
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
                                    TooltipLineEdit {
                                        text <=> root.entry_filter;
                                        horizontal-stretch: 1;
                                        enabled: !root.running;
                                        placeholder_text: "path or name";
                                        edited(value) => {
                                            root.filter_changed(value);
                                        }
                                        tooltip_requested(value, anchor_x, anchor_y, anchor_bottom) => {
                                            root.show_tooltip(
                                                value,
                                                anchor_x - root.absolute-position.x,
                                                anchor_y - root.absolute-position.y,
                                                anchor_bottom - root.absolute-position.y,
                                            );
                                        }
                                        tooltip_closed => {
                                            root.hide_tooltip();
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
                                HorizontalLayout {
                                    spacing: 8px;
                                    Text {
                                        text: "Results / Navigator";
                                        color: #374656;
                                        font-size: 15px;
                                        vertical-alignment: center;
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                    SegmentedRail {
                                        width: 118px;
                                        height: 24px;
                                        HorizontalLayout {
                                            spacing: 0px;
                                            SegmentItem {
                                                label: "Tree";
                                                selected: root.navigator_effective_view_mode == "tree";
                                                show_divider: false;
                                                enabled: !root.navigator_search_forces_flat_mode
                                                    && root.navigator_runtime_view_mode != "tree";
                                                tapped => {
                                                    root.navigator_view_mode_tree_requested();
                                                }
                                            }
                                            SegmentItem {
                                                label: "Flat";
                                                selected: root.navigator_effective_view_mode == "flat";
                                                show_divider: true;
                                                enabled: !root.navigator_search_forces_flat_mode
                                                    && root.navigator_runtime_view_mode != "flat";
                                                tapped => {
                                                    root.navigator_view_mode_flat_requested();
                                                }
                                            }
                                        }
                                    }
                                }
                                Text {
                                    text: root.results_collection_text;
                                    color: #6f7e8d;
                                    overflow: elide;
                                }
                                navigator_tree_view := NavigatorTree {
                                    visible: root.navigator_effective_view_mode == "tree";
                                    min-height: 0px;
                                    max-height: self.visible ? 12000px : 0px;
                                    vertical-stretch: self.visible ? 1 : 0;
                                    row_keys: root.tree_row_keys;
                                    row_display_names: root.tree_row_display_names;
                                    row_statuses: root.tree_row_statuses;
                                    row_tooltip_texts: root.tree_row_tooltip_texts;
                                    row_depths: root.tree_row_depths;
                                    row_is_directories: root.tree_row_is_directories;
                                    row_is_expandable: root.tree_row_is_expandable;
                                    row_is_expanded: root.tree_row_is_expanded;
                                    row_is_selectable: root.tree_row_is_selectable;
                                    row_source_indices: root.tree_row_source_indices;
                                    selected_row: root.selected_row;
                                    interaction_enabled: !root.diff_loading;
                                    file_selected(source_index) => {
                                        root.hide_tooltip();
                                        root.navigator_tree_file_selected(source_index);
                                    }
                                    directory_toggled(key) => {
                                        root.hide_tooltip();
                                        root.navigator_tree_directory_toggled(key);
                                    }
                                    context_menu_requested(key, status, directory, source_index, anchor_x, anchor_y) => {
                                        root.hide_tooltip();
                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                        root.navigator_tree_context_menu_requested(
                                            key,
                                            status,
                                            directory,
                                            source_index,
                                        );
                                    }
                                    tooltip_requested(text, anchor_x, anchor_top, anchor_bottom) => {
                                        root.show_tooltip(
                                            text,
                                            anchor_x - root.absolute-position.x,
                                            anchor_top - root.absolute-position.y,
                                            anchor_bottom - root.absolute-position.y,
                                        );
                                    }
                                    tooltip_closed() => {
                                        root.hide_tooltip();
                                    }
                                }
                                flat_results_list := ListView {
                                    visible: root.navigator_effective_view_mode == "flat";
                                    min-height: 0px;
                                    max-height: self.visible ? 12000px : 0px;
                                    vertical-stretch: self.visible ? 1 : 0;
                                    property <length> row_height: 50px;
                                    property <length> ensure_visible_padding: 16px;
                                    function ensure_row_visible(target_row: int) {
                                        if target_row < 0 || target_row >= root.row_paths.length {
                                            return;
                                        }

                                        let top_limit = min(
                                            self.ensure_visible_padding,
                                            max(0px, self.visible-height - self.row_height),
                                        );
                                        let bottom_limit = max(
                                            self.row_height,
                                            self.visible-height - self.ensure_visible_padding,
                                        );
                                        let target_top = self.viewport-y + target_row * self.row_height;
                                        let target_bottom = target_top + self.row_height;
                                        if target_top < top_limit {
                                            self.viewport-y += top_limit - target_top;
                                        }
                                        if target_bottom > bottom_limit {
                                            self.viewport-y -= target_bottom - bottom_limit;
                                        }
                                    }
                                    for row_path[index] in root.row_paths: row_item := Rectangle {
                                        property <length> tooltip_x_inset: 8px;
                                        property <int> source_index: root.row_source_indices[index];
                                        property <string> row_status: root.row_statuses[index];
                                        property <bool> row_unavailable: !root.row_can_load_diff[index];
                                        property <bool> row_selected: source_index == root.selected_row;
                                        property <string> display_name: root.row_display_names[index];
                                        property <string> parent_path: root.row_parent_paths[index];
                                        property <string> tooltip_text: root.row_tooltip_texts[index];
                                        property <string> secondary_text: root.row_secondary_texts[index];
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
                                        property <brush> item_border_color: row_selected
                                            ? UiPalette.results_row_selected_border
                                            : (row_unavailable
                                                ? UiPalette.results_row_unavailable_border
                                                : (row_status == "different"
                                                    ? UiPalette.results_row_tone_different_border
                                                    : (row_status == "equal"
                                                        ? UiPalette.results_row_tone_equal_border
                                                        : (row_status == "left-only"
                                                            ? UiPalette.results_row_tone_left_border
                                                            : (row_status == "right-only"
                                                                ? UiPalette.results_row_tone_right_border
                                                                : UiPalette.results_row_tone_neutral_border)))));
                                        property <brush> item_background_color: row_selected
                                            ? UiPalette.results_row_selected_background
                                            : (row_unavailable
                                                ? UiPalette.results_row_unavailable_background
                                                : (row_status == "different"
                                                    ? UiPalette.results_row_tone_different_background
                                                    : (row_status == "equal"
                                                        ? UiPalette.results_row_tone_equal_background
                                                        : (row_status == "left-only"
                                                            ? UiPalette.results_row_tone_left_background
                                                            : (row_status == "right-only"
                                                                ? UiPalette.results_row_tone_right_background
                                                                : UiPalette.results_row_tone_neutral_background)))));
                                        property <brush> path_text_color: row_selected
                                            ? UiPalette.results_row_selected_path_text
                                            : (row_unavailable ? UiPalette.results_row_unavailable_path_text : UiPalette.results_row_tone_neutral_path_text);
                                        property <brush> detail_text_color: row_selected
                                            ? UiPalette.results_row_selected_detail_text
                                            : (row_unavailable ? UiPalette.results_row_unavailable_detail_text : UiPalette.results_row_tone_neutral_detail_text);
                                        property <brush> context_text_color: row_selected
                                            ? UiPalette.results_row_selected_context_text
                                            : (row_unavailable ? UiPalette.results_row_unavailable_context_text : UiPalette.results_row_tone_neutral_context_text);
                                        property <brush> match_fill: row_selected
                                            ? UiPalette.results_row_selected_match_background
                                            : UiPalette.results_row_match_background;
                                        function update_row_tooltip(
                                            anchor_x: length,
                                            anchor_top: length,
                                            anchor_bottom: length,
                                        ) {
                                            if row_item.tooltip_text != ""
                                                && (display_name_label.is_truncated
                                                    || (parent_path_label.visible && parent_path_label.is_truncated)) {
                                                root.show_tooltip(
                                                    row_item.tooltip_text,
                                                    anchor_x + row_item.tooltip_x_inset - root.absolute-position.x,
                                                    anchor_top - root.absolute-position.y,
                                                    anchor_bottom - root.absolute-position.y,
                                                );
                                            } else {
                                                root.hide_tooltip();
                                            }
                                        }

                                        height: flat_results_list.row_height;
                                        border-width: 1px;
                                        border-color: row_item.item_border_color;
                                        border-radius: 3px;
                                        background: row_item.item_background_color;

                                        VerticalLayout {
                                            padding-left: 6px;
                                            padding-right: 6px;
                                            padding-top: 5px;
                                            padding-bottom: 5px;
                                            spacing: 2px;
                                            HorizontalLayout {
                                                spacing: 7px;
                                                StatusPill {
                                                    label: row_item.row_status_label;
                                                    tone: row_item.row_unavailable ? "neutral" : row_item.row_status_tone;
                                                }
                                                display_name_label := HighlightTextLabel {
                                                    text: row_item.display_name;
                                                    text_color: row_item.path_text_color;
                                                    highlight: root.row_display_name_matches[index];
                                                    highlight_fill: row_item.match_fill;
                                                    font_size: 13px;
                                                    font_weight: 600;
                                                    horizontal-stretch: 1;
                                                }
                                            }
                                            HorizontalLayout {
                                                spacing: 4px;
                                                secondary_text_label := Text {
                                                    text: row_item.secondary_text;
                                                    color: row_item.detail_text_color;
                                                    vertical-alignment: center;
                                                    font-size: 11px;
                                                    horizontal-stretch: 1;
                                                    overflow: elide;
                                                }
                                                Text {
                                                    visible: row_item.parent_path != "";
                                                    text: "·";
                                                    color: row_item.context_text_color;
                                                    vertical-alignment: center;
                                                    font-size: 11px;
                                                }
                                                parent_path_label := HighlightTextLabel {
                                                    visible: row_item.parent_path != "";
                                                    text: row_item.parent_path;
                                                    text_color: row_item.context_text_color;
                                                    highlight: root.row_parent_path_matches[index];
                                                    highlight_fill: row_item.match_fill;
                                                    font_size: 11px;
                                                    font_weight: 400;
                                                    max-width: 108px;
                                                }
                                            }
                                        }

                                        TouchArea {
                                            width: parent.width;
                                            height: parent.height;
                                            enabled: !root.diff_loading;
                                            changed has-hover => {
                                                if self.has-hover {
                                                    row_item.update_row_tooltip(
                                                        self.absolute-position.x,
                                                        self.absolute-position.y,
                                                        self.absolute-position.y + self.height,
                                                    );
                                                } else {
                                                    root.hide_tooltip();
                                                }
                                            }
                                            clicked => {
                                                root.hide_tooltip();
                                                root.row_selected(row_item.source_index);
                                            }
                                            pointer-event(event) => {
                                                if event.kind == PointerEventKind.move {
                                                    row_item.update_row_tooltip(
                                                        self.absolute-position.x,
                                                        self.absolute-position.y,
                                                        self.absolute-position.y + self.height,
                                                    );
                                                } else if event.kind == PointerEventKind.down {
                                                    root.hide_tooltip();
                                                }
                                                if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                    root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                    root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                    root.results_context_menu_requested(
                                                        row_item.source_index,
                                                        row_path,
                                                        row_item.row_status,
                                                        root.row_details[index],
                                                        row_item.row_unavailable,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    scrolled => {
                                        root.hide_tooltip();
                                        root.context_menu_close_requested();
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
                    border-color: transparent;
                    background: transparent;
                    VerticalLayout {
                        padding: 0px;
                        spacing: 0px;

                        Rectangle {
                            vertical-stretch: 1;
                            background: transparent;
                            workbench_host := Rectangle {
                                x: 0px;
                                y: 0px;
                                width: parent.width;
                                height: parent.height;
                                background: transparent;

                                property <brush> panel_border: #dbe4ee;
                                property <length> panel_corner_radius: 8px;
                                property <length> tab_row_height: 36px;
                                property <length> panel_overlap: 4px;
                                property <length> panel_top: self.tab_row_height - self.panel_overlap;

                                if root.workspace_sessions_visible : Rectangle {
                                    x: 0px;
                                    y: 0px;
                                    width: parent.width;
                                    height: root.workspace_session_strip_height;
                                    background: transparent;

                                    HorizontalLayout {
                                        padding-left: 2px;
                                        padding-right: 2px;
                                        padding-top: 3px;
                                        padding-bottom: 3px;
                                        spacing: 6px;

                                        for session_id[index] in root.workspace_session_ids : WorkspaceSessionTabButton {
                                            label: root.workspace_session_labels[index];
                                            kind: root.workspace_session_kinds[index];
                                            active: index == root.active_workspace_session_index;
                                            closable: root.workspace_session_closable[index];
                                            tapped => {
                                                root.context_menu_close_requested();
                                                root.workspace_session_selected(session_id);
                                            }
                                            close_requested() => {
                                                root.context_menu_close_requested();
                                                root.workspace_session_close_requested(session_id);
                                            }
                                        }

                                        Rectangle {
                                            horizontal-stretch: 1;
                                            background: transparent;
                                        }
                                    }
                                }

                                workbench_panel := Rectangle {
                                    visible: root.workspace_mode == "file-view" && !root.compare_file_view_active;
                                    x: 0px;
                                    y: root.workspace_session_strip_height + workbench_host.panel_top;
                                    width: parent.width;
                                    height: max(
                                        0px,
                                        parent.height - root.workspace_session_strip_height - workbench_host.panel_top,
                                    );
                                    border-width: 1px;
                                    border-color: workbench_host.panel_border;
                                    border-radius: workbench_host.panel_corner_radius;
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
                                                height: root.can_return_to_compare_view
                                                    ? root.workbench_compare_context_header_height
                                                    : root.workbench_header_height;
                                                background: #f9fbfe;
                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #dde5f0;
                                                }

                                                if root.can_return_to_compare_view : Rectangle {
                                                    width: parent.width;
                                                    height: parent.height;
                                                    background: transparent;

                                                    VerticalLayout {
                                                        padding: 10px;
                                                        spacing: 4px;

                                                        Text {
                                                            text: root.compare_root_pair_text;
                                                            color: #6e7f90;
                                                            font-size: 11px;
                                                            horizontal-stretch: 1;
                                                            overflow: elide;
                                                        }

                                                        Text {
                                                            text: root.file_view_title_text;
                                                            color: root.file_view_title_text == "No file selected" ? #607286 : #294b6b;
                                                            font-size: 16px;
                                                            font-weight: 600;
                                                            horizontal-stretch: 1;
                                                            overflow: elide;
                                                        }

                                                        HorizontalLayout {
                                                            spacing: 6px;

                                                            StatusPill {
                                                                label: "Diff";
                                                                tone: "neutral";
                                                            }
                                                            if root.has_selected_result : StatusPill {
                                                                label: root.diff_mode_label;
                                                                tone: root.diff_mode_tone;
                                                            }
                                                            if root.has_selected_result : StatusPill {
                                                                label: root.file_view_compare_status_label;
                                                                tone: root.file_view_compare_status_tone;
                                                            }
                                                            if !root.has_selected_result
                                                                || root.diff_shell_state_token == "stale-selection"
                                                                || root.diff_shell_state_token == "loading"
                                                                || root.diff_shell_state_token == "unavailable"
                                                                || root.diff_shell_state_token == "error" : StatusPill {
                                                                label: root.diff_shell_state_label;
                                                                tone: root.diff_shell_state_tone;
                                                            }
                                                            Text {
                                                                text: root.file_view_path_context_text;
                                                                color: #617285;
                                                                font-size: 12px;
                                                                vertical-alignment: center;
                                                                horizontal-stretch: 1;
                                                                overflow: elide;
                                                            }
                                                        }
                                                    }

                                                    TouchArea {
                                                        width: parent.width;
                                                        height: parent.height;
                                                        pointer-event(event) => {
                                                            if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                root.workspace_header_context_menu_requested(
                                                                    root.selected_relative_path_raw,
                                                                    root.diff_mode_label,
                                                                    root.file_view_compare_status_label,
                                                                    root.file_view_path_context_text,
                                                                    root.compare_root_pair_text,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }

                                                if !root.can_return_to_compare_view : VerticalLayout {
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
                                                        StatusPill {
                                                            label: "Diff";
                                                            tone: "neutral";
                                                        }
                                                        if root.has_selected_result : StatusPill {
                                                            label: root.diff_mode_label;
                                                            tone: root.diff_mode_tone;
                                                        }
                                                        if root.has_selected_result : StatusPill {
                                                            label: root.diff_result_status_label;
                                                            tone: root.diff_result_status_tone;
                                                        }
                                                        if !root.has_selected_result
                                                            || root.diff_shell_state_token == "stale-selection"
                                                            || root.diff_shell_state_token == "loading"
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

                                                if !root.can_return_to_compare_view : TouchArea {
                                                    pointer-event(event) => {
                                                        if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                            root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                            root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                            root.workspace_header_context_menu_requested(
                                                                root.selected_relative_path_raw,
                                                                root.diff_mode_label,
                                                                root.diff_result_status_label,
                                                                root.diff_context_summary_text,
                                                                root.diff_context_hint_text,
                                                            );
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
                                                        text: root.diff_helper_strip_text;
                                                        color: #617285;
                                                        font-size: 12px;
                                                        vertical-alignment: center;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
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
                                                                x: diff_body_scroll.viewport-x;
                                                                y: 0px;
                                                                width: diff_ready_surface.table_width;
                                                                height: parent.height;

                                                                Text {
                                                                    x: 0px;
                                                                    y: 0px;
                                                                    width: max(0px, root.diff_number_column_width - 8px);
                                                                    height: parent.height;
                                                                    text: root.diff_left_column_label;
                                                                    horizontal-alignment: right;
                                                                    vertical-alignment: center;
                                                                    font-size: 12px;
                                                                    color: #667789;
                                                                }
                                                                Rectangle {
                                                                    x: root.diff_first_separator_x;
                                                                    y: root.diff_header_separator_inset;
                                                                    width: root.diff_separator_width;
                                                                    height: max(0px, parent.height - root.diff_header_separator_inset * 2);
                                                                    background: #dce5f0;
                                                                }
                                                                Text {
                                                                    x: root.diff_new_column_x;
                                                                    y: 0px;
                                                                    width: max(0px, root.diff_number_column_width - 8px);
                                                                    height: parent.height;
                                                                    text: root.diff_right_column_label;
                                                                    horizontal-alignment: right;
                                                                    vertical-alignment: center;
                                                                    font-size: 12px;
                                                                    color: #667789;
                                                                }
                                                                Rectangle {
                                                                    x: root.diff_second_separator_x;
                                                                    y: root.diff_header_separator_inset;
                                                                    width: root.diff_separator_width;
                                                                    height: max(0px, parent.height - root.diff_header_separator_inset * 2);
                                                                    background: #dce5f0;
                                                                }
                                                                Text {
                                                                    x: root.diff_marker_column_x;
                                                                    y: 0px;
                                                                    width: root.diff_marker_column_width;
                                                                    height: parent.height;
                                                                    text: " ";
                                                                }
                                                                Rectangle {
                                                                    x: root.diff_third_separator_x;
                                                                    y: root.diff_header_separator_inset;
                                                                    width: root.diff_separator_width;
                                                                    height: max(0px, parent.height - root.diff_header_separator_inset * 2);
                                                                    background: #dce5f0;
                                                                }
                                                                Text {
                                                                    x: root.diff_content_column_x + 8px;
                                                                    y: 0px;
                                                                    width: max(0px, parent.width - root.diff_content_column_x - 8px);
                                                                    height: parent.height;
                                                                    text: "content";
                                                                    color: #667789;
                                                                    font-size: 12px;
                                                                    vertical-alignment: center;
                                                                }
                                                            }
                                                        }

                                                        diff_body_scroll := ScrollView {
                                                            x: 0px;
                                                            y: 30px;
                                                            width: parent.width;
                                                            height: max(0px, parent.height - 30px);
                                                            viewport-width: diff_ready_surface.table_width;
                                                            viewport-height: max(self.height, diff_rows_content.preferred-height);
                                                            horizontal-scrollbar-policy: ScrollBarPolicy.as-needed;
                                                            vertical-scrollbar-policy: ScrollBarPolicy.as-needed;
                                                            diff_rows_content := VerticalLayout {
                                                                width: diff_ready_surface.table_width;
                                                                spacing: 0px;

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

                                                                    DiffCopyHotspot {
                                                                        x: root.diff_old_column_x;
                                                                        y: 0px;
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
                                                                        x: root.diff_first_separator_x;
                                                                        y: 0px;
                                                                        width: root.diff_separator_width;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    DiffCopyHotspot {
                                                                        x: root.diff_new_column_x;
                                                                        y: 0px;
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
                                                                        x: root.diff_second_separator_x;
                                                                        y: 0px;
                                                                        width: root.diff_separator_width;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    DiffCopyHotspot {
                                                                        x: root.diff_marker_column_x;
                                                                        y: 0px;
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
                                                                        x: root.diff_third_separator_x;
                                                                        y: 0px;
                                                                        width: root.diff_separator_width;
                                                                        height: parent.height;
                                                                        background: #e4ebf4;
                                                                    }
                                                                    SelectableDiffText {
                                                                        x: root.diff_content_column_x;
                                                                        y: 0px;
                                                                        width: max(0px, parent.width - root.diff_content_column_x);
                                                                        height: parent.height;
                                                                        value: row_content;
                                                                        foreground: row_line.is_hunk
                                                                            ? #2f5376
                                                                            : #2f4357;
                                                                        font_weight: row_line.is_hunk ? 600 : 400;
                                                                        content_padding: 8px;
                                                                    }
                                                                }

                                                                Rectangle {
                                                                    width: diff_ready_surface.table_width;
                                                                    height: root.diff_scrollbar_safe_inset;
                                                                    background: transparent;
                                                                }
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
                                                height: root.can_return_to_compare_view
                                                    ? root.workbench_compare_context_header_height
                                                    : root.workbench_header_height;
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

                                                    analysis_header_surface := Rectangle {
                                                        horizontal-stretch: 1;
                                                        background: transparent;

                                                        if root.can_return_to_compare_view : VerticalLayout {
                                                            spacing: 4px;

                                                            Text {
                                                                text: root.compare_root_pair_text;
                                                                color: #6e7f90;
                                                                font-size: 11px;
                                                                horizontal-stretch: 1;
                                                                overflow: elide;
                                                            }

                                                            Text {
                                                                text: root.file_view_title_text;
                                                                color: root.file_view_title_text == "No file selected" ? #607286 : #294b6b;
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
                                                                if root.has_selected_result : StatusPill {
                                                                    label: root.file_view_compare_status_label;
                                                                    tone: root.file_view_compare_status_tone;
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
                                                                    text: root.file_view_path_context_text;
                                                                    color: #5f7184;
                                                                    font-size: 12px;
                                                                    vertical-alignment: center;
                                                                    horizontal-stretch: 1;
                                                                    overflow: elide;
                                                                }
                                                            }
                                                        }

                                                        if !root.can_return_to_compare_view : VerticalLayout {
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

                                                        if root.can_return_to_compare_view : TouchArea {
                                                            width: parent.width;
                                                            height: parent.height;
                                                            pointer-event(event) => {
                                                                if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                    root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                    root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                    root.workspace_header_context_menu_requested(
                                                                        root.selected_relative_path_raw,
                                                                        "Analysis",
                                                                        root.file_view_compare_status_label,
                                                                        root.file_view_path_context_text,
                                                                        root.compare_root_pair_text,
                                                                    );
                                                                }
                                                            }
                                                        }

                                                        if !root.can_return_to_compare_view : TouchArea {
                                                            pointer-event(event) => {
                                                                if event.button == PointerEventButton.right && event.kind == PointerEventKind.down {
                                                                    root.context_menu_anchor_x = self.absolute-position.x + self.mouse-x - root.absolute-position.x;
                                                                    root.context_menu_anchor_y = self.absolute-position.y + self.mouse-y - root.absolute-position.y;
                                                                    root.workspace_header_context_menu_requested(
                                                                        root.selected_relative_path_raw,
                                                                        "Analysis",
                                                                        root.analysis_state_label,
                                                                        root.analysis_header_summary_text,
                                                                        root.analysis_technical_context_text,
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }

                                                    ToolButton {
                                                        label: root.analysis_loading ? "Analyzing..." : "Analyze";
                                                        primary: true;
                                                        width: 132px;
                                                        button_min_width: 132px;
                                                        control_height: 30px;
                                                        label_font_size: 13px;
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
                                                        text: "Use Settings in App Bar to edit.";
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
                                                            scrolled => {
                                                                root.context_menu_close_requested();
                                                            }
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
                                                                    context_menu_requested(anchor_x, anchor_y, section_label, title, body, copy_value) => {
                                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                                        root.analysis_section_context_menu_requested(section_label, title, body, copy_value);
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
                                                                        ? UiPalette.result_section_tone_error_border
                                                                        : (root.analysis_risk_tone == "warn"
                                                                            ? UiPalette.result_section_tone_warn_border
                                                                            : (root.analysis_risk_tone == "success"
                                                                                ? UiPalette.result_section_tone_success_border
                                                                                : UiPalette.result_section_tone_neutral_border));
                                                                    background: root.analysis_risk_tone == "error"
                                                                        ? UiPalette.result_section_tone_error_background
                                                                        : (root.analysis_risk_tone == "warn"
                                                                            ? UiPalette.result_section_tone_warn_background
                                                                            : (root.analysis_risk_tone == "success"
                                                                                ? UiPalette.result_section_tone_success_background
                                                                                : UiPalette.result_section_tone_neutral_background));

                                                                    risk_layout := VerticalLayout {
                                                                        padding: 14px;
                                                                        spacing: 8px;

                                                                        risk_header_surface := Rectangle {
                                                                            height: 20px;
                                                                            background: transparent;

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
                                                                    context_menu_requested(anchor_x, anchor_y, section_label, title, body, copy_value) => {
                                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                                        root.analysis_section_context_menu_requested(section_label, title, body, copy_value);
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
                                                                    context_menu_requested(anchor_x, anchor_y, section_label, title, body, copy_value) => {
                                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                                        root.analysis_section_context_menu_requested(section_label, title, body, copy_value);
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
                                                                    context_menu_requested(anchor_x, anchor_y, section_label, title, body, copy_value) => {
                                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                                        root.analysis_section_context_menu_requested(section_label, title, body, copy_value);
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
                                                                    context_menu_requested(anchor_x, anchor_y, section_label, title, body, copy_value) => {
                                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                                        root.analysis_section_context_menu_requested(section_label, title, body, copy_value);
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
                                    visible: root.workspace_mode == "file-view" && !root.compare_file_view_active;
                                    x: workbench_host.panel_corner_radius;
                                    y: root.workspace_session_strip_height + workbench_host.panel_top;
                                    width: max(0px, parent.width - workbench_host.panel_corner_radius * 2);
                                    height: 1px;
                                    background: workbench_host.panel_border;
                                }

                                HorizontalLayout {
                                    visible: root.workspace_mode == "file-view" && !root.compare_file_view_active;
                                    x: 0px;
                                    y: root.workspace_session_strip_height;
                                    width: parent.width;
                                    height: workbench_host.tab_row_height;
                                    padding-right: 7px;
                                    spacing: 0px;

                                    WorkspaceTabButton {
                                        label: "Diff";
                                        selected: root.workspace_tab == 0;
                                        selected_fill: #f9fbfe;
                                        selected_border: workbench_host.panel_border;
                                        connector_depth: 5px;
                                        tapped => {
                                            root.context_menu_close_requested();
                                            root.file_view_diff_requested();
                                        }
                                    }

                                    WorkspaceTabButton {
                                        label: "Analysis";
                                        selected: root.workspace_tab == 1;
                                        selected_fill: #f9fbfe;
                                        selected_border: workbench_host.panel_border;
                                        connector_depth: 5px;
                                        tapped => {
                                            root.context_menu_close_requested();
                                            root.file_view_analysis_requested();
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

                                compare_file_surface := Rectangle {
                                    visible: root.workspace_mode == "file-view" && root.compare_file_view_active;
                                    x: 0px;
                                    y: root.workspace_session_strip_height;
                                    width: parent.width;
                                    height: max(0px, parent.height - root.workspace_session_strip_height);
                                    background: transparent;

                                    Rectangle {
                                        x: 0px;
                                        y: 0px;
                                        width: parent.width;
                                        height: parent.height;
                                        border-width: 1px;
                                        border-color: workbench_host.panel_border;
                                        border-radius: workbench_host.panel_corner_radius;
                                        background: #fcfdff;
                                        clip: true;

                                        VerticalLayout {
                                            padding: 0px;
                                            spacing: 0px;

                                            Rectangle {
                                                height: root.compare_file_header_height;
                                                background: #fafcfe;

                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #e1e8f0;
                                                }

                                                VerticalLayout {
                                                    padding-left: 10px;
                                                    padding-right: 10px;
                                                    padding-top: 9px;
                                                    padding-bottom: 9px;
                                                    spacing: 6px;

                                                    Text {
                                                        text: root.compare_root_pair_text;
                                                        color: #6f7f90;
                                                        font-size: 11px;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
                                                    }

                                                    Text {
                                                        text: root.file_view_title_text;
                                                        color: root.file_view_title_text == "No file selected" ? #607286 : #294b6b;
                                                        font-size: 16px;
                                                        font-weight: 600;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
                                                    }

                                                    HorizontalLayout {
                                                        spacing: 8px;

                                                        CompareHeaderGhostButton {
                                                            label: "Back";
                                                            tooltip_text: "Back to Compare Tree";
                                                            button_width: 58px;
                                                            enabled: root.can_return_to_compare_view;
                                                            tapped => {
                                                                root.compare_file_back_requested();
                                                            }
                                                            tooltip_requested(text, anchor_x, anchor_top, anchor_bottom) => {
                                                                root.show_tooltip(
                                                                    text,
                                                                    anchor_x - root.absolute-position.x,
                                                                    anchor_top - root.absolute-position.y,
                                                                    anchor_bottom - root.absolute-position.y,
                                                                );
                                                            }
                                                            tooltip_closed => {
                                                                root.hide_tooltip();
                                                            }
                                                        }

                                                        StatusPill {
                                                            label: "Compare File";
                                                            tone: "neutral";
                                                        }

                                                        if root.file_view_compare_status_label != "Unavailable" : StatusPill {
                                                            label: root.file_view_compare_status_label;
                                                            tone: root.file_view_compare_status_tone;
                                                        }

                                                        Text {
                                                            text: root.file_view_path_context_text;
                                                            color: #617285;
                                                            font-size: 12px;
                                                            vertical-alignment: center;
                                                            horizontal-stretch: 1;
                                                            overflow: elide;
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
                                                        text: root.compare_file_helper_text;
                                                        color: #617285;
                                                        font-size: 12px;
                                                        vertical-alignment: center;
                                                        horizontal-stretch: 1;
                                                        overflow: elide;
                                                    }
                                                }
                                            }

                                            Rectangle {
                                                height: 24px;
                                                background: #fafcfe;

                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #e1e8f0;
                                                }

                                                HorizontalLayout {
                                                    padding: 0px;
                                                    spacing: 0px;

                                                    Rectangle {
                                                        width: 48px;
                                                        background: #f7f9fc;

                                                        Text {
                                                            x: 0px;
                                                            width: parent.width - 8px;
                                                            height: parent.height;
                                                            text: "Line";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                            horizontal-alignment: right;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }
                                                    }

                                                    Rectangle {
                                                        horizontal-stretch: 1;
                                                        background: transparent;

                                                        Text {
                                                            x: 8px;
                                                            width: max(0px, parent.width - 16px);
                                                            height: parent.height;
                                                            text: "Base";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }
                                                    }

                                                    Rectangle {
                                                        width: 40px;
                                                        background: transparent;

                                                        Text {
                                                            width: parent.width;
                                                            height: parent.height;
                                                            text: "Rel";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                            horizontal-alignment: center;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }
                                                    }

                                                    Rectangle {
                                                        width: 48px;
                                                        background: #f7f9fc;

                                                        Text {
                                                            x: 8px;
                                                            width: parent.width - 8px;
                                                            height: parent.height;
                                                            text: "Line";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                            horizontal-alignment: left;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }
                                                    }

                                                    Rectangle {
                                                        horizontal-stretch: 1;
                                                        background: transparent;

                                                        Text {
                                                            x: 8px;
                                                            width: max(0px, parent.width - 16px);
                                                            height: parent.height;
                                                            text: "Target";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }
                                                    }
                                                }
                                            }

                                            if root.compare_file_has_rows : Rectangle {
                                                vertical-stretch: 1;
                                                background: #ffffff;
                                                clip: true;

                                                CompareFileView {
                                                    x: 0px;
                                                    y: 0px;
                                                    width: parent.width;
                                                    height: parent.height;
                                                    row_kinds: root.compare_file_row_kinds;
                                                    row_relation_labels: root.compare_file_row_relation_labels;
                                                    row_relation_tones: root.compare_file_row_relation_tones;
                                                    row_left_line_nos: root.compare_file_row_left_line_nos;
                                                    row_right_line_nos: root.compare_file_row_right_line_nos;
                                                    row_left_prefixes: root.compare_file_row_left_prefixes;
                                                    row_left_emphasis: root.compare_file_row_left_emphasis;
                                                    row_left_suffixes: root.compare_file_row_left_suffixes;
                                                    row_right_prefixes: root.compare_file_row_right_prefixes;
                                                    row_right_emphasis: root.compare_file_row_right_emphasis;
                                                    row_right_suffixes: root.compare_file_row_right_suffixes;
                                                    row_left_padding: root.compare_file_row_left_padding;
                                                    row_right_padding: root.compare_file_row_right_padding;
                                                    left_content_width_px: root.compare_file_left_content_width_px;
                                                    right_content_width_px: root.compare_file_right_content_width_px;
                                                    copy_requested(copy_value, feedback_label) => {
                                                        root.copy_requested(copy_value, feedback_label);
                                                    }
                                                }
                                            }

                                            if !root.compare_file_has_rows : DiffStateShell {
                                                vertical-stretch: 1;
                                                embedded: true;
                                                state_label: root.compare_file_shell_state_label;
                                                tone: root.compare_file_shell_state_tone;
                                                title: root.compare_file_shell_title_text;
                                                body: root.compare_file_shell_body_text;
                                                note: root.compare_file_shell_note_text;
                                            }
                                        }
                                    }
                                }

                                compare_workspace_surface := Rectangle {
                                    visible: root.workspace_mode == "compare-view";
                                    x: 0px;
                                    y: root.workspace_session_strip_height;
                                    width: parent.width;
                                    height: max(0px, parent.height - root.workspace_session_strip_height);
                                    background: transparent;

                                    Rectangle {
                                        x: 0px;
                                        y: 0px;
                                        width: parent.width;
                                        height: parent.height;
                                        border-width: 1px;
                                        border-color: workbench_host.panel_border;
                                        border-radius: workbench_host.panel_corner_radius;
                                        background: #fcfdff;
                                        clip: true;

                                        VerticalLayout {
                                            padding: 0px;
                                            spacing: 0px;

                                            Rectangle {
                                                height: root.compare_workspace_header_height;
                                                background: #fafcfe;

                                                Rectangle {
                                                    x: 0px;
                                                    y: parent.height - 1px;
                                                    width: parent.width;
                                                    height: 1px;
                                                    background: #e1e8f0;
                                                }

                                                VerticalLayout {
                                                    padding-left: 8px;
                                                    padding-right: 8px;
                                                    padding-top: 6px;
                                                    padding-bottom: 6px;
                                                    spacing: 8px;

                                                    HorizontalLayout {
                                                        spacing: 8px;

                                                        Text {
                                                            text: root.compare_root_pair_text;
                                                            color: #7a8b9c;
                                                            font-size: 11px;
                                                            font-weight: 500;
                                                            horizontal-stretch: 1;
                                                            vertical-alignment: center;
                                                            overflow: elide;
                                                        }

                                                        if root.compare_view_has_targets : StatusPill {
                                                            label: root.compare_view_target_status_label;
                                                            tone: root.compare_view_target_status_tone;
                                                        }
                                                    }

                                                    HorizontalLayout {
                                                        spacing: 10px;

                                                        breadcrumb_lane := Rectangle {
                                                            horizontal-stretch: 1;
                                                            clip: true;
                                                            background: transparent;

                                                            HorizontalLayout {
                                                                spacing: 8px;

                                                                CompareHeaderGhostButton {
                                                                    label: "Up";
                                                                    icon_kind: "up";
                                                                    button_width: 48px;
                                                                    enabled: root.compare_view_can_go_up;
                                                                    tapped => {
                                                                        root.compare_view_up_requested();
                                                                    }
                                                                }

                                                                CompareBreadcrumbViewport {
                                                                    horizontal-stretch: 1;
                                                                    height: 22px;
                                                                    labels: root.compare_view_breadcrumb_labels;
                                                                    paths: root.compare_view_breadcrumb_paths;
                                                                    segment_requested(segment_path) => {
                                                                        root.compare_view_breadcrumb_requested(segment_path);
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        Rectangle {
                                                            width: 250px;
                                                            background: transparent;

                                                            HorizontalLayout {
                                                                spacing: 6px;

                                                                Rectangle {
                                                                    horizontal-stretch: 1;
                                                                    background: transparent;
                                                                }

                                                                CompareHeaderGhostButton {
                                                                    label: root.compare_view_horizontal_scroll_locked ? "Locked" : "Unlocked";
                                                                    icon_kind: root.compare_view_horizontal_scroll_locked ? "lock" : "unlock";
                                                                    button_width: 88px;
                                                                    enabled: root.compare_view_has_targets;
                                                                    tapped => {
                                                                        root.compare_view_scroll_lock_toggled();
                                                                    }
                                                                }

                                                                CompareHeaderGhostButton {
                                                                    label: "Reset";
                                                                    icon_kind: "reset";
                                                                    button_width: 64px;
                                                                    enabled: root.compare_view_has_targets;
                                                                    tapped => {
                                                                        compare_workspace_view.reset_horizontal_scroll();
                                                                    }
                                                                }

                                                                CompareHeaderGhostButton {
                                                                    label: "Recenter";
                                                                    icon_kind: "recenter";
                                                                    button_width: 84px;
                                                                    enabled: root.compare_view_has_targets;
                                                                    tapped => {
                                                                        compare_workspace_view.recenter_focused_row();
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            Rectangle {
                                                height: 24px;
                                                background: #fafcfe;

                                                compare_column_header := Rectangle {
                                                    width: parent.width;
                                                    height: parent.height;
                                                    background: transparent;
                                                    property <length> inner_width: max(0px, self.width - root.compare_view_column_inset * 2);
                                                    property <length> side_column_width: max(
                                                        0px,
                                                        (self.inner_width
                                                            - root.compare_view_relation_column_width
                                                            - root.compare_view_column_divider_width * 2)
                                                            / 2,
                                                    );
                                                    property <length> left_column_x: root.compare_view_column_inset;
                                                    property <length> left_divider_x: self.left_column_x + self.side_column_width;
                                                    property <length> status_column_x: self.left_divider_x + root.compare_view_column_divider_width;
                                                    property <length> right_divider_x: self.status_column_x + root.compare_view_relation_column_width;
                                                    property <length> right_column_x: self.right_divider_x + root.compare_view_column_divider_width;

                                                        Text {
                                                            x: compare_column_header.left_column_x;
                                                            y: 0px;
                                                            width: compare_column_header.side_column_width;
                                                            height: parent.height;
                                                            text: "Base";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                        vertical-alignment: center;
                                                        overflow: elide;
                                                    }

                                                        Text {
                                                            x: compare_column_header.status_column_x;
                                                            y: 0px;
                                                            width: root.compare_view_relation_column_width;
                                                            height: parent.height;
                                                            text: "Relation";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                        horizontal-alignment: center;
                                                        vertical-alignment: center;
                                                        overflow: elide;
                                                    }

                                                        Text {
                                                            x: compare_column_header.right_column_x;
                                                            y: 0px;
                                                            width: compare_column_header.side_column_width;
                                                            height: parent.height;
                                                            text: "Target";
                                                            color: #6a7c8f;
                                                            font-size: 11px;
                                                            font-weight: 600;
                                                        horizontal-alignment: left;
                                                        vertical-alignment: center;
                                                        overflow: elide;
                                                    }
                                                }
                                            }

                                            Rectangle {
                                                vertical-stretch: 1;
                                                background: #fcfdff;
                                                clip: true;

                                                compare_workspace_view := CompareView {
                                                    x: 0px;
                                                    y: 0px;
                                                    width: parent.width;
                                                    height: parent.height;
                                                    row_paths: root.compare_row_paths;
                                                    row_depths: root.compare_row_depths;
                                                    row_left_icons: root.compare_row_left_icons;
                                                    row_left_names: root.compare_row_left_names;
                                                    row_left_present: root.compare_row_left_present;
                                                    row_status_labels: root.compare_row_status_labels;
                                                    row_status_tones: root.compare_row_status_tones;
                                                    row_right_icons: root.compare_row_right_icons;
                                                    row_right_names: root.compare_row_right_names;
                                                    row_right_present: root.compare_row_right_present;
                                                    row_is_directories: root.compare_row_is_directories;
                                                    row_is_expandable: root.compare_row_is_expandable;
                                                    row_is_expanded: root.compare_row_is_expanded;
                                                    column_inset: root.compare_view_column_inset;
                                                    column_divider_width: root.compare_view_column_divider_width;
                                                    status_column_width: root.compare_view_relation_column_width;
                                                    left_content_width_px: root.compare_view_left_content_width_px;
                                                    right_content_width_px: root.compare_view_right_content_width_px;
                                                    focused_row: root.compare_row_focused_index;
                                                    interaction_enabled: !root.running && !root.diff_loading;
                                                    horizontal_scroll_locked: root.compare_view_horizontal_scroll_locked;
                                                    row_focused(path) => {
                                                        root.compare_view_row_focused(path);
                                                    }
                                                    row_toggle_requested(path) => {
                                                        root.compare_view_row_toggle_requested(path);
                                                    }
                                                    row_activated(path) => {
                                                        root.compare_view_row_activated(path);
                                                    }
                                                    row_context_menu_requested(path, anchor_x, anchor_y) => {
                                                        root.context_menu_anchor_x = anchor_x - root.absolute-position.x;
                                                        root.context_menu_anchor_y = anchor_y - root.absolute-position.y;
                                                        root.compare_view_row_context_menu_requested(path);
                                                    }
                                                    back_requested() => {
                                                        root.compare_view_up_requested();
                                                    }
                                                }

                                                if root.compare_row_paths.length == 0 : DiffStateShell {
                                                    width: parent.width;
                                                    height: parent.height;
                                                    embedded: true;
                                                    state_label: "Empty";
                                                    tone: "neutral";
                                                    title: root.compare_view_empty_title_text;
                                                    body: root.compare_view_empty_body_text;
                                                    note: root.compare_view_empty_note_text;
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
                                    corner_radius: workbench_host.panel_corner_radius;
                                }
                            }
                        }
                    }
                }
            }
        }

        if root.tooltip_visible && root.tooltip_text != "" && !root.context_menu_open && !root.settings_open : Rectangle {
            property <length> panel_margin: 10px;
            property <length> shadow_offset: 4px;
            property <length> max_bubble_width: min(520px, max(180px, root.width - panel_margin * 2));
            property <length> bubble_width: tooltip_panel.width;
            property <length> bubble_height: tooltip_panel.height;
            property <length> preferred_above_y: root.tooltip_anchor_top - bubble_height - root.tooltip_gap;
            property <length> preferred_below_y: root.tooltip_anchor_bottom + root.tooltip_gap;
            x: max(
                panel_margin,
                min(root.tooltip_anchor_x, max(panel_margin, root.width - bubble_width - panel_margin))
            );
            y: max(
                panel_margin,
                min(
                    preferred_above_y >= panel_margin ? preferred_above_y : preferred_below_y,
                    max(panel_margin, root.height - bubble_height - panel_margin),
                )
            );
            width: bubble_width;
            height: bubble_height;
            background: transparent;

            Rectangle {
                x: 0px;
                y: parent.shadow_offset;
                width: parent.width;
                height: parent.height;
                border-radius: 6px;
                background: UiPalette.tooltip_shadow;
            }

            tooltip_panel := TooltipBubble {
                x: 0px;
                y: 0px;
                text: root.tooltip_text;
                max_panel_width: parent.max_bubble_width;
            }
        }

        if root.context_menu_open : Rectangle {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            background: transparent;

            TouchArea {
                clicked => {
                    root.context_menu_close_requested();
                }
                pointer-event(event) => {
                    if event.kind == PointerEventKind.down {
                        root.context_menu_close_requested();
                    }
                }
            }

            context_menu_stack := Rectangle {
                property <length> panel_width: 228px;
                property <length> item_height: 36px;
                property <length> panel_padding: 8px;
                property <length> panel_margin: 10px;
                property <length> shadow_margin: 16px;
                property <length> panel_height: panel_padding * 2 + root.context_menu_action_labels.length * item_height;
                property <length> panel_x: max(
                    panel_margin,
                    min(root.context_menu_anchor_x, max(panel_margin, parent.width - panel_width - panel_margin))
                );
                property <length> panel_y: max(
                    panel_margin,
                    min(root.context_menu_anchor_y, max(panel_margin, parent.height - panel_height - panel_margin))
                );
                x: self.panel_x - self.shadow_margin;
                y: self.panel_y - self.shadow_margin;
                width: self.panel_width + self.shadow_margin * 2;
                height: self.panel_height + self.shadow_margin * 2;
                background: transparent;

                Rectangle {
                    x: context_menu_stack.shadow_margin + 1px;
                    y: context_menu_stack.shadow_margin + 7px;
                    width: context_menu_stack.panel_width;
                    height: context_menu_stack.panel_height + 6px;
                    border-radius: 14px;
                    background: UiPalette.context_menu_core_shadow_soft;
                }

                Rectangle {
                    x: context_menu_stack.shadow_margin;
                    y: context_menu_stack.shadow_margin + 3px;
                    width: context_menu_stack.panel_width;
                    height: context_menu_stack.panel_height + 3px;
                    border-radius: 12px;
                    background: UiPalette.context_menu_core_shadow_strong;
                }

                context_menu_panel := Rectangle {
                    x: context_menu_stack.shadow_margin;
                    y: context_menu_stack.shadow_margin;
                    width: context_menu_stack.panel_width;
                    height: context_menu_stack.panel_height;
                    border-width: 1px;
                    border-radius: 10px;
                    border-color: UiPalette.context_menu_core_border;
                    background: UiPalette.context_menu_core_background;
                    clip: true;

                    Rectangle {
                        x: 1px;
                        y: 1px;
                        width: max(0px, parent.width - 2px);
                        height: min(22px, max(0px, parent.height - 2px));
                        border-radius: 9px;
                        background: UiPalette.context_menu_core_inner_highlight;
                    }

                    Rectangle {
                        x: 0px;
                        y: 0px;
                        width: parent.width;
                        height: 1px;
                        background: rgba(255, 255, 255, 0.78);
                    }

                    TouchArea {
                        clicked => {}
                        pointer-event(_) => {}
                    }

                    VerticalLayout {
                        padding: context_menu_stack.panel_padding;
                        spacing: 0px;

                        for action_label[index] in root.context_menu_action_labels: ContextMenuActionItem {
                            label: action_label;
                            action_id: root.context_menu_action_ids[index];
                            enabled: root.context_menu_action_enabled[index];
                            activated(action_id) => {
                                root.context_menu_action_triggered(action_id);
                            }
                        }
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
                ? UiPalette.toast_tone_error_border
                : (root.toast_feedback_tone == "warn"
                    ? UiPalette.toast_tone_warn_border
                    : (root.toast_feedback_tone == "success"
                        ? UiPalette.toast_tone_success_border
                        : UiPalette.toast_tone_info_border));
            background: root.toast_feedback_tone == "error"
                ? UiPalette.toast_tone_error_background
                : (root.toast_feedback_tone == "warn"
                    ? UiPalette.toast_tone_warn_background
                    : (root.toast_feedback_tone == "success"
                        ? UiPalette.toast_tone_success_background
                        : UiPalette.toast_tone_info_background));

            toast_message := Text {
                text: root.toast_feedback_text;
                x: 14px;
                y: 0px;
                width: max(0px, parent.width - 28px);
                height: parent.height;
                color: root.toast_feedback_tone == "error"
                    ? UiPalette.toast_tone_error_text
                    : (root.toast_feedback_tone == "warn"
                        ? UiPalette.toast_tone_warn_text
                        : (root.toast_feedback_tone == "success"
                            ? UiPalette.toast_tone_success_text
                            : UiPalette.toast_tone_info_text));
                horizontal-alignment: center;
                vertical-alignment: center;
                overflow: elide;
            }
        }

        if root.workspace_session_confirm_open : Rectangle {
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            background: rgba(17, 24, 34, 0.24);

            TouchArea {}

            SectionCard {
                property <length> dialog_width: min(420px, parent.width - 40px);
                property <length> dialog_height: 180px;
                width: self.dialog_width;
                height: self.dialog_height;
                x: (parent.width - self.width) / 2;
                y: max(24px, (parent.height - self.height) / 2);
                background: #fbfdff;
                border-color: #d8e2ec;

                VerticalLayout {
                    padding: 16px;
                    spacing: 12px;

                    Text {
                        text: root.workspace_session_confirm_title;
                        color: #274460;
                        font-size: 16px;
                        font-weight: 600;
                        horizontal-stretch: 1;
                        wrap: word-wrap;
                    }

                    Text {
                        text: root.workspace_session_confirm_body;
                        color: #5e7084;
                        font-size: 13px;
                        horizontal-stretch: 1;
                        wrap: word-wrap;
                    }

                    Rectangle {
                        vertical-stretch: 1;
                        background: transparent;
                    }

                    HorizontalLayout {
                        spacing: 8px;

                        Rectangle {
                            horizontal-stretch: 1;
                            background: transparent;
                        }

                        ToolButton {
                            label: "Cancel";
                            button_min_width: 88px;
                            tapped => {
                                root.workspace_session_cancelled();
                            }
                        }

                        ToolButton {
                            label: root.workspace_session_confirm_action_label;
                            primary: true;
                            button_min_width: 110px;
                            tapped => {
                                root.workspace_session_confirmed();
                            }
                        }
                    }
                }
            }
        }

        // Contract: Settings modal.
        // Edits global provider config plus a small set of persisted preferences without changing the main shell workflow.
        Rectangle {
            visible: root.settings_open;
            x: 0px;
            y: 0px;
            width: parent.width;
            height: parent.height;
            background: rgba(17, 24, 34, 0.24);

            TouchArea {}

            SectionCard {
                property <length> dialog_width: min(760px, parent.width - 36px);
                property <length> dialog_height: min(474px, parent.height - 40px);
                width: self.dialog_width;
                height: self.dialog_height;
                x: (parent.width - self.width) / 2;
                y: max(20px, (parent.height - self.height) / 2);
                border-color: #dfe5ed;
                background: #fcfdff;

                VerticalLayout {
                    padding: 14px;
                    spacing: 10px;

                    Text {
                        text: "Settings";
                        color: #2f4966;
                        font-size: 18px;
                    }
                    Text {
                        text: "Provider configuration and application defaults.";
                        color: #6a7888;
                    }

                    Rectangle {
                        height: 1px;
                        background: #e7ecf3;
                    }

                    HorizontalLayout {
                        vertical-stretch: 1;
                        spacing: 14px;

                        Rectangle {
                            width: 132px;
                            vertical-stretch: 1;
                            background: transparent;

                            VerticalLayout {
                                spacing: 8px;

                                Text {
                                    text: "Sections";
                                    color: #718091;
                                    font-size: 11px;
                                }
                                ToolButton {
                                    label: "Provider";
                                    active: root.settings_section == 0;
                                    button_min_width: 132px;
                                    control_height: 30px;
                                    tapped => {
                                        root.settings_section = 0;
                                    }
                                }
                                ToolButton {
                                    label: "Behavior";
                                    active: root.settings_section == 1;
                                    button_min_width: 132px;
                                    control_height: 30px;
                                    tapped => {
                                        root.settings_section = 1;
                                    }
                                }
                                Rectangle {
                                    vertical-stretch: 1;
                                }
                            }
                        }

                        Rectangle {
                            width: 1px;
                            vertical-stretch: 1;
                            background: #e7ecf3;
                        }

                        Rectangle {
                            horizontal-stretch: 1;
                            vertical-stretch: 1;
                            background: transparent;

                            if root.settings_section == 0 : VerticalLayout {
                                spacing: 10px;

                                Text {
                                    text: "Provider";
                                    color: #3b4a5b;
                                    font-size: 15px;
                                }
                                Text {
                                    text: "Configure the AI analysis provider used by the Analysis tab.";
                                    color: #6a7888;
                                    wrap: word-wrap;
                                }

                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Mode";
                                        width: 110px;
                                        color: #4f6074;
                                        vertical-alignment: center;
                                    }
                                    SegmentedRail {
                                        height: 30px;
                                        HorizontalLayout {
                                            spacing: 0px;
                                            SegmentItem {
                                                label: "Mock";
                                                selected: root.settings_provider_mode == 0;
                                                show_divider: false;
                                                tapped => {
                                                    root.settings_provider_mode = 0;
                                                }
                                            }
                                            SegmentItem {
                                                label: "OpenAI-compatible";
                                                selected: root.settings_provider_mode == 1;
                                                show_divider: true;
                                                tapped => {
                                                    root.settings_provider_mode = 1;
                                                }
                                            }
                                        }
                                    }
                                }

                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Timeout";
                                        width: 110px;
                                        color: #4f6074;
                                        vertical-alignment: center;
                                    }
                                    AppLineEdit {
                                        text <=> root.settings_provider_timeout;
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

                                if root.settings_provider_mode == 1 : VerticalLayout {
                                    spacing: 6px;
                                    HorizontalLayout {
                                        spacing: 6px;
                                        Text {
                                            text: "Endpoint";
                                            width: 110px;
                                            color: #4f6074;
                                            vertical-alignment: center;
                                        }
                                        AppLineEdit {
                                            text <=> root.settings_provider_endpoint;
                                            horizontal-stretch: 1;
                                            height: 28px;
                                        }
                                    }
                                    HorizontalLayout {
                                        spacing: 6px;
                                        Text {
                                            text: "API Key";
                                            width: 110px;
                                            color: #4f6074;
                                            vertical-alignment: center;
                                        }
                                        ApiKeyLineEdit {
                                            text <=> root.settings_provider_api_key;
                                            horizontal-stretch: 1;
                                            height: 28px;
                                            revealed <=> root.settings_provider_show_api_key;
                                        }
                                    }
                                    HorizontalLayout {
                                        spacing: 6px;
                                        Text {
                                            text: "Model";
                                            width: 110px;
                                            color: #4f6074;
                                            vertical-alignment: center;
                                        }
                                        AppLineEdit {
                                            text <=> root.settings_provider_model;
                                            horizontal-stretch: 1;
                                            height: 28px;
                                        }
                                    }
                                }

                                Rectangle {
                                    vertical-stretch: 1;
                                }
                            }

                            if root.settings_section == 1 : VerticalLayout {
                                spacing: 10px;

                                Text {
                                    text: "Behavior";
                                    color: #3b4a5b;
                                    font-size: 15px;
                                }
                                Text {
                                    text: "Control the default noise level and presentation of Results / Navigator.";
                                    color: #6a7888;
                                    wrap: word-wrap;
                                }

                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Default view";
                                        width: 110px;
                                        color: #4f6074;
                                        vertical-alignment: center;
                                    }
                                    SegmentedRail {
                                        width: 220px;
                                        height: 30px;
                                        HorizontalLayout {
                                            spacing: 0px;
                                            SegmentItem {
                                                label: "Tree";
                                                selected: root.settings_default_result_view == 0;
                                                show_divider: false;
                                                tapped => {
                                                    root.settings_default_result_view = 0;
                                                }
                                            }
                                            SegmentItem {
                                                label: "Flat";
                                                selected: root.settings_default_result_view == 1;
                                                show_divider: true;
                                                tapped => {
                                                    root.settings_default_result_view = 1;
                                                }
                                            }
                                        }
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                }

                                Text {
                                    text: "Applies when Search is empty. Search results still force Flat mode.";
                                    color: #6a7888;
                                    wrap: word-wrap;
                                }

                                HorizontalLayout {
                                    spacing: 6px;
                                    Text {
                                        text: "Hidden files";
                                        width: 110px;
                                        color: #4f6074;
                                        vertical-alignment: center;
                                    }
                                    SegmentedRail {
                                        width: 220px;
                                        height: 30px;
                                        HorizontalLayout {
                                            spacing: 0px;
                                            SegmentItem {
                                                label: "Show";
                                                selected: root.settings_show_hidden_files;
                                                show_divider: false;
                                                tapped => {
                                                    root.settings_show_hidden_files = true;
                                                }
                                            }
                                            SegmentItem {
                                                label: "Hide";
                                                selected: !root.settings_show_hidden_files;
                                                show_divider: true;
                                                tapped => {
                                                    root.settings_show_hidden_files = false;
                                                }
                                            }
                                        }
                                    }
                                    Rectangle {
                                        horizontal-stretch: 1;
                                    }
                                }

                                Text {
                                    text: "Applies to dot-prefixed files and folders in Results / Navigator. Save updates the current result list immediately and also affects future compares.";
                                    color: #6a7888;
                                    wrap: word-wrap;
                                }

                                Rectangle {
                                    vertical-stretch: 1;
                                }
                            }
                        }
                    }

                    Text {
                        visible: root.settings_error_text != "";
                        text: root.settings_error_text;
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
                                root.settings_open = false;
                                root.settings_cancel_clicked();
                            }
                        }
                        ToolButton {
                            label: "Save";
                            primary: true;
                            button_min_width: 108px;
                            control_height: 30px;
                            tapped => {
                                root.settings_save_clicked();
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
    ProgrammaticInputs,
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
struct LoadingMaskState {
    phase: LoadingMaskPhase,
    generation: u64,
    timeout_reached: bool,
    last_projection: LoadingMaskProjection,
}

impl Default for LoadingMaskState {
    fn default() -> Self {
        Self {
            phase: LoadingMaskPhase::Idle,
            generation: 0,
            timeout_reached: false,
            last_projection: LoadingMaskProjection::default(),
        }
    }
}

impl LoadingMaskState {
    fn advance(
        &mut self,
        running: bool,
        diff_loading: bool,
        analysis_loading: bool,
    ) -> (Option<LoadingMaskProjection>, Option<u64>) {
        let phase = derive_loading_mask_phase(running, diff_loading, analysis_loading);
        let phase_changed = self.phase != phase;
        if phase_changed {
            self.phase = phase;
            self.generation = self.generation.wrapping_add(1);
            self.timeout_reached = false;
        }

        let projection = derive_loading_mask_projection(
            running,
            diff_loading,
            analysis_loading,
            self.timeout_reached,
        );
        let timeout_generation = if phase_changed && phase != LoadingMaskPhase::Idle {
            Some(self.generation)
        } else {
            None
        };
        if projection == self.last_projection {
            return (None, timeout_generation);
        }
        self.last_projection = projection;
        (Some(projection), timeout_generation)
    }

    fn trigger_timeout(&mut self, generation: u64) -> Option<LoadingMaskProjection> {
        if self.phase == LoadingMaskPhase::Idle
            || self.generation != generation
            || self.timeout_reached
        {
            return None;
        }

        self.timeout_reached = true;
        let projection = match self.phase {
            LoadingMaskPhase::Comparing => derive_loading_mask_projection(true, false, false, true),
            LoadingMaskPhase::DiffLoading => {
                derive_loading_mask_projection(false, true, false, true)
            }
            LoadingMaskPhase::AnalysisLoading => {
                derive_loading_mask_projection(false, false, true, true)
            }
            LoadingMaskPhase::Idle => LoadingMaskProjection::default(),
        };
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

#[derive(Clone)]
struct LoadingMaskController {
    inner: Arc<Mutex<LoadingMaskControllerInner>>,
}

struct LoadingMaskControllerInner {
    window: slint::Weak<MainWindow>,
    state: LoadingMaskState,
}

impl LoadingMaskController {
    fn new(window: &MainWindow) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LoadingMaskControllerInner {
                window: window.as_weak(),
                state: LoadingMaskState::default(),
            })),
        }
    }

    fn sync(&self, running: bool, diff_loading: bool, analysis_loading: bool) {
        let (projection, timeout_generation) = {
            let mut inner = self
                .inner
                .lock()
                .expect("loading mask controller mutex poisoned");
            inner.state.advance(running, diff_loading, analysis_loading)
        };

        if let Some(projection) = projection {
            self.render_projection(projection);
        }
        if let Some(generation) = timeout_generation {
            self.schedule_timeout(generation);
        }
    }

    fn schedule_timeout(&self, generation: u64) {
        let controller = self.clone();
        Timer::single_shot(LOADING_MASK_TIMEOUT, move || {
            controller.on_timeout(generation);
        });
    }

    fn on_timeout(&self, generation: u64) {
        let projection = {
            let mut inner = self
                .inner
                .lock()
                .expect("loading mask controller mutex poisoned");
            inner.state.trigger_timeout(generation)
        };
        if let Some(projection) = projection {
            self.render_projection(projection);
        }
    }

    fn render_projection(&self, projection: LoadingMaskProjection) {
        let window = {
            let inner = self
                .inner
                .lock()
                .expect("loading mask controller mutex poisoned");
            inner.window.clone()
        };
        let Some(window) = window.upgrade() else {
            return;
        };
        apply_loading_mask_projection(&window, projection);
    }
}

#[derive(Clone)]
struct ContextMenuController {
    inner: Rc<RefCell<ContextMenuControllerInner>>,
}

#[derive(Default)]
struct ContextMenuControllerInner {
    window: slint::Weak<MainWindow>,
    target_token: String,
    text_payload: ContextMenuTextPayload,
    custom_actions: Vec<ContextMenuCustomAction>,
}

struct ContextMenuOpenRequest {
    target_token: String,
    text_payload: ContextMenuTextPayload,
    custom_actions: Vec<ContextMenuCustomAction>,
}

impl ContextMenuOpenRequest {
    fn builtin_only(target_token: impl Into<String>, text_payload: ContextMenuTextPayload) -> Self {
        Self {
            target_token: target_token.into(),
            text_payload,
            custom_actions: Vec::new(),
        }
    }
}

impl ContextMenuController {
    fn new(window: &MainWindow) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextMenuControllerInner {
                window: window.as_weak(),
                ..ContextMenuControllerInner::default()
            })),
        }
    }

    fn open(&self, request: ContextMenuOpenRequest) {
        let ContextMenuBuildResult {
            actions,
            truncated_custom_count: _,
        } = build_action_specs(
            &request.text_payload,
            &request
                .custom_actions
                .iter()
                .map(|action| action.descriptor.clone())
                .collect::<Vec<_>>(),
        );
        let Some(window) = self.inner.borrow().window.upgrade() else {
            return;
        };
        if actions.is_empty() {
            self.close();
            return;
        }

        let action_labels = actions
            .iter()
            .map(|action| SharedString::from(action.label.clone()))
            .collect::<Vec<_>>();
        let action_ids = actions
            .iter()
            .map(|action| SharedString::from(action.action_id.clone()))
            .collect::<Vec<_>>();
        let action_enabled = actions
            .iter()
            .map(|action| action.enabled)
            .collect::<Vec<_>>();

        {
            let mut inner = self.inner.borrow_mut();
            inner.target_token = request.target_token.clone();
            inner.text_payload = request.text_payload;
            inner.custom_actions = request.custom_actions;
        }

        window.set_context_menu_target_token(request.target_token.into());
        window.set_context_menu_action_labels(ModelRc::new(VecModel::from(action_labels)));
        window.set_context_menu_action_ids(ModelRc::new(VecModel::from(action_ids)));
        window.set_context_menu_action_enabled(ModelRc::new(VecModel::from(action_enabled)));
        window.set_context_menu_open(true);
    }

    fn close(&self) {
        let Some(window) = self.inner.borrow().window.upgrade() else {
            return;
        };
        {
            let mut inner = self.inner.borrow_mut();
            inner.target_token.clear();
            inner.text_payload = ContextMenuTextPayload::default();
            inner.custom_actions.clear();
        }
        window.set_context_menu_target_token("".into());
        window.set_context_menu_open(false);
        window.set_context_menu_action_labels(ModelRc::new(VecModel::from(
            Vec::<SharedString>::new(),
        )));
        window
            .set_context_menu_action_ids(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
        window.set_context_menu_action_enabled(ModelRc::new(VecModel::from(Vec::<bool>::new())));
    }

    fn activate(&self, action_id: &str, toast_controller: &ToastController) {
        let action_id = action_id.trim().to_string();
        if action_id.is_empty() {
            return;
        }

        let invocation = {
            let inner = self.inner.borrow();
            match action_id.as_str() {
                CONTEXT_MENU_COPY_ACTION_ID => Some(ContextMenuInvocation {
                    target_token: inner.target_token.clone(),
                    action_id: action_id.clone(),
                }),
                CONTEXT_MENU_COPY_SUMMARY_ACTION_ID => Some(ContextMenuInvocation {
                    target_token: inner.target_token.clone(),
                    action_id: action_id.clone(),
                }),
                _ => inner.custom_actions.iter().find_map(|action| {
                    (action.descriptor.action_id == action_id && action.descriptor.enabled).then(
                        || ContextMenuInvocation {
                            target_token: inner.target_token.clone(),
                            action_id: action_id.clone(),
                        },
                    )
                }),
            }
        };
        if invocation.is_none() {
            return;
        }

        let custom_handler = {
            let inner = self.inner.borrow();
            inner
                .custom_actions
                .iter()
                .find(|action| {
                    action.descriptor.action_id == action_id && action.descriptor.enabled
                })
                .map(|action| action.handler.clone())
        };
        let text_payload = self.inner.borrow().text_payload.clone();

        self.close();

        match action_id.as_str() {
            CONTEXT_MENU_COPY_ACTION_ID if text_payload.copy_enabled() => {
                copy_text_with_feedback(
                    toast_controller,
                    text_payload.copy_text.as_str(),
                    text_payload.copy_feedback_label.as_str(),
                );
            }
            CONTEXT_MENU_COPY_SUMMARY_ACTION_ID if text_payload.summary_enabled() => {
                copy_text_with_feedback(
                    toast_controller,
                    text_payload.summary_text.as_str(),
                    text_payload.summary_feedback_label.as_str(),
                );
            }
            _ => {
                if let (Some(handler), Some(invocation)) = (custom_handler, invocation) {
                    handler(invocation);
                }
            }
        }
    }
}

fn derive_context_menu_sync_state(state: &AppState) -> ContextMenuSyncState {
    ContextMenuSyncState {
        selected_row: state.selected_row,
        running: state.running,
        diff_loading: state.diff_loading,
        analysis_loading: state.analysis_loading,
    }
}

// Contract: sync mode gate for editable UI fields.
// Full mode and ProgrammaticInputs mode pull editable inputs from state;
// Passive mode preserves in-flight user typing.
fn should_sync_editable_inputs(mode: SyncMode) -> bool {
    matches!(mode, SyncMode::Full | SyncMode::ProgrammaticInputs)
}

// Contract: state cache guard.
// Prevents redundant property/model writes when the presenter state snapshot is unchanged.
fn should_skip_sync(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    last_state == Some(next_state)
}

// Contract: flat navigator refresh boundary.
// Rebuild flat list models only when flat projection inputs changed.
fn should_refresh_flat_result_models(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.navigator_flat_projection_revision != next_state.navigator_flat_projection_revision
        }
    }
}

// Contract: tree navigator refresh boundary.
// Rebuild tree models only when tree projection inputs changed.
fn should_refresh_tree_result_models(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.navigator_tree_projection_revision != next_state.navigator_tree_projection_revision
        }
    }
}

fn should_apply_flat_scroll_request(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => next_state.navigator_flat_scroll_request_revision != 0,
        Some(last) => {
            last.navigator_flat_scroll_request_revision
                != next_state.navigator_flat_scroll_request_revision
        }
    }
}

fn should_apply_tree_scroll_request(last_state: Option<&AppState>, next_state: &AppState) -> bool {
    match last_state {
        None => next_state.navigator_tree_scroll_request_revision != 0,
        Some(last) => {
            last.navigator_tree_scroll_request_revision
                != next_state.navigator_tree_scroll_request_revision
        }
    }
}

fn should_refresh_compare_view_models(
    last_state: Option<&AppState>,
    next_state: &AppState,
) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.compare_view_projection_revision != next_state.compare_view_projection_revision
        }
    }
}

fn should_refresh_workspace_session_models(
    last_state: Option<&AppState>,
    next_state: &AppState,
) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.workspace_sessions != next_state.workspace_sessions
                || last.active_session_id != next_state.active_session_id
        }
    }
}

fn should_refresh_compare_file_models(
    last_state: Option<&AppState>,
    next_state: &AppState,
) -> bool {
    match last_state {
        None => true,
        Some(last) => {
            last.selected_compare_file != next_state.selected_compare_file
                || last.compare_file_view_active() != next_state.compare_file_view_active()
        }
    }
}

fn should_apply_compare_scroll_request(
    last_state: Option<&AppState>,
    next_state: &AppState,
) -> bool {
    match last_state {
        None => next_state.compare_view_scroll_request_revision != 0,
        Some(last) => {
            last.compare_view_scroll_request_revision
                != next_state.compare_view_scroll_request_revision
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

fn initialize_window_models(window: &MainWindow) {
    window.set_workspace_session_ids(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_workspace_session_labels(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_workspace_session_tooltips(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_workspace_session_kinds(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_workspace_session_closable(Rc::new(VecModel::<bool>::default()).into());
    window.set_row_statuses(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_paths(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_details(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_display_names(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_parent_paths(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_tooltip_texts(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_secondary_texts(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_row_source_indices(Rc::new(VecModel::<i32>::default()).into());
    window.set_row_can_load_diff(Rc::new(VecModel::<bool>::default()).into());
    window.set_row_display_name_matches(Rc::new(VecModel::<bool>::default()).into());
    window.set_row_parent_path_matches(Rc::new(VecModel::<bool>::default()).into());
    window.set_tree_row_keys(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_tree_row_display_names(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_tree_row_statuses(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_tree_row_tooltip_texts(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_tree_row_depths(Rc::new(VecModel::<i32>::default()).into());
    window.set_tree_row_is_directories(Rc::new(VecModel::<bool>::default()).into());
    window.set_tree_row_is_expandable(Rc::new(VecModel::<bool>::default()).into());
    window.set_tree_row_is_expanded(Rc::new(VecModel::<bool>::default()).into());
    window.set_tree_row_is_selectable(Rc::new(VecModel::<bool>::default()).into());
    window.set_tree_row_source_indices(Rc::new(VecModel::<i32>::default()).into());
    window.set_compare_row_paths(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_depths(Rc::new(VecModel::<i32>::default()).into());
    window.set_compare_row_left_icons(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_left_names(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_left_present(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_row_status_labels(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_status_tones(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_right_icons(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_right_names(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_row_right_present(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_row_is_directories(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_row_is_expandable(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_row_is_expanded(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_view_breadcrumb_labels(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_view_breadcrumb_paths(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_kinds(Rc::new(VecModel::<SharedString>::default()).into());
    window
        .set_compare_file_row_relation_labels(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_relation_tones(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_left_line_nos(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_right_line_nos(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_left_prefixes(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_left_emphasis(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_left_suffixes(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_right_prefixes(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_right_emphasis(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_right_suffixes(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_compare_file_row_left_padding(Rc::new(VecModel::<bool>::default()).into());
    window.set_compare_file_row_right_padding(Rc::new(VecModel::<bool>::default()).into());
    window.set_diff_row_kinds(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_diff_old_line_nos(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_diff_new_line_nos(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_diff_markers(Rc::new(VecModel::<SharedString>::default()).into());
    window.set_diff_contents(Rc::new(VecModel::<SharedString>::default()).into());
}

fn replace_model_contents<T: Clone + 'static>(model: ModelRc<T>, next_rows: Vec<T>, name: &str) {
    let vec_model = model
        .as_any()
        .downcast_ref::<VecModel<T>>()
        .unwrap_or_else(|| panic!("{name} must be initialized as VecModel"));
    vec_model.set_vec(next_rows);
}

fn compare_file_segment_slots(
    segments: &[crate::view_models::CompareFileTextSegmentViewModel],
) -> (String, String, String) {
    let mut prefix = String::new();
    let mut emphasis = String::new();
    let mut suffix = String::new();
    let mut seen_emphasis = false;

    for segment in segments {
        if segment.tone == "emphasis" {
            emphasis.push_str(&segment.text);
            seen_emphasis = true;
        } else if seen_emphasis {
            suffix.push_str(&segment.text);
        } else {
            prefix.push_str(&segment.text);
        }
    }

    (prefix, emphasis, suffix)
}

fn estimate_compare_file_text_width_px(text: &str) -> i32 {
    const PX_PER_WIDTH_UNIT: usize = 8;
    const EXTRA_PADDING_PX: usize = 28;
    const MIN_TEXT_WIDTH_PX: usize = 32;

    let normalized = text.replace('\t', "    ");
    let width_units = UnicodeWidthStr::width(normalized.as_str());
    let width_px = width_units
        .saturating_mul(PX_PER_WIDTH_UNIT)
        .saturating_add(EXTRA_PADDING_PX)
        .max(MIN_TEXT_WIDTH_PX);
    i32::try_from(width_px).unwrap_or(i32::MAX)
}

fn compare_file_left_content_width_px(rows: &[crate::view_models::CompareFileRowViewModel]) -> i32 {
    rows.iter()
        .map(|row| row.left_text.as_str())
        .filter(|text| !text.is_empty())
        .map(estimate_compare_file_text_width_px)
        .max()
        .unwrap_or(0)
}

fn compare_file_right_content_width_px(
    rows: &[crate::view_models::CompareFileRowViewModel],
) -> i32 {
    rows.iter()
        .map(|row| row.right_text.as_str())
        .filter(|text| !text.is_empty())
        .map(estimate_compare_file_text_width_px)
        .max()
        .unwrap_or(0)
}

fn estimate_compare_view_label_width_px(text: &str) -> i32 {
    const PX_PER_WIDTH_UNIT: usize = 8;
    const EXTRA_PADDING_PX: usize = 12;
    const MIN_LABEL_WIDTH_PX: usize = 24;

    let normalized = text.replace('\t', "    ");
    let width_units = UnicodeWidthStr::width(normalized.as_str());
    let width_px = width_units
        .saturating_mul(PX_PER_WIDTH_UNIT)
        .saturating_add(EXTRA_PADDING_PX)
        .max(MIN_LABEL_WIDTH_PX);
    i32::try_from(width_px).unwrap_or(i32::MAX)
}

fn compare_view_side_content_width_px(rows: &[CompareViewRowProjection], left_side: bool) -> i32 {
    const INDENT_PX_PER_LEVEL: i32 = 12;
    const SIDE_FIXED_WIDTH_PX: i32 = 60;

    rows.iter()
        .filter_map(|row| {
            let (present, name) = if left_side {
                (row.left_present, row.left_name.as_str())
            } else {
                (row.right_present, row.right_name.as_str())
            };
            if !present || name.is_empty() {
                return None;
            }

            Some(
                i32::from(row.depth)
                    .saturating_mul(INDENT_PX_PER_LEVEL)
                    .saturating_add(SIDE_FIXED_WIDTH_PX)
                    .saturating_add(estimate_compare_view_label_width_px(name)),
            )
        })
        .max()
        .unwrap_or(0)
}

fn sync_navigator_scroll_requests(
    window: &MainWindow,
    state: &AppState,
    last_state: Option<&AppState>,
) {
    if should_apply_flat_scroll_request(last_state, state) {
        if let Some(visual_index) = state
            .navigator_flat_scroll_target_source_index
            .and_then(|source_index| {
                state.navigator_flat_visual_row_index_for_source_index(source_index)
            })
            .and_then(|index| i32::try_from(index).ok())
        {
            window.invoke_ensure_flat_row_visible(visual_index);
        }
    }

    if should_apply_tree_scroll_request(last_state, state) {
        if let Some(visual_index) = state
            .navigator_tree_scroll_target_source_index
            .and_then(|source_index| {
                state.navigator_tree_visual_row_index_for_source_index(source_index)
            })
            .and_then(|index| i32::try_from(index).ok())
        {
            window.invoke_ensure_tree_row_visible(visual_index);
        }
    }

    if should_apply_compare_scroll_request(last_state, state) {
        if let Some(visual_index) = state
            .compare_view_scroll_target_relative_path
            .as_deref()
            .and_then(|path| state.compare_view_visual_row_index_for_path(path))
            .and_then(|index| i32::try_from(index).ok())
        {
            window.invoke_ensure_compare_row_visible(visual_index);
        }
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
    window.set_compare_status_note_text(state.compare_status_note_text().into());
    window.set_compare_status_has_detail(state.compare_status_has_detail());
    window.set_compare_summary_copy_text(state.compare_summary_copy_text().into());
    window.set_compare_detail_copy_text(state.compare_detail_copy_text().into());
    window.set_warnings_text(state.warnings_text().into());
    window.set_error_text(state.error_message.clone().unwrap_or_default().into());
    window.set_compare_truncated(state.truncated);
    window.set_compare_has_deferred(state.compare_has_deferred());
    window.set_compare_has_oversized(state.compare_has_oversized());
    window.set_results_collection_text(state.results_collection_text().into());
    window.set_navigator_runtime_view_mode(state.navigator_runtime_view_mode_text().into());
    window.set_navigator_effective_view_mode(state.navigator_effective_view_mode_text().into());
    window.set_navigator_search_forces_flat_mode(state.navigator_search_forces_flat_mode());
    window.set_sidebar_visible(state.sidebar_visible());
    window.set_workspace_mode(state.workspace_mode_text().into());
    window.set_active_workspace_session_index(state.active_workspace_session_index());
    window.set_workspace_sessions_visible(state.workspace_sessions_visible());
    window.set_compare_focus_path_raw(state.compare_focus_path_raw_text().into());
    window.set_compare_root_pair_text(state.compare_root_pair_text().into());
    window.set_compare_view_current_path_text(state.compare_view_current_path_text().into());
    window.set_compare_view_has_targets(state.compare_view_has_targets());
    window.set_compare_view_target_status_label(state.compare_view_target_status_label().into());
    window.set_compare_view_target_status_tone(state.compare_view_target_status_tone().into());
    window.set_compare_view_empty_title_text(state.compare_view_empty_title_text().into());
    window.set_compare_view_empty_body_text(state.compare_view_empty_body_text().into());
    window.set_compare_view_can_go_up(state.compare_view_can_go_up());
    window.set_compare_view_horizontal_scroll_locked(state.compare_view_horizontal_scroll_locked());
    window.set_workspace_session_confirm_open(state.workspace_session_confirmation_open());
    window.set_workspace_session_confirm_title(
        state.workspace_session_confirmation_title_text().into(),
    );
    window.set_workspace_session_confirm_body(
        state.workspace_session_confirmation_body_text().into(),
    );
    window.set_workspace_session_confirm_action_label(
        state
            .workspace_session_confirmation_action_label_text()
            .into(),
    );
    window.set_can_return_to_compare_view(state.can_return_to_compare_view());
    window.set_compare_row_focused_index(
        state
            .compare_view_focused_row_index()
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
    );
    window.set_compare_file_view_active(state.compare_file_view_active());
    window.set_compare_file_summary_text(state.compare_file_summary_text().into());
    window.set_compare_file_warning_text(state.compare_file_warning_text().into());
    window.set_compare_file_truncated(state.compare_file_truncated());
    window.set_compare_file_has_rows(state.compare_file_has_rows());
    window.set_compare_file_helper_text(state.compare_file_helper_text().into());
    window.set_compare_file_shell_state_label(state.compare_file_shell_state_label().into());
    window.set_compare_file_shell_state_tone(state.compare_file_shell_state_tone().into());
    window.set_compare_file_shell_title_text(state.compare_file_shell_title_text().into());
    window.set_compare_file_shell_body_text(state.compare_file_shell_body_text().into());
    window.set_compare_file_shell_note_text(state.compare_file_shell_note_text().into());
    window.set_workspace_tab(state.file_view_mode_tab_index());
    window.set_diff_loading(state.diff_loading);
    window.set_selected_relative_path(state.selected_relative_path_text().into());
    window.set_selected_relative_path_raw(
        state
            .selected_relative_path
            .clone()
            .unwrap_or_default()
            .into(),
    );
    window.set_file_view_title_text(state.file_view_title_text().into());
    window.set_file_view_compare_status_label(state.file_view_compare_status_label().into());
    window.set_file_view_compare_status_tone(state.file_view_compare_status_tone().into());
    window.set_file_view_path_context_text(state.file_view_path_context_text().into());
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
    window.set_show_hidden_files(state.show_hidden_files);
    window.set_default_navigator_view_mode(state.default_navigator_view_mode_text().into());
    window.set_settings_error_text(state.settings_error_text().into());
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
    if should_refresh_workspace_session_models(last_state, state) {
        let session_ids = state
            .workspace_session_ids()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_workspace_session_ids(),
            session_ids,
            "workspace_session_ids",
        );
        let session_labels = state
            .workspace_session_labels()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_workspace_session_labels(),
            session_labels,
            "workspace_session_labels",
        );
        let session_tooltips = state
            .workspace_session_tooltips()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_workspace_session_tooltips(),
            session_tooltips,
            "workspace_session_tooltips",
        );
        let session_kinds = state
            .workspace_session_kinds()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_workspace_session_kinds(),
            session_kinds,
            "workspace_session_kinds",
        );
        replace_model_contents(
            window.get_workspace_session_closable(),
            state.workspace_session_closable(),
            "workspace_session_closable",
        );
    }
    if should_refresh_flat_result_models(last_state, state) {
        let projected_rows = state.navigator_row_projections();
        let row_statuses = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.row.status.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_row_statuses(), row_statuses, "row_statuses");
        let row_paths = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.row.relative_path.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_row_paths(), row_paths, "row_paths");
        let row_details = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.row.detail.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_row_details(), row_details, "row_details");
        let row_display_names = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.display_name.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_display_names(),
            row_display_names,
            "row_display_names",
        );
        let row_parent_paths = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.parent_path.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_parent_paths(),
            row_parent_paths,
            "row_parent_paths",
        );
        let row_tooltip_texts = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.tooltip_text.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_tooltip_texts(),
            row_tooltip_texts,
            "row_tooltip_texts",
        );
        let row_secondary_texts = projected_rows
            .iter()
            .map(|projection| SharedString::from(projection.secondary_text.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_secondary_texts(),
            row_secondary_texts,
            "row_secondary_texts",
        );
        let row_source_indices = projected_rows
            .iter()
            .map(|projection| projection.source_index as i32)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_source_indices(),
            row_source_indices,
            "row_source_indices",
        );
        let row_can_load_diff = projected_rows
            .iter()
            .map(|projection| projection.row.can_load_diff)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_can_load_diff(),
            row_can_load_diff,
            "row_can_load_diff",
        );
        let row_display_name_matches = projected_rows
            .iter()
            .map(|projection| projection.display_name_matches_filter)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_display_name_matches(),
            row_display_name_matches,
            "row_display_name_matches",
        );
        let row_parent_path_matches = projected_rows
            .iter()
            .map(|projection| projection.parent_path_matches_filter)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_row_parent_path_matches(),
            row_parent_path_matches,
            "row_parent_path_matches",
        );
    }

    if should_refresh_tree_result_models(last_state, state) {
        let projected_tree_rows = state.navigator_tree_row_projections();
        let tree_row_keys = projected_tree_rows
            .iter()
            .map(|projection| SharedString::from(projection.key.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_tree_row_keys(), tree_row_keys, "tree_row_keys");
        let tree_row_display_names = projected_tree_rows
            .iter()
            .map(|projection| SharedString::from(projection.display_name.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_display_names(),
            tree_row_display_names,
            "tree_row_display_names",
        );
        let tree_row_statuses = projected_tree_rows
            .iter()
            .map(|projection| SharedString::from(projection.display_status.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_statuses(),
            tree_row_statuses,
            "tree_row_statuses",
        );
        let tree_row_tooltip_texts = projected_tree_rows
            .iter()
            .map(|projection| SharedString::from(projection.tooltip_text.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_tooltip_texts(),
            tree_row_tooltip_texts,
            "tree_row_tooltip_texts",
        );
        let tree_row_depths = projected_tree_rows
            .iter()
            .map(|projection| i32::from(projection.depth))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_depths(),
            tree_row_depths,
            "tree_row_depths",
        );
        let tree_row_is_directories = projected_tree_rows
            .iter()
            .map(|projection| projection.is_directory)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_is_directories(),
            tree_row_is_directories,
            "tree_row_is_directories",
        );
        let tree_row_is_expandable = projected_tree_rows
            .iter()
            .map(|projection| projection.is_expandable)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_is_expandable(),
            tree_row_is_expandable,
            "tree_row_is_expandable",
        );
        let tree_row_is_expanded = projected_tree_rows
            .iter()
            .map(|projection| projection.is_expanded)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_is_expanded(),
            tree_row_is_expanded,
            "tree_row_is_expanded",
        );
        let tree_row_is_selectable = projected_tree_rows
            .iter()
            .map(|projection| projection.is_selectable)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_is_selectable(),
            tree_row_is_selectable,
            "tree_row_is_selectable",
        );
        let tree_row_source_indices = projected_tree_rows
            .iter()
            .map(|projection| {
                projection
                    .source_index
                    .and_then(|value| i32::try_from(value).ok())
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_tree_row_source_indices(),
            tree_row_source_indices,
            "tree_row_source_indices",
        );
    }

    if should_refresh_compare_view_models(last_state, state) {
        let projected_compare_rows = state.compare_view_row_projections();
        let compare_view_breadcrumb_labels = state
            .compare_view_breadcrumb_labels()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_view_breadcrumb_labels(),
            compare_view_breadcrumb_labels,
            "compare_view_breadcrumb_labels",
        );
        let compare_view_breadcrumb_paths = state
            .compare_view_breadcrumb_paths()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_view_breadcrumb_paths(),
            compare_view_breadcrumb_paths,
            "compare_view_breadcrumb_paths",
        );
        let compare_row_paths = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.relative_path.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_paths(),
            compare_row_paths,
            "compare_row_paths",
        );
        let compare_row_depths = projected_compare_rows
            .iter()
            .map(|projection| i32::from(projection.depth))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_depths(),
            compare_row_depths,
            "compare_row_depths",
        );
        let compare_row_left_icons = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.left_icon.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_left_icons(),
            compare_row_left_icons,
            "compare_row_left_icons",
        );
        let compare_row_left_names = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.left_name.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_left_names(),
            compare_row_left_names,
            "compare_row_left_names",
        );
        let compare_row_left_present = projected_compare_rows
            .iter()
            .map(|projection| projection.left_present)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_left_present(),
            compare_row_left_present,
            "compare_row_left_present",
        );
        let compare_row_status_labels = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.status_label.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_status_labels(),
            compare_row_status_labels,
            "compare_row_status_labels",
        );
        let compare_row_status_tones = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.status_tone.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_status_tones(),
            compare_row_status_tones,
            "compare_row_status_tones",
        );
        let compare_row_right_icons = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.right_icon.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_right_icons(),
            compare_row_right_icons,
            "compare_row_right_icons",
        );
        let compare_row_right_names = projected_compare_rows
            .iter()
            .map(|projection| SharedString::from(projection.right_name.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_right_names(),
            compare_row_right_names,
            "compare_row_right_names",
        );
        let compare_row_right_present = projected_compare_rows
            .iter()
            .map(|projection| projection.right_present)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_right_present(),
            compare_row_right_present,
            "compare_row_right_present",
        );
        let compare_row_is_directories = projected_compare_rows
            .iter()
            .map(|projection| projection.is_directory)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_is_directories(),
            compare_row_is_directories,
            "compare_row_is_directories",
        );
        let compare_row_is_expandable = projected_compare_rows
            .iter()
            .map(|projection| projection.is_expandable)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_is_expandable(),
            compare_row_is_expandable,
            "compare_row_is_expandable",
        );
        let compare_row_is_expanded = projected_compare_rows
            .iter()
            .map(|projection| projection.is_expanded)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_row_is_expanded(),
            compare_row_is_expanded,
            "compare_row_is_expanded",
        );
        window.set_compare_view_left_content_width_px(compare_view_side_content_width_px(
            &projected_compare_rows,
            true,
        ));
        window.set_compare_view_right_content_width_px(compare_view_side_content_width_px(
            &projected_compare_rows,
            false,
        ));
    }

    if should_refresh_compare_file_models(last_state, state) {
        let compare_file_rows = state.compare_file_row_projections();
        window.set_compare_file_left_content_width_px(compare_file_left_content_width_px(
            &compare_file_rows,
        ));
        window.set_compare_file_right_content_width_px(compare_file_right_content_width_px(
            &compare_file_rows,
        ));
        let compare_file_row_kinds = compare_file_rows
            .iter()
            .map(|row| SharedString::from(row.row_kind.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_kinds(),
            compare_file_row_kinds,
            "compare_file_row_kinds",
        );
        let compare_file_row_relation_labels = compare_file_rows
            .iter()
            .map(|row| SharedString::from(row.relation_label.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_relation_labels(),
            compare_file_row_relation_labels,
            "compare_file_row_relation_labels",
        );
        let compare_file_row_relation_tones = compare_file_rows
            .iter()
            .map(|row| SharedString::from(row.relation_tone.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_relation_tones(),
            compare_file_row_relation_tones,
            "compare_file_row_relation_tones",
        );
        let compare_file_row_left_line_nos = compare_file_rows
            .iter()
            .map(|row| {
                SharedString::from(
                    row.left_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_left_line_nos(),
            compare_file_row_left_line_nos,
            "compare_file_row_left_line_nos",
        );
        let compare_file_row_right_line_nos = compare_file_rows
            .iter()
            .map(|row| {
                SharedString::from(
                    row.right_line_no
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_right_line_nos(),
            compare_file_row_right_line_nos,
            "compare_file_row_right_line_nos",
        );
        let compare_file_row_left_prefixes = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.left_segments).0))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_left_prefixes(),
            compare_file_row_left_prefixes,
            "compare_file_row_left_prefixes",
        );
        let compare_file_row_left_emphasis = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.left_segments).1))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_left_emphasis(),
            compare_file_row_left_emphasis,
            "compare_file_row_left_emphasis",
        );
        let compare_file_row_left_suffixes = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.left_segments).2))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_left_suffixes(),
            compare_file_row_left_suffixes,
            "compare_file_row_left_suffixes",
        );
        let compare_file_row_right_prefixes = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.right_segments).0))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_right_prefixes(),
            compare_file_row_right_prefixes,
            "compare_file_row_right_prefixes",
        );
        let compare_file_row_right_emphasis = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.right_segments).1))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_right_emphasis(),
            compare_file_row_right_emphasis,
            "compare_file_row_right_emphasis",
        );
        let compare_file_row_right_suffixes = compare_file_rows
            .iter()
            .map(|row| SharedString::from(compare_file_segment_slots(&row.right_segments).2))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_right_suffixes(),
            compare_file_row_right_suffixes,
            "compare_file_row_right_suffixes",
        );
        let compare_file_row_left_padding = compare_file_rows
            .iter()
            .map(|row| row.left_padding)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_left_padding(),
            compare_file_row_left_padding,
            "compare_file_row_left_padding",
        );
        let compare_file_row_right_padding = compare_file_rows
            .iter()
            .map(|row| row.right_padding)
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_compare_file_row_right_padding(),
            compare_file_row_right_padding,
            "compare_file_row_right_padding",
        );
    }

    sync_navigator_scroll_requests(window, state, last_state);

    if should_refresh_diff_models(last_state, state) {
        let diff_rows = state.diff_viewer_rows();
        let diff_row_kinds = diff_rows
            .iter()
            .map(|row| SharedString::from(row.row_kind.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_diff_row_kinds(),
            diff_row_kinds,
            "diff_row_kinds",
        );
        let diff_old_line_nos = diff_rows
            .iter()
            .map(|row| SharedString::from(row.old_line_no.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_diff_old_line_nos(),
            diff_old_line_nos,
            "diff_old_line_nos",
        );
        let diff_new_line_nos = diff_rows
            .iter()
            .map(|row| SharedString::from(row.new_line_no.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(
            window.get_diff_new_line_nos(),
            diff_new_line_nos,
            "diff_new_line_nos",
        );
        let diff_markers = diff_rows
            .iter()
            .map(|row| SharedString::from(row.marker.clone()))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_diff_markers(), diff_markers, "diff_markers");
        let diff_contents = diff_rows
            .into_iter()
            .map(|row| SharedString::from(row.content))
            .collect::<Vec<_>>();
        replace_model_contents(window.get_diff_contents(), diff_contents, "diff_contents");
    }
}

fn sync_window_snapshot_if_changed(
    window: &MainWindow,
    state: AppState,
    cache: &Arc<Mutex<Option<AppState>>>,
    context_menu_controller: Option<&ContextMenuController>,
    loading_mask_controller: &LoadingMaskController,
    mode: SyncMode,
) {
    let mut cache_guard = cache.lock().expect("sync cache mutex poisoned");
    if should_skip_sync(cache_guard.as_ref(), &state) {
        return;
    }
    if let Some(last_state) = cache_guard.as_ref() {
        if let Some(context_menu_controller) = context_menu_controller {
            if should_close_for_sync_transition(
                derive_context_menu_sync_state(last_state),
                derive_context_menu_sync_state(&state),
            ) {
                context_menu_controller.close();
            }
        }
    }
    sync_window_state(window, &state, mode, cache_guard.as_ref());
    loading_mask_controller.sync(state.running, state.diff_loading, state.analysis_loading);
    *cache_guard = Some(state);
}

// Contract: cache-aware sync wrapper used by UI-thread callbacks.
fn sync_window_state_if_changed(
    window: &MainWindow,
    bridge: &UiBridge,
    cache: &Arc<Mutex<Option<AppState>>>,
    context_menu_controller: Option<&ContextMenuController>,
    loading_mask_controller: &LoadingMaskController,
    mode: SyncMode,
) {
    sync_window_snapshot_if_changed(
        window,
        bridge.snapshot(),
        cache,
        context_menu_controller,
        loading_mask_controller,
        mode,
    );
}

#[derive(Clone)]
struct UiSyncController {
    inner: Arc<UiSyncControllerInner>,
}

struct UiSyncControllerInner {
    window: slint::Weak<MainWindow>,
    state: Arc<Mutex<AppState>>,
    cache: Arc<Mutex<Option<AppState>>>,
    loading_mask_controller: LoadingMaskController,
    pending: AtomicBool,
}

impl UiSyncController {
    fn new(
        window: &MainWindow,
        state: Arc<Mutex<AppState>>,
        cache: Arc<Mutex<Option<AppState>>>,
        loading_mask_controller: LoadingMaskController,
    ) -> Self {
        Self {
            inner: Arc::new(UiSyncControllerInner {
                window: window.as_weak(),
                state,
                cache,
                loading_mask_controller,
                pending: AtomicBool::new(false),
            }),
        }
    }

    fn request_passive_sync(&self) {
        if self.inner.pending.swap(true, Ordering::AcqRel) {
            return;
        }

        let controller = self.clone();
        let enqueue_result = self.inner.window.upgrade_in_event_loop(move |window| {
            controller.inner.pending.store(false, Ordering::Release);
            let state = controller
                .inner
                .state
                .lock()
                .expect("app state mutex poisoned")
                .clone();
            sync_window_snapshot_if_changed(
                &window,
                state,
                &controller.inner.cache,
                None,
                &controller.inner.loading_mask_controller,
                SyncMode::Passive,
            );
        });
        if enqueue_result.is_err() {
            self.inner.pending.store(false, Ordering::Release);
        }
    }
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
    window_chrome::install_platform_windowing()?;
    let app = MainWindow::new().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    app.set_immersive_titlebar_enabled(window_chrome::immersive_titlebar_enabled());
    app.set_titlebar_visual_height(window_chrome::titlebar_visual_height().into());
    app.set_titlebar_leading_inset(window_chrome::titlebar_leading_inset().into());
    initialize_window_models(&app);

    let state = Arc::new(Mutex::new(AppState::default()));
    let presenter = Presenter::new(Arc::clone(&state));
    let bridge = UiBridge::new(presenter.clone());
    bridge.dispatch(UiCommand::Initialize);
    let initial_state = bridge.snapshot();
    sync_window_state(&app, &initial_state, SyncMode::Full, None);
    let sync_cache = Arc::new(Mutex::new(Some(initial_state.clone())));
    let toast_controller = ToastController::new(&app);
    let context_menu_controller = ContextMenuController::new(&app);
    let loading_mask_controller = LoadingMaskController::new(&app);
    loading_mask_controller.sync(
        initial_state.running,
        initial_state.diff_loading,
        initial_state.analysis_loading,
    );
    let async_sync_controller = UiSyncController::new(
        &app,
        Arc::clone(&state),
        Arc::clone(&sync_cache),
        loading_mask_controller.clone(),
    );
    presenter.set_state_change_notifier(Arc::new(move || {
        async_sync_controller.request_passive_sync();
    }));

    // Contract: UI event dispatch and bridge binding.
    // Each callback converts UI intent into UiCommand(s), then triggers passive sync.

    let close_context_menu_controller = context_menu_controller.clone();
    app.on_context_menu_close_requested(move || {
        close_context_menu_controller.close();
    });

    let app_weak = app.as_weak();
    app.on_titlebar_drag_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        window_chrome::request_titlebar_drag(&window.window());
    });

    let app_weak = app.as_weak();
    let compare_status_context_menu_controller = context_menu_controller.clone();
    app.on_compare_status_context_menu_requested(move |summary_text, detail_text| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        let payload = build_compare_status_payload(summary_text.as_str(), detail_text.as_str());
        let target_token = format!(
            "compare-status:{}",
            window
                .get_status_text()
                .to_string()
                .trim()
                .to_ascii_lowercase()
        );
        compare_status_context_menu_controller
            .open(ContextMenuOpenRequest::builtin_only(target_token, payload));
    });

    let app_weak = app.as_weak();
    let results_context_menu_bridge = bridge.clone();
    let results_context_menu_cache = Arc::clone(&sync_cache);
    let results_context_menu_controller = context_menu_controller.clone();
    let results_context_menu_loading_mask_controller = loading_mask_controller.clone();
    app.on_results_context_menu_requested(
        move |source_index, path, status, detail, unavailable| {
            let payload = build_results_row_payload(
                path.as_str(),
                status.as_str(),
                detail.as_str(),
                unavailable,
            );
            let target_token = format!("results:{}", path.as_str().trim());
            let source_index = usize::try_from(source_index).ok();
            let relative_path = path.to_string();
            let snapshot = results_context_menu_bridge.snapshot();
            let row = source_index
                .and_then(|index| snapshot.entry_rows.get(index))
                .cloned();
            let can_locate_and_open = row.as_ref().is_some_and(|row| row.entry_kind == "file");
            let can_open_compare_view = row
                .as_ref()
                .is_some_and(|row| row.entry_kind == "directory");
            let mut custom_actions = Vec::new();
            if can_locate_and_open {
                let locate_relative_path = relative_path.clone();
                let action_app_weak = app_weak.clone();
                let action_bridge = results_context_menu_bridge.clone();
                let action_cache = Arc::clone(&results_context_menu_cache);
                let action_context_menu_controller = results_context_menu_controller.clone();
                let action_loading_mask_controller =
                    results_context_menu_loading_mask_controller.clone();
                custom_actions.push(ContextMenuCustomAction {
                    descriptor: ContextMenuCustomActionDescriptor {
                        label: "Locate and Open".to_string(),
                        action_id: RESULTS_LOCATE_AND_OPEN_ACTION_ID.to_string(),
                        enabled: true,
                    },
                    handler: Rc::new(move |_invocation| {
                        let Some(window) = action_app_weak.upgrade() else {
                            return;
                        };

                        window.set_workspace_tab(0);
                        action_bridge
                            .dispatch(UiCommand::LocateAndOpen(locate_relative_path.clone()));
                        sync_window_state_if_changed(
                            &window,
                            &action_bridge,
                            &action_cache,
                            Some(&action_context_menu_controller),
                            &action_loading_mask_controller,
                            SyncMode::ProgrammaticInputs,
                        );
                    }),
                });
            }
            if can_open_compare_view {
                let compare_relative_path = relative_path.clone();
                let action_app_weak = app_weak.clone();
                let action_bridge = results_context_menu_bridge.clone();
                let action_cache = Arc::clone(&results_context_menu_cache);
                let action_context_menu_controller = results_context_menu_controller.clone();
                let action_loading_mask_controller =
                    results_context_menu_loading_mask_controller.clone();
                custom_actions.push(ContextMenuCustomAction {
                    descriptor: ContextMenuCustomActionDescriptor {
                        label: "Open in Compare View".to_string(),
                        action_id: RESULTS_OPEN_IN_COMPARE_VIEW_ACTION_ID.to_string(),
                        enabled: true,
                    },
                    handler: Rc::new(move |_invocation| {
                        let Some(window) = action_app_weak.upgrade() else {
                            return;
                        };

                        action_bridge
                            .dispatch(UiCommand::OpenCompareView(compare_relative_path.clone()));
                        sync_window_state_if_changed(
                            &window,
                            &action_bridge,
                            &action_cache,
                            Some(&action_context_menu_controller),
                            &action_loading_mask_controller,
                            SyncMode::ProgrammaticInputs,
                        );
                        if !window.get_workspace_session_confirm_open()
                            && window.get_workspace_mode() == "compare-view"
                        {
                            window.invoke_focus_compare_rows();
                        }
                    }),
                });
            }
            results_context_menu_controller.open(ContextMenuOpenRequest {
                target_token,
                text_payload: payload,
                custom_actions,
            });
        },
    );

    let app_weak = app.as_weak();
    let tree_context_menu_controller = context_menu_controller.clone();
    let tree_context_menu_bridge = bridge.clone();
    let tree_context_menu_cache = Arc::clone(&sync_cache);
    let tree_context_menu_loading_mask_controller = loading_mask_controller.clone();
    app.on_navigator_tree_context_menu_requested(move |key, status, directory, source_index| {
        if app_weak.upgrade().is_none() {
            return;
        }

        let payload = build_results_row_payload(
            key.as_str(),
            status.as_str(),
            if directory {
                "directory compare target"
            } else {
                "tree result entry"
            },
            false,
        );
        let target_token = format!("tree-results:{}", key.as_str().trim());
        let mut custom_actions = Vec::new();
        if directory && key.as_str().trim() != "" {
            let action_app_weak = app_weak.clone();
            let action_bridge = tree_context_menu_bridge.clone();
            let action_cache = Arc::clone(&tree_context_menu_cache);
            let action_context_menu_controller = tree_context_menu_controller.clone();
            let action_loading_mask_controller = tree_context_menu_loading_mask_controller.clone();
            let relative_path = key.to_string();
            custom_actions.push(ContextMenuCustomAction {
                descriptor: ContextMenuCustomActionDescriptor {
                    label: "Open in Compare View".to_string(),
                    action_id: RESULTS_OPEN_IN_COMPARE_VIEW_ACTION_ID.to_string(),
                    enabled: true,
                },
                handler: Rc::new(move |_invocation| {
                    let Some(window) = action_app_weak.upgrade() else {
                        return;
                    };

                    action_bridge.dispatch(UiCommand::OpenCompareView(relative_path.clone()));
                    sync_window_state_if_changed(
                        &window,
                        &action_bridge,
                        &action_cache,
                        Some(&action_context_menu_controller),
                        &action_loading_mask_controller,
                        SyncMode::ProgrammaticInputs,
                    );
                    if !window.get_workspace_session_confirm_open()
                        && window.get_workspace_mode() == "compare-view"
                    {
                        window.invoke_focus_compare_rows();
                    }
                }),
            });
        } else if source_index >= 0 {
            let source_index = source_index;
            let action_app_weak = app_weak.clone();
            let action_bridge = tree_context_menu_bridge.clone();
            let action_cache = Arc::clone(&tree_context_menu_cache);
            let action_context_menu_controller = tree_context_menu_controller.clone();
            let action_loading_mask_controller = tree_context_menu_loading_mask_controller.clone();
            custom_actions.push(ContextMenuCustomAction {
                descriptor: ContextMenuCustomActionDescriptor {
                    label: "Open File View".to_string(),
                    action_id: RESULTS_LOCATE_AND_OPEN_ACTION_ID.to_string(),
                    enabled: true,
                },
                handler: Rc::new(move |_invocation| {
                    let Some(window) = action_app_weak.upgrade() else {
                        return;
                    };

                    window.set_workspace_tab(0);
                    action_bridge.dispatch(UiCommand::SelectRow(source_index));
                    action_bridge.dispatch(UiCommand::LoadSelectedDiff);
                    sync_window_state_if_changed(
                        &window,
                        &action_bridge,
                        &action_cache,
                        Some(&action_context_menu_controller),
                        &action_loading_mask_controller,
                        SyncMode::ProgrammaticInputs,
                    );
                }),
            });
        }

        tree_context_menu_controller.open(ContextMenuOpenRequest {
            target_token,
            text_payload: payload,
            custom_actions,
        });
    });

    let app_weak = app.as_weak();
    let header_context_menu_controller = context_menu_controller.clone();
    app.on_workspace_header_context_menu_requested(
        move |relative_path, mode_label, status_label, summary_text, hint_text| {
            let Some(window) = app_weak.upgrade() else {
                return;
            };
            let payload = build_workspace_header_payload(
                relative_path.as_str(),
                mode_label.as_str(),
                status_label.as_str(),
                summary_text.as_str(),
                hint_text.as_str(),
            );
            let target_token = format!(
                "workspace-header:{}:{}",
                mode_label.as_str().trim().to_ascii_lowercase(),
                window.get_selected_relative_path_raw().to_string()
            );
            header_context_menu_controller
                .open(ContextMenuOpenRequest::builtin_only(target_token, payload));
        },
    );

    let app_weak = app.as_weak();
    let analysis_context_menu_controller = context_menu_controller.clone();
    app.on_analysis_section_context_menu_requested(
        move |section_label, title, body, copy_value| {
            let Some(window) = app_weak.upgrade() else {
                return;
            };
            let payload = build_analysis_section_payload(
                section_label.as_str(),
                title.as_str(),
                body.as_str(),
                copy_value.as_str(),
            );
            let target_token = format!(
                "analysis-section:{}:{}",
                section_label
                    .as_str()
                    .trim()
                    .to_ascii_lowercase()
                    .replace(' ', "-"),
                window.get_selected_relative_path_raw().to_string()
            );
            analysis_context_menu_controller
                .open(ContextMenuOpenRequest::builtin_only(target_token, payload));
        },
    );

    let context_menu_toast_controller = toast_controller.clone();
    let action_context_menu_controller = context_menu_controller.clone();
    app.on_context_menu_action_triggered(move |action_id| {
        action_context_menu_controller.activate(action_id.as_str(), &context_menu_toast_controller);
    });

    // Compare flow callbacks.
    let app_weak = app.as_weak();
    let compare_bridge = bridge.clone();
    let compare_cache = Arc::clone(&sync_cache);
    let compare_context_menu_controller = context_menu_controller.clone();
    let compare_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_context_menu_controller.close();
        window.set_compare_status_details_expanded(false);
        compare_bridge.dispatch(UiCommand::UpdateLeftRoot(
            window.get_left_root().to_string(),
        ));
        compare_bridge.dispatch(UiCommand::UpdateRightRoot(
            window.get_right_root().to_string(),
        ));
        compare_bridge.dispatch(UiCommand::RunCompare);
        sync_window_state_if_changed(
            &window,
            &compare_bridge,
            &compare_cache,
            Some(&compare_context_menu_controller),
            &compare_loading_mask_controller,
            SyncMode::Passive,
        );
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
    let row_context_menu_controller = context_menu_controller.clone();
    let row_loading_mask_controller = loading_mask_controller.clone();
    app.on_row_selected(move |index| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_diff_loading() {
            return;
        }
        row_context_menu_controller.close();
        window.set_workspace_tab(0);
        if window.get_selected_row() == index
            && !window.get_workspace_sessions_visible()
            && window.get_workspace_mode() == "file-view"
            && (window.get_diff_loaded() || window.get_diff_loading())
        {
            return;
        }

        row_bridge.dispatch(UiCommand::SelectRow(index));
        sync_window_state_if_changed(
            &window,
            &row_bridge,
            &row_cache,
            Some(&row_context_menu_controller),
            &row_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_session_confirm_open() {
            return;
        }
        row_bridge.dispatch(UiCommand::LoadSelectedDiff);
        sync_window_state_if_changed(
            &window,
            &row_bridge,
            &row_cache,
            Some(&row_context_menu_controller),
            &row_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let filter_bridge = bridge.clone();
    let filter_cache = Arc::clone(&sync_cache);
    let filter_context_menu_controller = context_menu_controller.clone();
    let filter_loading_mask_controller = loading_mask_controller.clone();
    app.on_filter_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        filter_bridge.dispatch(UiCommand::UpdateEntryFilter(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &filter_bridge,
            &filter_cache,
            Some(&filter_context_menu_controller),
            &filter_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let tree_mode_bridge = bridge.clone();
    let tree_mode_cache = Arc::clone(&sync_cache);
    let tree_mode_context_menu_controller = context_menu_controller.clone();
    let tree_mode_loading_mask_controller = loading_mask_controller.clone();
    app.on_navigator_view_mode_tree_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        tree_mode_context_menu_controller.close();
        tree_mode_bridge.dispatch(UiCommand::SetNavigatorViewModeTree);
        sync_window_state_if_changed(
            &window,
            &tree_mode_bridge,
            &tree_mode_cache,
            Some(&tree_mode_context_menu_controller),
            &tree_mode_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let flat_mode_bridge = bridge.clone();
    let flat_mode_cache = Arc::clone(&sync_cache);
    let flat_mode_context_menu_controller = context_menu_controller.clone();
    let flat_mode_loading_mask_controller = loading_mask_controller.clone();
    app.on_navigator_view_mode_flat_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        flat_mode_context_menu_controller.close();
        flat_mode_bridge.dispatch(UiCommand::SetNavigatorViewModeFlat);
        sync_window_state_if_changed(
            &window,
            &flat_mode_bridge,
            &flat_mode_cache,
            Some(&flat_mode_context_menu_controller),
            &flat_mode_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let tree_toggle_bridge = bridge.clone();
    let tree_toggle_cache = Arc::clone(&sync_cache);
    let tree_toggle_context_menu_controller = context_menu_controller.clone();
    let tree_toggle_loading_mask_controller = loading_mask_controller.clone();
    app.on_navigator_tree_directory_toggled(move |key| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_diff_loading() {
            return;
        }

        tree_toggle_context_menu_controller.close();
        tree_toggle_bridge.dispatch(UiCommand::ToggleNavigatorTreeNode(key.to_string()));
        sync_window_state_if_changed(
            &window,
            &tree_toggle_bridge,
            &tree_toggle_cache,
            Some(&tree_toggle_context_menu_controller),
            &tree_toggle_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let tree_file_bridge = bridge.clone();
    let tree_file_cache = Arc::clone(&sync_cache);
    let tree_file_context_menu_controller = context_menu_controller.clone();
    let tree_file_loading_mask_controller = loading_mask_controller.clone();
    app.on_navigator_tree_file_selected(move |index| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };
        if window.get_diff_loading() {
            return;
        }
        tree_file_context_menu_controller.close();
        window.set_workspace_tab(0);
        if window.get_selected_row() == index
            && !window.get_workspace_sessions_visible()
            && window.get_workspace_mode() == "file-view"
            && (window.get_diff_loaded() || window.get_diff_loading())
        {
            return;
        }

        tree_file_bridge.dispatch(UiCommand::SelectRow(index));
        sync_window_state_if_changed(
            &window,
            &tree_file_bridge,
            &tree_file_cache,
            Some(&tree_file_context_menu_controller),
            &tree_file_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_session_confirm_open() {
            return;
        }
        tree_file_bridge.dispatch(UiCommand::LoadSelectedDiff);
        sync_window_state_if_changed(
            &window,
            &tree_file_bridge,
            &tree_file_cache,
            Some(&tree_file_context_menu_controller),
            &tree_file_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let status_filter_bridge = bridge.clone();
    let status_filter_cache = Arc::clone(&sync_cache);
    let status_filter_context_menu_controller = context_menu_controller.clone();
    let status_filter_loading_mask_controller = loading_mask_controller.clone();
    app.on_status_filter_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        status_filter_bridge.dispatch(UiCommand::UpdateEntryStatusFilter(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &status_filter_bridge,
            &status_filter_cache,
            Some(&status_filter_context_menu_controller),
            &status_filter_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let sidebar_toggle_bridge = bridge.clone();
    let sidebar_toggle_cache = Arc::clone(&sync_cache);
    let sidebar_toggle_context_menu_controller = context_menu_controller.clone();
    let sidebar_toggle_loading_mask_controller = loading_mask_controller.clone();
    app.on_sidebar_toggle_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        sidebar_toggle_context_menu_controller.close();
        sidebar_toggle_bridge.dispatch(UiCommand::ToggleSidebarVisibility);
        sync_window_state_if_changed(
            &window,
            &sidebar_toggle_bridge,
            &sidebar_toggle_cache,
            Some(&sidebar_toggle_context_menu_controller),
            &sidebar_toggle_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let session_select_bridge = bridge.clone();
    let session_select_cache = Arc::clone(&sync_cache);
    let session_select_context_menu_controller = context_menu_controller.clone();
    let session_select_loading_mask_controller = loading_mask_controller.clone();
    app.on_workspace_session_selected(move |session_id| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        session_select_context_menu_controller.close();
        session_select_bridge.dispatch(UiCommand::SelectWorkspaceSession(session_id.to_string()));
        sync_window_state_if_changed(
            &window,
            &session_select_bridge,
            &session_select_cache,
            Some(&session_select_context_menu_controller),
            &session_select_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let session_close_bridge = bridge.clone();
    let session_close_cache = Arc::clone(&sync_cache);
    let session_close_context_menu_controller = context_menu_controller.clone();
    let session_close_loading_mask_controller = loading_mask_controller.clone();
    app.on_workspace_session_close_requested(move |session_id| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        session_close_context_menu_controller.close();
        session_close_bridge.dispatch(UiCommand::CloseWorkspaceSession(session_id.to_string()));
        sync_window_state_if_changed(
            &window,
            &session_close_bridge,
            &session_close_cache,
            Some(&session_close_context_menu_controller),
            &session_close_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let confirm_close_bridge = bridge.clone();
    let confirm_close_cache = Arc::clone(&sync_cache);
    let confirm_close_context_menu_controller = context_menu_controller.clone();
    let confirm_close_loading_mask_controller = loading_mask_controller.clone();
    app.on_workspace_session_confirmed(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        confirm_close_context_menu_controller.close();
        confirm_close_bridge.dispatch(UiCommand::ConfirmWorkspaceSessionAction);
        sync_window_state_if_changed(
            &window,
            &confirm_close_bridge,
            &confirm_close_cache,
            Some(&confirm_close_context_menu_controller),
            &confirm_close_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let cancel_close_bridge = bridge.clone();
    let cancel_close_cache = Arc::clone(&sync_cache);
    let cancel_close_context_menu_controller = context_menu_controller.clone();
    let cancel_close_loading_mask_controller = loading_mask_controller.clone();
    app.on_workspace_session_cancelled(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        cancel_close_context_menu_controller.close();
        cancel_close_bridge.dispatch(UiCommand::CancelWorkspaceSessionAction);
        sync_window_state_if_changed(
            &window,
            &cancel_close_bridge,
            &cancel_close_cache,
            Some(&cancel_close_context_menu_controller),
            &cancel_close_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let diff_mode_bridge = bridge.clone();
    let diff_mode_cache = Arc::clone(&sync_cache);
    let diff_mode_context_menu_controller = context_menu_controller.clone();
    let diff_mode_loading_mask_controller = loading_mask_controller.clone();
    app.on_file_view_diff_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        diff_mode_context_menu_controller.close();
        diff_mode_bridge.dispatch(UiCommand::SetFileViewModeDiff);
        sync_window_state_if_changed(
            &window,
            &diff_mode_bridge,
            &diff_mode_cache,
            Some(&diff_mode_context_menu_controller),
            &diff_mode_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let analysis_mode_bridge = bridge.clone();
    let analysis_mode_cache = Arc::clone(&sync_cache);
    let analysis_mode_context_menu_controller = context_menu_controller.clone();
    let analysis_mode_loading_mask_controller = loading_mask_controller.clone();
    app.on_file_view_analysis_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        analysis_mode_context_menu_controller.close();
        analysis_mode_bridge.dispatch(UiCommand::SetFileViewModeAnalysis);
        sync_window_state_if_changed(
            &window,
            &analysis_mode_bridge,
            &analysis_mode_cache,
            Some(&analysis_mode_context_menu_controller),
            &analysis_mode_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let compare_root_bridge = bridge.clone();
    let compare_root_cache = Arc::clone(&sync_cache);
    let compare_root_context_menu_controller = context_menu_controller.clone();
    let compare_root_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_root_view_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_root_context_menu_controller.close();
        compare_root_bridge.dispatch(UiCommand::OpenCompareView(String::new()));
        sync_window_state_if_changed(
            &window,
            &compare_root_bridge,
            &compare_root_cache,
            Some(&compare_root_context_menu_controller),
            &compare_root_loading_mask_controller,
            SyncMode::Passive,
        );
        if !window.get_workspace_session_confirm_open()
            && window.get_workspace_mode() == "compare-view"
        {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let compare_up_bridge = bridge.clone();
    let compare_up_cache = Arc::clone(&sync_cache);
    let compare_up_context_menu_controller = context_menu_controller.clone();
    let compare_up_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_view_up_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_up_context_menu_controller.close();
        compare_up_bridge.dispatch(UiCommand::CompareViewUpOneLevel);
        sync_window_state_if_changed(
            &window,
            &compare_up_bridge,
            &compare_up_cache,
            Some(&compare_up_context_menu_controller),
            &compare_up_loading_mask_controller,
            SyncMode::Passive,
        );
        window.invoke_focus_compare_rows();
    });

    let app_weak = app.as_weak();
    let compare_breadcrumb_bridge = bridge.clone();
    let compare_breadcrumb_cache = Arc::clone(&sync_cache);
    let compare_breadcrumb_context_menu_controller = context_menu_controller.clone();
    let compare_breadcrumb_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_view_breadcrumb_requested(move |relative_path| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_breadcrumb_context_menu_controller.close();
        compare_breadcrumb_bridge
            .dispatch(UiCommand::NavigateCompareView(relative_path.to_string()));
        sync_window_state_if_changed(
            &window,
            &compare_breadcrumb_bridge,
            &compare_breadcrumb_cache,
            Some(&compare_breadcrumb_context_menu_controller),
            &compare_breadcrumb_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let compare_scroll_lock_bridge = bridge.clone();
    let compare_scroll_lock_cache = Arc::clone(&sync_cache);
    let compare_scroll_lock_context_menu_controller = context_menu_controller.clone();
    let compare_scroll_lock_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_view_scroll_lock_toggled(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_scroll_lock_context_menu_controller.close();
        compare_scroll_lock_bridge.dispatch(UiCommand::ToggleCompareViewScrollLock);
        sync_window_state_if_changed(
            &window,
            &compare_scroll_lock_bridge,
            &compare_scroll_lock_cache,
            Some(&compare_scroll_lock_context_menu_controller),
            &compare_scroll_lock_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let compare_file_back_bridge = bridge.clone();
    let compare_file_back_cache = Arc::clone(&sync_cache);
    let compare_file_back_context_menu_controller = context_menu_controller.clone();
    let compare_file_back_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_file_back_requested(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_file_back_context_menu_controller.close();
        compare_file_back_bridge.dispatch(UiCommand::SelectWorkspaceSession(
            "compare-tree".to_string(),
        ));
        sync_window_state_if_changed(
            &window,
            &compare_file_back_bridge,
            &compare_file_back_cache,
            Some(&compare_file_back_context_menu_controller),
            &compare_file_back_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_workspace_mode() == "compare-view" {
            window.invoke_focus_compare_rows();
        }
    });

    let app_weak = app.as_weak();
    let compare_focus_bridge = bridge.clone();
    let compare_focus_cache = Arc::clone(&sync_cache);
    let compare_focus_context_menu_controller = context_menu_controller.clone();
    let compare_focus_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_view_row_focused(move |relative_path| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_focus_context_menu_controller.close();
        compare_focus_bridge.dispatch(UiCommand::FocusCompareRow(relative_path.to_string()));
        sync_window_state_if_changed(
            &window,
            &compare_focus_bridge,
            &compare_focus_cache,
            Some(&compare_focus_context_menu_controller),
            &compare_focus_loading_mask_controller,
            SyncMode::Passive,
        );
        window.invoke_focus_compare_rows();
    });

    let app_weak = app.as_weak();
    let compare_toggle_bridge = bridge.clone();
    let compare_toggle_cache = Arc::clone(&sync_cache);
    let compare_toggle_context_menu_controller = context_menu_controller.clone();
    let compare_toggle_loading_mask_controller = loading_mask_controller.clone();
    app.on_compare_view_row_toggle_requested(move |relative_path| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_toggle_context_menu_controller.close();
        compare_toggle_bridge.dispatch(UiCommand::ToggleCompareTreeNode(relative_path.to_string()));
        sync_window_state_if_changed(
            &window,
            &compare_toggle_bridge,
            &compare_toggle_cache,
            Some(&compare_toggle_context_menu_controller),
            &compare_toggle_loading_mask_controller,
            SyncMode::Passive,
        );
        window.invoke_focus_compare_rows();
    });

    let app_weak = app.as_weak();
    let compare_activate_bridge = bridge.clone();
    let compare_activate_cache = Arc::clone(&sync_cache);
    let compare_activate_context_menu_controller = context_menu_controller.clone();
    let compare_activate_loading_mask_controller = loading_mask_controller.clone();
    let compare_activate_toast_controller = toast_controller.clone();
    app.on_compare_view_row_activated(move |relative_path| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        compare_activate_context_menu_controller.close();
        let snapshot = compare_activate_bridge.snapshot();
        match snapshot.compare_view_row_action(relative_path.as_str()) {
            Some(CompareViewRowAction::ToggleDirectory) => {
                compare_activate_bridge
                    .dispatch(UiCommand::ToggleCompareTreeNode(relative_path.to_string()));
                sync_window_state_if_changed(
                    &window,
                    &compare_activate_bridge,
                    &compare_activate_cache,
                    Some(&compare_activate_context_menu_controller),
                    &compare_activate_loading_mask_controller,
                    SyncMode::Passive,
                );
                window.invoke_focus_compare_rows();
            }
            Some(CompareViewRowAction::OpenFileView) => {
                window.set_workspace_tab(0);
                compare_activate_bridge.dispatch(UiCommand::OpenFileViewFromCompare(
                    relative_path.to_string(),
                ));
                sync_window_state_if_changed(
                    &window,
                    &compare_activate_bridge,
                    &compare_activate_cache,
                    Some(&compare_activate_context_menu_controller),
                    &compare_activate_loading_mask_controller,
                    SyncMode::Passive,
                );
            }
            Some(CompareViewRowAction::TypeMismatch) => {
                compare_activate_toast_controller.dispatch(
                    ToastRequest::new(
                        "Type mismatch cannot be opened yet",
                        ToastTone::Warn,
                        ToastPlacement::Toast,
                    )
                    .with_duration(Duration::from_millis(1600))
                    .with_strategy(ToastStrategy::Replace),
                );
            }
            None => {}
        }
    });

    let app_weak = app.as_weak();
    let compare_row_context_menu_controller = context_menu_controller.clone();
    app.on_compare_view_row_context_menu_requested(move |_relative_path| {
        if app_weak.upgrade().is_none() {
            return;
        }
        compare_row_context_menu_controller.close();
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

    // Settings lifecycle callbacks (open/cancel/save).
    let app_weak = app.as_weak();
    let settings_bridge = bridge.clone();
    let settings_cache = Arc::clone(&sync_cache);
    let settings_context_menu_controller = context_menu_controller.clone();
    let settings_loading_mask_controller = loading_mask_controller.clone();
    app.on_settings_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        settings_context_menu_controller.close();
        settings_bridge.dispatch(UiCommand::ClearSettingsError);
        sync_window_state_if_changed(
            &window,
            &settings_bridge,
            &settings_cache,
            Some(&settings_context_menu_controller),
            &settings_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let settings_cancel_bridge = bridge.clone();
    let settings_cancel_cache = Arc::clone(&sync_cache);
    let settings_cancel_context_menu_controller = context_menu_controller.clone();
    let settings_cancel_loading_mask_controller = loading_mask_controller.clone();
    app.on_settings_cancel_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        settings_cancel_context_menu_controller.close();
        settings_cancel_bridge.dispatch(UiCommand::ClearSettingsError);
        sync_window_state_if_changed(
            &window,
            &settings_cancel_bridge,
            &settings_cancel_cache,
            Some(&settings_cancel_context_menu_controller),
            &settings_cancel_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let settings_save_bridge = bridge.clone();
    let settings_save_cache = Arc::clone(&sync_cache);
    let settings_toast_controller = toast_controller.clone();
    let settings_save_context_menu_controller = context_menu_controller.clone();
    let settings_save_loading_mask_controller = loading_mask_controller.clone();
    app.on_settings_save_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        settings_save_context_menu_controller.close();
        let provider_kind = if window.get_settings_provider_mode() == 1 {
            AiProviderKind::OpenAiCompatible
        } else {
            AiProviderKind::Mock
        };
        settings_save_bridge.dispatch(UiCommand::SaveAppSettings {
            provider_kind,
            endpoint: window.get_settings_provider_endpoint().to_string(),
            api_key: window.get_settings_provider_api_key().to_string(),
            model: window.get_settings_provider_model().to_string(),
            timeout_secs_text: window.get_settings_provider_timeout().to_string(),
            show_hidden_files: window.get_settings_show_hidden_files(),
            default_results_view: if window.get_settings_default_result_view() == 1 {
                NavigatorViewMode::Flat
            } else {
                NavigatorViewMode::Tree
            },
        });
        sync_window_state_if_changed(
            &window,
            &settings_save_bridge,
            &settings_save_cache,
            Some(&settings_save_context_menu_controller),
            &settings_save_loading_mask_controller,
            SyncMode::Passive,
        );
        if window.get_settings_error_text().is_empty() {
            window.set_settings_open(false);
            settings_toast_controller.dispatch(ToastRequest::new(
                "Settings saved",
                ToastTone::Success,
                ToastPlacement::Toast,
            ));
        }
    });

    // Analysis action callbacks.
    let app_weak = app.as_weak();
    let analysis_bridge = bridge.clone();
    let analysis_cache = Arc::clone(&sync_cache);
    let analyze_context_menu_controller = context_menu_controller.clone();
    let analysis_loading_mask_controller = loading_mask_controller.clone();
    app.on_analyze_clicked(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        analyze_context_menu_controller.close();
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
            Some(&analyze_context_menu_controller),
            &analysis_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    // Analysis provider mode callbacks.
    let app_weak = app.as_weak();
    let provider_bridge = bridge.clone();
    let provider_cache = Arc::clone(&sync_cache);
    let provider_mock_context_menu_controller = context_menu_controller.clone();
    let provider_mock_loading_mask_controller = loading_mask_controller.clone();
    app.on_analysis_provider_mock_selected(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_bridge.dispatch(UiCommand::SetAiProviderModeMock);
        sync_window_state_if_changed(
            &window,
            &provider_bridge,
            &provider_cache,
            Some(&provider_mock_context_menu_controller),
            &provider_mock_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let provider_bridge = bridge.clone();
    let provider_cache = Arc::clone(&sync_cache);
    let provider_openai_context_menu_controller = context_menu_controller.clone();
    let provider_openai_loading_mask_controller = loading_mask_controller.clone();
    app.on_analysis_provider_openai_selected(move || {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        provider_bridge.dispatch(UiCommand::SetAiProviderModeOpenAiCompatible);
        sync_window_state_if_changed(
            &window,
            &provider_bridge,
            &provider_cache,
            Some(&provider_openai_context_menu_controller),
            &provider_openai_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    // Analysis remote config field callbacks.
    let app_weak = app.as_weak();
    let endpoint_bridge = bridge.clone();
    let endpoint_cache = Arc::clone(&sync_cache);
    let endpoint_context_menu_controller = context_menu_controller.clone();
    let endpoint_loading_mask_controller = loading_mask_controller.clone();
    app.on_analysis_endpoint_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        endpoint_bridge.dispatch(UiCommand::UpdateAiEndpoint(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &endpoint_bridge,
            &endpoint_cache,
            Some(&endpoint_context_menu_controller),
            &endpoint_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let api_key_bridge = bridge.clone();
    let api_key_cache = Arc::clone(&sync_cache);
    let api_key_context_menu_controller = context_menu_controller.clone();
    let api_key_loading_mask_controller = loading_mask_controller.clone();
    app.on_analysis_api_key_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        api_key_bridge.dispatch(UiCommand::UpdateAiApiKey(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &api_key_bridge,
            &api_key_cache,
            Some(&api_key_context_menu_controller),
            &api_key_loading_mask_controller,
            SyncMode::Passive,
        );
    });

    let app_weak = app.as_weak();
    let model_bridge = bridge.clone();
    let model_cache = Arc::clone(&sync_cache);
    let model_context_menu_controller = context_menu_controller.clone();
    let model_loading_mask_controller = loading_mask_controller.clone();
    app.on_analysis_model_changed(move |value| {
        let Some(window) = app_weak.upgrade() else {
            return;
        };

        model_bridge.dispatch(UiCommand::UpdateAiModel(value.to_string()));
        sync_window_state_if_changed(
            &window,
            &model_bridge,
            &model_cache,
            Some(&model_context_menu_controller),
            &model_loading_mask_controller,
            SyncMode::Passive,
        );
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
    fn programmatic_inputs_mode_syncs_editable_inputs() {
        assert!(should_sync_editable_inputs(SyncMode::ProgrammaticInputs));
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
    fn loading_mask_state_resets_timeout_on_phase_change() {
        let mut state = LoadingMaskState::default();
        let (started, generation) = state.advance(false, true, false);
        let started = started.expect("first diff-loading projection should be emitted");
        assert_eq!(started.message, "Loading diff...");
        let generation = generation.expect("diff-loading phase should schedule a timeout");

        let timed_out = state
            .trigger_timeout(generation)
            .expect("timeout transition should emit degraded copy");
        assert_eq!(timed_out.message, "Taking longer than expected...");

        let (analysis_reset, _) = state.advance(false, false, true);
        let analysis_reset = analysis_reset.expect("phase switch should reset timeout copy");
        assert_eq!(analysis_reset.message, "Running AI analysis...");
    }

    #[test]
    fn compare_file_content_width_tracks_wide_cjk_lines() {
        let ascii = estimate_compare_file_text_width_px("aaaaaaaa");
        let cjk = estimate_compare_file_text_width_px("测试测试测试测试");
        assert!(cjk > ascii);
    }

    #[test]
    fn compare_file_content_width_tracks_each_side_independently() {
        let rows = vec![
            crate::view_models::CompareFileRowViewModel {
                left_text: "short".to_string(),
                right_text: "also short".to_string(),
                ..Default::default()
            },
            crate::view_models::CompareFileRowViewModel {
                left_text: "非常长的比较行，用来验证横向滚动范围".to_string(),
                right_text: String::new(),
                ..Default::default()
            },
        ];

        assert_eq!(
            compare_file_left_content_width_px(&rows),
            estimate_compare_file_text_width_px("非常长的比较行，用来验证横向滚动范围")
        );
        assert_eq!(
            compare_file_right_content_width_px(&rows),
            estimate_compare_file_text_width_px("also short")
        );
    }
}
