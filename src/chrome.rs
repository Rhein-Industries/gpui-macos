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
        .py(rems(0.5))
        .px(crate::tokens::pt(crate::tokens::SIDEBAR_ROW_INSET))
        .child(
            crate::native::native_glass((id, "sidebar-material"), cx)
                .sidebar_material()
                .absolute()
                .inset_0()
                .corner_radius(px(0.)),
        )
}

/// The opaque colour a [`sidebar`] shows behind its rows, for covers such as
/// [`crate::native::ScrollEdgeEffect::surface_color`]. With real glass it is
/// the measured result of the sidebar glass (#ECECEC / #262626, by
/// capture); otherwise the sidebar's fallback material
/// ([`crate::Theme::sidebar_material`]) for the window's activity,
/// composited over the window background ([`crate::Theme::background`])
/// where that material is translucent.
pub fn sidebar_surface(window: &gpui_kit::Window, cx: &App) -> gpui_kit::Hsla {
    let t = cx.macos();
    if !t.reduce_transparency
        && crate::native::glass_available(window)
        && !crate::native::is_glass_popup(window)
    {
        gpui_kit::rgb(crate::tokens::SIDEBAR_GLASS_SURFACE[usize::from(t.is_dark())]).into()
    } else {
        t.background
            .blend(t.sidebar_material(States::in_window(window).window_active))
    }
}

/// The titlebar band over a detail column next to a full-height sidebar,
/// from the sidebar's trailing edge to the window's, the native toolbar's
/// height (`NSTitlebarBackgroundView` at (220, 0, 840, 52) in the SwiftUI
/// view dump). As in SwiftUI's `NavigationSplitView`, it is clear until the
/// pointer is over it, then shows a faint fill (none in light mode, white
/// @0.034 in dark) and a one-pixel hairline directly below the band (black
/// @0.10 light, white @0.071 dark), measured from the SwiftUI reference with
/// and without the pointer over the band. It has no glass or blur. Use
/// [`TitlebarBackground::revealed`] to show it regardless of the pointer.
/// Children (for example a GPUI title) paint above the band.
#[derive(IntoElement)]
pub struct TitlebarBackground {
    id: ElementId,
    base: Stateful<Div>,
    revealed: bool,
    children: Vec<gpui_kit::AnyElement>,
}

/// Start a titlebar band; see [`TitlebarBackground`].
pub fn titlebar_background(id: impl Into<ElementId>, cx: &App) -> TitlebarBackground {
    let id = id.into();
    TitlebarBackground {
        base: div()
            .id(id.clone())
            .relative()
            .w_full()
            .flex_shrink_0()
            .font_family(cx.macos().font.clone())
            .text_color(cx.macos().foreground),
        id,
        revealed: false,
        children: Vec::new(),
    }
}

impl TitlebarBackground {
    /// Show the band even without the pointer over it.
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
}

impl Styled for TitlebarBackground {
    fn style(&mut self) -> &mut gpui_kit::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for TitlebarBackground {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui_kit::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TitlebarBackground {
    fn render(self, window: &mut gpui_kit::Window, cx: &mut App) -> impl IntoElement {
        let hovered = window.use_keyed_state((self.id.clone(), "titlebar-hover"), cx, |_, _| false);
        let shown = self.revealed || *hovered.read(cx);
        let t = cx.macos();
        let dark = usize::from(t.is_dark());
        let fill = crate::tokens::ink(t).opacity(crate::tokens::TITLEBAR_BAND_FILL[dark]);
        let hairline = crate::tokens::ink(t).opacity(crate::tokens::TITLEBAR_BAND_SEPARATOR[dark]);
        // Device pixels, not points: the hairline does not follow zoom.
        let hairline_width =
            px(crate::tokens::TITLEBAR_BAND_SEPARATOR_WIDTH / window.scale_factor());
        // Hover follows pointer events only: a move inside the band shows it,
        // a move elsewhere or the pointer leaving the window hides it. GPUI's
        // own hover recomputes from the last known pointer position on every
        // redraw (stale after the pointer leaves through the titlebar) and
        // drops hover on keyboard input; the band must do neither.
        let tracker = gpui_kit::canvas(
            |bounds, window, _| bounds.intersect(&window.content_mask().bounds),
            move |_, band, window, _| {
                let set = move |inside: bool, window: &mut gpui_kit::Window, cx: &mut App| {
                    let changed = hovered.update(cx, |value, cx| {
                        let changed = *value != inside;
                        if changed {
                            *value = inside;
                            cx.notify();
                        }
                        changed
                    });
                    if changed {
                        window.refresh();
                    }
                };
                let on_move = set.clone();
                window.on_mouse_event(
                    move |event: &gpui_kit::MouseMoveEvent, phase, window, cx| {
                        if phase == gpui_kit::DispatchPhase::Capture {
                            on_move(band.contains(&event.position), window, cx);
                        }
                    },
                );
                window.on_mouse_event(move |_: &gpui_kit::MouseExitEvent, phase, window, cx| {
                    if phase == gpui_kit::DispatchPhase::Capture {
                        set(false, window, cx);
                    }
                });
            },
        )
        .absolute()
        .inset_0();
        // The fill goes first, so caller children paint above it. The
        // hairline lies just below the band, over the content that follows;
        // deferred painting keeps that later sibling from covering it.
        self.base
            .child(tracker)
            .when(shown, |band| {
                band.when(fill.a > 0., |band| {
                    band.child(div().absolute().inset_0().bg(fill))
                })
                .child(gpui_kit::deferred(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top_full()
                        .h(hairline_width)
                        .bg(hairline),
                ))
            })
            .children(self.children)
    }
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
    // The symbol sits in a 26 pt slot at the regular row size, drawn at its
    // natural size: the label size at SF Symbols' large scale, medium weight
    // (tokens::SIDEBAR_SYMBOL_SLOT, SIDEBAR_SYMBOL_WEIGHT).
    let symbol_slot =
        rems(metrics.icon_column_width.as_f32() / 24. * crate::tokens::SIDEBAR_SYMBOL_SLOT / 16.);
    let symbol_points = rems(metrics.label_size.as_f32() / 16.);
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
        // The label stays 46 pt from the sidebar's edge (10 + 6 + 26 + 4).
        .px(rems(0.375))
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
                .size(symbol_slot)
                .natural_symbol(symbol_points, true)
                .font_weight(crate::tokens::SIDEBAR_SYMBOL_WEIGHT)
                .text_color(if highlighted { on_accent } else { accent })
        })
        .child_with_highlight(move |_| div().min_w_0().truncate().child(label))
}

/// A sidebar section with a subdued heading and caller-supplied row children.
/// The heading is 19 pt with an 11 pt tertiary label, after 13 pt of leading
/// space (spec §5.2).
pub fn sidebar_section(title: impl Into<SharedString>, cx: &App) -> Div {
    use crate::tokens::{SIDEBAR_GROUP_HEADER_HEIGHT, SIDEBAR_GROUP_SPACING, pt};
    div().flex().flex_col().pt(pt(SIDEBAR_GROUP_SPACING)).child(
        div()
            .flex()
            .items_center()
            .h(pt(SIDEBAR_GROUP_HEADER_HEIGHT))
            .px(pt(4.))
            .native_text_style(crate::TextStyle::Subheadline, cx.macos())
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(cx.macos().tertiary_label())
            .child(title.into()),
    )
}

/// A collapsible sidebar group header, as in an AppKit source list (spec
/// §5.2): 19 pt tall with an 11 pt semibold label in the tertiary label
/// color. The disclosure chevron appears only while the pointer is over the
/// header or it has keyboard focus; the header stays a focusable button that
/// reports its expanded state. Supply the click handler that toggles the
/// group. Place [`sidebar_group_spacing`] above every group but the first.
pub fn sidebar_section_header(
    id: impl Into<ElementId>,
    title: impl Into<SharedString>,
    expanded: bool,
    cx: &App,
) -> Button {
    use crate::tokens::{SIDEBAR_GROUP_HEADER_HEIGHT, pt};
    let t = cx.macos();
    let id = id.into();
    let title = title.into();
    let chevron = SharedString::from(format!("{id}-chevron"));
    button(id, "", ButtonVariant::Secondary, cx)
        .accessibility_label(title.clone())
        .aria_expanded(expanded)
        .fill_duration(Duration::ZERO)
        .h(pt(SIDEBAR_GROUP_HEADER_HEIGHT))
        .min_h(pt(SIDEBAR_GROUP_HEADER_HEIGHT))
        // The title stays 14 pt from the sidebar's edge (10 + 4).
        .px(pt(4.))
        .py(rems(0.))
        .w_full()
        .justify_between()
        .border_0()
        .shadow_none()
        .bg(t.foreground.opacity(0.))
        .active(|style| style)
        .native_text_style(crate::TextStyle::Subheadline, t)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(t.tertiary_label())
        .child(div().min_w_0().truncate().child(title))
        // The chevron keeps its space while hidden, so the label never moves.
        .child_with_reveal(move |revealed| {
            div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .size(rems(0.625))
                .when(revealed, |slot| {
                    slot.debug_selector(move || chevron.to_string()).child(
                        icon(if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        })
                        .size(rems(0.625)),
                    )
                })
        })
}

/// The leading space above a sidebar group after the first (13 pt, spec
/// §5.2). It is a spacer rather than a margin, so virtualized lists, which
/// measure an item's border box, keep it.
pub fn sidebar_group_spacing() -> Div {
    div()
        .flex_shrink_0()
        .h(crate::tokens::pt(crate::tokens::SIDEBAR_GROUP_SPACING))
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

    struct SectionHarness {
        focus: gpui_kit::FocusHandle,
    }
    impl Render for SectionHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(rems(13.75))
                .p(rems(1.))
                .child(
                    sidebar_section_header("section", "Explore", true, cx)
                        .track_focus(&self.focus)
                        .debug_selector(|| "section-header".into()),
                )
                .child(
                    div().w(rems(5.)).child(
                        sidebar_section_header(
                            "narrow",
                            "A very long sidebar group title that cannot fit",
                            false,
                            cx,
                        )
                        .debug_selector(|| "narrow-header".into()),
                    ),
                )
        }
    }

    /// Spec §5.2: a 19 pt header; its disclosure
    /// chevron shows only under the pointer or with keyboard focus, and the
    /// header stays an expandable button for assistive technology.
    #[gpui_kit::test]
    fn sidebar_section_header_reveals_its_chevron_on_hover_or_focus(cx: &mut TestAppContext) {
        use gpui_kit::{Modifiers, point};
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| SectionHarness {
            focus: cx.focus_handle(),
        });
        cx.update(|window, _| window.activate_window());
        let redraw = |cx: &mut gpui_kit::VisualTestContext| {
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
        };
        cx.simulate_mouse_move(point(px(600.), px(600.)), None, Modifiers::default());
        redraw(cx);
        let header = cx.debug_bounds("section-header").unwrap();
        assert_eq!(header.size.height, px(19.));
        assert!(
            cx.debug_bounds("section-chevron").is_none(),
            "hidden at rest"
        );
        cx.update(|window, _| {
            let node = gpui_kit::base::test_support::find(window, &[], &"section".into()).unwrap();
            assert_eq!(node.expanded(), Some(true));
        });
        cx.simulate_mouse_move(header.center(), None, Modifiers::default());
        redraw(cx);
        assert!(
            cx.debug_bounds("section-chevron").is_some(),
            "shown on hover"
        );
        cx.simulate_mouse_move(point(px(600.), px(600.)), None, Modifiers::default());
        redraw(cx);
        assert!(cx.debug_bounds("section-chevron").is_none(), "hidden again");
        cx.update(|window, cx| {
            let focus = view.read(cx).focus.clone();
            focus.focus(window, cx);
        });
        cx.simulate_keystrokes("a");
        redraw(cx);
        assert!(
            cx.debug_bounds("section-chevron").is_some(),
            "shown with keyboard focus"
        );
        // A long label in a narrow sidebar truncates on one 19 pt line; the
        // chevron keeps its 10 pt slot inside the header in both states.
        let mut slots = Vec::new();
        for hovered in [false, true] {
            let narrow = cx.debug_bounds("narrow-header").unwrap();
            let target = if hovered {
                narrow.center()
            } else {
                point(px(600.), px(600.))
            };
            cx.simulate_mouse_move(target, None, Modifiers::default());
            redraw(cx);
            let narrow = cx.debug_bounds("narrow-header").unwrap();
            assert_eq!(narrow.size.height, px(19.), "hovered {hovered}");
            assert_eq!(narrow.size.width, px(80.), "hovered {hovered}");
            let chevron = cx.debug_bounds("narrow-chevron");
            assert_eq!(chevron.is_some(), hovered);
            if let Some(chevron) = chevron {
                assert_eq!(chevron.size.width, px(10.));
                assert!(chevron.right() <= narrow.right() && chevron.left() >= narrow.left());
            }
            slots.push(narrow);
        }
        assert_eq!(slots[0], slots[1], "revealing the chevron moves nothing");
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
                    // Sidebar titles are full-strength ink (pure black / white).
                    "unfocused key" => (
                        ink(&theme).opacity(if dark { 0.088 } else { 0.085 }),
                        ink(&theme),
                    ),
                    _ => (
                        gpui_kit::rgb(if dark { 0x363636 } else { 0xe5e5e5 }).into(),
                        ink(&theme),
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

    struct Band {
        revealed: bool,
        edge: bool,
    }
    impl Render for Band {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                // The probe comes first: the test context reports the pointer
                // at the origin while it redraws.
                .child(
                    div()
                        .debug_selector(|| "band-probe".into())
                        .w(rems(25.))
                        .h(rems(3.25)),
                )
                .child(
                    div().w(rems(25.)).child(
                        titlebar_background("band", cx)
                            .revealed(self.revealed)
                            .h(rems(3.25))
                            .child(div().debug_selector(|| "band-child".into()).size(rems(1.))),
                    ),
                )
                .child(
                    crate::native::scroll_edge_effect("edge")
                        .active(self.edge)
                        .surface_color(gpui_kit::rgb(0x123456).into())
                        .w(rems(10.))
                        .h(rems(3.25)),
                )
        }
    }

    /// The titlebar band (as the SwiftUI reference under the pointer): clear at
    /// rest, the sidebar surface and a bottom hairline while hovered or
    /// revealed, with caller children above it. The headless scroll-edge
    /// fallback paints only while active.
    #[gpui_kit::test]
    fn titlebar_band_shows_on_hover_and_scroll_edge_fallback_follows_activity(
        cx: &mut TestAppContext,
    ) {
        use gpui_kit::{Modifiers, point, size};
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| Band {
            revealed: false,
            edge: false,
        });
        cx.update(|window, _| window.activate_window());
        let redraw = |cx: &mut gpui_kit::VisualTestContext| {
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
        };
        let band_quads = |cx: &mut gpui_kit::VisualTestContext| {
            cx.update(|window, cx| {
                let scale = window.scale_factor();
                let t = cx.macos();
                let dark = usize::from(t.is_dark());
                let fill = crate::tokens::ink(t).opacity(crate::tokens::TITLEBAR_BAND_FILL[dark]);
                let hairline =
                    crate::tokens::ink(t).opacity(crate::tokens::TITLEBAR_BAND_SEPARATOR[dark]);
                let quads = window.painted_quads();
                let full = size(px(400.), px(52.)).scale(scale);
                (
                    // The fill covers the band (dark only).
                    quads.iter().any(|quad| {
                        quad.bounds.size == full && quad.background.as_solid() == Some(fill)
                    }),
                    // The hairline, one device pixel, lies directly below the
                    // band (from 52 pt of the band, which starts at 52 here).
                    quads.iter().any(|quad| {
                        quad.bounds.size == size(px(400.), px(1.) / scale).scale(scale)
                            && quad.bounds.origin.y == px(52. + 52.).scale(scale)
                            && quad.background.as_solid() == Some(hairline)
                    }),
                )
            })
        };
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|_, cx| theme.clone().apply(cx));
            // Light mode has no fill, only the hairline.
            let shown = (theme.is_dark(), true);
            let probe = cx.debug_bounds("band-probe").unwrap().center();
            cx.simulate_mouse_move(probe, None, Modifiers::default());
            redraw(cx);
            assert_eq!(band_quads(cx), (false, false), "clear at rest");
            cx.simulate_mouse_move(point(px(100.), px(72.)), None, Modifiers::default());
            redraw(cx);
            assert_eq!(band_quads(cx), shown, "shown under the pointer");
            assert!(cx.debug_bounds("band-child").is_some());
            cx.simulate_mouse_move(probe, None, Modifiers::default());
            redraw(cx);
            assert_eq!(band_quads(cx), (false, false), "hidden again");
            // Keyboard input does not drop the hover of a stationary pointer.
            cx.simulate_mouse_move(point(px(100.), px(72.)), None, Modifiers::default());
            redraw(cx);
            cx.simulate_keystrokes("a");
            redraw(cx);
            assert_eq!(band_quads(cx), shown, "kept through key input");
            // Leaving the window through the titlebar hides the band, and it
            // stays hidden through later redraws until the pointer returns.
            cx.simulate_event(gpui_kit::MouseExitEvent {
                position: point(px(100.), px(53.)),
                pressed_button: None,
                modifiers: Modifiers::default(),
            });
            for redraw_after_exit in 0..3 {
                redraw(cx);
                assert_eq!(
                    band_quads(cx),
                    (false, false),
                    "hidden after exit, redraw {redraw_after_exit}"
                );
            }
            cx.simulate_mouse_move(point(px(100.), px(72.)), None, Modifiers::default());
            redraw(cx);
            assert_eq!(band_quads(cx), shown, "shown on re-entry");
            cx.simulate_mouse_move(probe, None, Modifiers::default());
            redraw(cx);
            view.update(cx, |band, _| band.revealed = true);
            redraw(cx);
            assert_eq!(band_quads(cx), shown, "revealed");
            // At 200% zoom the hairline stays one device pixel thick.
            cx.update(|window, _| window.set_rem_size(px(32.)));
            redraw(cx);
            let thin = cx.update(|window, cx| {
                let t = cx.macos();
                let hairline = crate::tokens::ink(t)
                    .opacity(crate::tokens::TITLEBAR_BAND_SEPARATOR[usize::from(t.is_dark())]);
                window.painted_quads().iter().any(|quad| {
                    quad.bounds.size
                        == size(px(800.), px(1.) / window.scale_factor())
                            .scale(window.scale_factor())
                        && quad.background.as_solid() == Some(hairline)
                })
            });
            assert!(thin, "one device pixel at 200% zoom");
            cx.update(|window, _| window.set_rem_size(px(16.)));
            view.update(cx, |band, _| band.revealed = false);
        }
        // The cover: the surface colour at alpha 0.85 down to 31.5 pt of the
        // 62 pt pocket, falling to zero at 61.5 pt (CSS 180 degrees runs top
        // to bottom); it never hides content fully.
        let surface: gpui_kit::Hsla = gpui_kit::rgb(0x123456).into();
        let fade = crate::native::scroll_edge_cover(surface);
        assert_eq!(
            fade,
            gpui_kit::linear_gradient(
                180.,
                gpui_kit::linear_color_stop(surface.opacity(0.85), 31.5 / 62.),
                gpui_kit::linear_color_stop(surface.opacity(0.), 61.5 / 62.),
            )
        );
        let edge_quad = |cx: &mut gpui_kit::VisualTestContext| {
            cx.update(|window, _| {
                let scale = window.scale_factor();
                window
                    .painted_quads()
                    .iter()
                    .filter(|quad| quad.bounds.size == size(px(160.), px(52.)).scale(scale))
                    .map(|quad| quad.background)
                    .collect::<Vec<_>>()
            })
        };
        redraw(cx);
        assert!(edge_quad(cx).is_empty(), "nothing scrolled under the band");
        view.update(cx, |band, _| band.edge = true);
        redraw(cx);
        assert_eq!(edge_quad(cx), vec![fade], "the cover while scrolled");
        view.update(cx, |band, _| band.edge = false);
        redraw(cx);
        assert!(edge_quad(cx).is_empty(), "removed when no longer scrolled");
    }

    /// Without real glass (headless), the sidebar surface is its fallback
    /// material for the window's activity, made opaque over the window
    /// background.
    #[gpui_kit::test]
    fn sidebar_surface_without_glass_is_the_fallback_material(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| SidebarRows { emphasized: true });
        cx.update(|window, cx| {
            let t = cx.macos();
            let surface = sidebar_surface(window, cx);
            assert_eq!(surface.a, 1.);
            assert_eq!(
                surface,
                t.background
                    .blend(t.sidebar_material(window.is_window_active()))
            );
        });
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
