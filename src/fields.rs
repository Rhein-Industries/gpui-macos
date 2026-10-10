//! Familiar macOS text-field configurations on the shared editor behavior.
use crate::{IconName, Input, icon, input};
use gpui_kit::base::input::InputState;
use gpui_kit::{App, ElementId, Entity, Styled, Window, rems};

/// A rounded search field with a leading search glyph. The app owns the query.
pub fn search_field(
    id: impl Into<ElementId>,
    state: &Entity<InputState>,
    window: &Window,
    cx: &mut App,
) -> Input {
    input(id, state, window, cx)
        .rounded_full()
        .prefix(icon(IconName::Search).size(rems(0.875)))
}

/// A masked editor. Initialize a dedicated state for the secret value.
/// Base supplies secure clipboard and accessibility behavior while masked.
pub fn secure_field(
    id: impl Into<ElementId>,
    state: &Entity<InputState>,
    window: &mut Window,
    cx: &mut App,
) -> Input {
    state.update(cx, |state, cx| state.set_masked(true, window, cx));
    input(id, state, window, cx)
}
