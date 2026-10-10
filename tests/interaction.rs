extern crate gpui_kit as gpui;

use gpui_kit::rems;
use gpui_kit::{
    Context, IntoElement, Modifiers, Render, TestAppContext, Window, div, point, prelude::*, px,
};
use gpui_macos::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Harness {
    clicks: Rc<Cell<usize>>,
    disabled: bool,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        div().id("root").tab_group().size(rems(12.5)).child(
            button("apply", "Apply", ButtonVariant::Primary, cx)
                .w(rems(7.5))
                .disabled(self.disabled)
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
        )
    }
}

#[gpui_kit::test]
fn styled_button_accepts_pointer_return_and_space(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let clicks = Rc::new(Cell::new(0));
    let handle = clicks.clone();
    let (_, cx) = cx.add_window_view(move |_, _| Harness {
        clicks: handle,
        disabled: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    assert_eq!(clicks.get(), 1);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for key in ["enter", "space"] {
        let keystroke = gpui_kit::Keystroke::parse(key).unwrap();
        cx.simulate_event(gpui_kit::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
    }
    assert_eq!(clicks.get(), 3);
}

#[gpui_kit::test]
fn disabled_styled_button_cannot_activate(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let clicks = Rc::new(Cell::new(0));
    let handle = clicks.clone();
    let (_, cx) = cx.add_window_view(move |_, _| Harness {
        clicks: handle,
        disabled: true,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    cx.update(|window, cx| window.focus_next(cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for key in ["enter", "space"] {
        let keystroke = gpui_kit::Keystroke::parse(key).unwrap();
        cx.simulate_event(gpui_kit::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
    }
    assert_eq!(clicks.get(), 0);
}

#[gpui_kit::test]
fn applying_theme_updates_base_tokens_and_geometry(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        Theme::light().apply(cx);
        let base = gpui_kit::base::Theme::global(cx);
        assert_eq!(base.tokens.colors, cx.macos().tokens());
        assert_eq!(base.appearance, gpui_kit::base::ThemeAppearance::Light);
        for (radius, expected) in [
            (base.tokens.radius.none, 0.),
            (base.tokens.radius.sm, 4.),
            (base.tokens.radius.md, 6.),
            (base.tokens.radius.lg, 10.),
            (base.tokens.radius.xl, 16.),
            (base.tokens.radius.full, 9999.),
        ] {
            assert_eq!(radius, px(expected));
        }
        assert_eq!(base.tokens.typography.md.size, px(13.));
        assert_eq!(base.tokens.typography.sans.as_ref(), ".SystemUIFont");
        for theme in [Theme::dark(), Theme::light()] {
            let style = theme
                .input_style()
                .resolved(&gpui_kit::base::SemanticThemeTokens {
                    colors: theme.tokens(),
                    ..Default::default()
                });
            assert_eq!(style.selection, theme.selection);
            assert_eq!(style.caret, theme.accent);
            assert_ne!(style.selection, style.background);
        }
        Theme::dark().apply(cx);
        assert_eq!(
            gpui_kit::base::Theme::global(cx).tokens.colors,
            cx.macos().tokens()
        );
    });
}

struct MenuHarness(Rc<Cell<usize>>);
impl Render for MenuHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.0.clone();
        div().size(rems(18.75)).child(menu(
            "actions",
            button("trigger", "Actions", ButtonVariant::Secondary, cx),
            vec![
                MenuItem::new("First"),
                MenuItem::new("Disabled").disabled(true),
                MenuItem::new("Last"),
            ],
            move |index, _, _| selected.set(index),
        ))
    }
}
#[gpui_kit::test]
fn menu_keyboard_skips_disabled_and_selects(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let selected = Rc::new(Cell::new(usize::MAX));
    let result = selected.clone();
    let (_, cx) = cx.add_window_view(move |_, _| MenuHarness(result));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        cx.debug_bounds("macos-menu-content").is_some(),
        "menu opened"
    );
    cx.simulate_keystrokes("down enter");
    assert_eq!(selected.get(), 2);
}

struct SubmenuHarness(Rc<Cell<usize>>);
impl Render for SubmenuHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.0.clone();
        div().size(rems(37.5)).child(menu(
            "actions",
            button("trigger", "Actions", ButtonVariant::Secondary, cx),
            vec![
                MenuItem::new("First"),
                MenuItem::new("Export").submenu(vec![
                    (10, MenuItem::new("PDF")),
                    (11, MenuItem::new("Markdown")),
                ]),
                MenuItem::new("Last"),
            ],
            move |index, _, _| selected.set(index),
        ))
    }
}
#[gpui_kit::test]
fn submenu_opens_beside_its_row_and_reports_child_index(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let selected = Rc::new(Cell::new(usize::MAX));
    let result = selected.clone();
    let (_, cx) = cx.add_window_view(move |_, _| SubmenuHarness(result));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let closed = cx.debug_bounds("macos-menu-content").expect("menu opened");
    assert!(cx.debug_bounds("macos-submenu-content").is_none());

    cx.simulate_keystrokes("down right");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let main = cx.debug_bounds("macos-menu-content").unwrap();
    let submenu = cx
        .debug_bounds("macos-submenu-content")
        .expect("submenu opened");
    let parent = cx.debug_bounds("macos-menu-item-1").unwrap();
    let child = cx.debug_bounds("macos-submenu-item-0").unwrap();
    assert_eq!(
        main.origin, closed.origin,
        "opening a submenu must not move the menu"
    );
    assert!(
        submenu.left() >= main.right(),
        "submenu should sit to the right of the menu: submenu {submenu:?}, menu {main:?}"
    );
    assert!(
        (child.top() - parent.top()).abs() <= px(0.5),
        "first submenu row should line up with its parent row: child {child:?}, parent {parent:?}"
    );

    cx.simulate_keystrokes("enter");
    assert_eq!(selected.get(), 10);
}

#[gpui_kit::test]
fn submenu_rows_take_pointer_clicks_and_outside_presses_dismiss(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let selected = Rc::new(Cell::new(usize::MAX));
    let result = selected.clone();
    let (_, cx) = cx.add_window_view(move |_, _| SubmenuHarness(result));
    let open_submenu = |cx: &mut gpui_kit::VisualTestContext| {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("down right");
        cx.update(|window, cx| window.draw(cx).clear(cx));
    };

    // The submenu floats outside the menu's bounds; pressing it must not read
    // as an outside press that closes the menu before the click lands.
    open_submenu(cx);
    let child = cx.debug_bounds("macos-submenu-item-1").unwrap().center();
    cx.simulate_click(child, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(selected.get(), 11);
    assert!(cx.debug_bounds("macos-menu-content").is_none());

    // A press outside both panels dismisses without selecting.
    selected.set(usize::MAX);
    open_submenu(cx);
    let submenu = cx.debug_bounds("macos-submenu-content").unwrap();
    cx.simulate_click(
        point(submenu.right() + px(20.), submenu.bottom() + px(20.)),
        Modifiers::default(),
    );
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(cx.debug_bounds("macos-menu-content").is_none());
    assert!(cx.debug_bounds("macos-submenu-content").is_none());
    assert_eq!(selected.get(), usize::MAX);
}

/// A menu pinned to the window's right edge, like a title bar menu.
struct EdgeSubmenuHarness(Rc<Cell<usize>>);
impl Render for EdgeSubmenuHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.0.clone();
        div().size_full().flex().items_start().justify_end().child(
            menu(
                "actions",
                button("trigger", "Actions", ButtonVariant::Secondary, cx)
                    .debug_selector(|| "edge-trigger".into()),
                vec![
                    MenuItem::new("First"),
                    MenuItem::new("Export").submenu(vec![
                        (10, MenuItem::new("PDF")),
                        (11, MenuItem::new("Markdown")),
                    ]),
                ],
                move |index, _, _| selected.set(index),
            )
            .anchor(gpui_kit::Anchor::TopRight),
        )
    }
}
#[gpui_kit::test]
fn submenu_flips_left_when_the_menu_meets_the_window_edge(cx: &mut TestAppContext) {
    cx.update(gpui_macos::init);
    let selected = Rc::new(Cell::new(usize::MAX));
    let result = selected.clone();
    let (_, cx) = cx.add_window_view(move |_, _| EdgeSubmenuHarness(result));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = cx.debug_bounds("edge-trigger").unwrap().center();
    cx.simulate_click(trigger, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let closed = cx.debug_bounds("macos-menu-content").expect("menu opened");

    cx.simulate_keystrokes("down right");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let main = cx.debug_bounds("macos-menu-content").unwrap();
    let submenu = cx
        .debug_bounds("macos-submenu-content")
        .expect("submenu opened");
    let parent = cx.debug_bounds("macos-menu-item-1").unwrap();
    let child = cx.debug_bounds("macos-submenu-item-0").unwrap();
    assert_eq!(
        main.origin, closed.origin,
        "opening a submenu must not move the menu"
    );
    assert!(
        submenu.right() <= main.left(),
        "submenu should flip to the left of the menu: submenu {submenu:?}, menu {main:?}"
    );
    assert!(
        (child.top() - parent.top()).abs() <= px(0.5),
        "first submenu row should line up with its parent row: child {child:?}, parent {parent:?}"
    );

    cx.simulate_click(child.center(), Modifiers::default());
    assert_eq!(selected.get(), 10);
}

#[gpui_kit::test]
fn links_open_urls_and_disabled_links_remain_inert(cx: &mut TestAppContext) {
    struct Links;
    impl Render for Links {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().flex().flex_col().children([
                link(
                    "enabled",
                    "Project",
                    "https://github.com/huacnlee/gpui-macos",
                    cx,
                )
                .debug_selector(|| "enabled-link".into()),
                link(
                    "disabled",
                    "Unavailable",
                    "https://example.com/disabled",
                    cx,
                )
                .disabled(true)
                .debug_selector(|| "disabled-link".into()),
            ])
        }
    }
    cx.update(gpui_macos::init);
    let (_, cx) = cx.add_window_view(|_, _| Links);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let disabled = cx.debug_bounds("disabled-link").unwrap().center();
    cx.simulate_click(disabled, Modifiers::default());
    assert_eq!(cx.opened_url(), None);
    let enabled = cx.debug_bounds("enabled-link").unwrap().center();
    cx.simulate_click(enabled, Modifiers::default());
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://github.com/huacnlee/gpui-macos")
    );
}

#[derive(Clone, Copy)]
enum ActivationPopup {
    Popover,
    Menu,
}

struct PopupActivationHarness {
    popup: ActivationPopup,
    disabled: bool,
    trigger: gpui_kit::FocusHandle,
    changes: Rc<RefCell<Vec<bool>>>,
    selections: Rc<Cell<usize>>,
    selected: Rc<Cell<usize>>,
}

impl Render for PopupActivationHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger = button(
            "keyboard-trigger",
            "Open options",
            ButtonVariant::Secondary,
            cx,
        )
        .accessibility_label("Open options")
        .debug_selector(|| "keyboard-trigger".into())
        .track_focus(&self.trigger)
        .disabled(self.disabled);
        let popup = match self.popup {
            ActivationPopup::Popover => popover("keyboard-popup", trigger, |_, _, _| {
                div()
                    .debug_selector(|| "keyboard-popover-body".into())
                    .child("Additional options")
            }),
            ActivationPopup::Menu => {
                let selected = self.selected.clone();
                let selections = self.selections.clone();
                menu(
                    "keyboard-popup",
                    trigger,
                    vec![
                        MenuItem::new("First"),
                        MenuItem::new("Unavailable").disabled(true),
                        MenuItem::new("Last"),
                    ],
                    move |index, _, _| {
                        selected.set(index);
                        selections.set(selections.get() + 1);
                    },
                )
            }
        };
        let changes = self.changes.clone();
        div()
            .id("activation-root")
            .tab_group()
            .size_full()
            .child(popup.on_open_change(move |open, _, _| changes.borrow_mut().push(*open)))
    }
}

fn isolate_trigger_keyboard_click(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        // The normal Popover Confirm binding can open on key-down before
        // Button emits its keyboard click on key-up. Mask only that enclosing
        // binding so this regression specifically exercises trigger_clickable,
        // the click-listener route used by non-pointer activation.
        cx.bind_keys([
            gpui_kit::KeyBinding::new("enter", gpui_kit::NoAction, Some("Popover")),
            gpui_kit::KeyBinding::new("space", gpui_kit::NoAction, Some("Popover")),
        ]);
    });
}

fn key_step(cx: &mut gpui_kit::VisualTestContext, key: &str, down: bool) {
    let keystroke = gpui_kit::Keystroke::parse(key).unwrap();
    if down {
        cx.simulate_event(gpui_kit::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        });
    } else {
        cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
    }
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

#[gpui_kit::test]
fn popover_trigger_keyboard_click_opens_once_and_escape_restores_focus(cx: &mut TestAppContext) {
    isolate_trigger_keyboard_click(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let (view, cx) = cx.add_window_view({
        let changes = changes.clone();
        move |_, cx| PopupActivationHarness {
            popup: ActivationPopup::Popover,
            disabled: false,
            trigger: cx.focus_handle(),
            changes,
            selections: Rc::new(Cell::new(0)),
            selected: Rc::new(Cell::new(usize::MAX)),
        }
    });
    cx.update(|window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
        window.draw(cx).clear(cx);
    });
    for key in ["enter", "space"] {
        let before = changes.borrow().len();
        key_step(cx, key, true);
        assert_eq!(
            changes.borrow().len(),
            before,
            "no enclosing Confirm action"
        );
        key_step(cx, key, false);
        assert_eq!(
            &changes.borrow()[before..],
            &[true],
            "one keyboard ClickEvent opens once"
        );
        assert!(cx.debug_bounds("keyboard-popover-body").is_some());
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(view.read(cx).trigger.is_focused(window));
        });
        assert!(cx.debug_bounds("keyboard-popover-body").is_none());
        assert_eq!(&changes.borrow()[before..], &[true, false]);
    }
}

#[gpui_kit::test]
fn menu_trigger_keyboard_click_opens_once_and_selection_closes(cx: &mut TestAppContext) {
    isolate_trigger_keyboard_click(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let selected = Rc::new(Cell::new(usize::MAX));
    let selections = Rc::new(Cell::new(0));
    let (view, cx) = cx.add_window_view({
        let changes = changes.clone();
        let selected = selected.clone();
        let selections = selections.clone();
        move |_, cx| PopupActivationHarness {
            popup: ActivationPopup::Menu,
            disabled: false,
            trigger: cx.focus_handle(),
            changes,
            selected,
            selections,
        }
    });
    cx.update(|window, cx| {
        view.read(cx).trigger.clone().focus(window, cx);
        window.draw(cx).clear(cx);
    });
    key_step(cx, "space", true);
    assert!(changes.borrow().is_empty());
    key_step(cx, "space", false);
    assert_eq!(*changes.borrow(), vec![true]);
    assert!(cx.debug_bounds("macos-menu-content").is_some());
    cx.simulate_keystrokes("down enter");
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert!(view.read(cx).trigger.is_focused(window));
    });
    assert_eq!(selected.get(), 2, "keyboard navigation skips disabled row");
    assert_eq!(selections.get(), 1, "selection is delivered once");
    assert!(cx.debug_bounds("macos-menu-content").is_none());
    assert_eq!(*changes.borrow(), vec![true, false]);
}

#[gpui_kit::test]
fn disabled_popup_triggers_reject_keyboard_click_and_pointer_activation(cx: &mut TestAppContext) {
    isolate_trigger_keyboard_click(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let (view, cx) = cx.add_window_view({
        let changes = changes.clone();
        move |_, cx| PopupActivationHarness {
            popup: ActivationPopup::Menu,
            disabled: true,
            trigger: cx.focus_handle(),
            changes,
            selected: Rc::new(Cell::new(usize::MAX)),
            selections: Rc::new(Cell::new(0)),
        }
    });
    cx.update(|window, cx| {
        // Even a stale programmatic focus handle cannot activate a disabled
        // trigger; normal tab navigation already omits this control.
        view.read(cx).trigger.clone().focus(window, cx);
        window.draw(cx).clear(cx);
    });
    for key in ["enter", "space"] {
        key_step(cx, key, true);
        key_step(cx, key, false);
    }
    let trigger = cx.debug_bounds("keyboard-trigger").unwrap().center();
    cx.simulate_click(trigger, Modifiers::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(changes.borrow().is_empty());
    assert!(cx.debug_bounds("macos-menu-content").is_none());
    cx.update(|_, cx| assert_eq!(view.read(cx).selections.get(), 0));
}
