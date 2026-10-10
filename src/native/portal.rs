//! Presentation of Base's measured, retained popup factories in native glass.
use super::{GlassPanelHandle, GlassPanelOptions, glass_available, open_glass_panel};
use crate::ActiveTheme;
use gpui_kit::base::{
    PopupContentFactory, PopupDismissCallback, PopupPresentation, PopupPresenter,
};
use gpui_kit::{App, AppContext, Bounds, Context, IntoElement, Pixels, Render, Window};
use std::{cell::RefCell, rc::Rc};

struct Content {
    factory: PopupContentFactory,
    rem_size: Pixels,
}

impl Render for Content {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(self.rem_size);
        (self.factory)(window, cx)
    }
}

#[derive(Default)]
struct Lease {
    panel: Option<GlassPanelHandle<Content>>,
    bounds: Option<Bounds<Pixels>>,
    on_dismiss: Rc<RefCell<Option<PopupDismissCallback>>>,
    failed: bool,
}

impl Lease {
    fn use_inline(&mut self, cx: &mut App) {
        // Closing a native presentation for an accessibility preference change
        // must not close its owner state. Retire the callback cell as well, so
        // deferred cleanup cannot invoke a replacement panel's callback.
        self.on_dismiss.borrow_mut().take();
        if let Some(panel) = self.panel.take() {
            let _ = panel.dismiss(cx);
        }
        self.on_dismiss = Rc::default();
        self.bounds = None;
        self.failed = false;
    }
}

/// Install real glass presentation for Base popup factories. Ordinary inline
/// presentation remains automatic off macOS, in headless tests, on older
/// systems, and when reduced transparency is selected.
///
/// A keyed lease lives only while its owner's popup continues to render.
/// Dropping it closes the corresponding child window. Factories are updated
/// on owner renders and produce fresh elements in the child window's frame.
pub fn install_glass_popup_presenter(cx: &mut App) {
    cx.set_global(PopupPresenter(Rc::new(present)));
}

fn present(request: PopupPresentation, owner: &mut Window, cx: &mut App) -> bool {
    let enabled = !cx.macos().reduce_transparency && glass_available(owner);
    present_with_availability(request, owner, cx, enabled)
}

fn present_with_availability(
    request: PopupPresentation,
    owner: &mut Window,
    cx: &mut App,
    enabled: bool,
) -> bool {
    let lease = owner.use_keyed_state((request.id.clone(), "native-popup-lease"), cx, |_, _| {
        Lease::default()
    });
    lease.update(cx, |lease, cx| {
        if !enabled {
            lease.use_inline(cx);
            return false;
        }
        if lease.failed {
            return false;
        }
        *lease.on_dismiss.borrow_mut() = request.on_dismiss;
        if let Some(panel) = lease.panel.as_ref() {
            // Native dismissal queues the owner callback. Do not temporarily
            // repaint its stale inline content before that callback closes it.
            if !panel.is_open() {
                return true;
            }
            if lease.bounds != Some(request.bounds)
                && panel.reposition(request.bounds, owner, cx).is_err()
            {
                return false;
            }
            let rem_size = owner.rem_size();
            panel.content().update(cx, |content, cx| {
                content.factory = request.content;
                content.rem_size = rem_size;
                cx.notify();
            });
            lease.bounds = Some(request.bounds);
            return true;
        }
        let mut options = GlassPanelOptions::new(request.bounds);
        options.focus = request.focus;
        options.dismiss_on_deactivate = request.focus;
        let rem_size = owner.rem_size();
        let on_dismiss = lease.on_dismiss.clone();
        let panel = open_glass_panel(
            options,
            owner,
            cx,
            |window, cx| {
                window.set_rem_size(rem_size);
                cx.new(|_| Content {
                    factory: request.content,
                    rem_size,
                })
            },
            move |owner, cx| {
                let callback = on_dismiss.borrow().clone();
                if let Some(callback) = callback {
                    callback(owner, cx);
                }
            },
        );
        match panel {
            Ok(panel) if panel.is_native() => {
                lease.panel = Some(panel);
                lease.bounds = Some(request.bounds);
                true
            }
            other => {
                // A bridge failure retains the original inline presentation.
                // A temporary fallback child must not dismiss the owner state.
                lease.on_dismiss.borrow_mut().take();
                if let Ok(panel) = other {
                    let _ = panel.dismiss(cx);
                }
                lease.failed = true;
                false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonVariant, Theme, button};
    use gpui_kit::InteractiveElement;
    use gpui_kit::base::Popup;
    use gpui_kit::{
        AnyWindowHandle, ParentElement, Styled, TestAppContext, VisualTestContext, div, point, px,
        size,
    };
    use std::cell::Cell;

    struct Harness {
        commands: Rc<Cell<usize>>,
        rendered_windows: Rc<RefCell<Vec<AnyWindowHandle>>>,
    }

    struct TransitionHarness {
        open: Rc<Cell<bool>>,
        commands: Rc<Cell<usize>>,
        dismissals: Rc<Cell<usize>>,
    }

    impl Render for TransitionHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let popup = Popup::new("transition-popup", div().w(px(100.)).h(px(20.))).focus(false);
            if !self.open.get() {
                return div().p(px(80.)).child(popup);
            }
            let commands = self.commands.clone();
            let open = self.open.clone();
            let dismissals = self.dismissals.clone();
            div().p(px(80.)).child(
                popup
                    .content_factory(move |_, cx| {
                        let commands = commands.clone();
                        div().w(px(160.)).h(px(60.)).child(
                            button(
                                "transition-command",
                                "Continue",
                                ButtonVariant::Secondary,
                                cx,
                            )
                            .debug_selector(|| "transition-command".into())
                            .on_click(move |_, window, _| {
                                commands.set(commands.get() + 1);
                                window.refresh();
                            }),
                        )
                    })
                    .on_dismiss(move |_, _| {
                        open.set(false);
                        dismissals.set(dismissals.get() + 1);
                    }),
            )
        }
    }

    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let commands = self.commands.clone();
            let rendered_windows = self.rendered_windows.clone();
            div().p(px(80.)).child(
                Popup::new("factory-popup", div().w(px(100.)).h(px(20.)))
                    .focus(false)
                    .content_factory(move |window, cx| {
                        rendered_windows.borrow_mut().push(window.window_handle());
                        let commands = commands.clone();
                        div().w(px(160.)).h(px(60.)).child(
                            button("factory-command", "Continue", ButtonVariant::Secondary, cx)
                                .debug_selector(|| "factory-command".into())
                                .on_click(move |_, _, _| commands.set(commands.get() + 1)),
                        )
                    }),
            )
        }
    }

    #[gpui_kit::test]
    fn factory_hook_receives_resolved_geometry_and_creates_fresh_foreign_frame_content(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let pane = Rc::new(RefCell::new(None::<GlassPanelHandle<Content>>));
        let resolved = Rc::new(RefCell::new(Vec::new()));
        let pane_for_hook = pane.clone();
        let observed = resolved.clone();
        cx.update(move |cx| {
            cx.set_global(PopupPresenter(Rc::new(move |request, owner, cx| {
                observed.borrow_mut().push(request.bounds);
                assert!(!request.focus);
                if pane_for_hook.borrow().is_none() {
                    let panel = open_glass_panel(
                        GlassPanelOptions::new(request.bounds),
                        owner,
                        cx,
                        |_, cx| {
                            cx.new(|_| Content {
                                factory: request.content,
                                rem_size: px(16.),
                            })
                        },
                        |_, _| {},
                    )
                    .unwrap();
                    *pane_for_hook.borrow_mut() = Some(panel);
                }
                true
            })));
        });
        let commands = Rc::new(Cell::new(0));
        let rendered_windows = Rc::new(RefCell::new(Vec::new()));
        let (_, owner_cx) = cx.add_window_view({
            let commands = commands.clone();
            let rendered_windows = rendered_windows.clone();
            move |_, _| Harness {
                commands,
                rendered_windows,
            }
        });
        owner_cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(!resolved.borrow().is_empty());
        assert_eq!(
            resolved.borrow()[0],
            Bounds::new(point(px(80.), px(100.)), size(px(160.), px(60.)))
        );
        assert!(
            owner_cx.debug_bounds("factory-command").is_none(),
            "Foreign presentation has no inline hit target"
        );
        let panel_window = pane.borrow().as_ref().unwrap().window_handle();
        let mut panel_cx = VisualTestContext::from_window(panel_window, owner_cx);
        panel_cx.update(|window, cx| window.draw(cx).clear(cx));
        let target = panel_cx.debug_bounds("factory-command").unwrap().center();
        panel_cx.simulate_click(target, Default::default());
        assert_eq!(commands.get(), 1);
        let windows = rendered_windows.borrow();
        assert!(windows.contains(&panel_window));
        assert!(windows.iter().any(|window| *window != panel_window));
    }

    #[gpui_kit::test]
    fn transparency_transition_keeps_inline_state_and_retires_old_native_callback(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        // Establish the transition's starting state independently of the host's
        // accessibility preference, which can enable Reduce Transparency in CI.
        cx.update(|cx| Theme::light().with_reduce_transparency(false).apply(cx));
        let panels = Rc::new(RefCell::new(Vec::new()));
        let observed = panels.clone();
        cx.update(move |cx| {
            cx.set_global(PopupPresenter(Rc::new(move |request, owner, cx| {
                let enabled = !cx.macos().reduce_transparency;
                if enabled {
                    // Headless windows cannot install AppKit glass. Seed an
                    // ordinary interactive child into the real keyed lease to
                    // exercise its native lifecycle and transition logic.
                    let lease = owner.use_keyed_state(
                        (request.id.clone(), "native-popup-lease"),
                        cx,
                        |_, _| Lease::default(),
                    );
                    lease.update(cx, |lease, cx| {
                        if lease.panel.is_none() && !lease.failed {
                            *lease.on_dismiss.borrow_mut() = request.on_dismiss.clone();
                            let callback = lease.on_dismiss.clone();
                            let rem_size = owner.rem_size();
                            let panel = open_glass_panel(
                                GlassPanelOptions::new(request.bounds),
                                owner,
                                cx,
                                |_, cx| {
                                    cx.new(|_| Content {
                                        factory: request.content.clone(),
                                        rem_size,
                                    })
                                },
                                move |owner, cx| {
                                    let callback = callback.borrow().clone();
                                    if let Some(callback) = callback {
                                        callback(owner, cx);
                                    }
                                },
                            )
                            .unwrap();
                            observed.borrow_mut().push(panel.window_handle());
                            lease.panel = Some(panel);
                            lease.bounds = Some(request.bounds);
                        }
                    });
                }
                present_with_availability(request, owner, cx, enabled)
            })));
        });
        let open = Rc::new(Cell::new(true));
        let commands = Rc::new(Cell::new(0));
        let dismissals = Rc::new(Cell::new(0));
        let (_, owner_cx) = cx.add_window_view({
            let open = open.clone();
            let commands = commands.clone();
            let dismissals = dismissals.clone();
            move |_, _| TransitionHarness {
                open,
                commands,
                dismissals,
            }
        });
        owner_cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(panels.borrow().len(), 1);
        let first_panel = panels.borrow()[0];
        let mut panel_cx = VisualTestContext::from_window(first_panel, owner_cx);
        panel_cx.update(|window, cx| window.draw(cx).clear(cx));
        let command = panel_cx
            .debug_bounds("transition-command")
            .unwrap()
            .center();
        panel_cx.simulate_click(command, Default::default());
        assert_eq!(commands.get(), 1);

        owner_cx.update(|window, cx| {
            Theme::light().with_reduce_transparency(true).apply(cx);
            window.draw(cx).clear(cx);
        });
        owner_cx.run_until_parked();
        assert!(open.get(), "Changing materials does not dismiss the popup");
        assert_eq!(dismissals.get(), 0);
        assert_eq!(owner_cx.windows().len(), 1);
        assert!(!owner_cx.windows().contains(&first_panel));
        let inline = owner_cx
            .debug_bounds("transition-command")
            .unwrap()
            .center();
        owner_cx.simulate_click(inline, Default::default());
        assert_eq!(commands.get(), 2, "Inline fallback retains live state");

        owner_cx.update(|window, cx| {
            Theme::light().apply(cx);
            window.draw(cx).clear(cx);
        });
        owner_cx.run_until_parked();
        assert_eq!(panels.borrow().len(), 2);
        let second_panel = panels.borrow()[1];
        assert_ne!(first_panel, second_panel);
        assert!(open.get());
        assert_eq!(dismissals.get(), 0);
        let mut panel_cx = VisualTestContext::from_window(second_panel, owner_cx);
        panel_cx.update(|window, cx| window.draw(cx).clear(cx));
        let command = panel_cx
            .debug_bounds("transition-command")
            .unwrap()
            .center();
        panel_cx.simulate_click(command, Default::default());
        assert_eq!(commands.get(), 3);
        panel_cx.simulate_keystrokes("escape");
        owner_cx.run_until_parked();
        assert!(!open.get(), "The replacement's real dismissal still works");
        assert_eq!(dismissals.get(), 1);
    }
}
