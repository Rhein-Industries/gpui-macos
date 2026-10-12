// Modified by gpui-macos contributors in 2026: render-time control state children, default styles and keyboard focus visibility.
// Original: longbridge/gpui-kit 0bbd9870ce420af9feacbf948b21d3824d82dc0b.
// Copyright 2024-2026 Longbridge; Apache-2.0. See LICENSE-APACHE and MACOS-PATCH.md.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, MouseButton, ParentElement, Refineable as _, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
    relative,
};
use smallvec::SmallVec;

use crate::{StateStyle, StyledExt as _, TestSupportExt as _};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An unstyled tab that owns pointer activation and accessibility behavior.
///
/// Tabs do not participate in keyboard focus by themselves. A compound tab
/// list may add keyboard navigation when that behavior is introduced.
// TODO: Add compound keyboard navigation with roving focus, arrow keys,
// Home/End, and Enter/Space activation before treating Tabs as a complete
// desktop tab-list primitive.
#[derive(IntoElement)]
pub struct Tab {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    semantic_styles: TabStyles,
    selected: bool,
    disabled: bool,
    children: SmallVec<[crate::state_child::StateChild; 2]>,
    on_click: Option<ClickHandler>,
    accessibility_label: Option<SharedString>,
    position_in_set: Option<usize>,
    size_of_set: Option<usize>,
    default_styles: Option<crate::state_child::DefaultStyles>,
    interaction: crate::state_child::CallerInteraction,
    tracked_focus: Option<FocusHandle>,
}

impl Tab {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            semantic_styles: TabStyles::default(),
            selected: false,
            disabled: false,
            children: SmallVec::new(),
            on_click: None,
            accessibility_label: None,
            position_in_set: None,
            size_of_set: None,
            default_styles: None,
            interaction: Default::default(),
            tracked_focus: None,
        }
    }

    /// Supplies render-time default styles from the final control state.
    /// `checked` is the final selected value. Caller styles, including
    /// semantic `styles`, `hover` and `active`, take precedence; default hover
    /// and active styles are dropped while disabled.
    pub fn default_styles(
        mut self,
        build: impl FnOnce(crate::ControlState, &mut Window, &mut App) -> crate::StateStyles + 'static,
    ) -> Self {
        self.default_styles = Some(Box::new(build));
        self
    }

    /// Updates the element identity used when the tab is rendered.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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
    /// `focus_visible` is true while the control shows keyboard focus.
    pub fn child_with_state<E: IntoElement>(
        mut self,
        build: impl FnOnce(crate::ControlState) -> E + 'static,
    ) -> Self {
        self.children
            .push(crate::state_child::StateChild::state(build));
        self
    }

    /// Adds a child built with the final render-time control state and the
    /// control's final resolved style (defaults beneath caller styles), for
    /// example a focus ring that follows the control's radius and border.
    pub fn child_with_resolved_style<E: IntoElement>(
        mut self,
        build: impl FnOnce(crate::ControlState, &StyleRefinement) -> E + 'static,
    ) -> Self {
        self.children
            .push(crate::state_child::StateChild::styled(build));
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets this tab's one-based position and the tab list's total size, so
    /// assistive technology can announce "tab 2 of 5".
    pub fn set_position(mut self, position: usize, size: usize) -> Self {
        self.position_in_set = Some(position);
        self.size_of_set = Some(size);
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn styles(mut self, build: impl FnOnce(TabStyles) -> TabStyles) -> Self {
        self.semantic_styles = build(self.semantic_styles);
        self
    }

    fn resolved_style(&self) -> StyleRefinement {
        crate::state_style::resolve_style(
            &self.style,
            [
                self.selected.then_some(&self.semantic_styles.selected),
                self.disabled.then_some(&self.semantic_styles.disabled),
            ]
            .into_iter()
            .flatten(),
        )
    }
}

#[derive(Default)]
pub struct TabStyles {
    selected: StyleRefinement,
    disabled: StyleRefinement,
}

impl TabStyles {
    pub fn selected(mut self, build: impl FnOnce(StateStyle) -> StateStyle) -> Self {
        self.selected
            .refine(&build(StateStyle::default()).into_refinement());
        self
    }

    pub fn disabled(mut self, build: impl FnOnce(StateStyle) -> StateStyle) -> Self {
        self.disabled
            .refine(&build(StateStyle::default()).into_refinement());
        self
    }
}

impl Styled for Tab {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Tab {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(
            elements
                .into_iter()
                .map(crate::state_child::StateChild::Ready),
        );
    }
}

impl InteractiveElement for Tab {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }

    /// Captured so render-time default styles can sit beneath it.
    fn hover(mut self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.interaction.hover = Some(f(StyleRefinement::default()));
        self
    }

    /// Captured so `ControlState::focus_visible` can follow the handle; the
    /// tab still tracks it as before.
    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.tracked_focus = Some(focus_handle.clone());
        self
    }
}

impl StatefulInteractiveElement for Tab {
    /// Captured so render-time default styles can sit beneath it.
    fn active(mut self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.interaction.active = Some(f(StyleRefinement::default()));
        self
    }
}

impl RenderOnce for Tab {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let control_state = crate::ControlState::new(disabled, self.selected)
            .with_focus(self.tracked_focus.as_ref(), window);
        let styles = crate::state_child::resolve_styles(
            self.default_styles.take(),
            control_state,
            self.resolved_style(),
            std::mem::take(&mut self.interaction),
            window,
            cx,
        );

        self.base
            .when_some(self.tracked_focus, |this, focus| this.track_focus(&focus))
            .id(self.id)
            .test_support()
            .role(Role::Tab)
            // Match Button's neutral control geometry: a fixed-size tab
            // centers ordinary content, while callers still own its size,
            // spacing and visual treatment.
            .flex()
            .items_center()
            .justify_center()
            .line_height(relative(1.))
            .when_some(self.accessibility_label, |this, label| {
                this.aria_label(label)
            })
            .aria_selected(self.selected)
            .when_some(self.position_in_set, |this, position| {
                this.aria_position_in_set(position)
            })
            .when_some(self.size_of_set, |this, size| this.aria_size_of_set(size))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when_some(
                (!disabled).then_some(self.on_click).flatten(),
                |this, on_click| {
                    this.on_click(move |event, window, cx| on_click(event, window, cx))
                },
            )
            .when_some(styles.hover, |this, style| this.hover(move |_| style))
            .when_some(styles.active, |this, style| this.active(move |_| style))
            .children(
                self.children
                    .into_iter()
                    .map(|child| child.resolve(control_state, &styles.base)),
            )
            .refine_style(&styles.base)
    }
}

/// An unstyled collection root for [`Tab`] elements.
///
/// Controlled selection and activation are expressed directly by its child
/// tabs. The root supplies collection accessibility without introducing a
/// context hierarchy or new keyboard behavior.
#[derive(IntoElement)]
pub struct Tabs {
    base: gpui::Stateful<Div>,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 2]>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            style: StyleRefinement::default(),
            children: SmallVec::new(),
        }
    }
}

impl Styled for Tabs {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Tabs {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InteractiveElement for Tabs {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Tabs {}

impl RenderOnce for Tabs {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.base
            .role(Role::TabList)
            .children(self.children)
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ElementExt as _;
    use std::{
        cell::Cell,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use gpui::{
        Context, Element as _, Modifiers, Render, Role, VisualTestContext, accesskit, canvas, hsla,
        point, px,
    };

    struct TabHarness {
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for TabHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            Tab::new("tab")
                .disabled(self.disabled)
                .size(px(100.))
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
        }
    }

    fn harness(
        cx: &mut gpui::TestAppContext,
        disabled: bool,
    ) -> (&mut VisualTestContext, Rc<Cell<usize>>) {
        let clicks = Rc::new(Cell::new(0));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| TabHarness { disabled, clicks }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cx, clicks)
    }

    #[gpui::test]
    fn pointer_activation_and_disabled_gating_match_tabs(cx: &mut gpui::TestAppContext) {
        let (cx, clicks) = harness(cx, false);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        assert_eq!(clicks.get(), 1);

        let (cx, clicks) = harness(cx, true);
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
        assert_eq!(clicks.get(), 0);
    }

    #[gpui::test]
    fn state_children_receive_the_final_state(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct View(Rc<RefCell<Vec<crate::ControlState>>>);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let seen = self.0.clone();
                Tabs::new("tabs").children([false, true].map(|disabled| {
                    let state = seen.clone();
                    let flag = seen.clone();
                    Tab::new(("tab-state", disabled as usize))
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
    fn default_styles_see_the_final_selected_value(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct View(Rc<RefCell<Vec<crate::ControlState>>>);
        impl Render for View {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let seen = self.0.clone();
                Tab::new("final-value")
                    .default_styles(move |state, _, _| {
                        seen.borrow_mut().push(state);
                        crate::StateStyles::default()
                    })
                    .selected(true)
            }
        }
        let seen = Rc::new(RefCell::new(Vec::new()));
        let view_seen = seen.clone();
        let (_, cx) = cx.add_window_view(move |_, _| View(view_seen));
        seen.borrow_mut().clear();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(&*seen.borrow(), &[crate::ControlState::new(false, true)]);
    }

    #[gpui::test]
    fn fixed_height_tab_centers_ordinary_child_geometry(cx: &mut gpui::TestAppContext) {
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
                Tab::new("alignment-tab")
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
            root.expect("tab bounds").center()
        );
    }

    #[gpui::test]
    fn exposes_tab_accessibility_state(cx: &mut gpui::TestAppContext) {
        type Captured = Arc<Mutex<Option<(accesskit::Node, accesskit::Node)>>>;

        struct Probe(Captured);

        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let captured = self.0.clone();
                canvas(
                    move |_, window, cx| {
                        let mut info = |tab: Tab| {
                            let mut node = accesskit::Node::new(Role::Tab);
                            tab.render(window, cx)
                                .into_element()
                                .write_a11y_info(&mut node);
                            node
                        };
                        let selected = info(
                            Tab::new("selected")
                                .selected(true)
                                .accessibility_label("Account")
                                .on_click(|_, _, _| {}),
                        );
                        let disabled =
                            info(Tab::new("disabled").disabled(true).on_click(|_, _, _| {}));
                        *captured.lock().unwrap() = Some((selected, disabled));
                    },
                    |_, _, _, _| {},
                )
            }
        }

        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Probe(captured));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let (selected, disabled) = result.lock().unwrap().take().unwrap();

        assert_eq!(selected.role(), Role::Tab);
        assert_eq!(selected.label(), Some("Account"));
        assert_eq!(selected.is_selected(), Some(true));
        assert!(selected.supports_action(accesskit::Action::Click));
        assert!(!disabled.supports_action(accesskit::Action::Click));
    }

    #[gpui::test]
    fn semantic_styles_preserve_the_legacy_state_priority(_cx: &mut gpui::TestAppContext) {
        let expected = hsla(0.3, 0.4, 0.5, 1.0);
        let mut tab = Tab::new("tab")
            .selected(true)
            .disabled(true)
            .styles(|styles| {
                styles
                    .selected(|style| style.opacity(0.8))
                    .disabled(|style| style.opacity(0.5))
            })
            .opacity(0.9);

        assert_eq!(tab.resolved_style().opacity, Some(0.5));
        tab.style().background = Some(expected.into());
        assert_eq!(tab.resolved_style().background, Some(expected.into()));
    }

    #[gpui::test]
    fn tabs_exposes_tab_list_role(cx: &mut gpui::TestAppContext) {
        type Captured = Arc<Mutex<Option<accesskit::Node>>>;
        struct Probe(Captured);

        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let captured = self.0.clone();
                canvas(
                    move |_, window, cx| {
                        let mut node = accesskit::Node::new(Role::TabList);
                        Tabs::new("tabs")
                            .render(window, cx)
                            .into_element()
                            .write_a11y_info(&mut node);
                        *captured.lock().unwrap() = Some(node);
                    },
                    |_, _, _, _| {},
                )
            }
        }

        let captured: Captured = Arc::new(Mutex::new(None));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Probe(captured));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(result.lock().unwrap().take().unwrap().role(), Role::TabList);
    }
}
