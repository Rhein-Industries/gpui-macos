//! Composable macOS window chrome, sidebars, inspectors and grouped settings.
//!
//! Sidebars host AppKit glass on macOS 26 and newer. Other compositions can
//! opt in using [`native_material`]. [`material`] supplies a deterministic fill
//! for canvas overlays and other platforms.
use crate::NativeTextStyle as _;
use crate::{ActiveTheme, Button, ButtonVariant, IconName, button, icon};
use gpui_kit::{
    App, Div, ElementId, FontWeight, IntoElement, RenderOnce, SharedString, Stateful, Window,
    WindowControlArea, div, prelude::*, px, rems,
};
use std::time::Duration;

/// Semantic material roles with light and dark theme support.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Material {
    Sidebar,
    #[default]
    Toolbar,
    Popover,
    Hud,
}

/// A composable material surface. Add geometry, content and elevation as needed.
/// The translucent fill does not blur or refract the content behind it.
pub fn material(kind: Material, cx: &App) -> Div {
    let t = cx.macos();
    div()
        .bg(match kind {
            Material::Sidebar | Material::Hud => {
                t.inset
                    .opacity(if t.reduce_transparency { 1. } else { 0.94 })
            }
            Material::Toolbar | Material::Popover => t.glass_fill(),
        })
        .border_color(t.glass_border())
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
}

/// Real AppKit glass for navigation and command surfaces, with a themed fallback.
/// Keep ancestor fills transparent; this underlay refracts the window backdrop.
/// GPUI content above the native view remains on its own rendering plane.
pub fn native_material(
    id: impl Into<ElementId>,
    kind: Material,
    cx: &App,
) -> crate::native::NativeGlass {
    crate::native::native_glass(id, cx)
        .corner_radius(if kind == Material::Sidebar {
            px(0.)
        } else {
            px(12.)
        })
        .clear(kind == Material::Hud)
}

/// A clear toolbar layout. Compose glass command groups, an unbacked title and search.
pub fn toolbar(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let t = cx.macos();
    div()
        .id(id)
        .flex()
        .items_center()
        .w_full()
        .min_h(rems(3.25))
        .gap(rems(0.625))
        .px(rems(0.875))
        .py(rems(0.25))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
        .flex_shrink_0()
}

/// Shared glass for related toolbar commands. Compose flat `toolbar_button` children.
pub fn toolbar_group(cx: &App) -> Div {
    material(Material::Toolbar, cx)
        .flex()
        .items_center()
        .gap(rems(0.125))
        .p(rems(0.125))
        .rounded_full()
        .border_1()
        .shadow(cx.macos().control_shadow())
}

/// A command group backed by actual system glass on supported macOS windows.
/// Supply a stable, sibling-unique ID and compose `toolbar_button` children.
pub fn native_toolbar_group(id: impl Into<ElementId>, cx: &App) -> crate::native::NativeGlass {
    native_material(id, Material::Toolbar, cx)
        .corner_radius(rems(1.125).to_pixels(px(16.)))
        .flex()
        .items_center()
        .gap(rems(0.125))
        .p(rems(0.125))
        .border_1()
        .shadow(cx.macos().control_shadow())
}

/// A flat, icon-only command inside a shared toolbar glass group.
/// The label names the action for assistive technology; add a tooltip if useful.
pub fn toolbar_button(
    id: impl Into<ElementId>,
    name: IconName,
    label: impl Into<SharedString>,
    cx: &App,
) -> Button {
    let t = cx.macos();
    button(id, "", ButtonVariant::Secondary, cx)
        .accessibility_label(label)
        .fill_duration(Duration::ZERO)
        .size(rems(2.))
        .p(rems(0.))
        .rounded_full()
        .border_0()
        .shadow_none()
        .bg(t.foreground.opacity(0.))
        .hover(|s| s.bg(t.hover_fill()))
        .active(|s| s.bg(t.pressed_fill()))
        .focus_visible(|s| s.bg(t.hover_fill()).shadow(t.focus_shadow()))
        .child(icon(name).size(rems(1.)))
}

/// A navigation sidebar. Scrollable content can be supplied inside this shell.
pub fn sidebar(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let id = id.into();
    div()
        .id(id.clone())
        .relative()
        .font_family(cx.macos().font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(cx.macos().foreground)
        .flex()
        .flex_col()
        .w(rems(13.75))
        .h_full()
        .flex_shrink_0()
        .p(rems(0.5))
        .child(
            crate::native::native_glass((id, "sidebar-material"), cx)
                .absolute()
                .inset_0()
                .corner_radius(px(0.)),
        )
}

#[derive(IntoElement)]
struct SidebarSymbol {
    name: IconName,
    selected: bool,
}

impl RenderOnce for SidebarSymbol {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.macos();
        let accent =
            if !window.is_window_active() && !t.reduce_transparency && !t.increased_contrast {
                // Source-list vibrancy deemphasizes symbols in an inactive window.
                // Blend the app accent so a custom or system accent keeps its hue.
                t.accent.blend(if t.is_dark() {
                    gpui_kit::Hsla::black().opacity(0.4)
                } else {
                    gpui_kit::Hsla::white().opacity(0.4)
                })
            } else {
                t.accent
            };
        icon(self.name)
            .size(rems(
                crate::native::sidebar_metrics().icon_column_width.as_f32() / 16.,
            ))
            .font_weight(FontWeight::NORMAL)
            .text_color(if self.selected && window.is_window_active() {
                t.on_accent
            } else {
                accent
            })
    }
}

/// A keyboard-accessible sidebar command with a blue selected row.
/// The application owns selection and supplies its click handler.
pub fn sidebar_item(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    name: IconName,
    selected: bool,
    cx: &App,
) -> Button {
    let t = cx.macos();
    let metrics = crate::native::sidebar_metrics();
    let text_style = match metrics.label_size.as_f32().round() as u32 {
        11 => crate::TextStyle::Subheadline,
        15 => crate::TextStyle::Title3,
        _ => crate::TextStyle::Body,
    };
    let label = label.into();
    button(id, "", ButtonVariant::Secondary, cx)
        .accessibility_label(label.clone())
        .selected(selected)
        .aria_selected(selected)
        .selected_fill(t.accent)
        .inactive_style(
            if selected {
                crate::native::sidebar_inactive_selection(cx)
            } else {
                t.foreground.opacity(0.)
            },
            if selected {
                if t.is_dark() && !t.increased_contrast {
                    t.accent.blend(gpui_kit::Hsla::white().opacity(0.2))
                } else {
                    t.accent
                }
            } else {
                t.foreground
            },
        )
        // Native row selection is immediate. A transparent resting row should
        // not interpolate through a tinted opaque color when highlighted.
        .fill_duration(Duration::ZERO)
        .bg(if selected {
            t.accent
        } else {
            t.foreground.opacity(0.)
        })
        .w_full()
        .h(rems(metrics.row_height.as_f32() / 16.))
        .native_text_style(text_style, t)
        .px(rems(0.5))
        .py(rems(0.))
        .rounded(rems(0.5))
        .border_0()
        .border_color(t.foreground.opacity(0.))
        .shadow_none()
        .justify_start()
        .gap(rems(0.25))
        .text_color(if selected { t.on_accent } else { t.foreground })
        .font_weight(if selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .hover(|s| {
            s.bg(if selected { t.accent } else { t.hover_fill() })
                .border_color(t.foreground.opacity(0.))
        })
        .active(|s| s.bg(if selected { t.accent } else { t.pressed_fill() }))
        .focus_visible(|s| {
            s.bg(if selected { t.accent } else { t.hover_fill() })
                .border_color(t.accent)
                .shadow(t.focus_shadow())
        })
        .child(SidebarSymbol { name, selected })
        .child(div().min_w_0().truncate().child(label))
}

/// A sidebar section with a subdued heading and caller-supplied row children.
pub fn sidebar_section(title: impl Into<SharedString>, cx: &App) -> Div {
    div().flex().flex_col().pt(rems(0.8125)).child(
        div()
            .flex()
            .items_center()
            .h(rems(1.1875))
            .px(rems(0.375))
            .native_text_style(crate::TextStyle::Subheadline, cx.macos())
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.macos().secondary)
            .child(title.into()),
    )
}

/// A draggable title row that leaves room for the system window controls.
/// Configure `WindowOptions::titlebar` with native traffic lights; this builder
/// reserves their leading space and never draws replacement window controls.
pub fn title_bar(title: impl Into<SharedString>, cx: &App) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .items_center()
        .w_full()
        .h(rems(3.25))
        .pl(rems(5.))
        .pr(rems(0.875))
        .gap(rems(0.625))
        .flex_shrink_0()
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::ToolbarTitle, cx.macos())
        .text_color(t.foreground)
        .font_weight(FontWeight::SEMIBOLD)
        .window_control_area(WindowControlArea::Drag)
        .child(div().min_w_0().truncate().child(title.into()))
}

/// A quiet bottom row for selection counts, activity and auxiliary commands.
pub fn status_bar(cx: &App) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .min_h(rems(1.75))
        .px(rems(0.875))
        .gap(rems(0.625))
        .border_t_1()
        .border_color(t.divider())
        .flex_shrink_0()
        .bg(t.background)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Subheadline, cx.macos())
        .text_color(t.secondary)
}

/// A trailing inspector pane with a section title and caller-supplied controls.
pub fn inspector(title: impl Into<SharedString>, cx: &App) -> Div {
    material(Material::Sidebar, cx)
        .flex()
        .flex_col()
        .w(rems(17.5))
        .h_full()
        .flex_shrink_0()
        .gap(rems(0.75))
        .p(rems(0.875))
        .border_l_1()
        .border_color(cx.macos().divider())
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title.into()))
}

/// A rounded System Settings group. Compose `settings_row` children inside it.
pub fn settings_group(title: impl Into<SharedString>, cx: &App) -> Div {
    let t = cx.macos();
    let title = title.into();
    div()
        .flex()
        .flex_col()
        .w_full()
        .rounded(rems(0.75))
        .overflow_hidden()
        .bg(t.normal_fill())
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
        .when(!title.is_empty(), |group| {
            group.child(
                div()
                    .px(rems(0.875))
                    .py(rems(0.75))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
        })
}

/// A settings label and optional description; append the control as a child.
pub fn settings_row(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    cx: &App,
) -> Div {
    let description = description.into();
    div()
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .min_h(rems(2.875))
        .gap(rems(1.))
        .px(rems(0.875))
        .py(rems(0.625))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(rems(0.1875))
                .child(label.into())
                .when(!description.is_empty(), |labels| {
                    labels.child(
                        div()
                            .native_text_style(crate::TextStyle::Subheadline, cx.macos())
                            .text_color(cx.macos().secondary)
                            .child(description),
                    )
                }),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left(rems(0.875))
                .right(rems(0.875))
                .h(rems(0.0625))
                .bg(cx.macos().divider()),
        )
}
