// Modified by gpui-macos contributors in 2026: render-time control state children and default styles.
// Original: longbridge/gpui-kit 0bbd9870ce420af9feacbf948b21d3824d82dc0b.
// Copyright 2024-2026 Longbridge; Apache-2.0. See LICENSE-APACHE and MACOS-PATCH.md.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, MouseButton, ParentElement, Refineable as _, RenderOnce, Role, SharedString,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Toggled, Window, div,
    prelude::FluentBuilder as _, relative,
};
use smallvec::SmallVec;

use crate::{StateStyle, StyledExt as _, TestSupportExt as _};

type ChangeHandler = Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App)>;

/// An unstyled, controlled toggle button.
///
/// This primitive owns activation, focus, and accessibility behavior. The
/// application owns all layout, visual states, sizes, and variants.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    base: Stateful<Div>,
    style: StyleRefinement,
    semantic_styles: ToggleStyles,
    pressed: bool,
    disabled: bool,
    children: SmallVec<[crate::state_child::StateChild; 2]>,
    on_change: Option<ChangeHandler>,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
    tab_stop: bool,
    tracked_focus: Option<FocusHandle>,
    default_styles: Option<crate::state_child::DefaultStyles>,
    interaction: crate::state_child::CallerInteraction,
}

/// Semantic root styles supported by [`Toggle`].
#[derive(Default)]
pub struct ToggleStyles {
    pressed: StyleRefinement,
    disabled: StyleRefinement,
}

impl ToggleStyles {
    pub fn pressed(mut self, build: impl FnOnce(StateStyle) -> StateStyle) -> Self {
        self.pressed
            .refine(&build(StateStyle::default()).into_refinement());
        self
    }

    pub fn disabled(mut self, build: impl FnOnce(StateStyle) -> StateStyle) -> Self {
        self.disabled
            .refine(&build(StateStyle::default()).into_refinement());
        self
    }
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            base: div().id(id.clone()),
            id,
            style: StyleRefinement::default(),
            semantic_styles: ToggleStyles::default(),
            pressed: false,
            disabled: false,
            children: SmallVec::new(),
            on_change: None,
            accessibility_label: None,
            tab_index: 0,
            tab_stop: true,
            tracked_focus: None,
            default_styles: None,
            interaction: Default::default(),
        }
    }

    /// Supplies render-time default styles from the final control state.
    /// `checked` is the final pressed value. Caller styles, including
    /// semantic `styles`, `hover` and `active`, take precedence; default hover
    /// and active styles are dropped while disabled.
    pub fn default_styles(
        mut self,
        build: impl FnOnce(crate::ControlState, &mut Window, &mut App) -> crate::StateStyles + 'static,
    ) -> Self {
        self.default_styles = Some(Box::new(build));
        self
    }

    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Adds a child built with the final disabled state, so it can present
    /// disabled colors even when `disabled` is set after it is added.
    pub fn child_with_disabled<E: IntoElement>(
        self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.child_with_state(move |state| build(state.disabled))
    }

    /// Adds a child built with the final render-time control state.
    /// `focus_visible` is currently false; focus producers are added separately.
    pub fn child_with_state<E: IntoElement>(
        mut self,
        build: impl FnOnce(crate::ControlState) -> E + 'static,
    ) -> Self {
        self.children
            .push(crate::state_child::StateChild::state(build));
        self
    }

    /// Configures application-owned styles for the toggle's semantic states.
    pub fn styles(mut self, build: impl FnOnce(ToggleStyles) -> ToggleStyles) -> Self {
        self.semantic_styles = build(self.semantic_styles);
        self
    }

    fn resolved_style(&self) -> StyleRefinement {
        crate::state_style::resolve_style(
            &self.style,
            [
                self.pressed.then_some(&self.semantic_styles.pressed),
                self.disabled.then_some(&self.semantic_styles.disabled),
            ]
            .into_iter()
            .flatten(),
        )
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Handles a request to change the controlled pressed state.
    pub fn on_change(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.tracked_focus = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone()
    }
}

impl Styled for Toggle {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Toggle {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(
            elements
                .into_iter()
                .map(crate::state_child::StateChild::Ready),
        );
    }
}

impl InteractiveElement for Toggle {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }

    /// Captured so render-time default styles can sit beneath it.
    fn hover(mut self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.interaction.hover = Some(f(StyleRefinement::default()));
        self
    }
}

impl StatefulInteractiveElement for Toggle {
    /// Captured so render-time default styles can sit beneath it.
    fn active(mut self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.interaction.active = Some(f(StyleRefinement::default()));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self
            .tracked_focus
            .clone()
            .unwrap_or_else(|| self.focus_handle(window, cx));
        let pressed = self.pressed;
        let disabled = self.disabled;
        let control_state = crate::ControlState::new(disabled, pressed);
        let styles = crate::state_child::resolve_styles(
            self.default_styles.take(),
            control_state,
            self.resolved_style(),
            std::mem::take(&mut self.interaction),
            window,
            cx,
        );
        let on_change = self.on_change;

        self.base
            .test_support()
            .role(Role::Button)
            // Match Button's neutral control geometry: a fixed-size toggle
            // centers ordinary content, while callers still own its size,
            // spacing and visual treatment.
            .flex()
            .items_center()
            .justify_center()
            .line_height(relative(1.))
            .aria_toggled(if pressed {
                Toggled::True
            } else {
                Toggled::False
            })
            .when_some(self.accessibility_label, |this, label| {
                this.aria_label(label)
            })
            .when(!disabled, |this| {
                this.track_focus(
                    &focus_handle
                        .tab_index(self.tab_index)
                        .tab_stop(self.tab_stop),
                )
            })
            .when(disabled, |this| {
                this.on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
            })
            .when_some(
                (!disabled).then_some(on_change).flatten(),
                |this, on_change| {
                    this.on_click(move |event, window, cx| {
                        on_change(!pressed, event, window, cx);
                    })
                },
            )
            .when_some(styles.hover, |this, style| this.hover(move |_| style))
            .when_some(styles.active, |this, style| this.active(move |_| style))
            .children(
                self.children
                    .into_iter()
                    .map(|child| child.resolve(control_state)),
            )
            .refine_style(&styles.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ElementExt as _;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use gpui::{
        Context, Element as _, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, accesskit, canvas, point, px,
    };

    struct Harness {
        pressed: bool,
        disabled: bool,
        changes: Rc<RefCell<Vec<bool>>>,
        keyboard_changes: Rc<Cell<usize>>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let keyboard_changes = self.keyboard_changes.clone();
            Toggle::new("toggle")
                .pressed(self.pressed)
                .disabled(self.disabled)
                .size(px(100.))
                .on_change(move |pressed, event, _, _| {
                    changes.borrow_mut().push(pressed);
                    if matches!(event, ClickEvent::Keyboard(_)) {
                        keyboard_changes.set(keyboard_changes.get() + 1);
                    }
                })
        }
    }

    fn harness(
        cx: &mut TestAppContext,
        pressed: bool,
        disabled: bool,
    ) -> (
        &mut VisualTestContext,
        Rc<RefCell<Vec<bool>>>,
        Rc<Cell<usize>>,
    ) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let keyboard_changes = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let changes = changes.clone();
            let keyboard_changes = keyboard_changes.clone();
            move |_, _| Harness {
                pressed,
                disabled,
                changes,
                keyboard_changes,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cx, changes, keyboard_changes)
    }

    #[gpui::test]
    fn pointer_requests_inverse_controlled_state_once(cx: &mut TestAppContext) {
        for (pressed, expected) in [(false, true), (true, false)] {
            let (cx, changes, _) = harness(cx, pressed, false);
            cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
            assert_eq!(changes.borrow().as_slice(), &[expected]);
        }
    }

    #[gpui::test]
    fn enter_and_space_use_one_native_keyboard_click_each(cx: &mut TestAppContext) {
        let (cx, changes, keyboard_changes) = harness(cx, false, false);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        changes.borrow_mut().clear();
        cx.update(|window, cx| window.draw(cx).clear(cx));

        for key in ["enter", "space"] {
            let keystroke = Keystroke::parse(key).unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke });
        }
        assert_eq!(changes.borrow().as_slice(), &[true, true]);
        assert_eq!(keyboard_changes.get(), 2);
    }

    #[gpui::test]
    fn state_children_receive_the_final_state(cx: &mut TestAppContext) {
        struct View(Rc<RefCell<Vec<crate::ControlState>>>);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let seen = self.0.clone();
                div().children([false, true].map(|disabled| {
                    let state = seen.clone();
                    let flag = seen.clone();
                    Toggle::new(("toggle-state", disabled as usize))
                        .child_with_state(move |control| {
                            state.borrow_mut().push(control);
                            div()
                        })
                        .child_with_disabled(move |disabled| {
                            flag.borrow_mut()
                                .push(crate::ControlState::new(disabled, false));
                            div()
                        })
                        .child("Label")
                        // Set after the children were added.
                        .disabled(disabled)
                }))
            }
        }
        let seen = Rc::new(RefCell::new(Vec::new()));
        let view_seen = seen.clone();
        let (_, cx) = cx.add_window_view(move |_, _| View(view_seen));
        seen.borrow_mut().clear();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let seen = seen.borrow();
        assert_eq!(
            seen.iter().map(|s| s.disabled).collect::<Vec<_>>(),
            [false, false, true, true]
        );
        assert!(!seen[0].focus_visible && !seen[2].focus_visible);
    }

    #[gpui::test]
    fn default_styles_use_final_state_beneath_caller_styles(cx: &mut TestAppContext) {
        use crate::state_child::{CallerInteraction, resolve_styles};
        use gpui::{Hsla, hsla};
        let red: Hsla = hsla(0., 1., 0.5, 1.);
        let blue: Hsla = hsla(0.6, 1., 0.5, 1.);
        let green: Hsla = hsla(0.3, 1., 0.5, 1.);
        let defaults = move |state: crate::ControlState| {
            crate::StateStyles::new(StyleRefinement::default().bg(red).text_color(red))
                .hover(StyleRefinement::default().bg(green))
                .active(
                    StyleRefinement::default()
                        .bg(if state.checked { blue } else { green })
                        .text_color(green),
                )
        };
        let resolve = |state, caller, interaction, window: &mut Window, cx: &mut App| {
            resolve_styles(
                Some(Box::new(move |state, _, _| defaults(state))),
                state,
                caller,
                interaction,
                window,
                cx,
            )
        };
        let (_, cx) = cx.add_window_view(|_, _| gpui::Empty);
        cx.update(|window, cx| {
            // A caller background wins at rest and while pressed or hovered;
            // the default label colour remains.
            let resolved = resolve(
                crate::ControlState::new(false, true),
                StyleRefinement::default().bg(blue),
                CallerInteraction::default(),
                window,
                cx,
            );
            assert_eq!(resolved.base.background, Some(blue.into()));
            assert_eq!(resolved.base.text.color, Some(red));
            assert_eq!(resolved.hover.unwrap().background, None);
            let active = resolved.active.unwrap();
            assert_eq!(active.background, None);
            assert_eq!(active.text.color, Some(green));
            // A caller label colour persists while pressed.
            let label = resolve(
                crate::ControlState::new(false, true),
                StyleRefinement::default().text_color(blue),
                CallerInteraction::default(),
                window,
                cx,
            );
            let active = label.active.unwrap();
            assert_eq!(active.background, Some(blue.into()));
            assert_eq!(active.text.color, None);
            // Caller hover / active replace the defaults; an empty one clears them.
            let replaced = resolve(
                crate::ControlState::new(false, true),
                StyleRefinement::default(),
                CallerInteraction {
                    hover: Some(StyleRefinement::default().text_color(blue)),
                    active: Some(StyleRefinement::default()),
                },
                window,
                cx,
            );
            let hover = replaced.hover.unwrap();
            assert_eq!(hover.background, None, "caller hover replaces");
            assert_eq!(hover.text.color, Some(blue));
            let active = replaced.active.unwrap();
            assert_eq!((active.background, active.text.color), (None, None));
            // Disabled drops the default transient styles but keeps the caller's.
            let disabled = resolve(
                crate::ControlState::new(true, false),
                StyleRefinement::default(),
                CallerInteraction {
                    hover: Some(StyleRefinement::default().text_color(blue)),
                    active: None,
                },
                window,
                cx,
            );
            assert_eq!(disabled.base.background, Some(red.into()));
            assert_eq!(disabled.hover.unwrap().text.color, Some(blue));
            assert!(disabled.active.is_none());
        });
    }

    #[gpui::test]
    fn default_styles_see_the_final_pressed_value(cx: &mut TestAppContext) {
        struct View(Rc<RefCell<Vec<crate::ControlState>>>);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let seen = self.0.clone();
                Toggle::new("final-value")
                    .pressed(false)
                    .default_styles(move |state, _, _| {
                        seen.borrow_mut().push(state);
                        crate::StateStyles::default()
                    })
                    // Value and disabled set after the default styles.
                    .pressed(true)
                    .disabled(true)
            }
        }
        let seen = Rc::new(RefCell::new(Vec::new()));
        let view_seen = seen.clone();
        let (_, cx) = cx.add_window_view(move |_, _| View(view_seen));
        seen.borrow_mut().clear();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(&*seen.borrow(), &[crate::ControlState::new(true, true)]);
    }

    #[gpui::test]
    fn disabled_toggle_is_inert(cx: &mut TestAppContext) {
        let (cx, changes, _) = harness(cx, false, true);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        cx.simulate_keystrokes("enter space");
        assert!(changes.borrow().is_empty());
    }

    #[gpui::test]
    fn fixed_height_toggle_centers_ordinary_child_geometry(cx: &mut TestAppContext) {
        type Captured = Arc<
            Mutex<(
                Option<gpui::Bounds<gpui::Pixels>>,
                Option<gpui::Bounds<gpui::Pixels>>,
            )>,
        >;

        struct AlignmentProbe(Captured);

        impl Render for AlignmentProbe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let root_capture = self.0.clone();
                let child_capture = self.0.clone();
                Toggle::new("alignment-toggle")
                    .w(px(120.))
                    .h(px(40.))
                    .child(
                        div()
                            .w(px(48.))
                            .h(px(12.))
                            .on_prepaint(move |bounds, _, _| {
                                child_capture.lock().unwrap().1 = Some(bounds);
                            }),
                    )
                    .on_prepaint(move |bounds, _, _| {
                        root_capture.lock().unwrap().0 = Some(bounds);
                    })
            }
        }

        let captured = Arc::new(Mutex::new((None, None)));
        let (_, context) = cx.add_window_view({
            let captured = captured.clone();
            move |_, _| AlignmentProbe(captured)
        });
        context.update(|window, cx| window.draw(cx).clear(cx));

        let (root, child) = *captured.lock().unwrap();
        assert_eq!(
            child.expect("child bounds").center(),
            root.expect("toggle bounds").center()
        );
    }

    #[test]
    fn state_styling_and_children_are_application_owned() {
        let _ = Toggle::new("styled")
            .child("Label")
            .styles(|styles| {
                styles
                    .pressed(|style| style.opacity(0.8))
                    .disabled(|style| style.opacity(0.5))
            })
            .hover(|style| style.opacity(0.9))
            .active(|style| style.opacity(0.8))
            .focus_visible(|style| style.opacity(0.7));
    }

    #[test]
    fn semantic_root_styles_follow_toggle_priority() {
        let styled = |toggle: Toggle| {
            toggle.styles(|styles| {
                styles
                    .pressed(|style| style.opacity(0.8))
                    .disabled(|style| style.opacity(0.5))
            })
        };

        assert_eq!(styled(Toggle::new("normal")).resolved_style().opacity, None);
        assert_eq!(
            styled(Toggle::new("pressed").pressed(true))
                .resolved_style()
                .opacity,
            Some(0.8)
        );
        assert_eq!(
            styled(Toggle::new("pressed-disabled").pressed(true).disabled(true))
                .resolved_style()
                .opacity,
            Some(0.5)
        );
        assert_eq!(
            styled(
                Toggle::new("state-over-instance")
                    .pressed(true)
                    .disabled(true)
                    .opacity(0.9),
            )
            .resolved_style()
            .opacity,
            Some(0.5)
        );
    }

    #[gpui::test]
    fn accessibility_exposes_button_role_toggled_state_and_action(cx: &mut TestAppContext) {
        type Captured = Arc<Mutex<Option<(accesskit::Node, accesskit::Node)>>>;
        struct Probe(Captured);
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let captured = self.0.clone();
                canvas(
                    move |_, window, cx| {
                        let mut info = |toggle: Toggle| {
                            let mut node = accesskit::Node::new(Role::Button);
                            toggle
                                .render(window, cx)
                                .into_element()
                                .write_a11y_info(&mut node);
                            node
                        };
                        let enabled = info(
                            Toggle::new("enabled")
                                .pressed(true)
                                .accessibility_label("Bold")
                                .on_change(|_, _, _, _| {}),
                        );
                        let disabled = info(
                            Toggle::new("disabled")
                                .disabled(true)
                                .on_change(|_, _, _, _| {}),
                        );
                        *captured.lock().unwrap() = Some((enabled, disabled));
                    },
                    |_, _, _, _| {},
                )
            }
        }
        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Probe(captured));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled, disabled) = result.lock().unwrap().take().unwrap();
        assert_eq!(enabled.role(), Role::Button);
        assert_eq!(enabled.label(), Some("Bold"));
        assert_eq!(enabled.toggled(), Some(Toggled::True));
        assert!(enabled.supports_action(accesskit::Action::Click));
        assert_eq!(disabled.toggled(), Some(Toggled::False));
        assert!(!disabled.supports_action(accesskit::Action::Click));
    }
}
