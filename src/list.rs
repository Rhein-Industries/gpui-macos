//! Virtualized lists retain base's sizing and scroll model.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use gpui_kit::rems;
use gpui_kit::{
    App, Context, Div, ElementId, Entity, Hsla, IntoElement, Pixels, Render, Size, Stateful,
    Styled, Window, div, prelude::*,
};
use std::{ops::Range, rc::Rc};

/// The (fill, foreground) of a selected list, table or tree row (spec §6.4).
/// Selection is application-owned; paint the selected row with these.
///
/// `emphasized` is true while the list has keyboard focus (for example
/// `focus.contains_focused(window, cx)`). Emphasized rows in a key window use
/// `selected_content` (#0064E1 / #0059D1) with white text; otherwise the row
/// is `unemphasized_selected_content` (#DCDCDC / #464646) with the label
/// color. An inactive window is never emphasized. Unselected rows stay clear,
/// or [`crate::Theme::alternating_row`] for striped rows; rows have no hover.
pub fn row_selection(emphasized: bool, window: &Window, cx: &App) -> (Hsla, Hsla) {
    use crate::state::{PaintRole, Part, RowKind, States, paint};
    let role = PaintRole::Row {
        kind: RowKind::List,
        selected: true,
        emphasized,
    };
    let states = States::in_window(window);
    (
        paint(role, Part::Fill, states, cx.macos()),
        paint(role, Part::Label, states, cx.macos()),
    )
}

/// Each supplied size must match the corresponding row's rendered height.
/// Only the visible range is built. Attach a base VirtualListScrollHandle to
/// preserve scroll position and support navigation to a specific item.
pub fn virtual_list<V: Render, R: IntoElement>(
    view: Entity<V>,
    id: impl Into<ElementId>,
    sizes: Rc<Vec<Size<Pixels>>>,
    render: impl Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R> + 'static,
    cx: &App,
) -> gpui_kit::base::VirtualList {
    let t = cx.macos();
    gpui_kit::base::v_virtual_list(view, id, sizes, render)
        .w_full()
        .h(rems(17.5))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
        .bg(t.background)
}

/// Overlay scrollbar for a scrollable region. Render after that region inside a
/// relative parent, using the same handle as the scrollable content.
pub fn scrollbar<H: gpui_kit::base::ScrollbarHandle + Clone>(
    id: impl Into<ElementId>,
    axis: gpui_kit::base::ScrollbarAxis,
    handle: &H,
    window: &Window,
    cx: &App,
) -> gpui_kit::base::Scrollbar {
    let t = cx.macos();
    gpui_kit::base::Scrollbar::new(handle)
        .id(id)
        .axis(axis)
        .mode(gpui_kit::base::ScrollbarMode::Scrolling)
        .styles(|style| {
            style
                .track(|track| {
                    track
                        .width(rems(0.5).to_pixels(window.rem_size()))
                        .bg(t.foreground.opacity(0.))
                })
                .thumb(|thumb| {
                    thumb
                        .width(rems(0.375).to_pixels(window.rem_size()))
                        .inset(rems(0.0625).to_pixels(window.rem_size()))
                        .radius(rems(0.1875).to_pixels(window.rem_size()))
                        .bg(t.foreground.opacity(0.35))
                })
                .thumb_hover(|thumb| thumb.bg(t.foreground.opacity(0.55)))
                .thumb_active(|thumb| thumb.bg(t.foreground.opacity(0.7)))
        })
}

/// The `.inset` list layout for [`crate::interpretation::list_item`] rows
/// (spec §5.2, §6.4): rows follow each other at the 24 pt row pitch with no
/// gap, 10 pt below the top and above the bottom edge, and inset 11.5 pt
/// from each side so the 5 pt rounded selection floats inside the list.
/// Give it the list's width; rows fill it.
pub fn list_rows(cx: &App) -> Div {
    use crate::tokens::{LIST_INSET_Y, LIST_SELECTION_INSET, pt};
    let t = cx.macos();
    div()
        .flex()
        .flex_col()
        .py(pt(LIST_INSET_Y))
        .px(pt(LIST_SELECTION_INSET))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, t)
        .text_color(t.foreground)
}

/// Grouped content, as in System Settings. Compose rows inside this surface.
pub fn inset_list(cx: &App) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .flex_col()
        .w_full()
        .overflow_hidden()
        .rounded(rems(0.75))
        .bg(t.normal_fill())
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
}

/// A composable inset row. Add a bottom separator when another row follows.
/// A stable id allows application-owned click, focus and selection behavior.
/// Rows are 40 pt: a grouped card-list specialization (plan D12), not the
/// 24 pt list row.
pub fn inset_list_row(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_between()
        .min_h(crate::tokens::pt(crate::tokens::INSET_LIST_ROW_HEIGHT))
        .px(rems(0.875))
        .py(rems(0.5))
        .gap(rems(0.75))
        .border_color(cx.macos().divider())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{TestAppContext, rgb};

    struct Empty;
    impl Render for Empty {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    /// `row_selection`, spec §6.4: emphasized only in an active window.
    #[gpui_kit::test]
    fn row_selection_matches_6_4(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| Empty);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            let dark = theme.is_dark();
            cx.update(|_, cx| theme.clone().apply(cx));
            let emphasized: Hsla = rgb(if dark { 0x0059d1 } else { 0x0064e1 }).into();
            let unemphasized: Hsla = rgb(if dark { 0x464646 } else { 0xdcdcdc }).into();
            cx.update(|window, _| window.activate_window());
            cx.run_until_parked();
            cx.update(|window, cx| {
                assert_eq!(
                    row_selection(true, window, cx),
                    (emphasized, theme.on_accent)
                );
                assert_eq!(
                    row_selection(false, window, cx),
                    (unemphasized, theme.foreground)
                );
            });
            cx.deactivate_window();
            cx.update(|window, cx| {
                for emphasis in [true, false] {
                    assert_eq!(
                        row_selection(emphasis, window, cx),
                        (unemphasized, theme.foreground),
                        "an inactive window is never emphasized"
                    );
                }
            });
        }
    }

    struct InsetList;
    impl Render for InsetList {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let (fill, text) = row_selection(false, window, cx);
            let mut rows = list_rows(cx)
                .w(rems(30.))
                .debug_selector(|| "inset-list".into());
            for index in 0..3_usize {
                rows = rows.child(
                    crate::interpretation::list_item(("pitch-row", index), cx)
                        .debug_selector(move || format!("pitch-row-{index}"))
                        .when(index == 0, |row| row.bg(fill).text_color(text))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(rems(0.625))
                                .child(crate::icon(crate::IconName::File))
                                .child("Project brief"),
                        ),
                );
            }
            div().p(rems(1.)).child(rows)
        }
    }

    /// The `.inset` list (spec §5.2, §6.4) on 1x and 2x displays at 100% and
    /// 200% zoom: consecutive rows are 24 pt apart (the row pitch, not just
    /// the row height), the first row starts 10 pt into the list, rows are
    /// inset 11.5 pt from each side and the selection has a 5 pt radius.
    #[gpui_kit::test]
    fn inset_list_rows_keep_the_native_pitch_and_insets(cx: &mut TestAppContext) {
        use gpui_kit::{Pixels, px};
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| InsetList);
        for scale in [1., 2.] {
            cx.simulate_scale_factor_change(scale);
            for zoom in [1., 2.] {
                let at = format!("{scale}x {zoom}");
                cx.update(|window, cx| {
                    window.set_rem_size(px(16. * zoom));
                    view.update(cx, |_, cx| cx.notify());
                    window.draw(cx).clear(cx);
                });
                // Layout rounds to the device grid; allow one device pixel.
                let near = |actual: Pixels, points: f32, what: &str| {
                    assert!(
                        (actual.as_f32() - points * zoom).abs() <= 1. / scale,
                        "{at} {what}: {actual:?} vs {}",
                        points * zoom
                    );
                };
                let list = cx.debug_bounds("inset-list").unwrap();
                let rows: Vec<_> = ["pitch-row-0", "pitch-row-1", "pitch-row-2"]
                    .into_iter()
                    .map(|selector| cx.debug_bounds(selector).unwrap())
                    .collect();
                near(rows[0].top() - list.top(), 10., "top inset");
                for row in &rows {
                    near(row.size.height, 24., "row height");
                    near(row.left() - list.left(), 11.5, "leading inset");
                    near(list.right() - row.right(), 11.5, "trailing inset");
                }
                for pair in rows.windows(2) {
                    near(pair[1].top() - pair[0].top(), 24., "row pitch");
                    assert_eq!(pair[0].bottom(), pair[1].top(), "{at}: rows have no gap");
                }
                near(list.bottom() - rows[2].bottom(), 10., "bottom inset");
                let radius = cx.update(|window, _| {
                    let bounds = rows[0].scale(window.scale_factor());
                    window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| {
                            quad.bounds == bounds
                                && quad.background.as_solid().is_some_and(|fill| fill.a > 0.)
                        })
                        .map(|quad| quad.corner_radii.top_left.0 / window.scale_factor())
                });
                near(
                    px(radius.expect("painted selection")),
                    5.,
                    "selection radius",
                );
            }
        }
    }

    struct Rows;
    impl Render for Rows {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let (fill, text) = row_selection(false, window, cx);
            div()
                .w(rems(20.))
                .flex()
                .flex_col()
                .child(
                    crate::interpretation::list_item("selected-item", cx)
                        .bg(fill)
                        .text_color(text)
                        .debug_selector(|| "selected-item".into())
                        .child("Project brief"),
                )
                .child(
                    crate::table_row("striped-row", 3, cx)
                        .h(rems(1.5))
                        .debug_selector(|| "striped-row".into()),
                )
        }
    }

    /// Rows have no hover (spec §6.5, C17): a selection or stripe fill stays
    /// under the pointer, including Kit list items (transparent hover token).
    #[gpui_kit::test]
    fn hovering_rows_keeps_selection_and_stripes(cx: &mut TestAppContext) {
        use gpui_kit::{Modifiers, point, px};
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| Rows);
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let fill = |cx: &mut gpui_kit::VisualTestContext, selector: &'static str| {
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
        for (selector, expected) in [
            (
                "selected-item",
                cx.update(|_, cx| cx.macos().unemphasized_selected_content()),
            ),
            (
                "striped-row",
                cx.update(|_, cx| cx.macos().alternating_row()),
            ),
        ] {
            cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::default());
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(fill(cx, selector), Some(expected), "{selector} at rest");
            let center = cx.debug_bounds(selector).unwrap().center();
            cx.simulate_mouse_move(center, None, Modifiers::default());
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(fill(cx, selector), Some(expected), "{selector} hovered");
        }
    }
}
