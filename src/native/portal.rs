//! Presentation of Base's measured, retained popup factories in native glass.
use super::panel::open_glass_panel_with_radius;
use super::{GlassPanelHandle, GlassPanelOptions, glass_available};
use crate::ActiveTheme;
use gpui_kit::base::{
    PopupContentFactory, PopupDismissCallback, PopupPresentation, PopupPresenter,
};
use gpui_kit::{App, AppContext, Bounds, Context, IntoElement, Pixels, Render, Window};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Content {
    factory: PopupContentFactory,
    rem_size: Pixels,
    /// The corner radius the content last declared (plan D13).
    radius: Rc<Cell<Option<Pixels>>>,
}

impl Content {
    fn new(factory: PopupContentFactory, rem_size: Pixels) -> Self {
        Self {
            factory,
            rem_size,
            radius: Rc::default(),
        }
    }
}

impl Render for Content {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(self.rem_size);
        // Only this popup's own content declares its radius: nested popups'
        // factories run later, during layout, outside this scope.
        let id = window.window_handle().window_id();
        let (content, declared) = super::popup_radius_scope(id, || (self.factory)(window, cx));
        if let Some(radius) = declared {
            self.radius.set(Some(radius));
            super::apply_popup_corner_radius(window, radius);
        }
        content
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
        let content = Content::new(request.content, rem_size);
        let radius = content.radius.clone();
        let panel = open_glass_panel_with_radius(
            options,
            owner,
            cx,
            |window, cx| {
                window.set_rem_size(rem_size);
                cx.new(|_| content)
            },
            move |owner, cx| {
                let callback = on_dismiss.borrow().clone();
                if let Some(callback) = callback {
                    callback(owner, cx);
                }
            },
            Some(radius),
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
    use crate::native::open_glass_panel;
    use crate::{ButtonVariant, Theme, button};
    use gpui_kit::InteractiveElement;
    use gpui_kit::base::Popup;
    use gpui_kit::{
        AnyWindowHandle, ElementId, ParentElement, Styled, TestAppContext, VisualTestContext, div,
        point, px, size,
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
                        |_, cx| cx.new(|_| Content::new(request.content, px(16.))),
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

    struct RadiusOwner;

    impl Render for RadiusOwner {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn radius_panel(
        owner: &mut Window,
        cx: &mut App,
        factory: PopupContentFactory,
    ) -> GlassPanelHandle<Content> {
        let rem_size = owner.rem_size();
        let content = Content::new(factory, rem_size);
        let radius = content.radius.clone();
        open_glass_panel_with_radius(
            GlassPanelOptions::new(Bounds::new(
                point(px(10.), px(10.)),
                size(px(200.), px(120.)),
            )),
            owner,
            cx,
            |window, cx| {
                window.set_rem_size(rem_size);
                cx.new(|_| content)
            },
            |_, _| {},
            Some(radius),
        )
        .unwrap()
    }

    /// Plan D13: the popup's own content declares its radius in points at
    /// the owner's rem size (menus 13, popovers 16; spec §7.3–7.4). The
    /// first frame GPUI draws inside `open_window` declares it, so it is in
    /// the options the glass is installed with; later declarations apply to
    /// the same panel.
    #[gpui_kit::test]
    fn popup_radius_is_installed_with_the_panel_at_the_owner_zoom(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, owner_cx) = cx.add_window_view(|_, _| RadiusOwner);
        for (zoom, points) in [(1f32, 13f32), (2., 13.), (1., 16.), (2., 16.)] {
            owner_cx.update(|window, _| window.set_rem_size(px(16. * zoom)));
            crate::native::take_applied_radii();
            let declared = Rc::new(Cell::new(points));
            let panel = owner_cx.update(|owner, cx| {
                let declared = declared.clone();
                let factory: PopupContentFactory = Rc::new(move |window, _| {
                    crate::native::set_popup_corner_radius(window, declared.get());
                    div().size_full().into_any_element()
                });
                radius_panel(owner, cx, factory)
            });
            let id = panel.window_handle().window_id();
            let expected = px(points * zoom);
            owner_cx.update(|_, cx| assert_eq!(panel.corner_radius(cx), expected));
            // The frame drawn inside `open_window` declared it before install
            // (checked above); every application is this panel's radius.
            let applied = crate::native::take_applied_radii();
            assert!(!applied.is_empty());
            assert!(applied.iter().all(|applied| *applied == (id, expected)));
            // A later declaration updates the same panel.
            declared.set(points - 4.);
            let mut panel_cx = VisualTestContext::from_window(panel.window_handle(), owner_cx);
            panel_cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            let applied = crate::native::take_applied_radii();
            assert!(!applied.is_empty());
            assert!(
                applied
                    .iter()
                    .all(|applied| *applied == (id, px((points - 4.) * zoom)))
            );
            panel_cx.update(|window, _| window.remove_window());
            owner_cx.run_until_parked();
        }
    }

    type Panels = Rc<RefCell<Vec<(ElementId, GlassPanelHandle<Content>)>>>;

    /// A presenter that shows every popup in a portal panel, as on a macOS
    /// 26 host, so nested popups open from inside other panels.
    fn install_panel_presenter(cx: &mut TestAppContext) -> Panels {
        let panels: Panels = Rc::default();
        let store = panels.clone();
        cx.update(move |cx| {
            cx.set_global(PopupPresenter(Rc::new(move |request, owner, cx| {
                let existing = store
                    .borrow()
                    .iter()
                    .find(|(id, panel)| *id == request.id && panel.is_open())
                    .map(|(_, panel)| panel.content());
                if let Some(content) = existing {
                    content.update(cx, |content, cx| {
                        content.factory = request.content;
                        cx.notify();
                    });
                } else {
                    let mut options = GlassPanelOptions::new(request.bounds);
                    options.focus = request.focus;
                    options.dismiss_on_deactivate = false;
                    let content = Content::new(request.content, owner.rem_size());
                    let radius = content.radius.clone();
                    let panel = open_glass_panel_with_radius(
                        options,
                        owner,
                        cx,
                        |_, cx| cx.new(|_| content),
                        |_, _| {},
                        Some(radius),
                    )
                    .unwrap();
                    store.borrow_mut().push((request.id, panel));
                }
                true
            })));
        });
        panels
    }

    fn panel_window(panels: &Panels, index: usize) -> AnyWindowHandle {
        panels.borrow()[index].1.window_handle()
    }

    struct NestedOwner {
        choice: gpui_kit::Entity<crate::ChoiceState>,
    }

    impl Render for NestedOwner {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let choice = self.choice.clone();
            div().p(px(20.)).child(crate::popover(
                "outer-popover",
                button("outer-trigger", "Options", ButtonVariant::Secondary, cx)
                    .debug_selector(|| "outer-trigger".into()),
                move |_, window, cx| crate::select("inner-select", &choice, window, cx),
            ))
        }
    }

    /// Review M5 #1: Base measures a nested select inside the popover's own
    /// panel, which must not turn the popover's 16 into 13. Each panel gets
    /// only its own content's radius; the owner gets none.
    #[gpui_kit::test]
    fn nested_select_keeps_the_outer_popover_radius(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let panels = install_panel_presenter(cx);
        let (view, owner_cx) = cx.add_window_view(|window, cx| NestedOwner {
            choice: cx.new(|cx| {
                crate::ChoiceState::new(
                    vec![
                        crate::ChoiceItem::new("a", "First"),
                        crate::ChoiceItem::new("b", "Second"),
                    ],
                    window,
                    cx,
                )
            }),
        });
        crate::native::take_applied_radii();
        owner_cx.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = owner_cx.debug_bounds("outer-trigger").unwrap().center();
        owner_cx.simulate_click(trigger, Default::default());
        owner_cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(panels.borrow().len(), 1, "the popover is presented");
        let outer = panel_window(&panels, 0);
        let mut outer_cx = VisualTestContext::from_window(outer, owner_cx);
        outer_cx.update(|window, cx| window.draw(cx).clear(cx));
        let choice = view.read_with(&outer_cx, |view, _| view.choice.clone());
        outer_cx.update(|window, cx| {
            choice.update(cx, |choice, cx| choice.set_open(true, window, cx));
        });
        for _ in 0..2 {
            outer_cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
        assert_eq!(panels.borrow().len(), 2, "the select is presented");
        let inner = panel_window(&panels, 1);
        let mut inner_cx = VisualTestContext::from_window(inner, &outer_cx);
        inner_cx.update(|window, cx| window.draw(cx).clear(cx));
        let applied = crate::native::take_applied_radii();
        let owner_id = owner_cx.update(|window, _| window.window_handle().window_id());
        assert!(applied.iter().any(|(id, _)| *id == outer.window_id()));
        assert!(applied.iter().any(|(id, _)| *id == inner.window_id()));
        for (id, radius) in applied {
            assert_ne!(id, owner_id, "the owner is never changed");
            if id == outer.window_id() {
                assert_eq!(radius, px(16.), "the popover keeps 16");
            } else {
                assert_eq!((id, radius), (inner.window_id(), px(13.)));
            }
        }
        owner_cx.update(|_, cx| {
            assert_eq!(panels.borrow()[0].1.corner_radius(cx), px(16.));
            assert_eq!(panels.borrow()[1].1.corner_radius(cx), px(13.));
        });
    }

    struct MenuHost;

    impl Render for MenuHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().p(px(12.)).child(crate::menu(
                "hosted-menu",
                button("hosted-trigger", "Actions", ButtonVariant::Secondary, cx)
                    .debug_selector(|| "hosted-trigger".into()),
                vec![
                    crate::MenuItem::new("Rename"),
                    crate::MenuItem::new("Delete"),
                ],
                |_, _, _| {},
            ))
        }
    }

    /// Review M5 #1: an application's own panel keeps its explicit radius
    /// when a menu opens from it; only the menu's panel gets 13.
    #[gpui_kit::test]
    fn custom_radius_panel_keeps_its_radius_when_a_menu_opens(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let panels = install_panel_presenter(cx);
        let (_, owner_cx) = cx.add_window_view(|_, _| RadiusOwner);
        let custom = owner_cx.update(|owner, cx| {
            let mut options = GlassPanelOptions::new(Bounds::new(
                point(px(10.), px(10.)),
                size(px(260.), px(200.)),
            ));
            options.corner_radius = px(8.);
            open_glass_panel(options, owner, cx, |_, cx| cx.new(|_| MenuHost), |_, _| {}).unwrap()
        });
        crate::native::take_applied_radii();
        let mut custom_cx = VisualTestContext::from_window(custom.window_handle(), owner_cx);
        custom_cx.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = custom_cx.debug_bounds("hosted-trigger").unwrap().center();
        custom_cx.simulate_click(trigger, Default::default());
        for _ in 0..2 {
            custom_cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
        }
        assert_eq!(panels.borrow().len(), 1, "the menu is presented");
        let menu = panel_window(&panels, 0);
        let mut menu_cx = VisualTestContext::from_window(menu, &custom_cx);
        menu_cx.update(|window, cx| window.draw(cx).clear(cx));
        let applied = crate::native::take_applied_radii();
        assert!(!applied.is_empty());
        assert!(
            applied
                .iter()
                .all(|(id, radius)| *id == menu.window_id() && *radius == px(13.)),
            "{applied:?}"
        );
        custom_cx.update(|_, cx| assert_eq!(custom.corner_radius(cx), px(8.)));
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
                                |_, cx| cx.new(|_| Content::new(request.content.clone(), rem_size)),
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
