// Modified by gpui-macos contributors in 2026: render-time state children with the resolved style.
// Original: longbridge/gpui-kit 0bbd9870ce420af9feacbf948b21d3824d82dc0b.
// Copyright 2024-2026 Longbridge; Apache-2.0. See LICENSE-APACHE and MACOS-PATCH.md.

use crate::TestSupportExt as _;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    div,
};

use crate::{
    StyledExt as _,
    actions::{Cancel, Confirm},
};

type OpenChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Unstyled controlled date-picker root. Applications own its trigger, calendar,
/// positioning, and visual presentation; Base owns focus and open/dismiss keyboard behavior.
#[derive(IntoElement)]
pub struct DatePicker {
    base: crate::ObservedElement<gpui::Stateful<Div>>,
    open: bool,
    disabled: bool,
    focus_handle: FocusHandle,
    style: StyleRefinement,
    children: Vec<crate::state_child::StateChild>,
    on_open_change: Option<OpenChange>,
}

impl DatePicker {
    /// Adds a child built at render with the final control state and the
    /// final resolved style, for example a focus ring that follows the
    /// control's radius and border. `checked` is the open state.
    pub fn child_with_resolved_style<E: IntoElement>(
        mut self,
        build: impl FnOnce(crate::ControlState, &StyleRefinement) -> E + 'static,
    ) -> Self {
        self.children
            .push(crate::state_child::StateChild::styled(build));
        self
    }

    pub fn new(id: impl Into<ElementId>, focus_handle: &FocusHandle) -> Self {
        Self {
            base: div().id(id).test_support(),
            open: false,
            disabled: false,
            focus_handle: focus_handle.clone(),
            style: StyleRefinement::default(),
            children: vec![],
            on_open_change: None,
        }
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
}
impl Styled for DatePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl ParentElement for DatePicker {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(
            children
                .into_iter()
                .map(crate::state_child::StateChild::Ready),
        );
    }
}
impl InteractiveElement for DatePicker {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }

    fn track_focus(mut self, handle: &gpui::FocusHandle) -> Self {
        self.base = self.base.track_focus(handle);
        self
    }
}
impl StatefulInteractiveElement for DatePicker {}
impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let open = self.open;
        let disabled = self.disabled;
        let control_state =
            crate::ControlState::new(disabled, open).with_focus(Some(&self.focus_handle), window);
        let style = self.style;
        let handler = self.on_open_change;
        self.base
            .role(Role::ComboBox)
            .aria_expanded(open)
            .track_focus(&self.focus_handle.tab_stop(!disabled))
            .on_action({
                let handler = handler.clone();
                move |_: &Confirm, window, cx| {
                    // Enter opens the picker, and closes it again once the
                    // value shown in the popup is the one the user wants.
                    if disabled {
                        cx.propagate();
                    } else if let Some(handler) = &handler {
                        handler(!open, window, cx);
                    }
                }
            })
            .on_action(move |_: &Cancel, window, cx| {
                if open {
                    if let Some(handler) = &handler {
                        handler(false, window, cx);
                    }
                } else {
                    cx.propagate();
                }
            })
            .refine_style(&style)
            .children(
                self.children
                    .into_iter()
                    .map(|child| child.resolve(control_state, &style)),
            )
    }
}
