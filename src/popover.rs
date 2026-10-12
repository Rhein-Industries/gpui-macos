//! Contextual controls in an anchored, non-modal surface.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use crate::motion::{POPUP_FADE, animated_duration, out_cubic};
use crate::tokens::{POPOVER_PADDING, POPOVER_RADIUS, pt};
use gpui_kit::base::motion::Presence;
use gpui_kit::base::{Popover, PopoverState};
use gpui_kit::rems;
use gpui_kit::{App, Context, Div, ElementId, Window, div, prelude::*};

pub(crate) fn init(cx: &mut App) {
    // Suppress the outer Popover toggle bindings inside its content. Deeper
    // control contexts (Button, Input, Select) still retain their own bindings.
    cx.bind_keys([
        gpui_kit::KeyBinding::new("enter", gpui_kit::NoAction, Some("macOSPopoverContent")),
        gpui_kit::KeyBinding::new("space", gpui_kit::NoAction, Some("macOSPopoverContent")),
    ]);
}

/// An anchored control surface with base-owned open state, outside dismissal,
/// and focus return. The content builder runs with the current theme.
///
/// To customize the surface itself, use the returned Popover's `content` builder
/// and compose `popover_surface(cx)` with your own dimensions and children.
pub fn popover<E: IntoElement>(
    id: impl Into<ElementId>,
    trigger: impl gpui_kit::base::Selectable
    + IntoElement
    + gpui_kit::StatefulInteractiveElement
    + 'static,
    content: impl Fn(&mut PopoverState, &mut Window, &mut Context<PopoverState>) -> E + 'static,
) -> Popover {
    let id = id.into();
    Popover::new(id.clone())
        .trigger_clickable(trigger)
        .content_factory(move |state, window, cx| {
            if crate::native::is_glass_popup(window) {
                let first = window.use_keyed_state("popover-native-focus", cx, |_, _| true);
                if *first.read(cx) {
                    first.update(cx, |first, _| *first = false);
                    gpui_kit::Focusable::focus_handle(state, cx).focus(window, cx);
                }
            }
            let body = content(state, window, cx);
            let opacity = popup_fade(&id, window, cx);
            div()
                .key_context("macOSPopoverContent")
                .opacity(opacity)
                .child(
                    presented_popover_surface(window, cx)
                        .debug_selector(|| "macos-popover-surface".into())
                        .child(body),
                )
        })
}

/// The opacity of a popup surface fading in on opening.
///
/// Base unmounts popover and hover card content the moment they close, so the
/// card fades in only; its fade-out would need Base to keep closing content
/// mounted. The fade state lives only while the content renders, so every
/// opening starts from transparent.
pub(crate) fn popup_fade(id: &ElementId, window: &mut Window, cx: &mut App) -> f32 {
    let fade_id = ElementId::NamedChild(id.clone().into(), "macos-popup-fade".into());
    let sample = Presence::new(fade_id, true)
        // Non-key native hover panels cannot drive animation frames. Keep
        // their content visible on the first frame, including inline fallbacks
        // shown by an inactive owner window.
        .transition(out_cubic(animated_duration(window, POPUP_FADE)))
        .sample(window, cx);
    #[cfg(test)]
    tests::LAST_FADE.with(|fade| fade.set(sample.progress));
    sample.progress
}

/// macOS popup chrome. Content remains free to use forms, rows, and actions.
///
/// Spec §7.4 (C26): radius 16 and SwiftUI's `.padding()` of 16, with no arrow.
pub fn popover_surface(cx: &App) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .flex_col()
        .gap(rems(0.875))
        .w(rems(17.5))
        .max_w(gpui_kit::relative(1.))
        .p(pt(POPOVER_PADDING))
        .border_1()
        .rounded(pt(POPOVER_RADIUS))
        .border_color(t.glass_border())
        .bg(t.glass_fill())
        .shadow(t.popover_shadow())
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
}

/// Chrome for either an inline fallback or a real native popup window. In a
/// glass popup the glass is the surface, so it declares the popover radius
/// (spec §7.4, plan D13).
pub(crate) fn presented_popover_surface(window: &Window, cx: &App) -> Div {
    let surface = popover_surface(cx);
    crate::native::set_popup_corner_radius(window, POPOVER_RADIUS);
    if crate::native::is_glass_popup(window) {
        surface
            .bg(gpui_kit::transparent_black())
            .border_0()
            .shadow_none()
    } else {
        let mut fill = cx.macos().glass_fill();
        fill.a = 1.;
        surface.bg(fill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{FocusHandle, Render, TestAppContext};
    use std::cell::Cell;

    thread_local! {
        /// The last opacity [`popup_fade`] sampled on this test thread.
        pub(super) static LAST_FADE: Cell<f32> = const { Cell::new(-1.) };
    }
    struct Harness {
        trigger: FocusHandle,
        checked: bool,
        checkbox_focus: FocusHandle,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let target = cx.entity();
            crate::focus_scope("root").size_full().child(popover(
                "options",
                crate::button(
                    "trigger",
                    "Display options",
                    crate::ButtonVariant::Secondary,
                    cx,
                )
                .track_focus(&self.trigger),
                move |_, _, cx| {
                    let checked = target.read(cx).checked;
                    let checkbox_focus = target.read(cx).checkbox_focus.clone();
                    let target = target.clone();
                    div()
                        .debug_selector(|| "test-popover-content".into())
                        .child(
                            crate::checkbox(
                                "hidden",
                                "Show hidden files",
                                if checked {
                                    gpui_kit::base::CheckboxState::Checked
                                } else {
                                    gpui_kit::base::CheckboxState::Unchecked
                                },
                                cx,
                            )
                            .track_focus(&checkbox_focus)
                            .on_change(move |value, _, _, cx| {
                                target.update(cx, |state, cx| {
                                    state.checked = value == gpui_kit::base::CheckboxState::Checked;
                                    cx.notify();
                                })
                            }),
                        )
                },
            ))
        }
    }
    #[gpui_kit::test]
    fn keyboard_edits_popup_without_closing_then_escape_restores_focus(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            checked: false,
            checkbox_focus: cx.focus_handle(),
        });
        cx.update(|window, cx| {
            view.read(cx).trigger.clone().focus(window, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("test-popover-content").is_some());
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.update(|window, cx| {
            assert!(
                view.read(cx).checkbox_focus.is_focused(window),
                "Tab should focus checkbox"
            )
        });
        let keystroke = gpui_kit::Keystroke::parse("space").unwrap();
        cx.simulate_event(gpui_kit::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(view.read(cx).checked);
        });
        assert!(
            cx.debug_bounds("test-popover-content").is_some(),
            "editing does not dismiss the popup"
        );
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(view.read(cx).trigger.is_focused(window));
        });
        assert!(cx.debug_bounds("test-popover-content").is_none());
    }

    /// Spec §7.4 (C26): radius 16 and padding 16 (inside the inline
    /// fallback's hairline border), no arrow, and the glass radius declared
    /// at the owner's zoom. On 1x and 2x displays at 100% and 200% zoom.
    #[gpui_kit::test]
    fn popover_radius_and_padding_match_c26(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            checked: false,
            checkbox_focus: cx.focus_handle(),
        });
        for (scale, zoom) in [(1f32, 1f32), (1., 2.), (2., 1.), (2., 2.)] {
            cx.simulate_scale_factor_change(scale);
            cx.update(|window, cx| {
                window.set_rem_size(gpui_kit::px(16. * zoom));
                view.read(cx).trigger.clone().focus(window, cx);
                window.refresh();
                window.draw(cx).clear(cx);
            });
            crate::native::take_popup_radii();
            cx.simulate_keystrokes("enter");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let surface = cx.debug_bounds("macos-popover-surface").expect("open");
            let content = cx.debug_bounds("test-popover-content").unwrap();
            let border = 1.;
            let what = format!("{scale}x {zoom}");
            for (inset, side) in [
                (content.left() - surface.left(), "leading"),
                (content.top() - surface.top(), "top"),
                (surface.right() - content.right(), "trailing"),
                (surface.bottom() - content.bottom(), "bottom"),
            ] {
                assert!(
                    (inset.as_f32() - border - 16. * zoom).abs() <= 0.5,
                    "{what} {side} padding {inset:?}"
                );
            }
            cx.update(|window, _| {
                let scaled = surface.scale(window.scale_factor());
                let quad = window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.bounds == scaled && quad.background.as_solid().is_some())
                    .unwrap_or_else(|| panic!("{what}: surface quad"));
                let radius = quad.corner_radii.top_left.0 / scale;
                assert!(
                    (radius - 16. * zoom).abs() <= 0.5,
                    "{what}: radius {radius}"
                );
            });
            let radii = crate::native::take_popup_radii();
            assert!(!radii.is_empty());
            assert!(
                radii.iter().all(|r| *r == gpui_kit::px(16. * zoom)),
                "{what}: {radii:?}"
            );
            cx.simulate_keystrokes("escape");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert!(cx.debug_bounds("macos-popover-surface").is_none());
        }
    }

    #[gpui_kit::test]
    fn opening_fades_the_card_in_and_every_opening_starts_transparent(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            checked: false,
            checkbox_focus: cx.focus_handle(),
        });
        let draw = |cx: &mut gpui_kit::VisualTestContext| {
            cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            LAST_FADE.with(Cell::get)
        };
        cx.update(|window, cx| {
            window.activate_window();
            view.read(cx).trigger.clone().focus(window, cx);
        });
        cx.run_until_parked();
        draw(cx);
        for _ in 0..2 {
            cx.simulate_keystrokes("enter");
            assert_eq!(draw(cx), 0., "an opening card starts transparent");
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(50));
            let partway = draw(cx);
            assert!(partway > 0. && partway < 1., "fading in: {partway}");
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            assert_eq!(draw(cx), 1., "the card settles opaque");
            cx.simulate_keystrokes("escape");
            draw(cx);
            assert!(cx.debug_bounds("test-popover-content").is_none());
        }
    }

    #[gpui_kit::test]
    fn inactive_popups_are_visible_without_animation_frames(cx: &mut TestAppContext) {
        struct BackgroundPopup;
        impl Render for BackgroundPopup {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().opacity(popup_fade(&"background-popup".into(), window, cx))
            }
        }
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| BackgroundPopup);
        cx.deactivate_window();
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(LAST_FADE.with(Cell::get), 1.);
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
    }
}
