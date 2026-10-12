//! Select and searchable Combobox presentations over the controlled base roots.
use crate::NativeTextStyle as _;
use crate::{ActiveTheme, ButtonVariant, IconName, button, icon, input};
use gpui_kit::base::{Combobox, ElementExt as _, Popup, Select, input::InputState};
use gpui_kit::rems;
use gpui_kit::{
    App, Context, ElementId, Entity, FocusHandle, Focusable, SharedString, Window, div, prelude::*,
};
use std::{cell::RefCell, rc::Rc};

type VisibleChoices = (Option<SharedString>, Rc<Vec<usize>>);

#[derive(Clone, Debug)]
pub struct ChoiceItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}
impl ChoiceItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Application-owned choice and editing state. Observe this entity for selection changes.
/// Use one entity per control; Select and Combobox share the same value model.
pub struct ChoiceState {
    items: Vec<ChoiceItem>,
    search_labels: Vec<String>,
    visible_cache: RefCell<Option<VisibleChoices>>,
    selected: Option<usize>,
    cursor: Option<usize>,
    open: bool,
    disabled: bool,
    label: SharedString,
    placeholder: SharedString,
    trigger_focus: FocusHandle,
    popup_focus: FocusHandle,
    query: Entity<InputState>,
    searchable: bool,
    scroll: gpui_kit::UniformListScrollHandle,
    popup_width: gpui_kit::Pixels,
}
impl ChoiceState {
    pub fn new(items: Vec<ChoiceItem>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Search options…"));
        cx.subscribe(
            &query,
            |this, _, event: &gpui_kit::base::input::InputEvent, cx| {
                if !matches!(event, gpui_kit::base::input::InputEvent::Change) {
                    return;
                }
                this.cursor = if this.query.read(cx).value().trim().is_empty() {
                    this.selected
                        .or_else(|| this.items.iter().position(|item| !item.disabled))
                } else {
                    this.visible(cx)
                        .iter()
                        .copied()
                        .find(|&i| !this.items[i].disabled)
                };
                let row = this
                    .cursor
                    .and_then(|i| this.visible(cx).binary_search(&i).ok())
                    .unwrap_or(0);
                this.scroll
                    .scroll_to_item_strict(row, gpui_kit::ScrollStrategy::Top);
                cx.notify();
            },
        )
        .detach();
        Self {
            search_labels: items.iter().map(|item| item.label.to_lowercase()).collect(),
            visible_cache: RefCell::default(),
            items,
            selected: None,
            cursor: None,
            open: false,
            disabled: false,
            label: "Choose an option".into(),
            placeholder: "Choose an option…".into(),
            trigger_focus: cx.focus_handle(),
            popup_focus: cx.focus_handle(),
            query,
            searchable: false,
            scroll: gpui_kit::UniformListScrollHandle::new(),
            popup_width: rems(17.5).to_pixels(window.rem_size()),
        }
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
    pub fn placeholder(mut self, label: impl Into<SharedString>) -> Self {
        self.placeholder = label.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn default_selected(mut self, index: usize) -> Self {
        self.selected = self
            .items
            .get(index)
            .filter(|item| !item.disabled)
            .map(|_| index);
        self
    }
    pub fn selected(&self) -> Option<&ChoiceItem> {
        self.selected.and_then(|i| self.items.get(i))
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn set_selected(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        self.selected = index.filter(|&i| self.items.get(i).is_some_and(|item| !item.disabled));
        cx.notify();
    }
    fn visible(&self, cx: &App) -> Rc<Vec<usize>> {
        let query = self.searchable.then(|| self.query.read(cx).value());
        let mut cache = self.visible_cache.borrow_mut();
        if let Some((cached_query, visible)) = cache.as_ref()
            && *cached_query == query
        {
            return visible.clone();
        }
        let normalized = query.as_deref().unwrap_or("").trim().to_lowercase();
        let visible: Rc<Vec<usize>> = Rc::new(if normalized.is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.search_labels
                .iter()
                .enumerate()
                .filter_map(|(i, label)| label.contains(&normalized).then_some(i))
                .collect()
        });
        *cache = Some((query, visible.clone()));
        visible
    }
    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.open == open {
            return;
        }
        self.open = open;
        if open {
            if self.searchable {
                let style = cx.macos().input_style();
                self.query.update(cx, |query, cx| {
                    // Opening happens before the search input's first render.
                    // Set its font before edits measure text (system fonts are
                    // unavailable in the browser).
                    query.set_editor_style(style);
                    query.set_value("", window, cx);
                });
            }
            self.cursor = self
                .selected
                .or_else(|| self.items.iter().position(|item| !item.disabled));
            if let Some(cursor) = self.cursor {
                self.scroll
                    .scroll_to_item(cursor, gpui_kit::ScrollStrategy::Nearest);
            }
            if self.searchable {
                self.query.update(cx, |query, cx| query.focus(window, cx));
            } else {
                self.popup_focus.focus(window, cx);
            }
        } else {
            self.trigger_focus.focus(window, cx);
        }
        cx.notify();
        window.refresh();
    }
    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.cursor.filter(|&i| !self.items[i].disabled) {
            self.selected = Some(index);
            self.set_open(false, window, cx);
        }
    }
    fn handle_key(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        match key {
            "up" | "down" | "home" | "end" => {
                if !self.open {
                    self.set_open(true, window, cx);
                } else {
                    self.navigate(key, cx);
                }
            }
            "enter" | "space" => {
                if self.open {
                    self.commit(window, cx);
                } else {
                    self.set_open(true, window, cx);
                }
            }
            "escape" => self.set_open(false, window, cx),
            "tab" | "shift-tab" => {
                self.set_open(false, window, cx);
                if key == "tab" {
                    window.focus_next(cx);
                } else {
                    window.focus_prev(cx);
                }
            }
            _ => {}
        }
        window.refresh();
    }
    fn navigate(&mut self, key: &str, cx: &mut Context<Self>) {
        let visible = self.visible(cx);
        let position = self.cursor.and_then(|i| visible.binary_search(&i).ok());
        let len = visible.len();
        let row = if len == 0 {
            None
        } else {
            let backwards = matches!(key, "up" | "end");
            let start = match key {
                "home" => 0,
                "end" => len - 1,
                "up" => position.map_or(len - 1, |i| (i + len - 1) % len),
                _ => position.map_or(0, |i| (i + 1) % len),
            };
            (0..len)
                .map(|offset| {
                    if backwards {
                        (start + len - offset) % len
                    } else {
                        (start + offset) % len
                    }
                })
                .find(|&row| !self.items[visible[row]].disabled)
        };
        self.cursor = row.map(|row| visible[row]);
        if let Some(row) = row {
            self.scroll
                .scroll_to_item(row, gpui_kit::ScrollStrategy::Nearest);
        }
        cx.notify();
    }
}
impl Focusable for ChoiceState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.trigger_focus.clone()
    }
}

/// An option picker with j/k and Ctrl-j/k/n/p navigation while open.
pub fn select(
    id: impl Into<ElementId>,
    state: &Entity<ChoiceState>,
    window: &mut Window,
    cx: &mut App,
) -> Select {
    let id = id.into();
    let body = presentation(id.clone(), state, false, window, cx);
    let choice = state.read(cx);
    let target = state.clone();
    Select::new(id)
        .open(choice.open)
        .disabled(choice.disabled)
        .accessibility_label(choice.label.clone())
        .focus_handle(&choice.trigger_focus)
        .on_open_change(move |open, window, cx| {
            target.update(cx, |state, cx| state.set_open(open, window, cx))
        })
        .w_full()
        .child(body)
}

/// A searchable picker with Ctrl-j/k/n/p navigation; plain letters edit the query.
pub fn combobox(
    id: impl Into<ElementId>,
    state: &Entity<ChoiceState>,
    window: &mut Window,
    cx: &mut App,
) -> Combobox {
    let id = id.into();
    let body = presentation(id.clone(), state, true, window, cx);
    let choice = state.read(cx);
    let target = state.clone();
    Combobox::new(id)
        .open(choice.open)
        .disabled(choice.disabled)
        .focus_handle(&choice.trigger_focus)
        .on_open_change(move |open, window, cx| {
            target.update(cx, |state, cx| state.set_open(open, window, cx))
        })
        .w_full()
        .child(body)
}

fn presentation(
    id: ElementId,
    state: &Entity<ChoiceState>,
    searchable: bool,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    state.update(cx, |state, _| state.searchable = searchable);
    let t = cx.macos().clone();
    let window_active = window.is_window_active();
    let choice = state.read(cx);
    let label = choice
        .selected()
        .map(|item| item.label.clone())
        .unwrap_or_else(|| choice.placeholder.clone());
    let open = choice.open;
    let disabled = choice.disabled;
    let trigger_focus = choice.trigger_focus.clone();
    let accessible_label = choice.label.clone();
    let target = state.clone();
    let trigger = button("choice-trigger", "", ButtonVariant::Secondary, cx)
        .focusable(false)
        .disabled(disabled)
        .w_full()
        .justify_start()
        .accessibility_label(accessible_label.clone())
        .border_color(t.foreground.opacity(0.))
        .child(div().flex_1().min_w_0().text_ellipsis().child(label))
        .child(
            icon(if searchable {
                IconName::ChevronDown
            } else {
                IconName::ChevronsUpDown
            })
            .size(rems(0.875)),
        )
        .on_click(move |_, window, cx| {
            target.update(cx, |state, cx| state.set_open(!state.open, window, cx))
        });
    let measure = state.clone();
    let trigger = div()
        .w_full()
        .child(trigger)
        .on_prepaint(move |bounds, window, cx| {
            measure.update(cx, |state, _| {
                if state.popup_width != bounds.size.width {
                    state.popup_width = bounds.size.width;
                    window.request_animation_frame();
                }
            });
        });
    let mut popup = Popup::new((id, "popup"), trigger).w_full();
    if open {
        let render = state.clone();
        let close = state.clone();
        popup = popup
            .offset(rems(0.25).to_pixels(window.rem_size()))
            .content_factory(move |window, cx| choice_content(&render, window, cx))
            .on_dismiss(move |window, cx| {
                close.update(cx, |state, cx| {
                    state.set_open(false, window, cx);
                    state.trigger_focus.focus(window, cx);
                })
            });
    }
    let root = div()
        .id("choice-keys")
        .aria_label(accessible_label)
        .when(!disabled, |root| {
            root.track_focus(&trigger_focus.tab_stop(true))
        })
        .w_full()
        .rounded(rems(0.375))
        .when(window_active, |root| {
            root.focus_visible(|s| s.shadow(t.focus_shadow()))
        });
    choice_keys(root, state, searchable).child(popup)
}

fn choice_keys(
    mut root: gpui_kit::Stateful<gpui_kit::Div>,
    state: &Entity<ChoiceState>,
    searchable: bool,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    macro_rules! action {
        ($ty:ty, $key:expr) => {{
            let target = state.clone();
            root = root.capture_action(move |_: &$ty, window, cx| {
                if target.read(cx).disabled {
                    return;
                }
                cx.stop_propagation();
                target.update(cx, |state, cx| state.handle_key($key, window, cx));
            });
        }};
    }
    action!(gpui_kit::base::actions::Confirm, "enter");
    action!(gpui_kit::base::actions::Cancel, "escape");
    action!(gpui_kit::base::actions::SelectUp, "up");
    action!(gpui_kit::base::actions::SelectDown, "down");
    action!(gpui_kit::base::input::Enter, "enter");
    action!(gpui_kit::base::input::Escape, "escape");
    action!(gpui_kit::base::input::MoveUp, "up");
    action!(gpui_kit::base::input::MoveDown, "down");
    action!(gpui_kit::base::input::IndentInline, "tab");
    action!(gpui_kit::base::input::OutdentInline, "shift-tab");
    let target = state.clone();
    root.capture_key_down(move |event, window, cx| {
        if target.read(cx).disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let navigation =
            if modifiers.control && !modifiers.alt && !modifiers.platform && !modifiers.shift {
                match key {
                    "j" | "n" => Some("down"),
                    "k" | "p" => Some("up"),
                    _ => None,
                }
            } else if !searchable && !modifiers.modified() {
                match key {
                    "j" => Some("down"),
                    "k" => Some("up"),
                    _ => None,
                }
            } else {
                None
            };
        if let Some(key) = navigation.filter(|_| target.read(cx).open) {
            target.update(cx, |state, cx| state.handle_key(key, window, cx));
        } else if key == "tab" {
            let backward = event.keystroke.modifiers.shift;
            target.update(cx, |state, cx| {
                state.handle_key(if backward { "shift-tab" } else { "tab" }, window, cx)
            });
        } else if !searchable
            && !event.keystroke.modifiers.modified()
            && matches!(key, "home" | "end" | "space")
        {
            target.update(cx, |state, cx| state.handle_key(key, window, cx));
        } else {
            return;
        }
        cx.stop_propagation();
    })
}

fn choice_content(
    state: &Entity<ChoiceState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let t = cx.macos().clone();
    let choice = state.read(cx);
    let searchable = choice.searchable;
    let focus = choice.popup_focus.clone();
    let query = choice.query.clone();
    let visible = choice.visible(cx);
    let scroll = choice.scroll.clone();
    let popup_width = choice.popup_width;
    if crate::native::is_glass_popup(window) {
        let first = window.use_keyed_state("choice-native-focus", cx, |_, _| true);
        if *first.read(cx) {
            first.update(cx, |first, _| *first = false);
            if searchable {
                query.update(cx, |query, cx| query.focus(window, cx));
            } else {
                focus.focus(window, cx);
            }
        }
    }
    let close = state.clone();
    let confirm = state.clone();
    let mut content = div()
        .id("choice-popup")
        .debug_selector(|| "macos-choice-popup".into())
        .track_focus(&focus)
        .w(popup_width)
        .max_w(gpui_kit::relative(1.))
        .p(rems(0.375))
        .border_1()
        .rounded(rems(0.75))
        .border_color(t.glass_border())
        .shadow(t.popover_shadow())
        .bg(t.surface)
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .flex()
        .flex_col()
        .gap(rems(0.375))
        .on_mouse_down_out(move |_, window, cx| {
            close.update(cx, |state, cx| {
                state.set_open(false, window, cx);
            })
        })
        .on_action(move |_: &gpui_kit::base::actions::Confirm, window, cx| {
            cx.stop_propagation();
            confirm.update(cx, |state, cx| state.commit(window, cx));
        });
    if searchable {
        content = content.child(
            input("choice-search", &query, window, cx)
                .h(rems(1.5))
                .py(rems(0.))
                .px(rems(0.375)),
        );
    }
    let rows = div()
        .id("choice-options")
        .role(gpui_kit::Role::ListBox)
        .flex()
        .flex_col();
    let rows = if visible.is_empty() {
        rows.child(
            div()
                .p(rems(0.625))
                .text_color(t.secondary)
                .child("No matching options"),
        )
    } else {
        let render = state.clone();
        let count = visible.len();
        rows.child(
            gpui_kit::uniform_list("choice-visible-options", count, move |range, window, cx| {
                range
                    .map(|row| choice_option(&render, visible[row], window, cx))
                    .collect()
            })
            .w_full()
            .h(rems((count as f32 * 1.5).min(14.)))
            .track_scroll(&scroll),
        )
    };
    if crate::native::is_glass_popup(window) {
        content = content
            .bg(gpui_kit::transparent_black())
            .border_0()
            .shadow_none();
    }
    choice_keys(content.child(rows), state, searchable)
}

fn choice_option(
    state: &Entity<ChoiceState>,
    index: usize,
    window: &Window,
    cx: &App,
) -> crate::Button {
    let choice = state.read(cx);
    let item = &choice.items[index];
    let cursor = choice.cursor;
    let selected = choice.selected;
    let t = cx.macos();
    let cursor_fill = crate::controls::control_selection(t, window.is_window_active());
    let cursor_text = crate::controls::control_mark(t, window.is_window_active());
    let target = state.clone();
    let hover = state.clone();
    button(("choice-option", index), "", ButtonVariant::Secondary, cx)
        .paint_role(crate::state::PaintRole::Custom)
        .fill_duration(std::time::Duration::ZERO)
        .accessibility_label(item.label.clone())
        .debug_selector(move || format!("macos-choice-option-{index}"))
        .role(gpui_kit::Role::ListBoxOption)
        .aria_selected(selected == Some(index))
        .when(cursor == Some(index), |row| row.aria_active_descendant())
        .focusable(false)
        .disabled(item.disabled)
        .w_full()
        .h(rems(1.5))
        .py(rems(0.))
        .px(rems(0.375))
        .gap(rems(0.375))
        .justify_start()
        .rounded(rems(0.375))
        .border_color(t.foreground.opacity(0.))
        .shadow(Vec::new())
        .bg(if cursor == Some(index) {
            cursor_fill
        } else {
            t.foreground.opacity(0.)
        })
        .text_color(if cursor == Some(index) {
            cursor_text
        } else {
            t.foreground
        })
        .hover(|s| s.bg(cursor_fill).text_color(cursor_text))
        .active(|s| s.bg(cursor_fill).text_color(cursor_text))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_ellipsis()
                .child(item.label.clone()),
        )
        .child(
            div()
                .w(rems(0.875))
                .flex_shrink_0()
                .when(selected == Some(index), |mark| {
                    mark.child(icon(IconName::Check).size(rems(0.875)))
                }),
        )
        .on_hover(move |hovered, window, cx| {
            if *hovered {
                hover.update(cx, |state, cx| {
                    if !state.items[index].disabled && state.cursor != Some(index) {
                        state.cursor = Some(index);
                        cx.notify();
                        window.refresh();
                    }
                });
            }
        })
        .on_click(move |_, window, cx| {
            target.update(cx, |state, cx| {
                state.cursor = Some(index);
                state.commit(window, cx);
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::px;
    use gpui_kit::{Render, TestAppContext, VisualTestContext};
    struct Harness {
        state: Entity<ChoiceState>,
        searchable: bool,
    }
    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            crate::focus_scope("root")
                .size_full()
                .child(button("before", "Before", ButtonVariant::Secondary, cx))
                .child(div().w(rems(17.5)).child(if self.searchable {
                    combobox("choice", &self.state, window, cx).into_any_element()
                } else {
                    select("choice", &self.state, window, cx).into_any_element()
                }))
                .child(button("after", "After", ButtonVariant::Secondary, cx))
        }
    }
    fn harness(
        cx: &mut TestAppContext,
        searchable: bool,
        disabled: bool,
    ) -> (Entity<ChoiceState>, &mut VisualTestContext) {
        harness_with_items(
            cx,
            searchable,
            disabled,
            vec![
                ChoiceItem::new("first", "First"),
                ChoiceItem::new("disabled", "Disabled").disabled(true),
                ChoiceItem::new("last", "Last"),
            ],
        )
    }

    fn harness_with_items(
        cx: &mut TestAppContext,
        searchable: bool,
        disabled: bool,
        items: Vec<ChoiceItem>,
    ) -> (Entity<ChoiceState>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            let state = cx.new(|cx| {
                ChoiceState::new(items, window, cx)
                    .default_selected(0)
                    .disabled(disabled)
            });
            cx.observe(&state, |_, _, cx| cx.notify()).detach();
            Harness { state, searchable }
        });
        let state = view.read_with(cx, |view, _| view.state.clone());
        cx.update(|window, cx| {
            state.read(cx).trigger_focus.clone().focus(window, cx);
            window.draw(cx).clear(cx);
        });
        (state, cx)
    }
    #[gpui_kit::test]
    fn select_skips_disabled_commits_and_restores_focus(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, false, false);
        cx.simulate_keystrokes("down");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(state.read(cx).open);
        });
        cx.simulate_keystrokes("down enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "last");
            assert!(!state.read(cx).open);
            assert!(state.read(cx).trigger_focus.is_focused(window));
        });
    }
    #[gpui_kit::test]
    fn select_vim_navigation_skips_disabled_and_wraps(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, false, false);
        cx.simulate_keystrokes("enter j");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(2)));
        cx.simulate_keystrokes("j");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(0)));
        cx.simulate_keystrokes("k enter");
        cx.update(|window, cx| {
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "last");
            assert!(!state.read(cx).open);
            assert!(state.read(cx).trigger_focus.is_focused(window));
        });
    }

    #[gpui_kit::test]
    fn combobox_vim_navigation_preserves_search_input(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, true, false);
        cx.simulate_keystrokes("enter ctrl-j");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(2)));
        cx.simulate_keystrokes("ctrl-k");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(0)));
        cx.simulate_keystrokes("ctrl-n");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(2)));
        cx.simulate_keystrokes("ctrl-p");
        cx.update(|_, cx| assert_eq!(state.read(cx).cursor, Some(0)));
        cx.simulate_keystrokes("j k");
        cx.update(|_, cx| {
            let choice = state.read(cx);
            assert_eq!(choice.query.read(cx).value().as_ref(), "jk");
            assert!(choice.visible(cx).is_empty());
            assert_eq!(choice.cursor, None);
        });
        cx.simulate_keystrokes("ctrl-j enter");
        cx.update(|_, cx| assert!(state.read(cx).open));
    }

    #[gpui_kit::test]
    fn combobox_filters_and_empty_result_cannot_commit(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, true, false);
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(state.read(cx).open);
        });
        cx.simulate_input("zzzz");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(state.read(cx).visible(cx).is_empty());
        });
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert!(state.read(cx).open));
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "first");
            assert!(state.read(cx).trigger_focus.is_focused(window));
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_input("LAST");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| {
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "last");
            assert!(!state.read(cx).open);
        });
    }
    #[gpui_kit::test]
    fn disabled_choice_does_not_open(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, false, true);
        cx.simulate_keystrokes("down enter space");
        cx.simulate_click(
            gpui_kit::point(px(10.), px(10.)),
            gpui_kit::Modifiers::default(),
        );
        cx.update(|_, cx| assert!(!state.read(cx).open));
    }
    #[gpui_kit::test]
    fn tab_leaves_combobox_without_focusing_hidden_search(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, true, false);
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let choice = state.read(cx);
            assert!(!choice.open);
            assert!(!choice.trigger_focus.is_focused(window));
            assert!(
                !choice.query.read(cx).focus_handle(cx).is_focused(window),
                "Tab must leave the closed popup's input"
            );
            assert!(window.focused(cx).is_some());
        });
        cx.simulate_keystrokes("shift-tab");
        cx.update(|window, cx| assert!(state.read(cx).trigger_focus.is_focused(window)));
    }

    #[gpui_kit::test]
    fn pointer_selects_and_outside_click_preserves_value(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, false, false);
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        let row = cx.debug_bounds("macos-choice-option-2").unwrap();
        cx.simulate_click(row.center(), gpui_kit::Modifiers::default());
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(!state.read(cx).open);
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "last");
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_click(
            gpui_kit::point(px(400.), px(400.)),
            gpui_kit::Modifiers::default(),
        );
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(!state.read(cx).open);
            assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "last");
        });
    }
    #[gpui_kit::test]
    fn keyboard_cursor_scrolls_long_list_into_view(cx: &mut TestAppContext) {
        let (state, cx) = harness_with_items(
            cx,
            false,
            false,
            (0..30)
                .map(|i| ChoiceItem::new(i.to_string(), format!("Workspace {i}")))
                .collect(),
        );
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("end");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        let last = cx.debug_bounds("macos-choice-option-29").unwrap();
        let popup = cx.debug_bounds("macos-choice-popup").unwrap();
        assert!(popup.contains(&last.center()));
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "29"));
    }

    #[gpui_kit::test]
    fn large_choices_render_visible_rows_and_reveal_the_keyboard_cursor(cx: &mut TestAppContext) {
        for searchable in [false, true] {
            let (state, cx) = harness_with_items(
                cx,
                searchable,
                false,
                (0..10_000)
                    .map(|i| ChoiceItem::new(i.to_string(), format!("Workspace {i}")))
                    .collect(),
            );
            cx.simulate_keystrokes("enter");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert!(cx.debug_bounds("macos-choice-option-0").is_some());
            assert!(
                cx.debug_bounds("macos-choice-option-5000").is_none(),
                "Offscreen options must not be built"
            );
            cx.simulate_keystrokes(if searchable { "ctrl-p" } else { "end" });
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let last = cx.debug_bounds("macos-choice-option-9999").unwrap();
            assert!(
                cx.debug_bounds("macos-choice-popup")
                    .unwrap()
                    .contains(&last.center())
            );
            cx.simulate_keystrokes("enter");
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "9999");
                assert!(!state.read(cx).open);
                assert!(state.read(cx).trigger_focus.is_focused(window));
            });
            cx.simulate_keystrokes("enter");
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert!(
                cx.debug_bounds("macos-choice-popup").unwrap().contains(
                    &cx.debug_bounds("macos-choice-option-9999")
                        .unwrap()
                        .center()
                ),
                "Reopening reveals the selected option"
            );
        }
    }

    #[gpui_kit::test]
    fn choice_filter_cache_tracks_query_and_control_mode(cx: &mut TestAppContext) {
        let (state, cx) = harness(cx, true, false);
        cx.simulate_keystrokes("enter");
        cx.simulate_input("  LAST  ");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let filtered = state.read(cx).visible(cx);
            assert_eq!(filtered.as_slice(), &[2]);
            assert!(
                Rc::ptr_eq(&filtered, &state.read(cx).visible(cx)),
                "Unchanged queries reuse the filtered indices"
            );
            state.update(cx, |state, _| state.searchable = false);
            assert_eq!(state.read(cx).visible(cx).as_slice(), &[0, 1, 2]);
            state.update(cx, |state, _| state.searchable = true);
            assert_eq!(state.read(cx).visible(cx).as_slice(), &[2]);
        });
        cx.simulate_keystrokes("escape enter");
        cx.update(|_, cx| assert_eq!(state.read(cx).visible(cx).as_slice(), &[0, 1, 2]));
    }

    #[gpui_kit::test]
    fn clearing_a_search_reveals_the_selected_option_in_a_large_combobox(cx: &mut TestAppContext) {
        let (state, cx) = harness_with_items(
            cx,
            true,
            false,
            (0..10_000)
                .map(|i| ChoiceItem::new(i.to_string(), format!("Workspace {i}")))
                .collect(),
        );
        cx.update(|_, cx| state.update(cx, |state, cx| state.set_selected(Some(9999), cx)));
        cx.simulate_keystrokes("enter");
        cx.simulate_input("Workspace 5");
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("cmd-a backspace");
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(cx.update(|_, cx| state.read(cx).cursor), Some(9999));
        assert!(
            cx.debug_bounds("macos-choice-popup").unwrap().contains(
                &cx.debug_bounds("macos-choice-option-9999")
                    .expect("Selected option must be rendered")
                    .center()
            )
        );
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert_eq!(state.read(cx).selected().unwrap().value.as_ref(), "9999"));
    }
}
