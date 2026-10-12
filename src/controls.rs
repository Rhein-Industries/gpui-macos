//! Styled constructors retain gpui-base's controlled-state builder APIs.
use crate::NativeTextStyle as _;
use crate::state::{PaintRole, Part, States, Tint, focus_ring_pt, over, paint};
use crate::tokens::{
    BEZEL_HEIGHT, BEZEL_PADDING_X, CHECKBOX_RADIUS, CHECKBOX_ROW_HEIGHT, CHECKBOX_SIZE,
    CONTROL_RADIUS, ICON_BUTTON_BEZEL, ICON_BUTTON_GLYPH, ICON_BUTTON_MARGIN, INDICATOR_LABEL_GAP,
    RADIO_DOT, RADIO_SIZE, SWITCH_LABEL_GAP, pt,
};
use crate::{ActiveTheme, Button, IconName, Link, icon};
use gpui_kit::base::{Checkbox, CheckboxState, Radio, StateStyles, Switch, Tab, Tabs, Toggle};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::rems;
use gpui_kit::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
};

/// Well and mark of a checkbox or radio, spec §6.2.6.
pub(crate) fn check_paint(theme: &crate::Theme, states: States, on: bool) -> (Hsla, Hsla) {
    let role = if on {
        PaintRole::CheckOn
    } else {
        PaintRole::CheckOff
    };
    (
        paint(role, Part::Fill, states, theme),
        paint(role, Part::Label, states, theme),
    )
}

/// `mixed` selects the dash glyph; the on state is the final checked value.
#[derive(IntoElement)]
struct NativeCheckboxIndicator(ElementId, bool, gpui_kit::base::ControlState);

impl RenderOnce for NativeCheckboxIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (id, mixed, control) = (self.0, self.1, self.2);
        let on = control.checked;
        // Base reports a left-button press anywhere on the control (§6.2.6 P).
        let states = States::in_window(window)
            .disabled(control.disabled)
            .pressed(control.pressed);
        let (well, mark) = check_paint(cx.macos(), states, on);
        // Spec §6.3: the ring surrounds the box only (22 × 22).
        let ring = control
            .focus_visible
            .then(|| focus_ring_pt(CHECKBOX_RADIUS, window, cx.macos()));
        // Spec §4.4 / §4.1 (C9): a 16 pt box with radius 5.5.
        div()
            .size(pt(CHECKBOX_SIZE))
            .debug_selector(move || format!("macos-checkbox-indicator-{id}"))
            .relative()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded(pt(CHECKBOX_RADIUS))
            .bg(well)
            .text_color(mark)
            .when(on, |indicator| {
                indicator.child(
                    icon(if mixed {
                        IconName::Minus
                    } else {
                        IconName::Check
                    })
                    .size(rems(0.875))
                    .text_color(mark),
                )
            })
            .children(ring)
    }
}

#[derive(IntoElement)]
struct NativeRadioIndicator(ElementId, gpui_kit::base::ControlState);

impl RenderOnce for NativeRadioIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (id, control) = (self.0, self.1);
        let on = control.checked;
        let states = States::in_window(window)
            .disabled(control.disabled)
            .pressed(control.pressed);
        let (well, mark) = check_paint(cx.macos(), states, on);
        // Spec §6.3: the focused radio's circle carries the ring.
        let ring = control
            .focus_visible
            .then(|| focus_ring_pt(RADIO_SIZE / 2., window, cx.macos()));
        // Spec §4.4: a 16 pt circle with a 5 pt dot.
        div()
            .size(pt(RADIO_SIZE))
            .debug_selector(move || format!("macos-radio-indicator-{id}"))
            .relative()
            .flex_shrink_0()
            .rounded_full()
            .bg(well)
            .flex()
            .items_center()
            .justify_center()
            .when(on, |indicator| {
                indicator.child(div().size(pt(RADIO_DOT)).rounded_full().bg(mark))
            })
            .children(ring)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Outline,
    #[default]
    Secondary,
    Danger,
    /// A text-only button without a bezel (`.borderless`): secondary label
    /// ink, no hover, darker while pressed.
    Borderless,
}

impl ButtonVariant {
    fn paint_role(self) -> PaintRole {
        match self {
            Self::Primary => PaintRole::Prominent(Tint::Accent),
            Self::Danger => PaintRole::Prominent(Tint::Red),
            Self::Outline | Self::Secondary => PaintRole::Bezel,
            Self::Borderless => PaintRole::Borderless,
        }
    }
}

pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    variant: ButtonVariant,
    cx: &App,
) -> Button {
    let t = cx.macos();
    let role = variant.paint_role();
    let rest = States::active();
    let bg = paint(role, Part::Fill, rest, t);
    let fg = paint(role, Part::Label, rest, t);
    let borderless = role == PaintRole::Borderless;
    // Restyled callers keep a pressed fill; the resolver also sets the label.
    let pressed_bg = paint(role, Part::Fill, rest.pressed(true), t);
    let label = label.into();
    Button::new(id)
        .accessibility_label(label.clone())
        .when(!label.is_empty(), |button| button.child(label))
        // Bezel family, spec §4.2: 24 tall (3 + 1 border + 16 Body line on
        // each side of the label), 12 pt padding, radius 6.
        .min_h(pt(BEZEL_HEIGHT))
        .py(rems(0.1875))
        .px(pt(BEZEL_PADDING_X))
        .gap(rems(0.375))
        .border_1()
        .rounded(pt(CONTROL_RADIUS))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        // A borderless button is its label: no bezel padding, border or
        // minimum height. Source: the SwiftUI probe's regular `.borderless`
        // "Button" frame is 41 x 16 pt, the label's 13 pt advance (40.29,
        // spec §4) by its Body line (16, §3.1). Keyboard focus keeps the ring.
        // No role here has hover (spec §6.5, C17); keyboard focus draws the
        // outset ring at render (§6.3) without an accent border.
        .when(borderless, |button| {
            button.min_h(rems(0.)).p(rems(0.)).border_0()
        })
        .bg(bg)
        .text_color(fg)
        .paint_role(role)
        .record_role_default()
        .active(|s| s.bg(pressed_bg))
        .selected_fill(match role {
            PaintRole::Prominent(_) => bg,
            _ => over(bg, t.selected_fill()),
        })
        // Only for buttons whose caller replaces the role paint (plan D6).
        .styles(|s| s.disabled(|s| s.opacity(crate::tokens::CUSTOM_DISABLED_OPACITY)))
}

/// An icon-only bordered button (37.5 × 20.5 bezel in a 39 × 21 frame). `label` names the action for assistive tech;
/// pair it with [`crate::with_tooltip`] so sighted users can read it too.
pub fn icon_button(
    id: impl Into<ElementId>,
    name: IconName,
    label: impl Into<SharedString>,
    variant: ButtonVariant,
    cx: &App,
) -> Button {
    let (width, height) = ICON_BUTTON_BEZEL;
    let (margin_x, margin_y) = ICON_BUTTON_MARGIN;
    let frame = (width + 2. * margin_x, height + 2. * margin_y);
    button(id, "", variant, cx)
        .accessibility_label(label)
        .fill_duration(crate::motion::CURSOR_COLOR)
        // Spec §4.2 (C7): a 37.5 × 20.5 bezel inside a 39 × 21 layout frame.
        // The margins are outside the bezel, so the hit area is the visible
        // bezel (no 24 pt hit-target exception). The text button's minimum
        // height, padding and border do not apply.
        .min_h(rems(0.))
        .p(rems(0.))
        .border_0()
        .w(pt(width))
        .h(pt(height))
        .mx(pt(margin_x))
        .my(pt(margin_y))
        // Half-point lengths round separately on the device grid; at render
        // the frame rounds once and the margins absorb the remainder.
        .snapped_frame(ICON_BUTTON_BEZEL, frame)
        .flex_shrink_0()
        .justify_center()
        .child(icon(name).size(pt(ICON_BUTTON_GLYPH)))
}

pub fn checkbox(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    state: CheckboxState,
    cx: &App,
) -> Checkbox {
    let t = cx.macos();
    let id = id.into();
    let label = label.into();
    Checkbox::new(id.clone())
        .state(state)
        .accessibility_label(label.clone())
        .flex()
        // Hug the control and its label instead of stretching across a column.
        .self_start()
        .items_center()
        // Spec §4.4 (C2): a 17 pt labeled row with the box at y 0.5, gap 5.
        .gap(pt(INDICATOR_LABEL_GAP))
        .min_h(pt(CHECKBOX_ROW_HEIGHT))
        .rounded(rems(0.25))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .child_with_state(move |control| {
            NativeCheckboxIndicator(id, state == CheckboxState::Indeterminate, control)
        })
        .child(label)
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
}

pub fn switch(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
    cx: &App,
) -> Switch {
    let t = cx.macos();
    let id = id.into();
    let label = label.into();
    Switch::new(id.clone())
        .checked(checked)
        .accessibility_label(label.clone())
        .flex()
        // Hug the control and its label instead of stretching across a column.
        .self_start()
        .items_center()
        .gap(pt(SWITCH_LABEL_GAP))
        .rounded(rems(0.75))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
        .child_with_state(move |control| {
            crate::switch_motion::SwitchMotion::new(id, control.checked)
                .disabled(control.disabled)
                .pressed(control.pressed)
                .focus_visible(control.focus_visible)
        })
        .child(label)
}

pub fn radio(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
    cx: &App,
) -> Radio {
    let t = cx.macos();
    let id = id.into();
    let label = label.into();
    Radio::new(id.clone())
        .checked(checked)
        .accessibility_label(label.clone())
        .flex()
        // Hug the control and its label instead of stretching across a column.
        .self_start()
        .items_center()
        .gap(pt(INDICATOR_LABEL_GAP))
        .min_h(pt(RADIO_SIZE))
        .rounded(rems(0.25))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .child_with_state(move |control| NativeRadioIndicator(id, control))
        .child(label)
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
}

/// Render-time default paint for a toggle button or tab from its final value,
/// disabled state and window activity. It sits beneath caller styles, so a
/// caller `bg` or `text_color` wins and extra children inherit the label ink.
/// Neither role has hover (spec §6.5, C17).
fn bezel_styles(role: PaintRole, disabled: bool, window: &Window, cx: &App) -> StateStyles {
    let t = cx.macos();
    let states = States::in_window(window).disabled(disabled);
    let part = |part, states| paint(role, part, states, t);
    let fill = part(Part::Fill, states);
    let mut styles = StateStyles::new(
        StyleRefinement::default()
            .bg(fill)
            .text_color(part(Part::Label, states)),
    );
    // Base drops default transient styles while disabled; an inactive window
    // has no pressed paint either.
    if states.window_active {
        let pressed = states.pressed(true);
        styles = styles.active(
            StyleRefinement::default()
                .bg(part(Part::Fill, pressed))
                .text_color(part(Part::Label, pressed)),
        );
    }
    styles
}

pub fn toggle(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    pressed: bool,
    cx: &App,
) -> Toggle {
    let t = cx.macos();
    let label = label.into();
    Toggle::new(id)
        .pressed(pressed)
        .accessibility_label(label.clone())
        .flex()
        // Hug the control and its label instead of stretching across a column.
        .self_start()
        .items_center()
        .gap(rems(0.5))
        .min_h(pt(BEZEL_HEIGHT))
        .py(rems(0.1875))
        .px(rems(0.625))
        .border_1()
        .rounded(pt(CONTROL_RADIUS))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        .child(label)
        .relative()
        .child_with_resolved_style(|state, style| {
            ControlFocusRing(state.focus_visible, style.clone())
        })
        // Toggle off is a Bezel (spec §6.1); on is ToggleOn (§6.2.4). The
        // role follows the final pressed value.
        .default_styles(|state, window, cx| {
            let role = if state.checked {
                PaintRole::ToggleOn
            } else {
                PaintRole::Bezel
            };
            bezel_styles(role, state.disabled, window, cx)
        })
}

/// The whole-control focus ring of a Base control, from its render-time
/// `focus_visible` state and final resolved style, so a caller's radius or
/// border is followed (spec §6.3).
#[derive(IntoElement)]
pub(crate) struct ControlFocusRing(pub bool, pub StyleRefinement);

impl RenderOnce for ControlFocusRing {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::state::focus_ring_if(self.0, &self.1, window, cx.macos())
    }
}

/// A multi-choice toolbar. Compose independent `toggle` children; each retains
/// its own keyboard focus and controlled pressed state.
pub fn toggle_group(id: impl Into<ElementId>, cx: &App) -> gpui_kit::base::ToggleGroup {
    let t = cx.macos();
    gpui_kit::base::ToggleGroup::new(id)
        .flex()
        .flex_wrap()
        .items_center()
        .gap(rems(0.375))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
}

pub fn link(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    href: impl Into<SharedString>,
    cx: &App,
) -> Link {
    let t = cx.macos();
    let label = label.into();
    // LinkLabel, spec §6.2.10: inactive equals active.
    let rest = States::active();
    let color = paint(PaintRole::Link, Part::Label, rest, t);
    let pressed = paint(PaintRole::Link, Part::Label, rest.pressed(true), t);
    let disabled = paint(PaintRole::Link, Part::Label, rest.disabled(true), t);
    Link::new(id)
        .href(href)
        .open_with(|url, _, _, cx| cx.open_url(url))
        .accessibility_label(label.clone())
        .child(label)
        .flex()
        .items_center()
        .gap(rems(0.375))
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_family(t.font.clone())
        .text_color(color)
        .rounded(rems(0.25))
        .border_1()
        .border_color(t.foreground.opacity(0.))
        .px(rems(0.25))
        .py(rems(0.375))
        // No hover (spec §6.5, C17); the pointer cursor is kept.
        .active(|s| s.text_color(pressed))
        .focus_visible(|s| s.shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.text_color(disabled)))
}

/// A rounded segmented strip with gpui-base Tab semantics.
pub fn tabs(id: impl Into<ElementId>, cx: &App) -> Tabs {
    let t = cx.macos();
    // Segmented track, spec §4.2: 24 tall, radius 6.
    Tabs::new(id)
        .flex()
        .h(pt(BEZEL_HEIGHT))
        .items_center()
        .gap(rems(0.125))
        .p(rems(0.0625))
        .rounded(pt(CONTROL_RADIUS))
        .bg(paint(
            PaintRole::Segment { selected: false },
            Part::Track,
            States::active(),
            t,
        ))
}

pub fn tab(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    cx: &App,
) -> Tab {
    let t = cx.macos();
    let label = label.into();
    Tab::new(id)
        .selected(selected)
        .accessibility_label(label.clone())
        .child(label)
        .px(pt(BEZEL_PADDING_X))
        .h(rems(1.375))
        .py(rems(0.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap_1()
        .border_1()
        .rounded(pt(TAB_RADIUS))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        // Spec §6.3: a focused segment carries the ring.
        .relative()
        .child_with_resolved_style(|state, style| {
            ControlFocusRing(state.focus_visible, style.clone())
        })
        // Segmented, spec §6.2.5 (C6): accent selection with a white label,
        // following the final selected value. No hover (§6.5, C17).
        .default_styles(|state, window, cx| {
            let role = PaintRole::Segment {
                selected: state.checked,
            };
            bezel_styles(role, state.disabled, window, cx)
        })
}

/// A segment's corner radius inside the 24 pt track, in points.
pub(crate) const TAB_RADIUS: f32 = 5.;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::ink;
    use gpui_kit::{
        AppContext, Context, Entity, FocusHandle, Render, TestAppContext, VisualTestContext, Window,
    };

    struct ActivationHarness {
        slider: Entity<gpui_kit::base::slider::SliderState>,
        checked: CheckboxState,
        checkbox_changes: usize,
    }

    impl Render for ActivationHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let target = cx.entity();
            crate::focus_scope("window-activation-controls")
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(rems(0.5))
                .child(
                    button("activation-primary", "Continue", ButtonVariant::Primary, cx)
                        .debug_selector(|| "activation-primary".into()),
                )
                .child(
                    button(
                        "activation-disabled",
                        "Disabled",
                        ButtonVariant::Primary,
                        cx,
                    )
                    .disabled(true)
                    .underline()
                    .debug_selector(|| "activation-disabled".into()),
                )
                .child(
                    checkbox("activation-checkbox", "Remember me", self.checked, cx)
                        .debug_selector(|| "activation-checkbox".into())
                        .on_change(move |state, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.checked = state;
                                this.checkbox_changes += 1;
                                cx.notify();
                            })
                        }),
                )
                .child(radio("activation-radio", "Automatic", true, cx))
                .child(radio("activation-off-radio", "Manual", false, cx))
                .child(switch("activation-switch", "Wi-Fi", true, cx))
                .child(switch("activation-off-switch", "AirDrop", false, cx))
                .child(
                    checkbox(
                        "activation-disabled-checkbox",
                        "Managed",
                        CheckboxState::Checked,
                        cx,
                    )
                    .disabled(true)
                    .underline()
                    .debug_selector(|| "activation-disabled-checkbox".into()),
                )
                .child(radio("activation-disabled-radio", "Locked", true, cx).disabled(true))
                .child(switch("activation-disabled-switch", "Managed", true, cx).disabled(true))
                .child(
                    switch("activation-disabled-off-switch", "Managed off", false, cx)
                        .disabled(true),
                )
                .child(crate::slider(&self.slider, false, window, cx).w(rems(12.)))
                .child(
                    button(
                        "activation-secondary",
                        "Options",
                        ButtonVariant::Secondary,
                        cx,
                    )
                    .debug_selector(|| "activation-secondary".into()),
                )
                .child(
                    button(
                        "activation-disabled-secondary",
                        "Unavailable",
                        ButtonVariant::Secondary,
                        cx,
                    )
                    .disabled(true)
                    .underline()
                    .debug_selector(|| "activation-disabled-secondary".into()),
                )
                .child(
                    button(
                        "activation-borderless",
                        "Details",
                        ButtonVariant::Borderless,
                        cx,
                    )
                    .underline()
                    .debug_selector(|| "activation-borderless".into()),
                )
                .child(
                    button(
                        "activation-disabled-borderless",
                        "Details",
                        ButtonVariant::Borderless,
                        cx,
                    )
                    .disabled(true)
                    .underline()
                    .debug_selector(|| "activation-disabled-borderless".into()),
                )
                .child(
                    button("activation-restyled", "Custom", ButtonVariant::Primary, cx)
                        .bg(restyled_fill())
                        .debug_selector(|| "activation-restyled".into()),
                )
                .child(
                    toggle("activation-toggle", "Bold", true, cx)
                        .debug_selector(|| "activation-toggle".into()),
                )
                .child(
                    toggle("activation-disabled-toggle", "Italic", true, cx)
                        .disabled(true)
                        .debug_selector(|| "activation-disabled-toggle".into()),
                )
                .child(
                    tabs("activation-tabs", cx).child(
                        tab("activation-tab", "General", true, cx)
                            .debug_selector(|| "activation-tab".into()),
                    ),
                )
        }
    }

    fn restyled_fill() -> Hsla {
        gpui_kit::rgb(0x34a853).into()
    }

    fn painted_background(cx: &mut VisualTestContext, selector: &'static str) -> Hsla {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.update(|window, _| {
            let bounds = bounds.scale(window.scale_factor());
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.bounds == bounds && quad.background.as_solid().is_some())
                .unwrap_or_else(|| panic!("{selector} has a painted background"))
                .background
                .as_solid()
                .unwrap()
        })
    }

    // An inherited underline paints in the actual label ink, exposing the
    // rendered text color without duplicating the control's style resolution.
    fn painted_label_color(cx: &mut VisualTestContext, selector: &'static str) -> Hsla {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.update(|window, _| {
            let bounds = bounds.scale(window.scale_factor());
            let colors: Vec<_> = window
                .painted_underlines()
                .into_iter()
                .filter(|line| bounds.contains(&line.bounds.center()))
                .map(|line| line.color)
                .collect();
            assert!(
                !colors.is_empty(),
                "{selector} must paint its label underline"
            );
            assert!(colors.iter().all(|color| *color == colors[0]));
            colors[0]
        })
    }

    fn painted_radio_mark(cx: &mut VisualTestContext) -> Hsla {
        let center = cx
            .debug_bounds("macos-radio-indicator-activation-radio")
            .unwrap()
            .center();
        cx.update(|window, _| {
            window
                .painted_quads()
                .into_iter()
                .find(|quad| {
                    quad.bounds.center() == center.scale(window.scale_factor())
                        && quad.bounds.size
                            == gpui_kit::size(gpui_kit::px(5.), gpui_kit::px(5.))
                                .scale(window.scale_factor())
                })
                .unwrap()
                .background
                .as_solid()
                .unwrap()
        })
    }

    #[gpui_kit::test]
    fn rendered_controls_follow_real_window_activation_and_keep_disabled_distinct(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, cx| ActivationHarness {
            slider: cx.new(|_| {
                gpui_kit::base::slider::SliderState::new()
                    .min(0.)
                    .max(100.)
                    .default_value(65.)
            }),
            checked: CheckboxState::Checked,
            checkbox_changes: 0,
        });
        cx.simulate_mouse_move(
            gpui_kit::point(gpui_kit::px(500.), gpui_kit::px(500.)),
            None,
            Default::default(),
        );
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            cx.simulate_mouse_move(
                gpui_kit::point(gpui_kit::px(500.), gpui_kit::px(500.)),
                None,
                Default::default(),
            );
            cx.update(|window, cx| {
                assert!(window.is_window_active());
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let states = States::active();
            let expect = |role, part, states| paint(role, part, states, &theme);
            let on_check = |s| expect(PaintRole::CheckOn, Part::Fill, s);
            for (selector, role) in [
                ("activation-primary", PaintRole::Prominent(Tint::Accent)),
                ("activation-secondary", PaintRole::Bezel),
                (
                    "macos-checkbox-indicator-activation-checkbox",
                    PaintRole::CheckOn,
                ),
                ("macos-radio-indicator-activation-radio", PaintRole::CheckOn),
                (
                    "macos-switch-track-activation-switch",
                    PaintRole::SwitchTrack { on: true },
                ),
                ("macos-slider-selection", PaintRole::Slider),
                ("activation-toggle", PaintRole::ToggleOn),
                ("activation-tab", PaintRole::Segment { selected: true }),
            ] {
                assert_eq!(
                    painted_background(cx, selector),
                    expect(role, Part::Fill, states),
                    "{selector} uses its active paint"
                );
            }
            assert_eq!(on_check(states), theme.indicator_accent());
            assert_eq!(painted_background(cx, "activation-primary"), theme.accent);
            assert_eq!(
                painted_background(cx, "activation-secondary"),
                theme.bezel_fill(),
                "the bezel is translucent (C14)"
            );
            assert_eq!(painted_radio_mark(cx), theme.on_accent);
            assert_eq!(
                painted_background(cx, "activation-disabled"),
                theme.accent.opacity(0.69),
                "a disabled primary keeps a faded accent bezel"
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled"),
                theme.on_accent.opacity(0.5)
            );
            assert_eq!(
                painted_background(cx, "activation-disabled-secondary"),
                ink(&theme).opacity(0.039),
                "Bezel disabled fill replaces the old 45% opacity"
            );
            let bezel_disabled_label =
                ink(&theme).opacity(if theme.is_dark() { 0.290 } else { 0.257 });
            assert_eq!(
                painted_label_color(cx, "activation-disabled-secondary"),
                bezel_disabled_label
            );
            assert_eq!(
                painted_label_color(cx, "activation-borderless"),
                theme.secondary
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled-borderless"),
                ink(&theme).opacity(0.25)
            );
            assert_eq!(
                painted_background(cx, "activation-restyled"),
                restyled_fill(),
                "a caller-restyled button keeps its paint (D6)"
            );
            assert_eq!(
                painted_background(cx, "activation-disabled-toggle"),
                theme.indicator_accent().opacity(0.70)
            );
            for selector in [
                "macos-checkbox-indicator-activation-disabled-checkbox",
                "macos-radio-indicator-activation-disabled-radio",
            ] {
                assert_eq!(
                    painted_background(cx, selector),
                    ink(&theme).opacity(0.075),
                    "{selector} is a neutral well rather than a faded accent"
                );
            }
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-switch"),
                theme.indicator_accent().opacity(0.71),
                "a disabled on switch keeps a faded accent track"
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled-checkbox"),
                theme.tertiary_label()
            );
            let expected_off = if theme.is_dark() {
                Hsla::from(gpui_kit::rgb(0xcfcfcf)).opacity(0.063)
            } else {
                Hsla::black().opacity(0.051)
            };
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-off-switch"),
                expected_off
            );
            cx.deactivate_window();
            cx.update(|window, cx| {
                assert!(!window.is_window_active());
                window.draw(cx).clear(cx);
            });
            let inactive = states.window_active(false);
            for (selector, role) in [
                ("activation-primary", PaintRole::Bezel),
                ("activation-secondary", PaintRole::Bezel),
                (
                    "macos-checkbox-indicator-activation-checkbox",
                    PaintRole::CheckOn,
                ),
                ("macos-radio-indicator-activation-radio", PaintRole::CheckOn),
                (
                    "macos-radio-indicator-activation-off-radio",
                    PaintRole::CheckOff,
                ),
                (
                    "macos-switch-track-activation-switch",
                    PaintRole::SwitchTrack { on: true },
                ),
                (
                    "macos-switch-track-activation-off-switch",
                    PaintRole::SwitchTrack { on: false },
                ),
                ("macos-slider-selection", PaintRole::Slider),
                ("activation-toggle", PaintRole::ToggleOn),
                ("activation-tab", PaintRole::Segment { selected: true }),
            ] {
                let fill = painted_background(cx, selector);
                assert_eq!(fill, expect(role, Part::Fill, inactive), "{selector}");
                assert_eq!(fill.s, 0., "{selector} becomes neutral on blur");
            }
            assert_eq!(
                painted_background(cx, "activation-primary"),
                theme.bezel_fill(),
                "an inactive prominent button is a Bezel"
            );
            assert!((on_check(inactive).a - 0.141).abs() < 1e-6);
            assert_eq!(
                painted_background(cx, "activation-toggle"),
                ink(&theme).opacity(0.125)
            );
            assert_eq!(
                painted_background(cx, "activation-disabled-toggle"),
                ink(&theme).opacity(0.067),
                "inactive disabled toggle-on is @0.067, not IN x AD"
            );
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-switch"),
                if theme.is_dark() {
                    Hsla::from(gpui_kit::rgb(0xdcdcdc)).opacity(0.086)
                } else {
                    Hsla::black().opacity(0.075)
                },
                "inactive disabled on-track (spec 6.2.7)"
            );
            assert_eq!(
                painted_background(cx, "activation-disabled"),
                crate::tokens::disabled_bezel(&theme).0,
                "an inactive disabled prominent button uses Bezel disabled"
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled"),
                bezel_disabled_label,
                "inactive and disabled prominent label uses Bezel disabled ink"
            );
            assert_eq!(
                painted_label_color(cx, "activation-borderless"),
                theme.secondary,
                "borderless is unchanged in an inactive window"
            );
            assert_eq!(
                painted_background(cx, "activation-restyled"),
                restyled_fill()
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled-checkbox"),
                theme.tertiary_label()
            );
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-off-switch"),
                expected_off
            );
            assert_eq!(
                painted_radio_mark(cx),
                theme.foreground,
                "inactive radio uses the label ink mark"
            );
            assert_eq!(
                painted_background(cx, "macos-switch-thumb-activation-switch"),
                expect(PaintRole::SwitchKnob { on: true }, Part::Fill, inactive)
            );
            assert_eq!(
                painted_background(cx, "macos-slider-thumb-end"),
                expect(PaintRole::Slider, Part::Knob, inactive)
            );
            cx.update(|window, _| window.activate_window());
            cx.run_until_parked();
            cx.simulate_mouse_move(
                gpui_kit::point(gpui_kit::px(500.), gpui_kit::px(500.)),
                None,
                Default::default(),
            );
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                painted_background(cx, "activation-primary"),
                theme.accent,
                "reactivation restores primary color without reconstruction"
            );
        }
    }

    fn custom_ink() -> Hsla {
        gpui_kit::rgb(0xaf52de).into()
    }

    struct StateHarness;

    impl Render for StateHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(rems(0.5))
                // Final values set after construction.
                .child(
                    toggle("late-toggle", "Bold", false, cx)
                        .pressed(true)
                        .debug_selector(|| "late-toggle".into()),
                )
                .child(
                    toggle("off-toggle", "Italic", false, cx)
                        .debug_selector(|| "off-toggle".into()),
                )
                .child(
                    toggle("disabled-off-toggle", "Underline", false, cx)
                        .disabled(true)
                        .debug_selector(|| "disabled-off-toggle".into()),
                )
                .child(
                    toggle("styled-toggle", "Strike", true, cx)
                        .text_color(custom_ink())
                        .underline()
                        .debug_selector(|| "styled-toggle".into()),
                )
                .child(
                    toggle("filled-toggle", "Code", true, cx)
                        .bg(custom_ink())
                        .debug_selector(|| "filled-toggle".into()),
                )
                .child(
                    tabs("state-tabs", cx)
                        .child(
                            tab("late-tab", "General", false, cx)
                                .selected(true)
                                .debug_selector(|| "late-tab".into()),
                        )
                        .child(
                            tab("disabled-tab", "Network", true, cx)
                                .disabled(true)
                                .underline()
                                // An extra child inherits the resolved label ink.
                                .child("⌘2")
                                .debug_selector(|| "disabled-tab".into()),
                        )
                        .child(
                            tab("plain-tab", "Advanced", false, cx)
                                .disabled(true)
                                .debug_selector(|| "plain-tab".into()),
                        ),
                )
                .child(
                    button("label-only", "Custom", ButtonVariant::Primary, cx)
                        .text_color(custom_ink())
                        .underline()
                        .debug_selector(|| "label-only".into()),
                )
                .child(
                    button("label-only-disabled", "Custom", ButtonVariant::Primary, cx)
                        .text_color(custom_ink())
                        .disabled(true)
                        .underline()
                        .debug_selector(|| "label-only-disabled".into()),
                )
                .child(
                    checkbox("press-checkbox", "Remember me", CheckboxState::Checked, cx)
                        .debug_selector(|| "press-checkbox".into()),
                )
                .child(
                    switch("press-switch", "Wi-Fi", false, cx)
                        .debug_selector(|| "press-switch".into()),
                )
                .child(
                    radio("press-radio", "Automatic", true, cx)
                        .debug_selector(|| "press-radio".into()),
                )
                // A caller group and a nested control keep their own presses.
                .child(
                    checkbox("group-checkbox", "Grouped", CheckboxState::Checked, cx)
                        .group("caller-row")
                        .child(
                            radio("nested-radio", "Nested", false, cx)
                                .debug_selector(|| "nested-radio".into()),
                        )
                        .debug_selector(|| "group-checkbox".into()),
                )
                // Caller interaction refinements replace the defaults.
                .child(
                    toggle("active-cleared", "Plain", false, cx)
                        .active(|s| s)
                        .debug_selector(|| "active-cleared".into()),
                )
                .child(
                    toggle("hover-custom", "Tinted", false, cx)
                        .hover(|s| s.bg(custom_ink()))
                        .debug_selector(|| "hover-custom".into()),
                )
        }
    }

    /// Every solid fill painted exactly at `selector`'s bounds.
    fn fills_at(cx: &mut VisualTestContext, selector: &'static str) -> Vec<Hsla> {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.update(|window, _| {
            let bounds = bounds.scale(window.scale_factor());
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.bounds == bounds)
                .filter_map(|quad| quad.background.as_solid())
                .filter(|fill| fill.a > 0.)
                .collect()
        })
    }

    #[gpui_kit::test]
    fn toggles_tabs_and_buttons_resolve_final_state_beneath_caller_styles(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, _| StateHarness);
        let away = gpui_kit::point(gpui_kit::px(700.), gpui_kit::px(700.));
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            cx.simulate_mouse_move(away, None, Default::default());
            let redraw = |cx: &mut VisualTestContext| {
                view.update(cx, |_, cx| cx.notify());
                cx.update(|window, cx| window.draw(cx).clear(cx));
            };
            redraw(cx);
            let rest = States::active();
            let expect = |role, part, states| paint(role, part, states, &theme);
            assert_eq!(
                fills_at(cx, "late-toggle"),
                [expect(PaintRole::ToggleOn, Part::Fill, rest)],
                "a later .pressed(true) paints ToggleOn once"
            );
            assert_eq!(
                fills_at(cx, "late-tab"),
                [expect(
                    PaintRole::Segment { selected: true },
                    Part::Fill,
                    rest
                )]
            );
            // Disabled selected tab and its extra child share the label ink.
            assert_eq!(
                fills_at(cx, "disabled-tab"),
                [theme.indicator_accent().opacity(0.71)]
            );
            assert_eq!(
                painted_label_color(cx, "disabled-tab"),
                expect(
                    PaintRole::Segment { selected: true },
                    Part::Label,
                    rest.disabled(true)
                )
            );
            assert!(
                fills_at(cx, "plain-tab").is_empty(),
                "no fill when unselected"
            );
            // Caller styles win over the defaults.
            assert_eq!(painted_label_color(cx, "styled-toggle"), custom_ink());
            assert_eq!(fills_at(cx, "styled-toggle"), [theme.indicator_accent()]);
            assert_eq!(fills_at(cx, "filled-toggle"), [custom_ink()]);
            // D6: a label-only customization keeps the role's fill.
            assert_eq!(fills_at(cx, "label-only"), [theme.accent]);
            assert_eq!(painted_label_color(cx, "label-only"), custom_ink());
            assert_eq!(
                fills_at(cx, "label-only-disabled"),
                [theme.accent.opacity(0.69)]
            );
            assert_eq!(painted_label_color(cx, "label-only-disabled"), custom_ink());

            // No hover (spec §6.5, C17): the rest fill stays under the pointer.
            for (selector, expected) in [
                ("off-toggle", theme.bezel_fill()),
                ("plain-tab", theme.foreground.opacity(0.)),
                ("disabled-off-toggle", ink(&theme).opacity(0.039)),
            ] {
                let center = cx.debug_bounds(selector).unwrap().center();
                cx.simulate_mouse_move(center, None, Default::default());
                redraw(cx);
                let fills = fills_at(cx, selector);
                if expected.a == 0. {
                    assert!(fills.is_empty(), "{selector} hovered: {fills:?}");
                } else {
                    assert_eq!(fills, [expected], "{selector} hovered");
                }
            }

            // Pressing a toggle paints its pressed arm (spec §6.2.4 P).
            let center = cx.debug_bounds("late-toggle").unwrap().center();
            cx.simulate_mouse_move(center, None, Default::default());
            cx.simulate_mouse_down(center, gpui_kit::MouseButton::Left, Default::default());
            redraw(cx);
            assert_eq!(
                fills_at(cx, "late-toggle"),
                [expect(PaintRole::ToggleOn, Part::Fill, rest.pressed(true))]
            );
            cx.simulate_mouse_up(center, gpui_kit::MouseButton::Left, Default::default());
            cx.simulate_mouse_move(away, None, Default::default());
            redraw(cx);

            // Caller paint and interaction refinements win over the defaults.
            let press = |cx: &mut VisualTestContext, selector: &'static str| {
                let center = cx.debug_bounds(selector).unwrap().center();
                cx.simulate_mouse_move(center, None, Default::default());
                cx.simulate_mouse_down(center, gpui_kit::MouseButton::Left, Default::default());
                center
            };
            let release = |cx: &mut VisualTestContext, at| {
                cx.simulate_mouse_up(at, gpui_kit::MouseButton::Left, Default::default());
                cx.simulate_mouse_move(away, None, Default::default());
            };
            let at = press(cx, "styled-toggle");
            redraw(cx);
            assert_eq!(
                painted_label_color(cx, "styled-toggle"),
                custom_ink(),
                "a caller label colour persists while pressed"
            );
            release(cx, at);
            let at = press(cx, "active-cleared");
            redraw(cx);
            assert_eq!(
                fills_at(cx, "active-cleared"),
                // Only the rest default remains: no pressed @0.165.
                [theme.bezel_fill()],
                ".active(|s| s) clears the default pressed paint"
            );
            release(cx, at);
            let center = cx.debug_bounds("hover-custom").unwrap().center();
            cx.simulate_mouse_move(center, None, Default::default());
            redraw(cx);
            assert_eq!(
                fills_at(cx, "hover-custom"),
                [custom_ink()],
                "a caller hover replaces the default hover"
            );
            cx.simulate_mouse_move(away, None, Default::default());
            redraw(cx);

            // Pressing the label presses the indicator (spec §6.2.6, §6.2.7).
            for (selector, indicator, role) in [
                (
                    "press-checkbox",
                    "macos-checkbox-indicator-press-checkbox",
                    PaintRole::CheckOn,
                ),
                (
                    "press-switch",
                    "macos-switch-track-press-switch",
                    PaintRole::SwitchTrack { on: false },
                ),
                (
                    "press-radio",
                    "macos-radio-indicator-press-radio",
                    PaintRole::CheckOn,
                ),
            ] {
                let control = cx.debug_bounds(selector).unwrap();
                let label = gpui_kit::point(control.right() - gpui_kit::px(4.), control.center().y);
                cx.simulate_mouse_move(label, None, Default::default());
                cx.simulate_mouse_down(label, gpui_kit::MouseButton::Left, Default::default());
                redraw(cx);
                assert_eq!(
                    fills_at(cx, indicator).last().copied(),
                    Some(expect(role, Part::Fill, rest.pressed(true))),
                    "{selector} pressed"
                );
                cx.simulate_mouse_up(label, gpui_kit::MouseButton::Left, Default::default());
                cx.simulate_mouse_move(away, None, Default::default());
                redraw(cx);
            }

            // A caller group keeps the press; a nested control is separate.
            let outer = cx.debug_bounds("group-checkbox").unwrap();
            let nested = cx.debug_bounds("nested-radio").unwrap();
            let outer_label = gpui_kit::point(nested.left() - gpui_kit::px(4.), outer.center().y);
            for (at, outer_pressed, nested_pressed) in
                [(outer_label, true, false), (nested.center(), true, true)]
            {
                cx.simulate_mouse_move(at, None, Default::default());
                cx.simulate_mouse_down(at, gpui_kit::MouseButton::Left, Default::default());
                redraw(cx);
                assert_eq!(
                    fills_at(cx, "macos-checkbox-indicator-group-checkbox"),
                    [expect(
                        PaintRole::CheckOn,
                        Part::Fill,
                        rest.pressed(outer_pressed)
                    )]
                );
                assert_eq!(
                    fills_at(cx, "macos-radio-indicator-nested-radio"),
                    [expect(
                        PaintRole::CheckOff,
                        Part::Fill,
                        rest.pressed(nested_pressed)
                    )]
                );
                cx.simulate_mouse_up(at, gpui_kit::MouseButton::Left, Default::default());
                cx.simulate_mouse_move(away, None, Default::default());
                redraw(cx);
            }
            // Only the primary button presses (spec §6.6).
            let label = gpui_kit::point(
                cx.debug_bounds("press-checkbox").unwrap().right() - gpui_kit::px(4.),
                cx.debug_bounds("press-checkbox").unwrap().center().y,
            );
            cx.simulate_mouse_move(label, None, Default::default());
            cx.simulate_mouse_down(label, gpui_kit::MouseButton::Right, Default::default());
            redraw(cx);
            assert_eq!(
                fills_at(cx, "macos-checkbox-indicator-press-checkbox"),
                [expect(PaintRole::CheckOn, Part::Fill, rest)]
            );
            cx.simulate_mouse_up(label, gpui_kit::MouseButton::Right, Default::default());
            cx.simulate_mouse_move(away, None, Default::default());
            redraw(cx);

            // Inactive: the final value still selects the role.
            cx.deactivate_window();
            redraw(cx);
            let inactive = rest.window_active(false);
            assert_eq!(
                fills_at(cx, "late-toggle"),
                [expect(PaintRole::ToggleOn, Part::Fill, inactive)]
            );
            assert_eq!(fills_at(cx, "label-only"), [theme.bezel_fill()]);
            assert_eq!(painted_label_color(cx, "label-only"), custom_ink());
        }
    }

    #[gpui_kit::test]
    fn toggle_and_tab_geometry_scales_with_zoom(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| StateHarness);
        for zoom in [1., 2.] {
            cx.update(|window, cx| {
                window.set_rem_size(gpui_kit::px(16. * zoom));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let toggle = cx.debug_bounds("late-toggle").unwrap();
            let tab = cx.debug_bounds("late-tab").unwrap();
            assert_eq!(
                toggle.size.height,
                gpui_kit::px(24. * zoom),
                "toggle at {zoom}"
            );
            assert_eq!(tab.size.height, gpui_kit::px(22. * zoom), "tab at {zoom}");
            // The bezel paint covers the whole border box.
            cx.update(|window, _| {
                for bounds in [toggle, tab] {
                    let bounds = bounds.scale(window.scale_factor());
                    assert!(window.painted_quads().iter().any(|quad| {
                        quad.bounds == bounds && quad.background.as_solid().is_some()
                    }));
                }
            });
        }
    }

    #[gpui_kit::test]
    fn borderless_label_darkens_while_pressed_without_a_bezel(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        struct Borderless;
        impl Render for Borderless {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().size(rems(25.)).child(
                    button("borderless", "Details", ButtonVariant::Borderless, cx)
                        .underline()
                        .debug_selector(|| "borderless".into()),
                )
            }
        }
        let (_, cx) = cx.add_window_view(|_, _| Borderless);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(painted_label_color(cx, "borderless"), theme.secondary);
            let bounds = cx.debug_bounds("borderless").unwrap();
            assert_eq!(
                bounds.size.height,
                gpui_kit::px(16.),
                "a borderless button is its Body label"
            );
            cx.update(|window, _| {
                let bounds = bounds.scale(window.scale_factor());
                assert!(
                    window
                        .painted_quads()
                        .iter()
                        .filter(|quad| quad.bounds == bounds)
                        .all(|quad| quad.background.as_solid().is_none_or(|fill| fill.a == 0.)),
                    "no bezel is painted"
                );
            });
            let center = bounds.center();
            cx.simulate_mouse_move(center, None, Default::default());
            cx.simulate_mouse_down(center, gpui_kit::MouseButton::Left, Default::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                painted_label_color(cx, "borderless"),
                ink(&theme).opacity(0.85),
                "pressed borderless label (spec 6.6)"
            );
            cx.simulate_mouse_up(center, gpui_kit::MouseButton::Left, Default::default());
            cx.simulate_mouse_move(
                gpui_kit::point(gpui_kit::px(390.), gpui_kit::px(390.)),
                None,
                Default::default(),
            );
        }
        // The frame is the label at every zoom (SwiftUI regular frame 41 x 16).
        cx.update(|window, cx| {
            window.set_rem_size(gpui_kit::px(32.));
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            cx.debug_bounds("borderless").unwrap().size.height,
            gpui_kit::px(32.)
        );
    }

    #[gpui_kit::test]
    fn inactive_checkbox_stays_interactive_and_preserves_its_controlled_state(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|_, cx| ActivationHarness {
            slider: cx.new(|_| gpui_kit::base::slider::SliderState::new()),
            checked: CheckboxState::Checked,
            checkbox_changes: 0,
        });
        cx.deactivate_window();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let control = cx.debug_bounds("activation-checkbox").unwrap();
        cx.simulate_click(control.center(), Default::default());
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).checked, CheckboxState::Unchecked);
            assert_eq!(
                view.read(cx).checkbox_changes,
                1,
                "a gray inactive control is still enabled"
            );
        });
    }

    struct GeometryHarness {
        input: Entity<gpui_kit::base::input::InputState>,
        choice: Entity<crate::ChoiceState>,
        slider: Entity<gpui_kit::base::slider::SliderState>,
    }

    impl Render for GeometryHarness {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            crate::focus_scope("measured-native-controls")
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(rems(0.5))
                .child(
                    button("measured-button", "Continue", ButtonVariant::Primary, cx)
                        .debug_selector(|| "measured-button".into()),
                )
                .child(
                    checkbox(
                        "measured-checkbox",
                        "Remember me",
                        CheckboxState::Checked,
                        cx,
                    )
                    .debug_selector(|| "measured-checkbox".into()),
                )
                .child(
                    radio("measured-radio", "Automatic", true, cx)
                        .debug_selector(|| "measured-radio".into()),
                )
                .child(switch("measured-switch", "Wi-Fi", false, cx))
                .child(
                    crate::input("measured-input", &self.input, window, cx)
                        .w(rems(12.))
                        .debug_selector(|| "measured-input".into()),
                )
                .child(
                    crate::button_group(
                        "measured-segments",
                        vec![
                            crate::ChoiceItem::new("general", "General"),
                            crate::ChoiceItem::new("network", "Network"),
                        ],
                        Some(0),
                        |_, _, _| {},
                        window,
                        cx,
                    )
                    .w(rems(12.))
                    .debug_selector(|| "measured-segments".into()),
                )
                .child(
                    div()
                        .w(rems(12.))
                        .debug_selector(|| "measured-popup".into())
                        .child(crate::select("measured-popup", &self.choice, window, cx)),
                )
                .child(
                    div()
                        .w(rems(12.))
                        .debug_selector(|| "measured-slider".into())
                        .child(crate::slider(&self.slider, false, window, cx)),
                )
        }
    }

    struct MetricsHarness {
        clicks: usize,
    }

    impl Render for MetricsHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    // A toolbar-style row: a text button beside an icon button.
                    div()
                        .flex()
                        .items_center()
                        .gap(rems(0.5))
                        .child(
                            button("metric-text", "Add", ButtonVariant::Outline, cx)
                                .debug_selector(|| "metric-text".into()),
                        )
                        .child(
                            div()
                                .flex()
                                .debug_selector(|| "metric-icon-frame".into())
                                .child(
                                    icon_button(
                                        "metric-icon",
                                        IconName::Plus,
                                        "Add",
                                        ButtonVariant::Outline,
                                        cx,
                                    )
                                    .debug_selector(|| "metric-icon".into())
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.clicks += 1;
                                            cx.notify();
                                        },
                                    )),
                                ),
                        ),
                )
                .child(
                    checkbox("metric-checkbox", "Remember me", CheckboxState::Checked, cx)
                        .debug_selector(|| "metric-checkbox".into()),
                )
                // Caller geometry wins over the snapped defaults.
                .child(
                    div()
                        .flex()
                        .debug_selector(|| "sized-icon-frame".into())
                        .child(
                            icon_button(
                                "sized-icon",
                                IconName::Plus,
                                "Add",
                                ButtonVariant::Outline,
                                cx,
                            )
                            .size(gpui_kit::px(32.))
                            .ml(gpui_kit::px(8.))
                            .debug_selector(|| "sized-icon".into()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .debug_selector(|| "margin-icon-frame".into())
                        .child(
                            icon_button(
                                "margin-icon",
                                IconName::Plus,
                                "Add",
                                ButtonVariant::Outline,
                                cx,
                            )
                            .ml(gpui_kit::px(8.))
                            .debug_selector(|| "margin-icon".into()),
                        ),
                )
        }
    }

    /// A point length on the device grid, rounded half toward zero as GPUI
    /// layout does, in logical pixels.
    fn device(points: f32, zoom: f32, scale: f32) -> f32 {
        ((points * zoom * scale).abs() - 0.5).ceil() / scale
    }

    /// Spec §4.2 (C7) and §4.4 (C2, C9) on 1x and 2x displays at 100%, 125%
    /// and 200% zoom.
    #[gpui_kit::test]
    fn icon_button_and_checkbox_row_follow_native_metrics(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| MetricsHarness { clicks: 0 });
        cx.simulate_mouse_move(
            gpui_kit::point(gpui_kit::px(600.), gpui_kit::px(600.)),
            None,
            Default::default(),
        );
        let near = |actual: gpui_kit::Pixels, expected: f32, what: &str| {
            assert!(
                (actual.as_f32() - expected).abs() <= 0.5,
                "{what}: {actual:?} vs {expected}"
            );
        };
        for scale in [1., 2.] {
            cx.simulate_scale_factor_change(scale);
            for zoom in [1., 1.25, 2.] {
                let at = format!("{scale}x {zoom}");
                cx.update(|window, cx| {
                    window.set_rem_size(gpui_kit::px(16. * zoom));
                    view.update(cx, |_, cx| cx.notify());
                    window.draw(cx).clear(cx);
                });
                // The layout frame is exactly 39 × 21 on the device grid; the
                // margins absorb the bezel's rounding.
                let frame = cx.debug_bounds("metric-icon-frame").unwrap();
                let bezel = cx.debug_bounds("metric-icon").unwrap();
                assert_eq!(frame.size.width.as_f32(), device(39., zoom, scale), "{at}");
                assert_eq!(frame.size.height.as_f32(), device(21., zoom, scale), "{at}");
                assert_eq!(bezel.size.width.as_f32(), device(37.5, zoom, scale), "{at}");
                assert_eq!(
                    bezel.size.height.as_f32(),
                    device(20.5, zoom, scale),
                    "{at}"
                );
                let margins = [
                    ("left", bezel.left() - frame.left(), 0.75),
                    ("right", frame.right() - bezel.right(), 0.75),
                    ("top", bezel.top() - frame.top(), 0.25),
                    ("bottom", frame.bottom() - bezel.bottom(), 0.25),
                ];
                // Each margin is within one device pixel of its nominal value.
                for (edge, margin, nominal) in margins {
                    assert!(
                        margin >= gpui_kit::px(0.)
                            && (margin.as_f32() - nominal * zoom).abs() <= 1. / scale,
                        "{at} {edge} margin {margin:?} vs {}",
                        nominal * zoom
                    );
                }
                // Text and icon buttons share a centerline in a row.
                let text = cx.debug_bounds("metric-text").unwrap();
                near(
                    bezel.center().y,
                    text.center().y.as_f32(),
                    &format!("{at} row alignment"),
                );
                assert_eq!(
                    painted_background(cx, "metric-icon"),
                    cx.update(|_, cx| cx.macos().bezel_fill()),
                    "the bezel fills exactly the hit bounds"
                );
                // A press in any margin misses; just inside each bezel edge
                // activates. The hit area is the visible bezel.
                let pixel = gpui_kit::px(1. / scale);
                let (x, y) = (bezel.center().x, bezel.center().y);
                for (edge, margin, _) in margins {
                    if margin < pixel {
                        continue;
                    }
                    let half = margin / 2.;
                    let point = match edge {
                        "left" => gpui_kit::point(frame.left() + half, y),
                        "right" => gpui_kit::point(frame.right() - half, y),
                        "top" => gpui_kit::point(x, frame.top() + half),
                        _ => gpui_kit::point(x, frame.bottom() - half),
                    };
                    let before = cx.update(|_, cx| view.read(cx).clicks);
                    cx.simulate_click(point, Default::default());
                    cx.update(|window, cx| window.draw(cx).clear(cx));
                    assert_eq!(
                        cx.update(|_, cx| view.read(cx).clicks),
                        before,
                        "{at} press in the {edge} margin"
                    );
                }
                for point in [
                    gpui_kit::point(bezel.left() + pixel / 2., y),
                    gpui_kit::point(bezel.right() - pixel / 2., y),
                    gpui_kit::point(x, bezel.top() + pixel / 2.),
                    gpui_kit::point(x, bezel.bottom() - pixel / 2.),
                ] {
                    let before = cx.update(|_, cx| view.read(cx).clicks);
                    cx.simulate_click(point, Default::default());
                    cx.update(|window, cx| window.draw(cx).clear(cx));
                    assert_eq!(
                        cx.update(|_, cx| view.read(cx).clicks),
                        before + 1,
                        "{at} press at {point:?} inside the bezel"
                    );
                }

                // A caller size and margin survive rendering; an axis the
                // caller left alone is still snapped.
                let sized = cx.debug_bounds("sized-icon").unwrap();
                let sized_frame = cx.debug_bounds("sized-icon-frame").unwrap();
                assert_eq!(
                    sized.size,
                    gpui_kit::size(gpui_kit::px(32.), gpui_kit::px(32.))
                );
                assert_eq!(sized.left() - sized_frame.left(), gpui_kit::px(8.), "{at}");
                let margin = cx.debug_bounds("margin-icon").unwrap();
                let margin_frame = cx.debug_bounds("margin-icon-frame").unwrap();
                assert_eq!(
                    margin.left() - margin_frame.left(),
                    gpui_kit::px(8.),
                    "{at}"
                );
                assert_eq!(
                    margin_frame.size.height.as_f32(),
                    device(21., zoom, scale),
                    "{at}"
                );
                assert_eq!(
                    margin.size.height.as_f32(),
                    device(20.5, zoom, scale),
                    "{at}"
                );

                // Labeled checkbox: 17 pt row, 16 pt box at y 0.5, radius 5.5.
                let row = cx.debug_bounds("metric-checkbox").unwrap();
                let indicator = cx
                    .debug_bounds("macos-checkbox-indicator-metric-checkbox")
                    .unwrap();
                near(row.size.height, 17. * zoom, &format!("{at} checkbox row"));
                near(indicator.size.width, 16. * zoom, "checkbox box width");
                near(indicator.size.height, 16. * zoom, "checkbox box height");
                // The 0.5 pt offset snaps to the device grid.
                let offset = (indicator.top() - row.top()).as_f32();
                assert!(
                    (offset - 0.5 * zoom).abs() <= 1. / scale,
                    "{at} checkbox box y {offset}"
                );
                near(indicator.left() - row.left(), 0., "checkbox box x");
                let radius = cx.update(|window, _| {
                    let scaled = indicator.scale(window.scale_factor());
                    let quad = window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| quad.bounds == scaled)
                        .expect("checkbox well");
                    quad.corner_radii.top_left.0 / window.scale_factor()
                });
                near(gpui_kit::px(radius), 5.5 * zoom, "checkbox radius");
            }
        }
    }

    /// Preserve the actual AppKit baseline while independently scaling GPUI.
    /// Native intrinsic widths depend on the platform font and are intentionally
    /// left flexible; intrinsic heights and switch geometry are authoritative.
    #[gpui_kit::test]
    fn native_intrinsic_geometry_matches_appkit_at_each_zoom(cx: &mut TestAppContext) {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../docs/native-metrics.json")).unwrap();
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| GeometryHarness {
            input: cx.new(|cx| {
                gpui_kit::base::input::InputState::new(window, cx).default_value("Taylor")
            }),
            choice: cx.new(|cx| {
                crate::ChoiceState::new(
                    vec![crate::ChoiceItem::new("standard", "Standard")],
                    window,
                    cx,
                )
                .default_selected(0)
            }),
            slider: cx.new(|_| {
                gpui_kit::base::slider::SliderState::new()
                    .min(0.)
                    .max(100.)
                    .default_value(65.)
            }),
        });
        for (appearance, theme) in [
            ("light", crate::Theme::light()),
            ("dark", crate::Theme::dark()),
        ] {
            for zoom in [1., 1.25, 2.] {
                cx.update(|window, cx| {
                    theme.clone().apply(cx);
                    window.set_rem_size(gpui_kit::px(16. * zoom));
                    view.update(cx, |_, cx| cx.notify());
                    window.draw(cx).clear(cx);
                });
                for (selector, native) in [
                    ("measured-button", "button_primary"),
                    ("measured-checkbox", "checkbox"),
                    ("measured-radio", "radio"),
                    ("macos-switch-track-measured-switch", "switch_off"),
                    ("measured-input", "text_field"),
                    ("measured-segments", "segmented"),
                    ("measured-popup", "popup"),
                    ("measured-slider", "slider"),
                ] {
                    // The labeled checkbox row follows SwiftUI's 17 pt over
                    // AppKit's 16 (spec §4.4, C2).
                    let native_height = if native == "checkbox" {
                        crate::tokens::CHECKBOX_ROW_HEIGHT
                    } else {
                        reference[appearance]["controls"][native]["intrinsic_height"]
                            .as_f64()
                            .unwrap() as f32
                    };
                    let expected = native_height * zoom;
                    let actual = cx.debug_bounds(selector).unwrap().size.height.as_f32();
                    // ±0.5 pt (plan Verification): layout snaps a fractional
                    // size such as 17 × 1.25 to whole device pixels.
                    assert!(
                        (actual - expected).abs() <= 0.5,
                        "{appearance} {zoom} {selector}: {actual} vs native {expected}"
                    );
                }
                let track = cx
                    .debug_bounds("macos-switch-track-measured-switch")
                    .unwrap();
                let native_width =
                    reference[appearance]["controls"]["switch_off"]["intrinsic_width"]
                        .as_f64()
                        .unwrap() as f32;
                assert_eq!(track.size.width, gpui_kit::px(native_width * zoom));
                let thumb = cx
                    .debug_bounds("macos-switch-thumb-measured-switch")
                    .unwrap();
                assert_eq!(
                    thumb.size,
                    gpui_kit::size(gpui_kit::px(32. * zoom), gpui_kit::px(20. * zoom))
                );
                assert_eq!(thumb.origin.x - track.origin.x, gpui_kit::px(2. * zoom));
                assert_eq!(thumb.origin.y - track.origin.y, gpui_kit::px(2. * zoom));
            }
        }
    }

    struct Harness {
        checkbox: CheckboxState,
        radio: bool,
        switch: bool,
        disabled: bool,
        changes: [usize; 3],
        focus: [FocusHandle; 3],
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
            let checkbox_target = cx.entity();
            let radio_target = cx.entity();
            let switch_target = cx.entity();
            crate::focus_scope("native-controls")
                .size_full()
                .flex()
                .flex_col()
                .gap(rems(0.5))
                .child(
                    checkbox("remember", "Remember me", self.checkbox, cx)
                        .debug_selector(|| "native-checkbox".into())
                        .track_focus(&self.focus[0])
                        .disabled(self.disabled)
                        .on_change(move |value, _, _, cx| {
                            checkbox_target.update(cx, |this, cx| {
                                this.checkbox = value;
                                this.changes[0] += 1;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    radio("automatic", "Automatic", self.radio, cx)
                        .debug_selector(|| "native-radio".into())
                        .track_focus(&self.focus[1])
                        .disabled(self.disabled)
                        .on_change(move |value, _, _, cx| {
                            radio_target.update(cx, |this, cx| {
                                this.radio = value;
                                this.changes[1] += 1;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    switch("wireless", "Wi-Fi", self.switch, cx)
                        .debug_selector(|| "native-switch".into())
                        .track_focus(&self.focus[2])
                        .disabled(self.disabled)
                        .on_change(move |value, _, _, cx| {
                            switch_target.update(cx, |this, cx| {
                                this.switch = value;
                                this.changes[2] += 1;
                                cx.notify();
                            });
                        }),
                )
        }
    }

    fn harness(cx: &mut TestAppContext) -> (Entity<Harness>, &mut VisualTestContext) {
        cx.update(crate::init);
        cx.add_window_view(|_, cx| Harness {
            checkbox: CheckboxState::Indeterminate,
            radio: false,
            switch: false,
            disabled: false,
            changes: [0; 3],
            focus: std::array::from_fn(|_| cx.focus_handle()),
        })
    }

    fn click(cx: &mut VisualTestContext, selector: &'static str) {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let point = cx.debug_bounds(selector).unwrap().center();
        cx.simulate_click(point, Default::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn press(cx: &mut VisualTestContext, key: &str) {
        let keystroke = gpui_kit::Keystroke::parse(key).unwrap();
        cx.simulate_event(gpui_kit::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }

    #[gpui_kit::test]
    fn native_controls_preserve_controlled_pointer_and_keyboard_activation(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = harness(cx);
        click(cx, "native-checkbox");
        cx.update(|window, cx| {
            assert_eq!(view.read(cx).checkbox, CheckboxState::Checked);
            assert!(view.read(cx).focus[0].is_focused(window));
        });
        press(cx, "space");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(view.read(cx).checkbox, CheckboxState::Unchecked);
        });

        click(cx, "native-radio");
        press(cx, "space");
        cx.update(|_, cx| {
            assert!(view.read(cx).radio);
            assert_eq!(
                view.read(cx).changes[1],
                1,
                "a radio cannot deselect itself"
            );
        });
        cx.update(|window, cx| {
            view.update(cx, |this, cx| {
                this.radio = false;
                cx.notify();
            });
            window.draw(cx).clear(cx);
        });
        press(cx, "enter");
        cx.update(|_, cx| assert!(view.read(cx).radio));

        click(cx, "native-switch");
        cx.update(|_, cx| assert!(view.read(cx).switch));
        press(cx, "space");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(!view.read(cx).switch);
        });
        press(cx, "enter");
        cx.update(|_, cx| {
            assert!(view.read(cx).switch);
            assert_eq!(view.read(cx).changes, [2, 2, 3]);
        });
    }

    #[gpui_kit::test]
    fn clicking_the_indicator_itself_activates_and_focuses_the_control(cx: &mut TestAppContext) {
        let (view, cx) = harness(cx);
        for (index, indicator) in [
            "macos-checkbox-indicator-remember",
            "macos-radio-indicator-automatic",
            "macos-switch-track-wireless",
        ]
        .into_iter()
        .enumerate()
        {
            click(cx, indicator);
            cx.update(|window, cx| {
                let state = view.read(cx);
                assert_eq!(state.changes[index], 1, "{indicator} activates its control");
                assert!(state.focus[index].is_focused(window), "{indicator} focuses");
            });
        }
    }

    #[gpui_kit::test]
    fn disabled_native_controls_reject_pointer_and_keyboard_changes(cx: &mut TestAppContext) {
        let (view, cx) = harness(cx);
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        for (index, selector) in ["native-checkbox", "native-radio", "native-switch"]
            .into_iter()
            .enumerate()
        {
            click(cx, selector);
            cx.update(|window, cx| {
                let focus = view.read(cx).focus[index].clone();
                focus.focus(window, cx);
            });
            press(cx, "enter");
            press(cx, "space");
        }
        cx.update(|_, cx| {
            let state = view.read(cx);
            assert_eq!(state.checkbox, CheckboxState::Indeterminate);
            assert!(!state.radio && !state.switch);
            assert_eq!(state.changes, [0; 3]);
        });
    }

    struct RingHarness {
        focus: Vec<FocusHandle>,
        field: Entity<gpui_kit::base::input::InputState>,
    }

    impl Render for RingHarness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(rems(1.))
                .p(rems(1.))
                .child(
                    checkbox("ring-checkbox", "Remember", CheckboxState::Unchecked, cx)
                        .track_focus(&self.focus[0]),
                )
                .child(radio("ring-radio", "Automatic", true, cx).track_focus(&self.focus[1]))
                .child(switch("ring-switch", "Wi-Fi", true, cx).track_focus(&self.focus[2]))
                .child(
                    button("ring-button", "Apply", ButtonVariant::Secondary, cx)
                        .track_focus(&self.focus[3])
                        .debug_selector(|| "ring-button".into()),
                )
                .child(
                    toggle("ring-toggle", "Bold", false, cx)
                        .track_focus(&self.focus[4])
                        .debug_selector(|| "ring-toggle".into()),
                )
                .child(
                    tabs("ring-tabs", cx).child(
                        tab("ring-tab", "General", true, cx)
                            .track_focus(&self.focus[5])
                            .debug_selector(|| "ring-tab".into()),
                    ),
                )
                .child(
                    crate::accordion_trigger("ring-accordion", "Details", false, cx)
                        .track_focus(&self.focus[6])
                        .w(rems(12.))
                        .debug_selector(|| "ring-accordion".into()),
                )
                .child(
                    crate::input("ring-field", &self.field, window, cx)
                        .w(rems(10.))
                        .debug_selector(|| "ring-field".into()),
                )
        }
    }

    fn rings(cx: &mut VisualTestContext) -> Vec<gpui_kit::Quad> {
        cx.update(|window, cx| crate::state::painted_rings(window, cx.macos()))
    }

    /// Spec §6.3 (C15, D4): one 3 pt ring outset 3 pt around the box, circle,
    /// track or whole control, corner radius + 3, at 100% and 200% zoom;
    /// keyboard focus in an active window only (fields: any focus), and no
    /// accent border.
    #[gpui_kit::test]
    fn focus_rings_surround_the_native_scope_for_keyboard_focus_only(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(|window, cx| RingHarness {
            focus: (0..7).map(|_| cx.focus_handle()).collect(),
            field: cx.new(|cx| gpui_kit::base::input::InputState::new(window, cx)),
        });
        let redraw = |cx: &mut VisualTestContext| {
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
        };
        for (theme, zoom) in [
            (crate::Theme::light(), 1.),
            (crate::Theme::dark(), 1.),
            (crate::Theme::light(), 2.),
        ] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.set_rem_size(gpui_kit::px(16. * zoom));
                window.activate_window();
            });
            cx.run_until_parked();
            // (focus index, ringed element, its size in points, radius in points)
            for (index, selector, size, radius) in [
                (0, "macos-checkbox-indicator-ring-checkbox", (16., 16.), 5.5),
                (1, "macos-radio-indicator-ring-radio", (16., 16.), 8.),
                (2, "macos-switch-track-ring-switch", (54., 24.), 12.),
                (3, "ring-button", (0., 24.), 6.),
                (4, "ring-toggle", (0., 24.), 6.),
                (5, "ring-tab", (0., 22.), 5.),
                (6, "ring-accordion", (192., 36.), 8.),
                (7, "ring-field", (160., 24.), 6.),
            ] {
                let field = index == 7;
                if field {
                    let state = view.read_with(cx, |view, _| view.field.clone());
                    cx.update(|window, cx| state.update(cx, |state, cx| state.focus(window, cx)));
                } else {
                    let focus = view.read_with(cx, |view, _| view.focus[index].clone());
                    cx.update(|window, cx| window.focus(&focus, cx));
                }
                cx.simulate_mouse_move(
                    gpui_kit::point(gpui_kit::px(900.), gpui_kit::px(900.)),
                    None,
                    Default::default(),
                );
                redraw(cx);
                // Pointer-driven focus rings only fields.
                assert_eq!(
                    rings(cx).len(),
                    usize::from(field),
                    "{selector}: pointer focus"
                );
                cx.simulate_keystrokes("x");
                redraw(cx);
                let target = cx.debug_bounds(selector).expect(selector);
                let close = |a: f32, b: f32| (a - b).abs() <= 0.5;
                assert!(
                    close(target.size.height.as_f32(), size.1 * zoom)
                        && (size.0 == 0. || close(target.size.width.as_f32(), size.0 * zoom)),
                    "{selector} at {zoom}: {target:?}"
                );
                let painted = rings(cx);
                assert_eq!(painted.len(), 1, "{selector} at {zoom}: exactly one ring");
                cx.update(|window, _| {
                    let scale = window.scale_factor();
                    let ring = &painted[0];
                    let expected = target.dilate(gpui_kit::px(3. * zoom)).scale(scale);
                    let near =
                        |a: gpui_kit::ScaledPixels, b: gpui_kit::ScaledPixels| close(a.0, b.0);
                    assert!(
                        near(ring.bounds.origin.x, expected.origin.x)
                            && near(ring.bounds.origin.y, expected.origin.y)
                            && near(ring.bounds.size.width, expected.size.width)
                            && near(ring.bounds.size.height, expected.size.height),
                        "{selector} at {zoom}: {:?} vs {expected:?}",
                        ring.bounds
                    );
                    assert!(
                        close(ring.border_widths.top.0, 3. * zoom * scale),
                        "{selector} width"
                    );
                    assert!(
                        close(ring.corner_radii.top_left.0, (radius + 3.) * zoom * scale),
                        "{selector} radius {:?}",
                        ring.corner_radii
                    );
                    assert_eq!(ring.background.as_solid().map_or(0., |fill| fill.a), 0.);
                    // No accent border on the control itself.
                    let control = target.scale(scale);
                    assert!(
                        window
                            .painted_quads()
                            .into_iter()
                            .filter(|quad| quad.bounds == control)
                            .all(|quad| quad.border_color != theme.accent),
                        "{selector}: no accent border while focused"
                    );
                });
                cx.deactivate_window();
                redraw(cx);
                assert!(
                    rings(cx).is_empty(),
                    "{selector}: no ring in an inactive window"
                );
                cx.update(|window, _| window.activate_window());
                cx.run_until_parked();
            }
            // Checkbox box ring 22 × 22, switch capsule 60 × 30 (spec §6.3).
            let focus = view.read_with(cx, |view, _| view.focus[0].clone());
            cx.update(|window, cx| window.focus(&focus, cx));
            cx.simulate_keystrokes("x");
            redraw(cx);
            let scale = cx.update(|window, _| window.scale_factor());
            let ring = rings(cx)[0].bounds.size;
            assert_eq!(
                (ring.width.0, ring.height.0),
                (22. * zoom * scale, 22. * zoom * scale)
            );
            let focus = view.read_with(cx, |view, _| view.focus[2].clone());
            cx.update(|window, cx| window.focus(&focus, cx));
            redraw(cx);
            let ring = rings(cx)[0].bounds.size;
            assert_eq!(
                (ring.width.0, ring.height.0),
                (60. * zoom * scale, 30. * zoom * scale)
            );
        }
    }
}
