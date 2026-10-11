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
    if !state.read(cx).presentation().is_masked() {
        state.update(cx, |state, cx| state.set_masked(true, window, cx));
    }
    input(id, state, window, cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{AppContext, Context, IntoElement, Render, TestAppContext};
    use std::{cell::Cell, rc::Rc};

    struct SecureField(Entity<InputState>);

    impl Render for SecureField {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            secure_field("password", &self.0, window, cx)
        }
    }

    #[gpui_kit::test]
    fn secure_field_does_not_notify_its_editor_on_unchanged_redraws(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let notifications = Rc::new(Cell::new(0));
        let observed = notifications.clone();
        let (view, cx) = cx.add_window_view(move |window, cx| {
            let state = cx.new(|cx| InputState::new(window, cx).default_value("secret"));
            cx.observe(&state, move |_, _, _| observed.set(observed.get() + 1))
                .detach();
            SecureField(state)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        notifications.set(0);
        for _ in 0..3 {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                assert!(view.read(cx).0.read(cx).presentation().is_masked());
            });
            cx.run_until_parked();
        }
        assert_eq!(
            notifications.get(),
            0,
            "Rendering a secure field must not schedule another redraw"
        );
    }
}
