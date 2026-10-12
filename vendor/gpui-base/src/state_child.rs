// New file by gpui-macos contributors in 2026: render-time control state children, default styles and focus visibility.
// Copyright 2026 gpui-macos contributors; Apache-2.0. See LICENSE-APACHE and MACOS-PATCH.md.

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement, MouseButton,
    MouseUpEvent, Refineable as _, StyleRefinement, Window,
};

/// Final render-time control state.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    pub disabled: bool,
    /// The control shows keyboard focus: it is enabled, its focus handle is
    /// focused, the last input was from the keyboard and the window is
    /// active. Produced by Checkbox, Radio, Switch and Toggle from their
    /// focus handle, and by Tab from a handle passed to `track_focus`.
    pub focus_visible: bool,
    /// The control's final value: a checked or mixed checkbox, an on switch,
    /// a checked radio, a pressed toggle or a selected tab.
    pub checked: bool,
    /// The primary (left) mouse button is held on the control, anywhere in
    /// its bounds. Reported by Checkbox, Radio and Switch; never while disabled.
    pub pressed: bool,
}

impl ControlState {
    pub(crate) fn new(disabled: bool, checked: bool) -> Self {
        Self {
            disabled,
            focus_visible: false,
            checked,
            pressed: false,
        }
    }

    pub(crate) fn with_pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed && !self.disabled;
        self
    }

    /// Sets `focus_visible` directly, for controls whose focus is
    /// application-owned (text field frames).
    pub(crate) fn with_focus_visible(mut self, visible: bool) -> Self {
        self.focus_visible = visible && !self.disabled;
        self
    }

    /// Sets `focus_visible` from the control's final focus handle.
    pub(crate) fn with_focus(mut self, focus: Option<&FocusHandle>, window: &Window) -> Self {
        self.focus_visible = !self.disabled
            && window.is_window_active()
            && window.last_input_was_keyboard()
            && focus.is_some_and(|focus| focus.is_focused(window));
        self
    }
}

/// Per-control primary-button press state, kept without using the caller's
/// GPUI `group` slot. A stored press is cancelled while the control is
/// disabled or the window is inactive, because the matching mouse-up may
/// never be delivered.
pub(crate) fn press_state(
    id: &ElementId,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<bool> {
    let state = window.use_keyed_state((id.clone(), "control-pressed"), cx, |_, _| false);
    if (disabled || !window.is_window_active()) && *state.read(cx) {
        state.update(cx, |pressed, _| *pressed = false);
    }
    state
}

/// Tracks left-button presses on `element`; other buttons are ignored.
pub(crate) fn track_press<E: InteractiveElement>(element: E, state: &Entity<bool>) -> E {
    fn set<Event: 'static>(
        state: &Entity<bool>,
        value: bool,
    ) -> impl Fn(&Event, &mut Window, &mut App) + 'static {
        let state = state.clone();
        move |_, _, cx| {
            state.update(cx, |pressed, cx| {
                if *pressed != value {
                    *pressed = value;
                    cx.notify();
                }
            })
        }
    }
    let release = set::<MouseUpEvent>(state, false);
    element
        .on_mouse_down(MouseButton::Left, set(state, true))
        // Capture phase, so a handler that stops propagation in the bubble
        // phase (for example in `on_change`) cannot leave the press stuck.
        .capture_any_mouse_up(move |event, window, cx| {
            if event.button == MouseButton::Left {
                release(event, window, cx);
            }
        })
        .on_mouse_up_out(MouseButton::Left, set(state, false))
}

type StyledChild = Box<dyn FnOnce(ControlState, &StyleRefinement) -> AnyElement>;

/// Children are resolved after builders finalize the control state.
pub(crate) enum StateChild {
    Ready(AnyElement),
    State(Box<dyn FnOnce(ControlState) -> AnyElement>),
    /// Also receives the control's final resolved style (defaults, caller
    /// instance and semantic styles), for example to follow its radius and
    /// border width.
    Styled(StyledChild),
}
impl StateChild {
    pub(crate) fn state<E: IntoElement>(build: impl FnOnce(ControlState) -> E + 'static) -> Self {
        Self::State(Box::new(move |state| build(state).into_any_element()))
    }
    pub(crate) fn styled<E: IntoElement>(
        build: impl FnOnce(ControlState, &StyleRefinement) -> E + 'static,
    ) -> Self {
        Self::Styled(Box::new(move |state, style| {
            build(state, style).into_any_element()
        }))
    }
    pub(crate) fn resolve(self, state: ControlState, style: &StyleRefinement) -> AnyElement {
        match self {
            Self::Ready(element) => element,
            Self::State(build) => build(state),
            Self::Styled(build) => build(state, style),
        }
    }
}

/// Render-time default styles for a control, built from its final state.
/// They sit beneath the caller's own styles. The caller's instance and
/// semantic-state styles win property by property, and the default hover and
/// active styles never override a background, text color or border color the
/// caller set. A caller `hover` or `active` refinement replaces the default
/// one entirely, as a constructor's refinement always could be replaced.
/// Default hover and active styles are dropped while the control is disabled.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct StateStyles {
    pub base: StyleRefinement,
    pub hover: Option<StyleRefinement>,
    pub active: Option<StyleRefinement>,
}

impl StateStyles {
    pub fn new(base: StyleRefinement) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }

    pub fn hover(mut self, style: StyleRefinement) -> Self {
        self.hover = Some(style);
        self
    }

    pub fn active(mut self, style: StyleRefinement) -> Self {
        self.active = Some(style);
        self
    }
}

pub(crate) type DefaultStyles = Box<dyn FnOnce(ControlState, &mut Window, &mut App) -> StateStyles>;

/// Caller-owned interaction refinements, captured so defaults can sit beneath them.
#[derive(Default)]
pub(crate) struct CallerInteraction {
    pub(crate) hover: Option<StyleRefinement>,
    pub(crate) active: Option<StyleRefinement>,
}

/// The base, hover and active refinements to apply at render.
pub(crate) struct ResolvedStyles {
    pub(crate) base: StyleRefinement,
    pub(crate) hover: Option<StyleRefinement>,
    pub(crate) active: Option<StyleRefinement>,
}

pub(crate) fn resolve_styles(
    defaults: Option<DefaultStyles>,
    state: ControlState,
    caller: StyleRefinement,
    interaction: CallerInteraction,
    window: &mut Window,
    cx: &mut App,
) -> ResolvedStyles {
    let defaults = defaults
        .map(|build| build(state, window, cx))
        .unwrap_or_default();
    let layer = |default: Option<StyleRefinement>, replacement: Option<StyleRefinement>| {
        replacement.or_else(|| {
            default.filter(|_| !state.disabled).map(|mut default| {
                // Caller paint persists through hover and press.
                if caller.background.is_some() {
                    default.background = None;
                }
                if caller.text.color.is_some() {
                    default.text.color = None;
                }
                if caller.border_color.is_some() {
                    default.border_color = None;
                }
                default
            })
        })
    };
    let mut base = defaults.base;
    base.refine(&caller);
    ResolvedStyles {
        base,
        hover: layer(defaults.hover, interaction.hover),
        active: layer(defaults.active, interaction.active),
    }
}
