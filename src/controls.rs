//! Styled constructors retain gpui-base's controlled-state builder APIs.
use crate::NativeTextStyle as _;
use crate::state::{PaintRole, Part, States, Tint, over, paint};
use crate::{ActiveTheme, Button, IconName, Link, icon};
use gpui_kit::base::{Checkbox, CheckboxState, Radio, StateStyles, Switch, Tab, Tabs, Toggle};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::rems;
use gpui_kit::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
};

/// AppKit keeps inactive controls enabled while replacing accent color with a
/// neutral selection. Used by selection rows until the row rule (plan M4).
pub(crate) fn control_selection(theme: &crate::Theme, active: bool) -> Hsla {
    if active {
        theme.accent
    } else {
        theme
            .control_fill()
            .blend(
                theme
                    .foreground
                    .opacity(if theme.is_dark() { 0.1 } else { 0.08 }),
            )
    }
}

pub(crate) fn control_mark(theme: &crate::Theme, active: bool) -> Hsla {
    if active {
        theme.on_accent
    } else {
        theme.foreground
    }
}

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
        div()
            .size(rems(1.))
            .debug_selector(move || format!("macos-checkbox-indicator-{id}"))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded(rems(0.25))
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
        div()
            .size(rems(1.))
            .debug_selector(move || format!("macos-radio-indicator-{id}"))
            .flex_shrink_0()
            .rounded_full()
            .bg(well)
            .flex()
            .items_center()
            .justify_center()
            .when(on, |indicator| {
                indicator.child(div().size(rems(0.25)).rounded_full().bg(mark))
            })
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
    // Hover is unchanged until the hover rule lands (plan M4); a borderless
    // button has none (C4). Overlays composite, so a translucent bezel darkens.
    let hover_bg = match role {
        PaintRole::Prominent(_) => bg.blend(Hsla::black().opacity(0.08)),
        _ => over(bg, t.hover_fill()),
    };
    // Restyled callers keep a pressed fill; the resolver also sets the label.
    let pressed_bg = paint(role, Part::Fill, rest.pressed(true), t);
    let label = label.into();
    Button::new(id)
        .accessibility_label(label.clone())
        .when(!label.is_empty(), |button| button.child(label))
        .min_h(rems(1.5))
        .py(rems(0.1875))
        .px(rems(0.75))
        .gap(rems(0.375))
        .border_1()
        .rounded(rems(0.375))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        // A borderless button is its label: no bezel padding, border or
        // minimum height. Source: the SwiftUI probe's regular `.borderless`
        // "Button" frame is 41 x 16 pt, the label's 13 pt advance (40.29,
        // spec §4) by its Body line (16, §3.1). Keyboard focus keeps the ring.
        .when(borderless, |button| {
            button.min_h(rems(0.)).p(rems(0.)).border_0()
        })
        .bg(bg)
        .text_color(fg)
        .paint_role(role)
        .record_role_default()
        .when(!borderless, |button| button.hover(|s| s.bg(hover_bg)))
        .active(|s| s.bg(pressed_bg))
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .selected_fill(match role {
            PaintRole::Prominent(_) => bg,
            _ => over(bg, t.selected_fill()),
        })
        // Only for buttons whose caller replaces the role paint (plan D6).
        .styles(|s| s.disabled(|s| s.opacity(crate::tokens::CUSTOM_DISABLED_OPACITY)))
}

/// A square, icon-only button. `label` names the action for assistive tech;
/// pair it with [`crate::with_tooltip`] so sighted users can read it too.
pub fn icon_button(
    id: impl Into<ElementId>,
    name: IconName,
    label: impl Into<SharedString>,
    variant: ButtonVariant,
    cx: &App,
) -> Button {
    button(id, "", variant, cx)
        .accessibility_label(label)
        .fill_duration(crate::motion::CURSOR_COLOR)
        // Equal padding around a fixed-size glyph keeps the hit target square.
        .p(rems(0.25))
        .size(rems(1.5))
        .justify_center()
        .child(icon(name).size(rems(0.875)))
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
        .gap(rems(0.3125))
        .min_h(rems(1.))
        .rounded(rems(0.25))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .child_with_state(move |control| {
            NativeCheckboxIndicator(id, state == CheckboxState::Indeterminate, control)
        })
        .child(label)
        .focus_visible(|s| s.shadow(t.focus_shadow()))
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
        .gap(rems(0.5))
        .rounded(rems(0.75))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .focus_visible(|s| s.shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
        .child_with_state(move |control| {
            crate::switch_motion::SwitchMotion::new(id, control.checked)
                .disabled(control.disabled)
                .pressed(control.pressed)
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
        .gap(rems(0.3125))
        .min_h(rems(1.))
        .rounded(rems(0.25))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .child_with_state(move |control| NativeRadioIndicator(id, control))
        .child(label)
        .focus_visible(|s| s.shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
}

/// Render-time default paint for a toggle button or tab from its final value,
/// disabled state and window activity. It sits beneath caller styles, so a
/// caller `bg` or `text_color` wins and extra children inherit the label ink.
fn bezel_styles(
    role: PaintRole,
    hover: impl FnOnce(Hsla, &crate::Theme) -> Option<StyleRefinement>,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> StateStyles {
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
    // has no hover or pressed paint either.
    if states.window_active {
        let pressed = states.pressed(true);
        styles = styles.active(
            StyleRefinement::default()
                .bg(part(Part::Fill, pressed))
                .text_color(part(Part::Label, pressed)),
        );
        if let Some(hover) = hover(fill, t) {
            styles = styles.hover(hover);
        }
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
        .min_h(rems(1.5))
        .py(rems(0.1875))
        .px(rems(0.625))
        .border_1()
        .rounded(rems(0.375))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        .child(label)
        // Toggle off is a Bezel (spec §6.1); on is ToggleOn (§6.2.4). The
        // role follows the final pressed value.
        .default_styles(|state, window, cx| {
            let role = if state.checked {
                PaintRole::ToggleOn
            } else {
                PaintRole::Bezel
            };
            // The hover border and bezel overlay stay until the hover rule (M4).
            bezel_styles(
                role,
                |fill, t| {
                    let hover = StyleRefinement::default().border_color(t.focus_border());
                    Some(if role == PaintRole::Bezel {
                        hover.bg(over(fill, t.hover_fill()))
                    } else {
                        hover
                    })
                },
                state.disabled,
                window,
                cx,
            )
        })
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
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
        .hover(|s| s.text_color(color).underline())
        .active(|s| s.text_color(pressed))
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.text_color(disabled)))
}

/// A rounded segmented strip with gpui-base Tab semantics.
pub fn tabs(id: impl Into<ElementId>, cx: &App) -> Tabs {
    let t = cx.macos();
    Tabs::new(id)
        .flex()
        .h(rems(1.5))
        .items_center()
        .gap(rems(0.125))
        .p(rems(0.0625))
        .rounded(rems(0.375))
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
        .px_3()
        .h(rems(1.375))
        .py(rems(0.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap_1()
        .border_1()
        .rounded(rems(0.3125))
        .border_color(t.foreground.opacity(0.))
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        // Segmented, spec §6.2.5 (C6): accent selection with a white label,
        // following the final selected value.
        .default_styles(|state, window, cx| {
            let role = PaintRole::Segment {
                selected: state.checked,
            };
            // Unselected tabs keep their hover overlay until the hover rule (M4).
            bezel_styles(
                role,
                |_, t| (!state.checked).then(|| StyleRefinement::default().bg(t.hover_fill())),
                state.disabled,
                window,
                cx,
            )
        })
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
}

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
                            == gpui_kit::size(gpui_kit::px(4.), gpui_kit::px(4.))
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

            // Hover: one composite fill, and none on a disabled toggle.
            for (selector, expected) in [
                ("off-toggle", over(theme.bezel_fill(), theme.hover_fill())),
                ("disabled-off-toggle", ink(&theme).opacity(0.039)),
            ] {
                let center = cx.debug_bounds(selector).unwrap().center();
                cx.simulate_mouse_move(center, None, Default::default());
                redraw(cx);
                assert_eq!(fills_at(cx, selector), [expected], "{selector} hovered");
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
                // Only the (hovered) default remains: no pressed @0.165.
                [over(theme.bezel_fill(), theme.hover_fill())],
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
                    let expected = reference[appearance]["controls"][native]["intrinsic_height"]
                        .as_f64()
                        .unwrap() as f32
                        * zoom;
                    let actual = cx.debug_bounds(selector).unwrap().size.height.as_f32();
                    assert!(
                        (actual - expected).abs() < 0.1,
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
}
