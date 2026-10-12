//! Short, non-interactive explanations with GPUI's tooltip lifecycle.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use crate::tokens::{TOOLTIP_HEIGHT, TOOLTIP_PADDING_X, TOOLTIP_RADIUS, pt};
use gpui_kit::rems;
use gpui_kit::{
    App, AppContext, Context, Render, SharedString, StatefulInteractiveElement, Window, prelude::*,
};
use std::time::Duration;

/// A composable tooltip surface, retaining the base tooltip role and styling API.
///
/// Spec §7.5 (C27): a 20 pt body with radius 5 and no border, 11 pt text in
/// the label colour, on the tooltip's own material (#E7E7E7 @0.898 light,
/// #3A3A3A @0.828 dark), opaque under Reduce Transparency.
pub fn tooltip(text: impl Into<SharedString>, cx: &App) -> gpui_kit::base::Tooltip {
    let t = cx.macos();
    let line = crate::TextStyle::Tooltip.token().line_height;
    gpui_kit::base::Tooltip::new("macos-tooltip")
        .max_w(rems(20.))
        .min_h(pt(TOOLTIP_HEIGHT))
        .px(pt(TOOLTIP_PADDING_X))
        .py(pt((TOOLTIP_HEIGHT - line) / 2.))
        .rounded(pt(TOOLTIP_RADIUS))
        .bg(t.tooltip_material())
        .shadow(t.control_shadow())
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Tooltip, cx.macos())
        .child(text.into())
}

/// Attach a macOS tooltip without changing the control's concrete type.
/// GPUI owns hover delay, placement, dismissal, and cleanup when the owner leaves.
/// Keep an accessible label on icon-only controls; a tooltip is supplementary.
pub fn with_tooltip<T: StatefulInteractiveElement>(control: T, text: impl Into<SharedString>) -> T {
    let text = text.into();
    control
        .tooltip_show_delay(Duration::from_millis(400))
        .tooltip(move |_, cx| cx.new(|_| TooltipText(text.clone())).into())
}

struct TooltipText(SharedString);
impl Render for TooltipText {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_kit::div()
            .debug_selector(|| "macos-tooltip-surface".into())
            .child(tooltip(self.0.clone(), cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::px;
    use gpui_kit::{Render, TestAppContext, point};
    struct Harness;
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            gpui_kit::div().size_full().child(with_tooltip(
                crate::button("favorite", "Favorite", crate::ButtonVariant::Secondary, cx),
                "Add to favorites",
            ))
        }
    }
    #[gpui_kit::test]
    fn tooltip_waits_for_hover_and_hides_on_pointer_leave(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_event(gpui_kit::MouseMoveEvent {
            position: point(px(10.), px(10.)),
            pressed_button: None,
            modifiers: Default::default(),
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.dispatcher.advance_clock(Duration::from_millis(399));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("macos-tooltip-surface").is_none());
        cx.dispatcher.advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("macos-tooltip-surface").is_some());
        cx.simulate_event(gpui_kit::MouseMoveEvent {
            position: point(px(400.), px(400.)),
            pressed_button: None,
            modifiers: Default::default(),
        });
        cx.dispatcher.advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("macos-tooltip-surface").is_none());
    }

    /// Spec §7.5 (C27): 20 pt tall, radius 5, no border, and the tooltip's
    /// own material, opaque under Reduce Transparency. On
    /// 1x and 2x displays at 100% and 200% zoom.
    #[gpui_kit::test]
    fn tooltip_surface_matches_c27(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        let mut shown = false;
        for (scale, zoom, theme) in [
            (1f32, 1f32, crate::Theme::light()),
            (2., 2., crate::Theme::dark()),
            (1., 2., crate::Theme::light().with_reduce_transparency(true)),
            (2., 1., crate::Theme::dark().with_reduce_transparency(true)),
        ] {
            cx.simulate_scale_factor_change(scale);
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.set_rem_size(px(16. * zoom));
                window.refresh();
                window.draw(cx).clear(cx);
            });
            if !shown {
                cx.simulate_event(gpui_kit::MouseMoveEvent {
                    position: point(px(10.), px(10.)),
                    pressed_button: None,
                    modifiers: Default::default(),
                });
                cx.dispatcher.advance_clock(Duration::from_millis(400));
                cx.run_until_parked();
                shown = true;
            }
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let bounds = cx.debug_bounds("macos-tooltip-surface").expect("shown");
            let expected = if theme.is_dark() {
                (gpui_kit::rgb(0x3a3a3a), 0.828)
            } else {
                (gpui_kit::rgb(0xe7e7e7), 0.898)
            };
            let fill = gpui_kit::Hsla::from(expected.0).opacity(if theme.reduce_transparency {
                1.
            } else {
                expected.1
            });
            let what = format!("{scale}x {zoom} dark {}", theme.is_dark());
            assert!(
                (bounds.size.height.as_f32() - 20. * zoom).abs() <= 0.5,
                "{what}: height {bounds:?}"
            );
            cx.update(|window, cx| {
                assert_eq!(cx.macos().tooltip_material(), fill, "{what}");
                let scaled = bounds.scale(window.scale_factor());
                let quad = window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.bounds == scaled && quad.background.as_solid().is_some())
                    .unwrap_or_else(|| panic!("{what}: tooltip quad"));
                assert_eq!(quad.background.as_solid(), Some(fill), "{what}");
                let border = quad.border_widths;
                assert_eq!(
                    [border.top, border.right, border.bottom, border.left].map(|width| width.0),
                    [0.; 4],
                    "{what}: no border"
                );
                let radius = quad.corner_radii.top_left.0 / scale;
                assert!((radius - 5. * zoom).abs() <= 0.5, "{what}: radius {radius}");
            });
        }
    }
}
