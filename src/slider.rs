//! Horizontal single-value and range sliders with native pointer interaction.
//! The rounded track follows pointer drags exactly and eases keyboard changes.
use crate::ActiveTheme;
use crate::motion::{SLIDER_KNOB_SCALE, SLIDER_TRAVEL, out_cubic};
use crate::state::{PaintRole, Part, States, paint};
use gpui_kit::base::motion::{Transition, transition};
use gpui_kit::base::{
    Slider, SliderIndicator, SliderThumb, SliderTrack,
    slider::{SliderEvent, SliderState, SliderValue},
};
use gpui_kit::rems;
use gpui_kit::{
    App, ElementId, Entity, FocusHandle, KeyDownEvent, MouseButton, Window, div, prelude::*, px,
    relative,
};
use std::time::Duration;

/// A subtle enlargement helps identify the thumb under the pointer.
const HOT_KNOB_SCALE: f32 = 1.06;
/// The native resting thumb is a 20 × 16 pill at 100% zoom.
const KNOB_WIDTH: f32 = 1.25;
const KNOB_HEIGHT: f32 = 1.;
/// The knob's vertical center within the native 16-point slider.
const KNOB_CENTER: f32 = 0.5;

/// Compose themed base parts. Pass disabled here so both track and thumbs are inert.
/// The returned base Slider remains available for layout/style customization.
pub fn slider(
    state: &Entity<SliderState>,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> Slider {
    let t = cx.macos().clone();
    // Spec §6.2.8 (C18): only the fill changes with state.
    let states = States::in_window(window).disabled(disabled);
    let active = states.window_active;
    let id = state.entity_id();
    let target_percentage = state.read(cx).percentage();
    let range = state.read(cx).value().is_range();
    // Base keeps its drag flag private, so the slider records its own: set when
    // the pointer presses the track or a knob, cleared when it is released.
    let pointer = window
        .use_keyed_state(("macos-slider-pointer", id), cx, |_, _| false)
        .clone();
    if !active {
        pointer.update(cx, |pressed, _| *pressed = false);
    }
    let travel = if *pointer.read(cx) || !active {
        Transition::new(Duration::ZERO)
    } else {
        out_cubic(SLIDER_TRAVEL)
    };
    let percentage = transition(
        ElementId::from(("macos-slider-start", id)),
        target_percentage.start,
        travel.clone(),
        window,
        cx,
    )
        ..transition(
            ElementId::from(("macos-slider-end", id)),
            target_percentage.end,
            travel,
            window,
            cx,
        );
    let press = pointer.clone();
    let release = pointer.clone();
    let release_out = pointer.clone();
    let mut track = SliderTrack::new(state)
        .disabled(disabled)
        .relative()
        .size_full()
        .when(!disabled, |track| {
            track
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    press.update(cx, |pressed, _| *pressed = true)
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    release.update(cx, |pressed, _| *pressed = false)
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    release_out.update(cx, |pressed, _| *pressed = false)
                })
        })
        .child(
            SliderIndicator::new(state)
                .absolute()
                .top(rems(0.3125))
                // The native bar fills the control's width, while the value
                // mapping keeps each thumb center half a thumb from the ends.
                .left(rems(KNOB_WIDTH / 2.))
                .right(rems(KNOB_WIDTH / 2.))
                .w_auto()
                .h(rems(0.375))
                .child(
                    div()
                        .absolute()
                        .debug_selector(|| "macos-slider-track".into())
                        .left(rems(-KNOB_WIDTH / 2.))
                        .right(rems(-KNOB_WIDTH / 2.))
                        .h_full()
                        .rounded_full()
                        .bg(paint(PaintRole::Slider, Part::Track, states, &t)),
                )
                .child(
                    div()
                        .absolute()
                        .debug_selector(|| "macos-slider-selection".into())
                        .h_full()
                        .left(relative(percentage.start))
                        .when(!range, |selection| selection.left(rems(-KNOB_WIDTH / 2.)))
                        .right(relative(1. - percentage.end))
                        .rounded_full()
                        .bg(paint(PaintRole::Slider, Part::Fill, states, &t)),
                ),
        );
    for start in [true, false] {
        if start && !range {
            continue;
        }
        let focus = window
            .use_keyed_state(
                (
                    gpui_kit::ElementId::from(("macos-slider-focus", state.entity_id())),
                    if start { "start" } else { "end" },
                ),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let edge = if start { "start" } else { "end" };
        let hovered = window
            .use_keyed_state(
                (ElementId::from(("macos-slider-hover", id)), edge),
                cx,
                |_, _| false,
            )
            .clone();
        let hot = active && !disabled && (*hovered.read(cx) || *pointer.read(cx));
        let scale = transition(
            (ElementId::from(("macos-slider-knob", id)), edge),
            if hot { HOT_KNOB_SCALE } else { 1. },
            out_cubic(crate::motion::animated_duration(window, SLIDER_KNOB_SCALE)),
            window,
            cx,
        );
        // Grow symmetrically by device pixels to keep the thumb centered.
        let rest_width = rems(KNOB_WIDTH).to_pixels(window.rem_size());
        let rest_height = rems(KNOB_HEIGHT).to_pixels(window.rem_size());
        let scale_factor = window.scale_factor();
        let grow = ((scale - 1.) * rest_width.as_f32() / 2. * scale_factor).ceil() / scale_factor;
        let width = rest_width + px(2. * grow);
        let height = rest_height + px(2. * grow);
        let center = rems(KNOB_CENTER).to_pixels(window.rem_size());
        let position = if start {
            percentage.start
        } else {
            percentage.end
        };
        let inset = rems(KNOB_WIDTH / 2.).to_pixels(window.rem_size());
        let target = state.clone();
        let pointer_focus: FocusHandle = focus.clone();
        let knob_press = pointer.clone();
        track = track.child(
            SliderThumb::new(state)
                .start(start)
                .disabled(disabled)
                .absolute()
                .top(center - height / 2.)
                .left(relative(position))
                .ml(inset * (1. - 2. * position) - width / 2.)
                .w(width)
                .h(height)
                .debug_selector(move || format!("macos-slider-thumb-{edge}"))
                .rounded_full()
                .border_1()
                .border_color(t.foreground.opacity(0.))
                .bg(paint(PaintRole::Slider, Part::Knob, states, &t))
                .shadow(t.control_shadow())
                .when(!disabled, |thumb| {
                    thumb
                        .track_focus(&focus)
                        .when(active, |thumb| {
                            thumb.focus_visible(|s| {
                                s.border_color(t.accent).shadow(t.focus_shadow())
                            })
                        })
                        .on_hover(move |hover, _, cx| {
                            hovered.update(cx, |hovered, cx| {
                                *hovered = *hover;
                                cx.notify();
                            })
                        })
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            knob_press.update(cx, |pressed, _| *pressed = true);
                            pointer_focus.focus(window, cx)
                        })
                        .on_key_down(move |event: &KeyDownEvent, window, cx| {
                            if event.keystroke.modifiers.modified() {
                                return;
                            }
                            let key = event.keystroke.key.as_str();
                            if !matches!(
                                key,
                                "left" | "right" | "down" | "up" | "h" | "l" | "home" | "end"
                            ) {
                                return;
                            }
                            target.update(cx, |state, cx| {
                                let next = keyboard_value(
                                    state.value(),
                                    start,
                                    key,
                                    state.min_value(),
                                    state.max_value(),
                                    state.step_value(),
                                );
                                if next != state.value() {
                                    state.set_value(next, window, cx);
                                    cx.emit(SliderEvent::Change(next));
                                    cx.emit(SliderEvent::Release(next));
                                }
                            });
                            cx.stop_propagation();
                        })
                }),
        );
    }
    Slider::new(state)
        .disabled(disabled)
        .w_full()
        .h(rems(1.))
        .child(track)
}

fn keyboard_value(
    value: SliderValue,
    start: bool,
    key: &str,
    min: f32,
    max: f32,
    step: f32,
) -> SliderValue {
    let (current, lower, upper) = match value {
        SliderValue::Single(value) => (value, min, max),
        SliderValue::Range(a, b) if start => (a, min, b),
        SliderValue::Range(a, b) => (b, a, max),
    };
    let next = match key {
        "home" => lower,
        "end" => upper,
        "left" | "down" | "h" => (current - step).max(lower),
        _ => (current + step).min(upper),
    };
    match value {
        SliderValue::Single(_) => SliderValue::Single(next),
        SliderValue::Range(_, b) if start => SliderValue::Range(next, b),
        SliderValue::Range(a, _) => SliderValue::Range(a, next),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, Modifiers, Render, TestAppContext, point, px};

    struct Harness {
        state: Entity<SliderState>,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(216.))
                .child(slider(&self.state, false, window, cx))
        }
    }

    fn knob_center(cx: &mut gpui_kit::VisualTestContext) -> f32 {
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = cx.debug_bounds("macos-slider-thumb-end").unwrap();
        bounds.center().x.as_f32()
    }

    #[gpui_kit::test]
    fn set_values_glide_but_pointer_presses_track_exactly(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let state = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(0.));
        let (_, cx) = cx.add_window_view(|_, _| Harness {
            state: state.clone(),
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let rest = knob_center(cx);

        cx.update(|window, cx| state.update(cx, |state, cx| state.set_value(100., window, cx)));
        let start = knob_center(cx);
        cx.executor().advance_clock(SLIDER_TRAVEL / 2);
        let partway = knob_center(cx);
        cx.executor().advance_clock(SLIDER_TRAVEL);
        let settled = knob_center(cx);
        assert_eq!(start, rest, "the knob starts from where it was");
        assert!(
            partway > rest && partway < settled,
            "{rest} {partway} {settled}"
        );

        // A press on the track follows the pointer without gliding.
        let track = cx.debug_bounds("macos-slider-thumb-end").unwrap();
        let target = point(px(rest + (settled - rest) * 0.25), track.center().y);
        cx.simulate_mouse_down(target, MouseButton::Left, Modifiers::default());
        let pressed = knob_center(cx);
        assert!(
            (pressed - target.x.as_f32()).abs() < 2.,
            "{pressed} {:?}",
            target.x
        );
        // The pressed knob grows around its center.
        cx.executor().advance_clock(SLIDER_KNOB_SCALE * 2);
        assert!((knob_center(cx) - pressed).abs() < 0.01, "grows in place");
        let grown = cx
            .debug_bounds("macos-slider-thumb-end")
            .unwrap()
            .size
            .width;
        // At least one device pixel on each side, and no more than the scale asks.
        assert!(grown.as_f32() > 20. && grown.as_f32() <= 22., "{grown:?}");
        cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::default());
        let released = knob_center(cx);
        assert!(
            (released - pressed).abs() < 0.5,
            "releasing does not move the knob"
        );
        // Still hot while hovered; it settles once the pointer leaves.
        cx.simulate_mouse_move(point(px(0.), px(200.)), None, Modifiers::default());
        knob_center(cx);
        cx.executor().advance_clock(SLIDER_KNOB_SCALE * 2);
        knob_center(cx);
        let rested = cx
            .debug_bounds("macos-slider-thumb-end")
            .unwrap()
            .size
            .width;
        assert_eq!(rested, px(20.));
    }

    #[gpui_kit::test]
    fn inactive_sliders_show_new_values_without_animation_frames(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for (initial, next) in [
            (SliderValue::Single(0.), SliderValue::Single(100.)),
            (SliderValue::Range(20., 80.), SliderValue::Range(40., 60.)),
        ] {
            let state = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(initial));
            let (_, cx) = cx.add_window_view(|_, _| Harness {
                state: state.clone(),
            });
            cx.deactivate_window();
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.update(|window, cx| state.update(cx, |state, cx| state.set_value(next, window, cx)));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let values = match next {
                SliderValue::Single(end) => vec![("macos-slider-thumb-end", end)],
                SliderValue::Range(start, end) => vec![
                    ("macos-slider-thumb-start", start),
                    ("macos-slider-thumb-end", end),
                ],
            };
            for (id, value) in values {
                let thumb = cx.debug_bounds(id).unwrap();
                let expected = 10. + 196. * value / 100.;
                assert!(
                    (thumb.center().x.as_f32() - expected).abs() <= 0.5,
                    "The first inactive frame must show {value}% at {expected}px: {thumb:?}"
                );
            }
            cx.update(|window, cx| assert_eq!(window.simulate_next_frame(cx), 0));
        }
    }

    #[gpui_kit::test]
    fn native_track_fills_the_width_and_end_clicks_preserve_inset_thumb_travel(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let state = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(50.));
        let (view, cx) = cx.add_window_view(|_, _| Harness {
            state: state.clone(),
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        for zoom in [1., 1.25, 2.] {
            cx.update(|window, cx| {
                window.set_rem_size(px(16. * zoom));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let track = cx.debug_bounds("macos-slider-track").unwrap();
            assert_eq!(track.size.width, px(216.));
            assert_eq!(track.size.height, px(6. * zoom));
            assert!((track.top().as_f32() - 5. * zoom).abs() <= 0.5);
            for (pointer, expected_value, expected_center) in [
                (track.left() + px(1.), 0., track.left() + px(10. * zoom)),
                (track.right() - px(1.), 100., track.right() - px(10. * zoom)),
            ] {
                let target = point(pointer, track.center().y);
                cx.simulate_mouse_down(target, MouseButton::Left, Modifiers::default());
                assert!((knob_center(cx) - expected_center.as_f32()).abs() < 0.01);
                cx.update(|_, cx| {
                    assert_eq!(state.read(cx).value(), SliderValue::Single(expected_value));
                });
                cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::default());
            }
        }
    }
    #[test]
    fn keys_respect_endpoints_and_prevent_range_crossing() {
        assert_eq!(
            keyboard_value(SliderValue::Single(100.), false, "right", 0., 100., 5.),
            SliderValue::Single(100.)
        );
        assert_eq!(
            keyboard_value(SliderValue::Range(20., 30.), true, "end", 0., 100., 5.),
            SliderValue::Range(30., 30.)
        );
        assert_eq!(
            keyboard_value(SliderValue::Range(20., 30.), false, "home", 0., 100., 5.),
            SliderValue::Range(20., 20.)
        );
    }
}
