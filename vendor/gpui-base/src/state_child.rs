// New file by gpui-macos contributors in 2026: render-time control state children.
// Copyright 2026 gpui-macos contributors; Apache-2.0. See LICENSE-APACHE and MACOS-PATCH.md.

use gpui::{AnyElement, IntoElement};

/// Final render-time control state. Focus visibility is reserved for a later
/// producer; it is currently always false.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    pub disabled: bool,
    pub focus_visible: bool,
}

/// Children are resolved after builders finalize the control state.
pub(crate) enum StateChild {
    Ready(AnyElement),
    State(Box<dyn FnOnce(ControlState) -> AnyElement>),
}
impl StateChild {
    pub(crate) fn state<E: IntoElement>(build: impl FnOnce(ControlState) -> E + 'static) -> Self {
        Self::State(Box::new(move |state| build(state).into_any_element()))
    }
    pub(crate) fn resolve(self, disabled: bool) -> AnyElement {
        match self {
            Self::Ready(element) => element,
            Self::State(build) => build(ControlState {
                disabled,
                focus_visible: false,
            }),
        }
    }
}
