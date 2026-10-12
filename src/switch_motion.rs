//! Rounded switch track and white thumb with smooth state transitions.
use crate::ActiveTheme;
use crate::motion::{CONTROL_COLOR, SWITCH_TRAVEL, color, out_cubic};
use crate::state::{PaintRole, Part, States, paint};
use crate::tokens::{SWITCH_INSET, SWITCH_KNOB, SWITCH_TRACK, pt};
use gpui_kit::base::motion::transition;
use gpui_kit::base::{SwitchThumb, SwitchTrack};
use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder as _,
};

/// Native 54 × 24 track, 32 × 20 pill, and two-point inset (spec §4.5).
const KNOB_TRAVEL: f32 = SWITCH_TRACK.0 - SWITCH_INSET * 2. - SWITCH_KNOB.0;

/// A switch's track and knob. Rendered inside the switch, so its motion state
/// is keyed within the switch's element scope.
#[derive(IntoElement)]
pub(crate) struct SwitchMotion {
    id: ElementId,
    checked: bool,
    disabled: bool,
    pressed: bool,
    focus_visible: bool,
}

impl SwitchMotion {
    pub(crate) fn new(id: ElementId, checked: bool) -> Self {
        Self {
            id,
            checked,
            disabled: false,
            pressed: false,
            focus_visible: false,
        }
    }

    /// A left-button press anywhere on the switch (§6.2.7 P).
    pub(crate) fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    /// Keyboard focus in an active window: the ring surrounds the track (§6.3).
    pub(crate) fn focus_visible(mut self, visible: bool) -> Self {
        self.focus_visible = visible;
        self
    }

    /// The final disabled state; track and knob use their disabled arms.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for SwitchMotion {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.macos().clone();
        let checked = self.checked;
        let states = States::in_window(window)
            .disabled(self.disabled)
            .pressed(self.pressed);
        let fade = color(crate::motion::animated_duration(window, CONTROL_COLOR));
        // Spec §6.2.7: explicit track and knob arms per state (C19).
        let track_bg = transition(
            "macos-switch-track-bg",
            paint(
                PaintRole::SwitchTrack { on: checked },
                Part::Fill,
                states,
                &t,
            ),
            fade.clone(),
            window,
            cx,
        );
        let thumb_bg: gpui_kit::Hsla = transition(
            "macos-switch-thumb-bg",
            paint(
                PaintRole::SwitchKnob { on: checked },
                Part::Fill,
                states,
                &t,
            ),
            fade,
            window,
            cx,
        );
        let travel = transition(
            "macos-switch-thumb-x",
            if checked { 1f32 } else { 0. },
            out_cubic(crate::motion::animated_duration(window, SWITCH_TRAVEL)),
            window,
            cx,
        );
        let id = self.id;
        SwitchTrack::new(id.clone())
            .checked(checked)
            .w(pt(SWITCH_TRACK.0))
            .h(pt(SWITCH_TRACK.1))
            .flex_shrink_0()
            .p(pt(SWITCH_INSET))
            .relative()
            .rounded_full()
            .bg(track_bg)
            .debug_selector({
                let id = id.clone();
                move || format!("macos-switch-track-{id}")
            })
            .child(
                SwitchThumb::new(checked)
                    .w(pt(SWITCH_KNOB.0))
                    .h(pt(SWITCH_KNOB.1))
                    .rounded_full()
                    .bg(thumb_bg)
                    .shadow(crate::tokens::switch_knob_shadow(&t))
                    .ml(pt(KNOB_TRAVEL * travel))
                    .child(
                        div()
                            .size_full()
                            .debug_selector(move || format!("macos-switch-thumb-{id}")),
                    ),
            )
            // Spec §6.3: a capsule ring around the track (60 × 30).
            .when(self.focus_visible, |track| {
                track.child(crate::state::focus_ring_pt(SWITCH_TRACK.1 / 2., window, &t))
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::ActiveTheme;
    use gpui_kit::{Context, Render, TestAppContext, VisualTestContext, prelude::*};
    use std::time::Duration;

    struct Harness {
        checked: bool,
    }

    impl Render for Harness {
        fn render(
            &mut self,
            _: &mut gpui_kit::Window,
            cx: &mut Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            crate::focus_scope("switch-test")
                .size_full()
                .child(crate::switch("wifi", "Wi-Fi", self.checked, cx))
        }
    }

    fn thumb_x(cx: &mut VisualTestContext) -> f32 {
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.debug_bounds("macos-switch-thumb-wifi")
            .unwrap()
            .origin
            .x
            .as_f32()
    }

    fn toggle(view: &gpui_kit::Entity<Harness>, cx: &mut VisualTestContext) {
        cx.update(|_, cx| {
            view.update(cx, |harness, cx| {
                harness.checked = !harness.checked;
                cx.notify();
            })
        });
    }

    #[gpui_kit::test]
    fn knob_slides_to_on_over_the_switch_travel(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| Harness { checked: false });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let off = thumb_x(cx);

        toggle(&view, cx);
        assert_eq!(thumb_x(cx), off, "the first frame starts from rest");
        cx.executor().advance_clock(Duration::from_millis(40));
        let midway = thumb_x(cx);
        cx.executor().advance_clock(Duration::from_millis(200));
        let on = thumb_x(cx);

        assert!(on > off);
        assert!(off < midway && midway < on, "{off} < {midway} < {on}");
        // Out-cubic covers most of the distance early.
        assert!(midway - off > (on - off) / 3.);
    }

    #[gpui_kit::test]
    fn reduced_motion_snaps_the_knob(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| Harness { checked: false });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let off = thumb_x(cx);
        toggle(&view, cx);
        let on = thumb_x(cx);
        assert!(on > off);
        cx.executor().advance_clock(Duration::from_millis(200));
        assert_eq!(thumb_x(cx), on);
    }

    #[gpui_kit::test]
    fn inactive_switches_apply_state_and_theme_changes_without_waiting_for_frames(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| Harness { checked: false });
        cx.deactivate_window();
        let off = thumb_x(cx);
        toggle(&view, cx);
        let on = thumb_x(cx);
        assert!(on > off, "The first inactive frame must show the new state");
        cx.update(|_, cx| crate::Theme::dark().apply(cx));
        assert_eq!(thumb_x(cx), on);
        let track = cx.debug_bounds("macos-switch-track-wifi").unwrap();
        cx.update(|window, cx| {
            let expected = crate::state::paint(
                crate::state::PaintRole::SwitchTrack { on: true },
                crate::state::Part::Fill,
                crate::state::States::active().window_active(false),
                cx.macos(),
            );
            assert!(window.painted_quads().iter().any(|quad| {
                quad.bounds == track.scale(window.scale_factor())
                    && quad.background.as_solid() == Some(expected)
            }));
            assert_eq!(window.simulate_next_frame(cx), 0);
        });
        toggle(&view, cx);
        assert_eq!(thumb_x(cx), off);
    }
}
