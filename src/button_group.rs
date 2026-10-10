//! Exclusive settings and page navigation with one keyboard stop per group.
use crate::{ActiveTheme, ChoiceItem};
use gpui_kit::base::{
    Radio, RadioGroup, Tabs,
    actions::{Confirm, SelectLeft, SelectRight},
};
use gpui_kit::rems;
use gpui_kit::{App, Context, ElementId, Entity, FocusHandle, KeyBinding, Window, prelude::*};
use std::rc::Rc;

type Change = Rc<dyn Fn(usize, &mut Window, &mut App)>;
struct Cursor {
    focus: FocusHandle,
    index: Option<usize>,
    was_focused: bool,
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("left", SelectLeft, Some("macOSOptionGroup")),
        KeyBinding::new("h", SelectLeft, Some("macOSOptionGroup")),
        KeyBinding::new("right", SelectRight, Some("macOSOptionGroup")),
        KeyBinding::new("l", SelectRight, Some("macOSOptionGroup")),
        KeyBinding::new(
            "enter",
            Confirm { secondary: false },
            Some("macOSOptionGroup"),
        ),
        KeyBinding::new(
            "space",
            Confirm { secondary: false },
            Some("macOSOptionGroup"),
        ),
    ]);
}

/// A single-choice setting rendered as content-sized segments in a shared frame.
/// The application owns `selected` and updates it in `on_change`.
/// Left/Right (or h/l) move the cursor; Return/Space commit the choice.
pub fn button_group(
    id: impl Into<ElementId>,
    items: Vec<ChoiceItem>,
    selected: Option<usize>,
    on_change: impl Fn(usize, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> RadioGroup {
    let id = id.into();
    let cursor = cursor(&id, &items, selected, window, cx);
    let on_change: Change = Rc::new(move |index, window, cx| {
        if selected != Some(index) {
            on_change(index, window, cx);
        }
    });
    let t = cx.macos().clone();
    let window_active = window.is_window_active();
    let count = items.len();
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let change = on_change.clone();
            let pointer_cursor = cursor.clone();
            let focus = cursor.read(cx).focus.clone();
            Radio::new(index)
                .debug_selector(move || format!("macos-option-{index}"))
                .checked(selected == Some(index))
                .disabled(item.disabled)
                .accessibility_label(item.label.clone())
                .set_position(index + 1, count)
                .tab_stop(false)
                .flex()
                .relative()
                .items_center()
                .flex_grow(1.)
                .flex_shrink(1.)
                .min_w_0()
                .justify_center()
                .px_3()
                .h(rems(1.375))
                .py(rems(0.))
                .border_1()
                .rounded(rems(0.3125))
                .border_color(t.foreground.opacity(0.))
                .font_family(t.font.clone())
                .text_size(rems(0.8125))
                .line_height(gpui_kit::relative(1.2))
                .text_color(if window_active {
                    t.foreground
                } else {
                    t.secondary
                })
                .bg(if selected == Some(index) {
                    if t.is_dark() {
                        t.control_fill().blend(t.foreground.opacity(0.16))
                    } else if window_active {
                        t.surface
                    } else {
                        crate::controls::control_selection(&t, false)
                    }
                } else {
                    t.foreground.opacity(0.)
                })
                .when(
                    window_active
                        && focus.is_focused(window)
                        && cursor.read(cx).index == Some(index),
                    |row| row.border_color(t.accent).shadow(t.focus_shadow()),
                )
                .when(window_active && !item.disabled, |row| {
                    row.hover(|row| row.bg(t.hover_fill()))
                        .active(|row| row.bg(t.pressed_fill()))
                })
                .styles(|styles| styles.disabled(|row| row.opacity(0.45)))
                .child(item.label.clone())
                .when(
                    index > 0 && selected != Some(index) && selected != Some(index - 1),
                    |row| {
                        row.child(
                            gpui_kit::div()
                                .absolute()
                                .left(rems(-0.0625))
                                .top(rems(0.25))
                                .bottom(rems(0.25))
                                .w(rems(0.0625))
                                .bg(t.divider()),
                        )
                    },
                )
                .when(!item.disabled, |row| {
                    row.on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
                        pointer_cursor.update(cx, |state, cx| {
                            state.focus.focus(window, cx);
                            // Preserve the pointer cursor when focus enters the group.
                            state.was_focused = true;
                            state.index = Some(index);
                            cx.notify();
                        });
                    })
                })
                .on_change(move |_, _, window, cx| {
                    focus.focus(window, cx);
                    change(index, window, cx);
                })
                .into_any_element()
        })
        .collect::<Vec<_>>();
    navigate(
        RadioGroup::new(id)
            .axis(gpui_kit::Axis::Horizontal)
            .flex()
            .w_full()
            .h(rems(1.5))
            .gap(rems(0.125))
            .p(rems(0.0625))
            .rounded(rems(0.375))
            .bg(t.control_fill())
            .children(rows),
        cursor,
        items,
        on_change,
        cx,
    )
}

/// A page tab list with the same restrained visual language and independent
/// tab-list semantics. Use the selected value to render the associated panel.
/// `show_shortcuts` renders one-based number hints; applications own key bindings.
pub fn tab_list(
    id: impl Into<ElementId>,
    items: Vec<ChoiceItem>,
    selected: Option<usize>,
    show_shortcuts: bool,
    on_change: impl Fn(usize, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Tabs {
    let id = id.into();
    let cursor = cursor(&id, &items, selected, window, cx);
    let on_change: Change = Rc::new(move |index, window, cx| {
        if selected != Some(index) {
            on_change(index, window, cx);
        }
    });
    let count = items.len();
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let change = on_change.clone();
            let focus = cursor.read(cx).focus.clone();
            crate::tab(index, item.label.clone(), selected == Some(index), cx)
                .debug_selector(move || format!("macos-tab-{index}"))
                .disabled(item.disabled)
                .set_position(index + 1, count)
                .when(show_shortcuts, |tab| {
                    tab.child(
                        gpui_kit::div()
                            .self_start()
                            .text_size(gpui_kit::rems(0.625))
                            .child((index + 1).to_string()),
                    )
                })
                .on_click({
                    let cursor = cursor.clone();
                    move |_, window, cx| {
                        cursor.update(cx, |state, cx| {
                            state.index = Some(index);
                            cx.notify();
                        });
                        focus.focus(window, cx);
                        change(index, window, cx);
                    }
                })
                .into_any_element()
        })
        .collect::<Vec<_>>();
    navigate(
        crate::tabs(id, cx).children(rows),
        cursor,
        items,
        on_change,
        cx,
    )
}

fn cursor(
    id: &ElementId,
    items: &[ChoiceItem],
    selected: Option<usize>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Cursor> {
    let initial = selected
        .filter(|&i| items.get(i).is_some_and(|item| !item.disabled))
        .or_else(|| items.iter().position(|item| !item.disabled));
    let cursor = window.use_keyed_state((id.clone(), "cursor"), cx, |_, cx| Cursor {
        focus: cx.focus_handle(),
        index: initial,
        was_focused: false,
    });
    cursor.update(cx, |state, _| {
        let focused = state.focus.is_focused(window);
        if (focused && !state.was_focused)
            || !state
                .index
                .is_some_and(|i| items.get(i).is_some_and(|item| !item.disabled))
        {
            state.index = initial;
        }
        state.was_focused = focused;
    });
    cursor
}

fn navigate<T: StatefulInteractiveElement + FluentBuilder>(
    root: T,
    cursor: Entity<Cursor>,
    items: Vec<ChoiceItem>,
    change: Change,
    cx: &App,
) -> T {
    let focus = cursor.read(cx).focus.clone();
    let enabled: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| (!item.disabled).then_some(i))
        .collect();
    let mut root = root
        .key_context("macOSOptionGroup")
        .when(!enabled.is_empty(), |root| {
            root.track_focus(&focus.clone().tab_stop(true))
        });
    if !enabled.is_empty() {
        let pointer_focus = focus.clone();
        root = root.on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
            pointer_focus.focus(window, cx)
        });
    }
    let left = cursor.clone();
    let left_items = enabled.clone();
    root = root.on_action(move |_: &SelectLeft, window, cx| {
        left.update(cx, |state, cx| step(state, &left_items, false, cx));
        window.refresh();
    });
    let right = cursor.clone();
    let right_items = enabled;
    root = root.on_action(move |_: &SelectRight, window, cx| {
        right.update(cx, |state, cx| step(state, &right_items, true, cx));
        window.refresh();
    });
    root.on_action(move |_: &Confirm, window, cx| {
        if let Some(index) = cursor
            .read(cx)
            .index
            .filter(|&i| items.get(i).is_some_and(|item| !item.disabled))
        {
            change(index, window, cx);
        }
    })
}
fn step(cursor: &mut Cursor, enabled: &[usize], forward: bool, cx: &mut Context<Cursor>) {
    if enabled.is_empty() {
        return;
    }
    let current = cursor
        .index
        .and_then(|i| enabled.iter().position(|&v| v == i))
        .unwrap_or(0);
    cursor.index = Some(
        enabled[if forward {
            (current + 1).min(enabled.len() - 1)
        } else {
            current.saturating_sub(1)
        }],
    );
    cx.notify();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Render, TestAppContext};

    #[gpui_kit::test]
    fn native_segments_fit_their_labels_and_share_extra_width(cx: &mut TestAppContext) {
        struct NativeSegments;
        impl Render for NativeSegments {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                button_group(
                    "native-segments",
                    vec![
                        ChoiceItem::new("general", "General"),
                        ChoiceItem::new("network", "Network"),
                        ChoiceItem::new("advanced", "Advanced"),
                    ],
                    Some(0),
                    |_, _, _| {},
                    window,
                    cx,
                )
                .w(rems(13.8125))
                .debug_selector(|| "native-segments".into())
            }
        }
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| NativeSegments);
        for zoom in [1., 1.25, 2.] {
            cx.update(|window, cx| {
                window.set_rem_size(gpui_kit::px(16. * zoom));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let frame = cx.debug_bounds("native-segments").unwrap();
            let general = cx.debug_bounds("macos-option-0").unwrap();
            let network = cx.debug_bounds("macos-option-1").unwrap();
            let advanced = cx.debug_bounds("macos-option-2").unwrap();
            assert!((frame.size.width.as_f32() - 221. * zoom).abs() <= 0.5);
            // The equal-length labels depend on font shaping and pixel snapping;
            // the longer label must keep a larger share of the frame.
            assert!(general.size.width < advanced.size.width);
            assert!(network.size.width < advanced.size.width);
            assert!(((general.left() - frame.left()).as_f32() - zoom).abs() <= 0.5);
            assert!(((frame.right() - advanced.right()).as_f32() - zoom).abs() <= 0.5);
            assert!((general.size.height.as_f32() - 22. * zoom).abs() <= 0.5);
        }
    }
    struct Harness {
        tab_list: bool,
        selected: usize,
        before: FocusHandle,
        after: FocusHandle,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let items = vec![
                ChoiceItem::new("first", "First"),
                ChoiceItem::new("disabled", "Disabled").disabled(true),
                ChoiceItem::new("last", "Last"),
            ];
            let target = cx.entity();
            let change = move |index, _: &mut Window, cx: &mut App| {
                target.update(cx, |this, cx| {
                    this.selected = index;
                    cx.notify();
                })
            };
            let group = if self.tab_list {
                tab_list(
                    "options",
                    items,
                    Some(self.selected),
                    true,
                    change,
                    window,
                    cx,
                )
                .into_any_element()
            } else {
                button_group("options", items, Some(self.selected), change, window, cx)
                    .into_any_element()
            };
            crate::focus_scope("root")
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    crate::button("before", "Before", crate::ButtonVariant::Secondary, cx)
                        .track_focus(&self.before),
                )
                .child(group)
                .child(
                    crate::button("after", "After", crate::ButtonVariant::Secondary, cx)
                        .track_focus(&self.after),
                )
        }
    }
    #[gpui_kit::test]
    fn clicking_a_tab_moves_the_keyboard_cursor_to_it(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            tab_list: true,
            selected: 0,
            before: cx.focus_handle(),
            after: cx.focus_handle(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        for (index, selector) in [(2, "macos-tab-2"), (0, "macos-tab-0")] {
            let point = cx.debug_bounds(selector).unwrap().center();
            cx.simulate_click(point, Default::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_keystrokes("enter");
            cx.update(|_, cx| {
                assert_eq!(view.read(cx).selected, index);
            });
        }
    }

    #[gpui_kit::test]
    fn clicking_a_button_moves_the_keyboard_cursor_to_it(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            tab_list: false,
            selected: 0,
            before: cx.focus_handle(),
            after: cx.focus_handle(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        for (index, selector) in [
            (2, "macos-option-2"),
            (0, "macos-option-0"),
            (2, "macos-option-2"),
            (0, "macos-option-0"),
        ] {
            let point = cx.debug_bounds(selector).unwrap().center();
            cx.simulate_click(point, Default::default());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                assert_eq!(view.read(cx).selected, index);
            });
            cx.simulate_keystrokes("enter space");
            cx.update(|_, cx| assert_eq!(view.read(cx).selected, index));
        }
        // Clicking the selected radio must also reset an uncommitted arrow cursor.
        cx.simulate_keystrokes("right");
        let point = cx.debug_bounds("macos-option-0").unwrap().center();
        cx.simulate_click(point, Default::default());
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert_eq!(view.read(cx).selected, 0));
    }

    #[gpui_kit::test]
    fn groups_have_one_tab_stop_and_skip_disabled_choices(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for tab_list in [false, true] {
            let (view, cx) = cx.add_window_view(move |_, cx| Harness {
                tab_list,
                selected: 0,
                before: cx.focus_handle(),
                after: cx.focus_handle(),
            });
            cx.update(|window, cx| {
                view.read(cx).before.clone().focus(window, cx);
                window.draw(cx).clear(cx);
            });
            cx.simulate_keystrokes("tab");
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.simulate_keystrokes("right");
            cx.update(|_, cx| {
                assert_eq!(
                    view.read(cx).selected,
                    0,
                    "arrows move the cursor without committing"
                )
            });
            cx.simulate_keystrokes("enter");
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                assert_eq!(view.read(cx).selected, 2, "disabled option is skipped");
            });
            cx.simulate_keystrokes("tab");
            cx.update(|window, cx| {
                assert!(
                    view.read(cx).after.is_focused(window),
                    "one tab stop per group"
                )
            });
            cx.simulate_keystrokes("shift-tab");
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            cx.simulate_keystrokes("left space");
            cx.update(|_, cx| assert_eq!(view.read(cx).selected, 0));
        }
    }
}
