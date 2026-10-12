//! Composable macOS window chrome, sidebars, inspectors and grouped settings.
//!
//! Sidebars host AppKit glass on macOS 26 and newer. Other compositions can
//! opt in using [`native_material`]. [`material`] supplies a deterministic fill
//! for canvas overlays and other platforms.
use crate::NativeTextStyle as _;
use crate::state::{PaintRole, Part, RowKind, States, paint};
use crate::{ActiveTheme, Button, ButtonVariant, IconName, button, icon};
use gpui_kit::{
    App, Div, ElementId, FontWeight, SharedString, Stateful, WindowControlArea, div, prelude::*,
    px, rems,
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
    // The radius is in rems so it follows zoom; the glass resolves it at render.
    crate::native::native_glass(id, cx)
        .rounded(if kind == Material::Sidebar {
            rems(0.)
        } else {
            rems(0.75)
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
        .rounded(rems(1.125))
        .flex()
        .items_center()
        .gap(rems(0.125))
        .p(rems(0.125))
        .border_1()
        .shadow(cx.macos().control_shadow())
}

/// A flat, icon-only command inside a shared toolbar glass group.
/// The label names the action for assistive technology; add a tooltip if useful.
/// It is a toolbar item: a hover disc (ink about 0.05 / 0.04), no bezel, and
/// the outset focus ring for keyboard focus (spec §6.3, §6.5, C17).
pub fn toolbar_button(
    id: impl Into<ElementId>,
    name: IconName,
    label: impl Into<SharedString>,
    cx: &App,
) -> Button {
    let t = cx.macos();
    let role = PaintRole::ToolbarItem;
    let rest = States::active();
    button(id, "", ButtonVariant::Secondary, cx)
        .accessibility_label(label)
        .fill_duration(Duration::ZERO)
        .size(rems(2.))
        .p(rems(0.))
        .rounded_full()
        .border_0()
        .shadow_none()
        .bg(paint(role, Part::Fill, rest, t))
        .text_color(paint(role, Part::Label, rest, t))
        .active(|s| s.bg(paint(role, Part::Fill, rest.pressed(true), t)))
        .paint_role(role)
        .record_role_default()
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
        // Ideal width, C3 (policy).
        .w(crate::tokens::pt(crate::tokens::SIDEBAR_IDEAL_WIDTH))
        .h_full()
        .flex_shrink_0()
        .p(rems(0.5))
        .child(
            crate::native::native_glass((id, "sidebar-material"), cx)
                .sidebar_material()
                .absolute()
                .inset_0()
                .corner_radius(px(0.)),
        )
}

/// A keyboard-accessible sidebar command with native row selection (spec
/// §6.4, C20): an opaque accent row with a white label while emphasized in a
/// key window, a neutral overlay over the material while unemphasized, and
/// an opaque neutral row in an inactive window. Pass the sidebar's focus
/// state to [`Button::selection_emphasis`]; the default is emphasized. The
/// application owns selection and supplies its click handler. Rows have no
/// hover (C17).
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
    // The emphasized default; the final selection and emphasis resolve at
    // render (`Button::selection_emphasis`).
    let role = PaintRole::Row {
        kind: RowKind::Sidebar,
        selected,
        emphasized: true,
    };
    let rest = States::active();
    let symbol_size = rems(crate::native::sidebar_metrics().icon_column_width.as_f32() / 16.);
    let (accent, on_accent) = (t.accent, t.on_accent);
    button(id, "", ButtonVariant::Secondary, cx)
        .accessibility_label(label.clone())
        .selected(selected)
        .aria_selected(selected)
        // Native row selection is immediate. A transparent resting row should
        // not interpolate through a tinted opaque color when highlighted.
        .fill_duration(Duration::ZERO)
        .bg(paint(role, Part::Fill, rest, t))
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
        .text_color(paint(role, Part::Label, rest, t))
        .font_weight(if selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        // Pressing a row does not change its paint (spec §6.4).
        .active(|s| s)
        .paint_role(role)
        .record_role_default()
        // The symbol is accent, and white on the emphasized selection.
        .child_with_highlight(move |highlighted| {
            icon(name)
                .size(symbol_size)
                .font_weight(FontWeight::NORMAL)
                .text_color(if highlighted { on_accent } else { accent })
        })
        .child_with_highlight(move |_| div().min_w_0().truncate().child(label))
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, InteractiveElement, IntoElement, Render, TestAppContext, Window};

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    native_material("material", Material::Popover, cx)
                        .size(rems(4.))
                        .flex_shrink_0()
                        .debug_selector(|| "material".into()),
                )
                .child(
                    native_toolbar_group("group", cx)
                        .w(rems(6.))
                        .h(rems(3.))
                        .flex_shrink_0()
                        .debug_selector(|| "group".into()),
                )
                .child(sidebar("sidebar", cx).debug_selector(|| "sidebar".into()))
        }
    }

    /// Glass corners and the sidebar's ideal width (C3) follow zoom.
    #[gpui_kit::test]
    fn glass_radii_and_sidebar_width_follow_zoom(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            // The opaque fallback paints the GPUI corners the glass also uses.
            let mut theme = crate::Theme::light();
            theme.reduce_transparency = true;
            theme.apply(cx);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        for zoom in [1., 2.] {
            cx.update(|window, cx| {
                window.set_rem_size(px(16. * zoom));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            for (selector, radius) in [("material", 12.), ("group", 18.)] {
                let bounds = cx.debug_bounds(selector).unwrap();
                let painted = cx.update(|window, _| {
                    let scaled = bounds.scale(window.scale_factor());
                    window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| quad.bounds == scaled)
                        .expect(selector)
                        .corner_radii
                        .top_left
                        .0
                        / window.scale_factor()
                });
                assert_eq!(painted, radius * zoom, "{selector} at {zoom}");
            }
            assert_eq!(
                cx.debug_bounds("sidebar").unwrap().size.width,
                px(220. * zoom)
            );
        }
    }

    struct SidebarRows {
        emphasized: bool,
    }
    impl Render for SidebarRows {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            sidebar("rows", cx)
                .child(
                    sidebar_item("selected-row", "Favorites", IconName::Star, true, cx)
                        .selection_emphasis(self.emphasized)
                        .underline()
                        .debug_selector(|| "selected-row".into()),
                )
                .child(
                    sidebar_item("other-row", "Documents", IconName::File, false, cx)
                        .selection_emphasis(self.emphasized)
                        .debug_selector(|| "other-row".into()),
                )
        }
    }

    /// Sidebar selection, spec §6.4 (C20), in a focused key window, an
    /// unfocused key window and an inactive window, light and dark.
    #[gpui_kit::test]
    fn sidebar_selection_follows_emphasis_and_window_state(cx: &mut TestAppContext) {
        use crate::tokens::ink;
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| SidebarRows { emphasized: true });
        let solid = |cx: &mut gpui_kit::VisualTestContext, selector: &'static str| {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.update(|window, _| {
                let bounds = bounds.scale(window.scale_factor());
                window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.bounds == bounds)
                    .filter_map(|quad| quad.background.as_solid())
                    .find(|fill| fill.a > 0.)
            })
        };
        let label = |cx: &mut gpui_kit::VisualTestContext| {
            let bounds = cx.debug_bounds("selected-row").unwrap();
            cx.update(|window, _| {
                let bounds = bounds.scale(window.scale_factor());
                window
                    .painted_underlines()
                    .into_iter()
                    .find(|line| bounds.contains(&line.bounds.center()))
                    .unwrap()
                    .color
            })
        };
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            let dark = theme.is_dark();
            cx.update(|_, cx| theme.clone().apply(cx));
            for (state, active, emphasized) in [
                ("focused key", true, true),
                ("unfocused key", true, false),
                ("inactive", false, true),
            ] {
                if active {
                    cx.update(|window, _| window.activate_window());
                    cx.run_until_parked();
                } else {
                    cx.deactivate_window();
                }
                view.update(cx, |view, cx| {
                    view.emphasized = emphasized;
                    cx.notify();
                });
                cx.update(|window, cx| window.draw(cx).clear(cx));
                let (fill, text) = match state {
                    "focused key" => (theme.accent, theme.on_accent),
                    "unfocused key" => (
                        ink(&theme).opacity(if dark { 0.088 } else { 0.085 }),
                        theme.foreground,
                    ),
                    _ => (
                        gpui_kit::rgb(if dark { 0x363636 } else { 0xe5e5e5 }).into(),
                        theme.foreground,
                    ),
                };
                assert_eq!(
                    solid(cx, "selected-row"),
                    Some(fill),
                    "{state} fill, dark {dark}"
                );
                assert_eq!(label(cx), text, "{state} label, dark {dark}");
                assert_eq!(
                    solid(cx, "other-row"),
                    None,
                    "{state}: unselected rows stay clear"
                );
                // No hover on rows (C17).
                let center = cx.debug_bounds("other-row").unwrap().center();
                cx.simulate_mouse_move(center, None, Default::default());
                cx.update(|window, cx| window.draw(cx).clear(cx));
                assert_eq!(solid(cx, "other-row"), None, "{state}: no row hover");
                cx.simulate_mouse_move(
                    gpui_kit::point(px(900.), px(900.)),
                    None,
                    Default::default(),
                );
            }
        }
    }

    struct OrderedRow;
    impl Render for OrderedRow {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            sidebar("ordered", cx)
                .debug_selector(|| "ordered-sidebar".into())
                .child(
                    sidebar_item("ordered-row", "Inbox", IconName::Star, true, cx)
                        .debug_selector(|| "ordered-row".into())
                        .child(div().debug_selector(|| "ordered-badge".into()).child("3")),
                )
        }
    }

    /// A caller's extra child follows the symbol and label (one ordered
    /// child sequence), and the sidebar material follows window activity
    /// (spec §7.1, §7.2; C20).
    #[gpui_kit::test]
    fn sidebar_children_keep_order_and_material_follows_activity(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| OrderedRow);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            let dark = theme.is_dark();
            cx.update(|_, cx| theme.clone().apply(cx));
            for active in [true, false] {
                if active {
                    cx.update(|window, _| window.activate_window());
                    cx.run_until_parked();
                } else {
                    cx.deactivate_window();
                }
                view.update(cx, |_, cx| cx.notify());
                cx.update(|window, cx| window.draw(cx).clear(cx));
                let row = cx.debug_bounds("ordered-row").unwrap();
                let badge = cx.debug_bounds("ordered-badge").unwrap();
                assert!(
                    badge.left() > row.left() + px(60.),
                    "the badge follows the symbol and label: {badge:?} in {row:?}"
                );
                let shell = cx.debug_bounds("ordered-sidebar").unwrap();
                let expected: gpui_kit::Hsla = match active {
                    true => {
                        gpui_kit::Hsla::from(gpui_kit::rgb(if dark { 0x2c2c2c } else { 0xe7e7e7 }))
                            .opacity(if dark { 0.816 } else { 0.898 })
                    }
                    false => gpui_kit::rgb(if dark { 0x282828 } else { 0xf0f0f0 }).into(),
                };
                cx.update(|window, _| {
                    let shell = shell.scale(window.scale_factor());
                    assert!(
                        window.painted_quads().iter().any(|quad| {
                            quad.bounds == shell && quad.background.as_solid() == Some(expected)
                        }),
                        "sidebar material, active {active}, dark {dark}"
                    );
                });
            }
        }
    }
}
