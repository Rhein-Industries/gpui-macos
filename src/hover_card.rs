use crate::popover::popup_fade;
use gpui_kit::base::{HoverCard, HoverCardState};
use gpui_kit::rems;
use gpui_kit::{ElementId, IntoElement, ParentElement, Styled, prelude::*};
use std::time::Duration;

/// Supplementary hover content. Keep essential information available inline.
pub fn hover_card<E: IntoElement>(
    id: impl Into<ElementId>,
    trigger: impl IntoElement,
    content: impl Fn(
        &mut HoverCardState,
        &mut gpui_kit::Window,
        &mut gpui_kit::Context<HoverCardState>,
    ) -> E
    + 'static,
) -> HoverCard {
    let id = id.into();
    HoverCard::new(id.clone())
        .trigger(trigger)
        .open_delay(Duration::from_millis(400))
        .close_delay(Duration::from_millis(200))
        .content_factory(move |state, window, cx| {
            let body = content(state, window, cx);
            // Fades in like the popover card; it closes at once (see `popup_fade`).
            let opacity = popup_fade(&id, window, cx);
            crate::popover::presented_popover_surface(window, cx)
                .id("hover-card-surface")
                .mt(rems(0.25))
                .opacity(opacity)
                .child(body)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::base::{PopupPresentation, PopupPresenter};
    use gpui_kit::{
        Context, Render, TestAppContext, VisualTestContext, Window, WindowId, div, point, px,
    };
    use std::{cell::RefCell, rc::Rc};

    struct Harness {
        count: usize,
        factory_windows: Rc<RefCell<Vec<WindowId>>>,
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let target = cx.entity();
            let factory_windows = self.factory_windows.clone();
            div().size_full().child(hover_card(
                "profile-card",
                div()
                    .debug_selector(|| "hover-factory-trigger".into())
                    .size(px(24.))
                    .child("Info"),
                move |state, window, cx| {
                    assert!(state.is_open(), "factories only build open content");
                    factory_windows
                        .borrow_mut()
                        .push(window.window_handle().window_id());
                    let count = target.read(cx).count;
                    let target = target.clone();
                    div()
                        .debug_selector(move || format!("hover-factory-count-{count}"))
                        .w(px(200.))
                        .h(px(80.))
                        .child(format!("Count: {count}"))
                        .child(
                            crate::button(
                                "hover-factory-command",
                                "Add one",
                                crate::ButtonVariant::Secondary,
                                cx,
                            )
                            .debug_selector(|| "hover-factory-command".into())
                            .on_click(move |_, _, cx| {
                                target.update(cx, |state, cx| {
                                    state.count += 1;
                                    cx.notify();
                                });
                            }),
                        )
                },
            ))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.draw(cx).clear(cx);
        });
    }

    fn advance(cx: &mut VisualTestContext, duration: Duration) {
        cx.executor().advance_clock(duration);
        cx.run_until_parked();
        draw(cx);
    }

    #[gpui_kit::test]
    fn hover_factory_rebuilds_interactive_content_on_each_open_and_preserves_delay(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let factory_windows = Rc::new(RefCell::new(Vec::new()));
        let (view, cx) = cx.add_window_view({
            let factory_windows = factory_windows.clone();
            move |_, _| Harness {
                count: 0,
                factory_windows,
            }
        });
        draw(cx);
        let trigger = cx.debug_bounds("hover-factory-trigger").unwrap().center();
        cx.simulate_mouse_move(trigger, None, Default::default());
        advance(cx, Duration::from_millis(399));
        assert!(factory_windows.borrow().is_empty());
        assert!(cx.debug_bounds("hover-factory-count-0").is_none());
        advance(cx, Duration::from_millis(1));
        assert!(cx.debug_bounds("hover-factory-count-0").is_some());

        let command = cx.debug_bounds("hover-factory-command").unwrap().center();
        cx.simulate_mouse_move(command, None, Default::default());
        cx.simulate_click(command, Default::default());
        draw(cx);
        cx.update(|_, cx| assert_eq!(view.read(cx).count, 1));
        assert!(cx.debug_bounds("hover-factory-count-1").is_some());
        advance(cx, Duration::from_millis(400));
        assert!(
            cx.debug_bounds("hover-factory-count-1").is_some(),
            "entering interactive content cancels pending close"
        );

        cx.simulate_mouse_move(point(px(500.), px(400.)), None, Default::default());
        advance(cx, Duration::from_millis(199));
        assert!(cx.debug_bounds("hover-factory-count-1").is_some());
        advance(cx, Duration::from_millis(1));
        assert!(cx.debug_bounds("hover-factory-count-1").is_none());

        // A second opening builds fresh elements around retained application
        // data; it must not reuse a consumed first-opening AnyElement.
        cx.update(|_, cx| {
            view.update(cx, |state, cx| {
                state.count = 2;
                cx.notify();
            });
        });
        cx.simulate_mouse_move(trigger, None, Default::default());
        advance(cx, Duration::from_millis(400));
        assert!(cx.debug_bounds("hover-factory-count-2").is_some());
        assert!(cx.debug_bounds("hover-factory-count-1").is_none());
    }

    struct PresentationHost(Rc<RefCell<Option<PopupPresentation>>>);

    impl Render for PresentationHost {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let presentation = self.0.borrow().clone();
            div()
                .size_full()
                .children(presentation.map(|popup| (popup.content)(window, cx)))
        }
    }

    #[gpui_kit::test]
    fn hover_factory_builds_in_the_presenting_window_and_dismisses_owner_state(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let presentation = Rc::new(RefCell::new(None));
        cx.update({
            let presentation = presentation.clone();
            move |cx| {
                cx.set_global(PopupPresenter(Rc::new(move |popup, _, _| {
                    assert!(!popup.focus, "hover cards preserve owner keyboard focus");
                    *presentation.borrow_mut() = Some(popup);
                    true
                })));
            }
        });
        let factory_windows = Rc::new(RefCell::new(Vec::new()));
        let (_, owner_cx) = cx.add_window_view({
            let factory_windows = factory_windows.clone();
            move |_, _| Harness {
                count: 0,
                factory_windows,
            }
        });
        draw(owner_cx);
        let owner_handle = owner_cx.update(|window, _| window.window_handle());
        let owner_id = owner_handle.window_id();
        let trigger = owner_cx
            .debug_bounds("hover-factory-trigger")
            .unwrap()
            .center();
        owner_cx.simulate_mouse_move(trigger, None, Default::default());
        advance(owner_cx, Duration::from_millis(400));
        assert!(presentation.borrow().is_some());
        assert!(owner_cx.debug_bounds("hover-factory-command").is_none());

        let popup = presentation.borrow().clone().unwrap();
        let (_, receiving_cx) = cx.add_window_view({
            let presentation = presentation.clone();
            move |_, _| PresentationHost(presentation)
        });
        draw(receiving_cx);
        let receiving_id = receiving_cx.update(|window, _| window.window_handle().window_id());
        assert_ne!(owner_id, receiving_id);
        assert_eq!(factory_windows.borrow().last(), Some(&receiving_id));
        assert!(receiving_cx.debug_bounds("hover-factory-command").is_some());

        let before = factory_windows.borrow().len();
        receiving_cx.update(|window, cx| {
            // A real presenter removes its popup content before announcing
            // dismissal; the headless host follows that same lifecycle.
            presentation.borrow_mut().take();
            popup.on_dismiss.as_ref().unwrap()(window, cx);
        });
        cx.update(|cx| {
            owner_handle
                .update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                    window.draw(cx).clear(cx);
                })
                .unwrap();
        });
        assert_eq!(
            factory_windows.borrow().len(),
            before,
            "presenter dismissal closes retained HoverCardState in the owner"
        );
    }
}
