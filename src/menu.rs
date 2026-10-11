//! A keyboard menu composed from the base Popover and Button primitives.
use crate::{ActiveTheme, ButtonVariant, IconName, button, icon, keystroke_label};
use gpui_kit::base::{ElementExt, Popover, Popup};
use gpui_kit::rems;
use gpui_kit::{
    Action, Anchor, AnyElement, App, Bounds, ElementId, Entity, Focusable, KeyDownEvent,
    ParentElement, Pixels, Point, SharedString, Window, anchored, deferred, div, point, prelude::*,
    px,
};
use std::{rc::Rc, time::Duration};

/// What a row does when chosen, shared by both panels and every handler.
type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Row height, and the gap that follows it, in rems.
const ROW_HEIGHT: f32 = 1.75;
const ROW_GAP: f32 = 0.125;
/// A separator costs its padding on both sides plus its own hairline.
const SEPARATOR_HEIGHT: f32 = 0.5625;
const MENU_PADDING: f32 = 0.375;
const SUBMENU_WIDTH: f32 = 10.625;
/// Space between the menu and its submenu, in rems.
const SUBMENU_GAP: f32 = 0.25;

#[derive(Clone)]
pub struct MenuItem {
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub shortcut: Option<SharedString>,
    /// Shows the keymap's binding for this action, taking precedence over `shortcut`.
    pub action: Option<Rc<dyn Action>>,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub separator_before: bool,
    /// Child rows, each paired with the index reported to `on_select`.
    pub children: Vec<(usize, MenuItem)>,
}
impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: None,
            action: None,
            disabled: false,
            checked: None,
            separator_before: false,
            children: Vec::new(),
        }
    }
    /// Open a second panel beside this row instead of selecting it.
    ///
    /// Each child carries the index `on_select` receives, so a submenu choice
    /// is reported like any other item and the parent needs no index of its own.
    pub fn submenu(mut self, children: Vec<(usize, MenuItem)>) -> Self {
        self.children = children;
        self
    }
    /// Display an exclusive choice with its check on the trailing edge.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    pub fn separator_before(mut self) -> Self {
        self.separator_before = true;
        self
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    /// Show the shortcut bound to `action` in the keymap, so the label always
    /// matches the platform and any user rebinding. Display only: selection is
    /// still reported through `on_select`.
    pub fn action(mut self, action: impl Action) -> Self {
        self.action = Some(Rc::new(action));
        self
    }
    /// Native menu shortcut text: the action's binding, else the literal shortcut.
    fn render_shortcut(&self, window: &Window, cx: &App) -> Option<AnyElement> {
        self.action
            .as_ref()
            .and_then(|action| window.highest_precedence_binding_for_action(action.as_ref()))
            .map(|binding| {
                binding
                    .keystrokes()
                    .iter()
                    .map(|stroke| keystroke_label(stroke.modifiers(), stroke.key()).to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .into()
            })
            .or_else(|| self.shortcut.clone())
            .map(|label: SharedString| {
                div()
                    .font_family(cx.macos().font.clone())
                    .text_size(rems(0.75))
                    .opacity(0.72)
                    .child(label)
                    .into_any_element()
            })
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Compact action menu. Main command-launcher menus can override row geometry separately.
/// Base positions the popup, dismisses outside/Escape and restores trigger focus.
pub fn menu(
    id: impl Into<ElementId>,
    trigger: impl gpui_kit::base::Selectable
    + IntoElement
    + gpui_kit::StatefulInteractiveElement
    + 'static,
    items: Vec<MenuItem>,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> Popover {
    let items = Rc::new(items);
    let on_select = Rc::new(on_select);
    Popover::new(id)
        .trigger_clickable(trigger)
        // The submenu floats outside the menu's bounds, so dismissal on an
        // outside press is handled below where both panels are known.
        .overlay_closable(false)
        .content_factory(move |popover, window, cx| {
            if crate::native::is_glass_popup(window) {
                let first = window.use_keyed_state("menu-native-focus", cx, |_, _| true);
                if *first.read(cx) {
                    first.update(cx, |first, _| *first = false);
                    popover.focus_handle(cx).focus(window, cx);
                }
            }
            let t = cx.macos().clone();
            let cursor = window.use_keyed_state("menu-cursor", cx, |_, _| {
                items.iter().position(|item| !item.disabled)
            });
            let focus = window
                .use_keyed_state("menu-focus", cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            if popover.focus_handle(cx).is_focused(window) {
                focus.focus(window, cx);
            }
            // The open submenu, and the row highlighted inside it.
            let page = window.use_keyed_state("menu-page", cx, |_, _| None::<usize>);
            let child_cursor = window.use_keyed_state("submenu-cursor", cx, |_, _| None::<usize>);
            // Last painted bounds of each panel, used to place the submenu
            // beside the menu and to tell presses inside it from outside ones.
            let menu_bounds = window.use_keyed_state("menu-bounds", cx, |_, _| Bounds::default());
            let submenu_bounds =
                window.use_keyed_state("submenu-bounds", cx, |_, _| None::<Bounds<Pixels>>);

            // Selection is one decision for both panels: open a submenu, or
            // dismiss and report. Every code path below routes through it, so
            // opening a submenu never closes the menu.
            let displayed = items.clone();
            let leaf_select = on_select.clone();
            let select_page = page.clone();
            let select_child = child_cursor.clone();
            let select_close = cx.entity();
            let command_owner = popover.owner_window();
            let on_select: Select = Rc::new(move |index, window, cx| {
                if let Some(parent) = *select_page.read(cx)
                    && let Some(child) = *select_child.read(cx)
                {
                    if let Some((action, item)) = displayed[parent].children.get(child)
                        && !item.disabled
                    {
                        select_close.update(cx, |state, cx| state.dismiss(window, cx));
                        dispatch_selection(command_owner, leaf_select.clone(), *action, window, cx);
                    }
                    return;
                }
                let Some(item) = displayed.get(index) else {
                    return;
                };
                if !item.children.is_empty() {
                    select_page.update(cx, |page, cx| {
                        *page = Some(index);
                        cx.notify();
                    });
                    select_child.update(cx, |child, cx| {
                        *child = item.children.iter().position(|(_, item)| !item.disabled);
                        cx.notify();
                    });
                    window.refresh();
                } else if !item.disabled {
                    select_close.update(cx, |state, cx| state.dismiss(window, cx));
                    dispatch_selection(command_owner, leaf_select.clone(), index, window, cx);
                }
            });

            let has_icons = items.iter().any(|item| item.icon.is_some());
            let open_page = *page.read(cx);
            let keyboard_items = items.clone();
            let keyboard_page = page.clone();
            let keyboard_child = child_cursor.clone();
            let keyboard_cursor = cursor.clone();
            let keyboard_select = on_select.clone();
            let confirm_cursor = cursor.clone();
            let confirm_select = on_select.clone();
            let submenu_items = items.clone();
            let submenu_cursor = child_cursor.clone();
            let submenu_page = page.clone();
            let submenu_select = on_select.clone();
            let main = div()
                .id("menu-items")
                .debug_selector(|| "macos-menu-content".into())
                .role(gpui_kit::Role::Menu)
                .track_focus(&focus)
                .w(rems(15.))
                .p(rems(MENU_PADDING))
                .flex()
                .flex_col()
                .gap(rems(ROW_GAP))
                .border_1()
                .rounded(rems(0.75))
                .border_color(t.glass_border())
                .bg(if crate::native::is_glass_popup(window) {
                    gpui_kit::transparent_black()
                } else {
                    let mut fill = t.glass_fill();
                    fill.a = 1.;
                    fill
                })
                .shadow(t.popover_shadow())
                .when(crate::native::is_glass_popup(window), |menu| {
                    menu.border_0().shadow_none()
                })
                .text_color(t.foreground)
                .font_family(t.font.clone())
                .text_size(rems(0.8125))
                .on_action(move |_: &gpui_kit::base::actions::Confirm, window, cx| {
                    cx.stop_propagation();
                    if let Some(index) = *confirm_cursor.read(cx) {
                        confirm_select(index, window, cx);
                        window.refresh();
                    }
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let key = event.keystroke.key.as_str();
                    let current = *keyboard_cursor.read(cx);
                    if key == "left" && keyboard_page.read(cx).is_some() {
                        keyboard_page.update(cx, |page, cx| {
                            *page = None;
                            cx.notify();
                        });
                        keyboard_child.update(cx, |child, cx| {
                            *child = None;
                            cx.notify();
                        });
                        window.refresh();
                    } else if key == "right" {
                        if keyboard_page.read(cx).is_none()
                            && let Some(index) =
                                current.filter(|&i| !keyboard_items[i].children.is_empty())
                        {
                            keyboard_select(index, window, cx);
                        }
                    } else if matches!(key, "enter" | "space") {
                        if let Some(index) = current.filter(|&i| !keyboard_items[i].disabled) {
                            keyboard_select(index, window, cx);
                            window.refresh();
                        }
                    } else if matches!(key, "down" | "j" | "up" | "k" | "home" | "end") {
                        // While a submenu is open the arrows belong to it.
                        if let Some(parent) = *keyboard_page.read(cx) {
                            let enabled: Vec<_> = keyboard_items[parent]
                                .children
                                .iter()
                                .enumerate()
                                .filter_map(|(i, (_, item))| (!item.disabled).then_some(i))
                                .collect();
                            let next = next_item(&enabled, *keyboard_child.read(cx), key);
                            keyboard_child.update(cx, |child, cx| {
                                *child = next;
                                cx.notify();
                            });
                            window.refresh();
                            cx.stop_propagation();
                            return;
                        }
                        let enabled: Vec<_> = keyboard_items
                            .iter()
                            .enumerate()
                            .filter_map(|(i, item)| (!item.disabled).then_some(i))
                            .collect();
                        let next = next_item(&enabled, current, key);
                        keyboard_cursor.update(cx, |cursor, cx| {
                            *cursor = next;
                            cx.notify();
                        });
                        window.refresh();
                    } else {
                        return;
                    }
                    cx.stop_propagation();
                })
                .children(items.iter().enumerate().map(|(index, item)| {
                    let disabled = item.disabled;
                    let hover_cursor = cursor.clone();
                    let hover_page = page.clone();
                    let hover_child = child_cursor.clone();
                    let item_has_children = !item.children.is_empty();
                    let click_page = page.clone();
                    let click_child = child_cursor.clone();
                    let current = *cursor.read(cx) == Some(index);
                    let select = on_select.clone();
                    let shortcut = item.render_shortcut(window, cx);
                    let row = button(("menu-item", index), "", ButtonVariant::Secondary, cx)
                        .debug_selector(move || format!("macos-menu-item-{index}"))
                        .accessibility_label(item.label.clone())
                        .role(gpui_kit::Role::MenuItem)
                        .when_some(item.checked, |row, checked| {
                            row.role(gpui_kit::Role::MenuItemRadio)
                                .aria_toggled(if checked {
                                    gpui_kit::accesskit::Toggled::True
                                } else {
                                    gpui_kit::accesskit::Toggled::False
                                })
                        })
                        .focusable(false)
                        .disabled(item.disabled)
                        .fill_duration(Duration::ZERO)
                        .w_full()
                        .h(rems(ROW_HEIGHT))
                        .py(rems(0.))
                        .px(rems(0.5))
                        .rounded(rems(0.375))
                        .border_0()
                        .shadow_none()
                        .justify_start()
                        .bg(if current {
                            t.accent
                        } else {
                            t.foreground.opacity(0.)
                        })
                        .text_color(if current { t.on_accent } else { t.foreground })
                        .hover(|s| s.bg(t.accent).text_color(t.on_accent))
                        .active(|s| s.bg(t.accent).text_color(t.on_accent))
                        .when(has_icons, |row| {
                            row.child(
                                div()
                                    .w(rems(0.875))
                                    .flex_shrink_0()
                                    .when_some(item.icon, |slot, name| {
                                        slot.child(icon(name).size(rems(0.875)))
                                    }),
                            )
                        })
                        .child(div().flex_1().min_w_0().child(item.label.clone()))
                        .when(item_has_children, |row| {
                            row.child(icon(IconName::ChevronRight).size(rems(0.875)))
                        })
                        .when_some(item.checked, |row, checked| {
                            row.child(div().w(rems(0.875)).when(checked, |slot| {
                                slot.child(icon(IconName::Check).size(rems(0.875)))
                            }))
                        })
                        .children(shortcut)
                        .on_hover(move |hovered, window, cx| {
                            if *hovered && !disabled {
                                hover_page.update(cx, |page, cx| {
                                    *page = item_has_children.then_some(index);
                                    cx.notify();
                                });
                                hover_child.update(cx, |child, cx| {
                                    *child = None;
                                    cx.notify();
                                });
                                hover_cursor.update(cx, |cursor, cx| {
                                    *cursor = Some(index);
                                    cx.notify();
                                });
                                window.refresh();
                            }
                        })
                        .on_click(move |_, window, cx| {
                            // A pointer selects the row under it, not whatever
                            // the keyboard left highlighted in a submenu.
                            click_page.update(cx, |page, _| *page = None);
                            click_child.update(cx, |child, _| *child = None);
                            select(index, window, cx);
                            window.refresh();
                        });
                    div()
                        .when(item.separator_before, |group| {
                            group.child(div().py(rems(0.25)).child(crate::separator(cx)))
                        })
                        .child(row)
                }));
            let dismiss_bounds = submenu_bounds.clone();
            let dismiss = cx.entity();
            if open_page.is_none() {
                submenu_bounds.update(cx, |slot, _| *slot = None);
            }
            let submenu = open_page.map(|parent| {
                let rem = window.rem_size();
                let native = crate::native::is_glass_popup(window);
                // Native children may extend beyond their owning popup's
                // viewport. Resolve against the display in owner-local points;
                // the child presenter translates those points to screen space.
                let viewport = if native {
                    window.display(cx).map(|display| {
                        let display = display.bounds();
                        Bounds::new(display.origin - window.bounds().origin, display.size)
                    })
                } else {
                    None
                }
                .unwrap_or_else(|| Bounds::new(point(px(0.), px(0.)), window.viewport_size()));
                let (anchor, origin) = submenu_placement(
                    *menu_bounds.read(cx),
                    rems(submenu_offset(&submenu_items, parent)).to_pixels(rem),
                    rems(SUBMENU_WIDTH).to_pixels(rem),
                    rems(SUBMENU_GAP).to_pixels(rem),
                    viewport.right(),
                );
                if native {
                    return native_submenu(
                        parent,
                        SubmenuPosition {
                            anchor,
                            origin,
                            viewport,
                        },
                        submenu_items[parent].children.clone(),
                        submenu_cursor,
                        submenu_page,
                        submenu_select,
                    )
                    .into_any_element();
                }
                let record_submenu = submenu_bounds.clone();
                deferred(
                    anchored()
                        .anchor(anchor)
                        .position(origin)
                        .snap_to_window_with_margin(px(4.))
                        .child(
                            // Measured on this unpadded wrapper: the probe
                            // `on_prepaint` adds is absolutely positioned, so on
                            // the padded panel it would be inset.
                            div()
                                .on_prepaint(move |bounds, _, cx| {
                                    record_submenu.update(cx, |slot, _| *slot = Some(bounds))
                                })
                                .child(submenu_panel(
                                    &submenu_items[parent].children,
                                    parent,
                                    submenu_cursor,
                                    submenu_page,
                                    submenu_select,
                                    window,
                                    cx,
                                )),
                        ),
                )
                // Above the popover surface itself, which paints at POPUP_PRIORITY.
                .with_priority(gpui_kit::base::POPUP_PRIORITY + 1)
                .into_any_element()
            });
            div()
                .on_mouse_down_out(move |_, window, cx| {
                    let inside_submenu = dismiss_bounds
                        .read(cx)
                        .is_some_and(|bounds| bounds.contains(&window.mouse_position()));
                    if !inside_submenu {
                        dismiss.update(cx, |state, cx| state.dismiss(window, cx));
                        window.refresh();
                    }
                })
                // Measured on this unpadded wrapper for the same reason.
                .on_prepaint(move |bounds, _, cx| menu_bounds.update(cx, |slot, _| *slot = bounds))
                .child(main)
                .children(submenu)
        })
}

/// Window commands belong to the original trigger's window even when the
/// pointer selected a row in a separate native popup or submenu window.
fn dispatch_selection(
    owner: Option<gpui_kit::AnyWindowHandle>,
    select: Select,
    index: usize,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(owner) = owner
        && owner.window_id() != window.window_handle().window_id()
    {
        cx.defer(move |cx| {
            let _ = owner.update(cx, |_, window, cx| {
                select(index, window, cx);
                window.refresh();
            });
        });
    } else {
        select(index, window, cx);
    }
}

/// A retained factory lets Base present the submenu in its own child panel.
/// The invisible trigger supplies a corner rather than a second visual row.
/// Keyboard navigation remains in the primary menu, sharing cursor entities.
struct SubmenuPosition {
    anchor: Anchor,
    origin: Point<Pixels>,
    viewport: Bounds<Pixels>,
}

fn native_submenu(
    parent: usize,
    position: SubmenuPosition,
    items: Vec<(usize, MenuItem)>,
    cursor: Entity<Option<usize>>,
    page: Entity<Option<usize>>,
    select: Select,
) -> Popup {
    let dismiss_page = page.clone();
    let dismiss_cursor = cursor.clone();
    Popup::new(("submenu-popup", parent), div().size_0())
        .absolute()
        .left(position.origin.x)
        .top(position.origin.y)
        .size_0()
        .anchor(position.anchor)
        .viewport_bounds(position.viewport)
        .focus(false)
        .content_factory(move |window, cx| {
            submenu_panel(
                &items,
                parent,
                cursor.clone(),
                page.clone(),
                select.clone(),
                window,
                cx,
            )
            .into_any_element()
        })
        .on_dismiss(move |window, cx| {
            // Closing an old panel after hovering another parent must not close
            // the replacement submenu.
            if *dismiss_page.read(cx) == Some(parent) {
                dismiss_page.update(cx, |page, cx| {
                    *page = None;
                    cx.notify();
                });
                dismiss_cursor.update(cx, |cursor, cx| {
                    *cursor = None;
                    cx.notify();
                });
                window.refresh();
            }
        })
}

/// Where the submenu opens: beside the menu, level with the row that opened
/// it. It prefers the right and flips to the left when it would run past the
/// viewport, the way a desktop menu does. `top` is the opening row's offset
/// from the top of the menu.
fn submenu_placement(
    menu: Bounds<Pixels>,
    top: Pixels,
    width: Pixels,
    gap: Pixels,
    viewport_width: Pixels,
) -> (Anchor, Point<Pixels>) {
    let top = menu.top() + top;
    if menu.right() + gap + width <= viewport_width {
        (Anchor::TopLeft, point(menu.right() + gap, top))
    } else {
        (Anchor::TopRight, point(menu.left() - gap, top))
    }
}

/// The floating panel listing a submenu's rows. It shares the menu's cursor
/// model: `cursor` is the highlighted row, and choosing a row routes through
/// `select` with the parent index so both panels make one decision.
fn submenu_panel(
    items: &[(usize, MenuItem)],
    parent: usize,
    cursor: Entity<Option<usize>>,
    page: Entity<Option<usize>>,
    select: Select,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let t = cx.macos().clone();
    let has_icons = items.iter().any(|(_, item)| item.icon.is_some());
    let highlighted = *cursor.read(cx);
    div()
        .id("submenu")
        .debug_selector(|| "macos-submenu-content".into())
        .role(gpui_kit::Role::Menu)
        .occlude()
        .w(rems(SUBMENU_WIDTH))
        .p(rems(MENU_PADDING))
        .flex()
        .flex_col()
        .gap(rems(ROW_GAP))
        .border_1()
        .rounded(rems(0.75))
        .border_color(t.glass_border())
        .bg(if crate::native::is_glass_popup(window) {
            gpui_kit::transparent_black()
        } else {
            t.glass_fill()
        })
        .shadow(t.popover_shadow())
        .when(crate::native::is_glass_popup(window), |menu| {
            menu.border_0().shadow_none()
        })
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .children(items.iter().enumerate().map(|(index, (_, item))| {
            let hover_cursor = cursor.clone();
            let click_cursor = cursor.clone();
            let click_page = page.clone();
            let select = select.clone();
            let disabled = item.disabled;
            button(("submenu-item", index), "", ButtonVariant::Secondary, cx)
                .debug_selector(move || format!("macos-submenu-item-{index}"))
                .accessibility_label(item.label.clone())
                .role(gpui_kit::Role::MenuItem)
                .when_some(item.checked, |row, checked| {
                    row.role(gpui_kit::Role::MenuItemRadio)
                        .aria_toggled(if checked {
                            gpui_kit::accesskit::Toggled::True
                        } else {
                            gpui_kit::accesskit::Toggled::False
                        })
                })
                .focusable(false)
                .disabled(disabled)
                .fill_duration(Duration::ZERO)
                .w_full()
                .h(rems(ROW_HEIGHT))
                .py(rems(0.))
                .px(rems(0.5))
                .rounded(rems(0.375))
                .border_0()
                .shadow_none()
                .justify_start()
                .bg(if highlighted == Some(index) {
                    t.accent
                } else {
                    t.foreground.opacity(0.)
                })
                .text_color(if highlighted == Some(index) {
                    t.on_accent
                } else {
                    t.foreground
                })
                .hover(|s| s.bg(t.accent).text_color(t.on_accent))
                .active(|s| s.bg(t.accent).text_color(t.on_accent))
                .when(has_icons, |row| {
                    row.child(
                        div()
                            .w(rems(0.875))
                            .flex_shrink_0()
                            .when_some(item.icon, |slot, name| {
                                slot.child(icon(name).size(rems(0.875)))
                            }),
                    )
                })
                .child(div().flex_1().min_w_0().child(item.label.clone()))
                .when_some(item.checked, |row, checked| {
                    row.child(div().w(rems(0.875)).when(checked, |slot| {
                        slot.child(icon(IconName::Check).size(rems(0.875)))
                    }))
                })
                .children(item.render_shortcut(window, cx))
                .on_hover(move |hovered, window, cx| {
                    if *hovered && !disabled {
                        hover_cursor.update(cx, |cursor, cx| {
                            *cursor = Some(index);
                            cx.notify();
                        });
                        window.refresh();
                    }
                })
                .on_click(move |_, window, cx| {
                    click_page.update(cx, |page, _| *page = Some(parent));
                    click_cursor.update(cx, |cursor, _| *cursor = Some(index));
                    select(parent, window, cx);
                    window.refresh();
                })
        }))
}

/// How far the submenu sits below the top of the menu, so its first row lines
/// up with the parent row that opened it. Both panels share the same border
/// and padding, so only the rows above the parent count.
fn submenu_offset(items: &[MenuItem], parent: usize) -> f32 {
    let separator = |item: &MenuItem| {
        if item.separator_before {
            SEPARATOR_HEIGHT
        } else {
            0.
        }
    };
    items
        .iter()
        .take(parent)
        .map(|item| ROW_HEIGHT + ROW_GAP + separator(item))
        .sum::<f32>()
        + items.get(parent).map_or(0., separator)
}

fn next_item(enabled: &[usize], current: Option<usize>, key: &str) -> Option<usize> {
    if enabled.is_empty() {
        return None;
    }
    let position = current.and_then(|i| enabled.iter().position(|&item| item == i));
    let index = match key {
        "home" => 0,
        "end" => enabled.len() - 1,
        "up" | "k" => position.map_or(enabled.len() - 1, |i| {
            (i + enabled.len() - 1) % enabled.len()
        }),
        _ => position.map_or(0, |i| (i + 1) % enabled.len()),
    };
    Some(enabled[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::base::{PopupContentFactory, PopupPresenter};
    use gpui_kit::{AppContext, Context, Render, TestAppContext, VisualTestContext, size};
    use std::cell::RefCell;

    struct SubmenuContent(PopupContentFactory);

    impl Render for SubmenuContent {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)(window, cx)
        }
    }

    struct NativeSubmenuHarness {
        page: Entity<Option<usize>>,
        cursor: Entity<Option<usize>>,
        commands: Rc<RefCell<Vec<(usize, gpui_kit::AnyWindowHandle)>>>,
    }

    impl Render for NativeSubmenuHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let owner = window.window_handle();
            let commands = self.commands.clone();
            let select: Select = Rc::new(move |index, window, cx| {
                let commands = commands.clone();
                dispatch_selection(
                    Some(owner),
                    Rc::new(move |index, window, _| {
                        commands.borrow_mut().push((index, window.window_handle()));
                    }),
                    index,
                    window,
                    cx,
                );
            });
            div().size_full().child(native_submenu(
                2,
                SubmenuPosition {
                    anchor: Anchor::TopLeft,
                    origin: point(px(244.), px(40.)),
                    viewport: Bounds::new(point(px(-600.), px(-400.)), size(px(1800.), px(1200.))),
                },
                vec![(20, MenuItem::new("PDF")), (21, MenuItem::new("Image"))],
                self.cursor.clone(),
                self.page.clone(),
                select,
            ))
        }
    }

    #[gpui_kit::test]
    fn native_submenu_uses_a_separate_panel_and_dispatches_to_original_owner(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let pane = Rc::new(RefCell::new(
            None::<crate::native::GlassPanelHandle<SubmenuContent>>,
        ));
        let resolved = Rc::new(RefCell::new(Vec::new()));
        let pane_for_hook = pane.clone();
        let observed = resolved.clone();
        cx.update(move |cx| {
            cx.set_global(PopupPresenter(Rc::new(move |request, owner, cx| {
                observed.borrow_mut().push(request.bounds);
                assert!(
                    !request.focus,
                    "Submenu keeps keyboard navigation in the main menu"
                );
                if pane_for_hook.borrow().is_none() {
                    let dismiss = request.on_dismiss;
                    let panel = crate::native::open_glass_panel(
                        crate::native::GlassPanelOptions::new(request.bounds),
                        owner,
                        cx,
                        |_, cx| cx.new(|_| SubmenuContent(request.content)),
                        move |owner, cx| {
                            if let Some(dismiss) = &dismiss {
                                dismiss(owner, cx);
                            }
                        },
                    )
                    .unwrap();
                    *pane_for_hook.borrow_mut() = Some(panel);
                }
                true
            })));
        });
        let commands = Rc::new(RefCell::new(Vec::new()));
        let (view, owner_cx) = cx.add_window_view({
            let commands = commands.clone();
            move |window, cx| {
                window.resize(size(px(240.), px(160.)));
                NativeSubmenuHarness {
                    page: cx.new(|_| Some(2)),
                    cursor: cx.new(|_| None),
                    commands,
                }
            }
        });
        let owner_handle = owner_cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.window_handle()
        });
        let bounds = resolved.borrow()[0];
        assert_eq!(bounds.origin, point(px(244.), px(40.)));
        assert!(
            bounds.left() > px(240.),
            "Child is outside the small owner's viewport"
        );
        assert!(owner_cx.debug_bounds("macos-submenu-content").is_none());
        let child_handle = pane.borrow().as_ref().unwrap().window_handle();
        let mut child_cx = VisualTestContext::from_window(child_handle, owner_cx);
        child_cx.update(|window, cx| window.draw(cx).clear(cx));
        let target = child_cx
            .debug_bounds("macos-submenu-item-1")
            .unwrap()
            .center();
        child_cx.simulate_click(target, Default::default());
        child_cx.run_until_parked();
        assert_eq!(&*commands.borrow(), &[(2, owner_handle)]);
        assert_eq!(
            view.read_with(&child_cx, |view, cx| *view.cursor.read(cx)),
            Some(1)
        );
        child_cx.update(|window, _| window.remove_window());
        child_cx.run_until_parked();
        view.read_with(&child_cx, |view, cx| {
            assert_eq!(*view.page.read(cx), None);
            assert_eq!(*view.cursor.read(cx), None);
        });
    }

    #[test]
    fn navigation_skips_disabled_and_wraps() {
        assert_eq!(next_item(&[0, 2], Some(0), "down"), Some(2));
        assert_eq!(next_item(&[0, 2], Some(2), "down"), Some(0));
        assert_eq!(next_item(&[0, 2], Some(0), "up"), Some(2));
        assert_eq!(next_item(&[], None, "home"), None);
    }

    #[test]
    fn submenu_opens_right_and_flips_left_at_the_viewport_edge() {
        let menu = Bounds::new(point(px(100.), px(40.)), gpui_kit::size(px(240.), px(160.)));
        let open = |viewport| submenu_placement(menu, px(30.), px(170.), px(4.), viewport);

        // Room on the right: the submenu's left edge sits past the menu.
        assert_eq!(open(px(800.)), (Anchor::TopLeft, point(px(344.), px(70.))));
        // Exactly enough room still opens to the right.
        assert_eq!(open(px(514.)).0, Anchor::TopLeft);
        // One pixel short: its right edge sits before the menu instead.
        assert_eq!(open(px(513.)), (Anchor::TopRight, point(px(96.), px(70.))));
    }

    #[test]
    fn submenu_lines_up_with_the_row_that_opened_it() {
        let items = vec![
            MenuItem::new("first"),
            MenuItem::new("second").separator_before(),
            MenuItem::new("third"),
        ];
        assert_eq!(submenu_offset(&items, 0), 0.);
        assert_eq!(
            submenu_offset(&items, 1),
            ROW_HEIGHT + ROW_GAP + SEPARATOR_HEIGHT
        );
        assert_eq!(
            submenu_offset(&items, 2),
            2. * (ROW_HEIGHT + ROW_GAP) + SEPARATOR_HEIGHT
        );
    }

    #[test]
    fn submenu_children_carry_their_own_action_index() {
        let item = MenuItem::new("Theme").submenu(vec![
            (20, MenuItem::new("System").checked(true)),
            (21, MenuItem::new("Light")),
        ]);
        assert_eq!(item.children.len(), 2);
        assert_eq!(item.children[0].0, 20);
        assert_eq!(item.children[1].0, 21);
    }
}
