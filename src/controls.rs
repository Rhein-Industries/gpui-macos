//! Styled constructors retain gpui-base's controlled-state builder APIs.
use crate::NativeTextStyle as _;
use crate::tokens::{disabled_control_fill, disabled_control_mark};
use crate::{ActiveTheme, Button, IconName, Link, icon};
use gpui_kit::base::{Checkbox, CheckboxState, Radio, Switch, Tab, Tabs, Toggle};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::rems;
use gpui_kit::{
    App, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

/// AppKit keeps inactive controls enabled while replacing accent color with a
/// neutral selection. Resolve this per window, including secondary windows.
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

pub(crate) fn control_thumb(theme: &crate::Theme, active: bool) -> Hsla {
    if !active && theme.is_dark() {
        gpui_kit::rgb(0xdedede).into()
    } else {
        gpui_kit::rgb(0xffffff).into()
    }
}

pub(crate) fn unchecked_control_fill(theme: &crate::Theme, active: bool) -> Hsla {
    if !active {
        gpui_kit::rgb(if theme.is_dark() { 0x3c3d40 } else { 0xdfdfdf }).into()
    } else {
        theme.control_fill()
    }
}

fn disabled_aware_fill(theme: &crate::Theme, active: bool, disabled: bool, selected: bool) -> Hsla {
    match (disabled, selected) {
        (true, selected) => disabled_control_fill(theme, selected),
        (false, true) => control_selection(theme, active),
        (false, false) => unchecked_control_fill(theme, active),
    }
}

pub(crate) fn slider_selection(theme: &crate::Theme, active: bool) -> Hsla {
    if active {
        theme.accent
    } else {
        theme
            .control_fill()
            .blend(
                theme
                    .foreground
                    .opacity(if theme.is_dark() { 0.25 } else { 0.22 }),
            )
    }
}

#[derive(IntoElement)]
struct NativeCheckboxIndicator(ElementId, CheckboxState, bool);

impl RenderOnce for NativeCheckboxIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.macos();
        let active = window.is_window_active();
        let disabled = self.2;
        let mark = if disabled {
            disabled_control_mark(theme)
        } else {
            control_mark(theme, active)
        };
        div()
            .size(rems(1.))
            .debug_selector(move || format!("macos-checkbox-indicator-{}", self.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded(rems(0.25))
            .bg(disabled_aware_fill(
                theme,
                active,
                disabled,
                self.1 != CheckboxState::Unchecked,
            ))
            .text_color(mark)
            .when(self.1 != CheckboxState::Unchecked, |indicator| {
                indicator.child(
                    icon(if self.1 == CheckboxState::Checked {
                        IconName::Check
                    } else {
                        IconName::Minus
                    })
                    .size(rems(0.875))
                    .text_color(mark),
                )
            })
    }
}

#[derive(IntoElement)]
struct NativeRadioIndicator(ElementId, bool, bool);

impl RenderOnce for NativeRadioIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.macos();
        let active = window.is_window_active();
        let disabled = self.2;
        div()
            .size(rems(1.))
            .debug_selector(move || format!("macos-radio-indicator-{}", self.0))
            .flex_shrink_0()
            .rounded_full()
            .bg(disabled_aware_fill(theme, active, disabled, self.1))
            .flex()
            .items_center()
            .justify_center()
            .when(self.1, |indicator| {
                indicator.child(div().size(rems(0.25)).rounded_full().bg(if disabled {
                    disabled_control_mark(theme)
                } else {
                    control_mark(theme, active)
                }))
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
}

pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    variant: ButtonVariant,
    cx: &App,
) -> Button {
    let t = cx.macos();
    let (bg, fg) = match variant {
        ButtonVariant::Primary => (t.accent, t.on_accent),
        ButtonVariant::Danger => (t.danger, t.on_accent),
        ButtonVariant::Outline | ButtonVariant::Secondary => (t.control_fill(), t.foreground),
    };
    let filled = matches!(variant, ButtonVariant::Primary | ButtonVariant::Danger);
    // Resolve overlays before animating so an opaque control stays opaque.
    let hover_bg = bg.blend(if filled {
        gpui_kit::Hsla::black().opacity(0.08)
    } else {
        t.hover_fill()
    });
    let pressed_bg = bg.blend(if filled {
        gpui_kit::Hsla::black().opacity(0.16)
    } else {
        t.pressed_fill()
    });
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
        .bg(bg)
        .text_color(fg)
        .when(filled, |button| {
            button
                .inactive_style(t.control_fill(), t.foreground)
                .disabled_style(
                    t.disabled_tint(bg),
                    fg.opacity(crate::tokens::DISABLED_TINT_LABEL_OPACITY),
                )
        })
        .hover(|s| s.bg(hover_bg))
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .active(|s| s.bg(pressed_bg))
        .selected_fill(if filled {
            bg
        } else {
            bg.blend(t.selected_fill())
        })
        .styles(|s| s.disabled(|s| s.opacity(0.45)))
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
        .child_with_disabled(move |disabled| NativeCheckboxIndicator(id, state, disabled))
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
        .child_with_disabled(move |disabled| {
            crate::switch_motion::SwitchMotion::new(id, checked).disabled(disabled)
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
        .child_with_disabled(move |disabled| NativeRadioIndicator(id, checked, disabled))
        .child(label)
        .focus_visible(|s| s.shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.text_color(t.disabled_foreground())))
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
        .bg(t.control_fill())
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .font_weight(FontWeight::NORMAL)
        .child(label)
        .hover(|s| s.bg(t.hover_fill()).border_color(t.focus_border()))
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .active(|s| s.bg(t.pressed_fill()))
        .styles(|s| {
            s.pressed(|s| {
                s.bg(t.accent)
                    .text_color(t.on_accent)
                    .border_color(t.accent)
            })
            .disabled(|s| s.opacity(0.45))
        })
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
        .text_color(t.accent)
        .rounded(rems(0.25))
        .border_1()
        .border_color(t.foreground.opacity(0.))
        .px(rems(0.25))
        .py(rems(0.375))
        .hover(|s| s.text_color(t.accent).underline())
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .styles(|s| s.disabled(|s| s.opacity(0.45)))
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
        .bg(t.control_fill())
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
        .text_color(t.secondary)
        .hover(|s| s.bg(t.hover_fill()))
        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
        .active(|s| s.bg(t.pressed_fill()))
        .styles(|s| {
            s.selected(|s| s.bg(t.surface).text_color(t.foreground))
                .disabled(|s| s.opacity(0.45))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
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
        }
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
            for selector in [
                "activation-primary",
                "macos-checkbox-indicator-activation-checkbox",
                "macos-radio-indicator-activation-radio",
                "macos-switch-track-activation-switch",
                "macos-slider-selection",
            ] {
                assert_eq!(
                    painted_background(cx, selector),
                    theme.accent,
                    "{selector} uses the active accent"
                );
            }
            let disabled = painted_background(cx, "activation-disabled");
            assert_eq!(painted_radio_mark(cx), theme.on_accent);
            assert_eq!(
                disabled,
                theme.accent.opacity(0.69),
                "a disabled primary keeps a faded accent bezel"
            );
            for selector in [
                "macos-checkbox-indicator-activation-disabled-checkbox",
                "macos-radio-indicator-activation-disabled-radio",
            ] {
                assert_eq!(
                    painted_background(cx, selector),
                    crate::tokens::disabled_control_fill(&theme, true),
                    "{selector} is a neutral well rather than a faded accent"
                );
            }
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-switch"),
                theme.accent.opacity(0.71),
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
            for selector in [
                "activation-primary",
                "macos-checkbox-indicator-activation-checkbox",
                "macos-radio-indicator-activation-radio",
                "macos-switch-track-activation-switch",
                "macos-slider-selection",
            ] {
                let fill = painted_background(cx, selector);
                assert_eq!(fill.s, 0., "{selector} becomes neutral on blur");
                assert_eq!(fill.a, 1., "inactive remains enabled rather than disabled");
                assert_ne!(fill, theme.accent);
            }
            assert_eq!(
                painted_background(cx, "activation-disabled"),
                crate::tokens::disabled_bezel(&theme).0,
                "an inactive disabled prominent button uses Bezel disabled"
            );
            let expected_label = if theme.is_dark() {
                Hsla::white().opacity(0.290)
            } else {
                Hsla::black().opacity(0.257)
            };
            assert_eq!(
                painted_label_color(cx, "activation-disabled"),
                expected_label,
                "inactive and disabled prominent label uses Bezel disabled ink"
            );
            assert_eq!(
                painted_label_color(cx, "activation-disabled-checkbox"),
                theme.tertiary_label()
            );
            assert_eq!(
                painted_background(cx, "macos-switch-track-activation-disabled-off-switch"),
                expected_off
            );
            let expected_thumb: Hsla =
                gpui_kit::rgb(if theme.is_dark() { 0xdedede } else { 0xffffff }).into();
            {
                let off_fill: Hsla =
                    gpui_kit::rgb(if theme.is_dark() { 0x3c3d40 } else { 0xdfdfdf }).into();
                assert_eq!(
                    painted_background(cx, "macos-radio-indicator-activation-off-radio"),
                    off_fill
                );
                assert_eq!(
                    painted_background(cx, "macos-switch-track-activation-off-switch"),
                    off_fill
                );
            }
            assert_eq!(
                painted_radio_mark(cx),
                theme.foreground,
                "inactive radio uses the native black/white foreground mark"
            );
            assert_eq!(
                painted_background(cx, "macos-switch-thumb-activation-switch"),
                expected_thumb
            );
            assert_eq!(
                painted_background(cx, "macos-slider-thumb-end"),
                expected_thumb
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
