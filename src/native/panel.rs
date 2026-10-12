//! Real glass foregrounds in separate, GPUI-owned popup windows.
//!
//! The owner window's rendered scene is below the popup at the WindowServer
//! level. AppKit can therefore sample that scene, while a transparent GPUI
//! renderer inside `NSGlassEffectView.contentView` owns the foreground.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use gpui_kit::{
    AnyWindowHandle, App, AppContext, AsyncApp, Bounds, Context, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement,
    Styled, Subscription, WeakEntity, Window, WindowBackgroundAppearance, WindowBounds,
    WindowHandle, WindowKind, WindowOptions, div, px,
};
use std::{cell::Cell, rc::Rc};

/// Geometry and material for a popup above its owning GPUI window.
#[derive(Clone, Debug)]
pub struct GlassPanelOptions {
    /// Bounds in the owning renderer's top-left, logical-point coordinates.
    pub bounds: Bounds<Pixels>,
    pub corner_radius: Pixels,
    pub clear: bool,
    pub tint: Option<Hsla>,
    /// Close when the user transfers keyboard focus to another window.
    pub dismiss_on_deactivate: bool,
    /// Give the panel keyboard focus. This does not activate the application.
    /// Leave false for programmatic preparation; set true for a user action.
    pub focus: bool,
}

impl GlassPanelOptions {
    pub fn new(bounds: Bounds<Pixels>) -> Self {
        Self {
            bounds,
            corner_radius: px(14.),
            clear: false,
            tint: None,
            dismiss_on_deactivate: true,
            focus: false,
        }
    }

    fn checked(mut self) -> anyhow::Result<Self> {
        check_bounds(self.bounds)?;
        self.corner_radius =
            super::native_corner_radius(self.corner_radius.into(), px(16.), self.bounds.size);
        Ok(self)
    }
}

fn check_bounds(bounds: Bounds<Pixels>) -> anyhow::Result<()> {
    let [x, y, width, height] = [
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    ]
    .map(f32::from);
    anyhow::ensure!(
        [x, y, width, height].iter().all(|value| value.is_finite()) && width >= 1. && height >= 1.,
        "Glass popup bounds must be finite and at least one logical point wide and high"
    );
    Ok(())
}

/// Owns the native contentView installation and child-window attachment.
/// Keep it in the popup's root entity. Dropping it restores the preceding
/// hierarchy and removes the child attachment without changing GPUI delegates.
pub struct NativeGlassPopup {
    #[cfg(target_os = "macos")]
    inner: super::macos::GlassPopup,
}

impl NativeGlassPopup {
    /// Move or resize using the same owner's local coordinates.
    pub fn reposition(&self, bounds: Bounds<Pixels>, owner: &Window) -> anyhow::Result<()> {
        check_bounds(bounds)?;
        #[cfg(target_os = "macos")]
        return self.inner.reposition(bounds, owner);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = owner;
            anyhow::bail!("Native glass popups require macOS")
        }
    }
}

/// Wrap a GPUI-created popup's own renderer in real system glass.
///
/// Both windows must be AppKit windows and the foreground must be a separate
/// `WindowKind::PopUp` window with a transparent background. This does not
/// move a renderer between windows or replace its input/delegate ownership.
/// Headless windows, older systems and reduced transparency return an error;
/// callers retain their ordinary opaque GPUI presentation in those cases.
pub fn install_glass_popup(
    options: &GlassPanelOptions,
    owner: &Window,
    popup: &Window,
    cx: &App,
) -> anyhow::Result<NativeGlassPopup> {
    let options = options.clone().checked()?;
    anyhow::ensure!(
        !cx.macos().reduce_transparency,
        "Reduced transparency uses the opaque GPUI popup fallback"
    );
    #[cfg(target_os = "macos")]
    {
        Ok(NativeGlassPopup {
            inner: super::macos::GlassPopup::new(
                super::macos::GlassPopupOwner::new(owner)?,
                popup,
                &options,
            )?,
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (options, owner, popup);
        anyhow::bail!("Native glass popups require macOS")
    }
}

type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// A GPUI root wrapping retained application content, never frame-local
/// `AnyElement` values. Its own window draws the foreground over native glass.
pub struct GlassPanel<V: Render + 'static> {
    content: Entity<V>,
    owner: AnyWindowHandle,
    previous_focus: Option<FocusHandle>,
    focus: FocusHandle,
    options: GlassPanelOptions,
    dismissed: Rc<Cell<bool>>,
    on_dismiss: Dismiss,
    material: Option<NativeGlassPopup>,
    _subscriptions: Vec<Subscription>,
    was_active: bool,
}

impl<V: Render + 'static> GlassPanel<V> {
    fn finish(&self, cx: &mut App) {
        if self.dismissed.replace(true) {
            return;
        }
        let owner = self.owner;
        let previous = self.previous_focus.clone();
        let took_focus = self.options.focus || self.was_active;
        let on_dismiss = self.on_dismiss.clone();
        // The owner can still be borrowed by the caller that dismissed us.
        // Defer restoration until both window updates have returned.
        cx.defer(move |cx| {
            let _ = owner.update(cx, |_, window, cx| {
                let current = window.focused(cx);
                // Hover panels must not steal focus, and a later owner Tab,
                // click or blur takes precedence over stale popup restoration.
                // Preserve that choice across adapter dismissal callbacks too.
                let preserve_current = !took_focus || current != previous;
                if !preserve_current && let Some(previous) = &previous {
                    previous.focus(window, cx);
                }
                on_dismiss(window, cx);
                if preserve_current {
                    if let Some(current) = current {
                        current.focus(window, cx);
                    } else {
                        window.blur(cx);
                    }
                }
                window.refresh();
            });
        });
    }
}

impl<V: Render + 'static> Render for GlassPanel<V> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let native = self.material.is_some() && !cx.macos().reduce_transparency;
        let mut fill = cx.macos().glass_fill();
        fill.a = 1.;
        let border = cx.macos().glass_border();
        let foreground = cx.macos().foreground;
        let mut pane = div()
            .id("native-glass-panel")
            .size_full()
            .rounded(self.options.corner_radius)
            .native_text_style(crate::TextStyle::Body, cx.macos())
            .text_color(foreground)
            .role(gpui_kit::accesskit::Role::Dialog)
            .tab_group()
            .track_focus(&self.focus)
            .key_context("NativeGlassPanel")
            .on_key_down(
                cx.listener(|_, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        window.remove_window();
                    }
                }),
            )
            .child(self.content.clone());
        if !native {
            pane = pane.bg(fill).border_1().border_color(border);
        }
        pane
    }
}

/// Keep this handle while a glass popup is open. Content remains an ordinary
/// retained GPUI entity and can be updated normally by the application.
pub struct GlassPanelHandle<V: Render + 'static> {
    window: WindowHandle<gpui_kit::base::Root>,
    panel: WeakEntity<GlassPanel<V>>,
    content: Entity<V>,
    dismissed: Rc<Cell<bool>>,
    native: bool,
    async_cx: AsyncApp,
}

impl<V: Render + 'static> Drop for GlassPanelHandle<V> {
    fn drop(&mut self) {
        if !self.dismissed.get() {
            let window = self.window;
            // Dropping a handle may occur inside another window update. A
            // deferred main-thread task avoids reentering GPUI's App borrow.
            self.async_cx
                .spawn(async move |cx| {
                    let _ = window.update(cx, |_, window, _| window.remove_window());
                })
                .detach();
        }
    }
}

impl<V: Render + 'static> GlassPanelHandle<V> {
    pub fn content(&self) -> Entity<V> {
        self.content.clone()
    }

    pub fn window_handle(&self) -> AnyWindowHandle {
        self.window.into()
    }

    pub fn is_native(&self) -> bool {
        self.native
    }

    pub fn is_open(&self) -> bool {
        !self.dismissed.get()
    }

    /// Transfer keyboard focus without bringing the application forward.
    pub fn focus(&self, cx: &mut App) -> anyhow::Result<()> {
        self.window.update(cx, |_, window, cx| {
            window.activate_window();
            self.panel
                .update(cx, |panel, cx| panel.focus.focus(window, cx))
        })?
    }

    pub fn dismiss(&self, cx: &mut App) -> anyhow::Result<()> {
        self.window
            .update(cx, |_, window, _| window.remove_window())
    }

    pub fn reposition(
        &self,
        bounds: Bounds<Pixels>,
        owner: &Window,
        cx: &mut App,
    ) -> anyhow::Result<()> {
        check_bounds(bounds)?;
        self.window.update(cx, |_, window, cx| {
            self.panel.update(cx, |panel, _| {
                if let Some(material) = panel.material.as_ref() {
                    material.reposition(bounds, owner)?;
                } else {
                    anyhow::ensure!(
                        bounds.origin == panel.options.bounds.origin,
                        "This platform cannot reposition a fallback popup window"
                    );
                    window.resize(bounds.size);
                }
                panel.options.bounds = bounds;
                Ok(())
            })
        })??
    }
}

/// Open a GPUI foreground panel over a system glass backdrop.
///
/// The root factory runs in the new panel's own window and element arena.
/// Retain entities/state, rather than moving an `AnyElement` from another
/// window or frame. Escape closes the panel; deactivation can close it too.
/// Closing the owner removes its panel. Dismissal restores the owner's GPUI
/// focus unless the owner has since changed it, and invokes `on_dismiss` once.
/// Nonfocusing panels preserve the owner's focus. Unsupported systems and accessibility
/// preferences use the same interactive content with an opaque GPUI surface.
pub fn open_glass_panel<V: Render + 'static>(
    options: GlassPanelOptions,
    owner: &mut Window,
    cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> anyhow::Result<GlassPanelHandle<V>> {
    let options = options.checked()?;
    let owner_handle = owner.window_handle();
    let previous_focus = owner.focused(cx);
    let bounds = Bounds::new(
        owner.bounds().origin + options.bounds.origin,
        options.bounds.size,
    );
    #[cfg(target_os = "macos")]
    let native_owner = if cx.macos().reduce_transparency {
        None
    } else {
        super::macos::GlassPopupOwner::new(owner).ok()
    };
    let dismissed = Rc::new(Cell::new(false));
    let on_dismiss: Dismiss = Rc::new(on_dismiss);
    let mut content = None;
    let mut panel_entity = None;
    let panel_options = options.clone();
    let panel_dismissed = dismissed.clone();
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            focus: options.focus,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        },
        |window, cx| {
            let body = build(window, cx);
            content = Some(body.clone());
            let panel = cx.new(|cx| {
                let focus = cx.focus_handle();
                if panel_options.focus {
                    focus.focus(window, cx);
                }
                let panel_handle = window.window_handle();
                let owner_closed = cx.on_window_closed(move |cx, id| {
                    if id == owner_handle.window_id() {
                        let _ = panel_handle.update(cx, |_, window, _| window.remove_window());
                    }
                });
                let activation =
                    cx.observe_window_activation(window, |panel: &mut GlassPanel<V>, window, _| {
                        let active = window.is_window_active();
                        if panel.options.dismiss_on_deactivate && panel.was_active && !active {
                            window.remove_window();
                        }
                        panel.was_active |= active;
                    });
                cx.on_release(|panel: &mut GlassPanel<V>, cx| panel.finish(cx))
                    .detach();
                GlassPanel {
                    content: body,
                    owner: owner_handle,
                    previous_focus,
                    focus,
                    options: panel_options,
                    dismissed: panel_dismissed,
                    on_dismiss,
                    material: None,
                    _subscriptions: vec![owner_closed, activation],
                    was_active: window.is_window_active(),
                }
            });
            panel_entity = Some(panel.downgrade());
            cx.new(|cx| {
                gpui_kit::base::Root::new(panel, window, cx).bg(gpui_kit::transparent_black())
            })
        },
    )?;
    let panel_entity = panel_entity.expect("GPUI constructed the retained panel view");
    let native = window.update(cx, |_, window, cx| {
        panel_entity.update(cx, |panel, cx| {
            #[cfg(target_os = "macos")]
            if let Some(owner) = native_owner {
                panel.material = super::macos::GlassPopup::new(owner, window, &options)
                    .ok()
                    .map(|inner| NativeGlassPopup { inner });
            }
            cx.notify();
            window.refresh();
            panel.material.is_some()
        })
    })??;
    Ok(GlassPanelHandle {
        window,
        panel: panel_entity,
        content: content.expect("GPUI invokes the root factory before returning its window"),
        dismissed,
        native,
        async_cx: cx.to_async(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonVariant, Theme, button};
    use gpui_kit::{KeyDownEvent, Keystroke, TestAppContext, VisualTestContext, point, size};

    struct Owner {
        focus: FocusHandle,
    }

    impl Render for Owner {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            button("panel-trigger", "Open", ButtonVariant::Secondary, cx).track_focus(&self.focus)
        }
    }

    struct FocusOwner {
        first: FocusHandle,
        second: FocusHandle,
    }

    impl Render for FocusOwner {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .tab_group()
                .child(
                    button("first", "First", ButtonVariant::Secondary, cx).track_focus(&self.first),
                )
                .child(
                    button("second", "Second", ButtonVariant::Secondary, cx)
                        .track_focus(&self.second),
                )
        }
    }

    struct Content {
        commands: usize,
    }

    impl Render for Content {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let _ = gpui_kit::base::Root::read(window, cx);
            div().p(px(12.)).child(
                button("panel-command", "Continue", ButtonVariant::Secondary, cx)
                    .debug_selector(|| "panel-command".into())
                    .on_click(cx.listener(|content, _, _, cx| {
                        content.commands += 1;
                        cx.notify();
                    })),
            )
        }
    }

    fn options() -> GlassPanelOptions {
        let mut options = GlassPanelOptions::new(Bounds::new(
            point(px(100.), px(80.)),
            size(px(240.), px(120.)),
        ));
        options.focus = true;
        options
    }

    #[gpui_kit::test]
    fn glass_panel_fallback_keeps_content_interactive_and_escape_restores_owner_focus(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            crate::init(cx);
            Theme::light().with_reduce_transparency(true).apply(cx);
        });
        let (owner, owner_cx) = cx.add_window_view(|window, cx| {
            let focus = cx.focus_handle();
            focus.focus(window, cx);
            Owner { focus }
        });
        let dismissed = Rc::new(Cell::new(0));
        let observed = dismissed.clone();
        let panel = owner_cx.update(|owner, cx| {
            open_glass_panel(
                options(),
                owner,
                cx,
                |_, cx| cx.new(|_| Content { commands: 0 }),
                move |_, _| observed.set(observed.get() + 1),
            )
            .unwrap()
        });
        assert!(!panel.is_native());
        assert!(panel.is_open());
        let mut panel_cx = VisualTestContext::from_window(panel.window_handle(), owner_cx);
        panel_cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let surface = window
                .painted_quads()
                .into_iter()
                .find(|quad| {
                    quad.bounds.size == size(px(240.), px(120.)).scale(window.scale_factor())
                })
                .expect("Fallback paints its complete popup surface");
            assert_eq!(surface.background.as_solid().unwrap().a, 1.);
        });
        let command = panel_cx.debug_bounds("panel-command").unwrap().center();
        panel_cx.simulate_click(command, Default::default());
        panel_cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(panel.content().read(cx).commands, 1);
        });
        panel_cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke::parse("escape").unwrap(),
            is_held: false,
            prefer_character_input: false,
        });
        assert!(!panel.is_open());
        assert_eq!(dismissed.get(), 1);
        owner_cx.update(|window, cx| assert!(owner.read(cx).focus.is_focused(window)));
        owner_cx.update(|_, cx| assert!(panel.dismiss(cx).is_err()));
        assert_eq!(dismissed.get(), 1, "Dismissal is reported exactly once");
    }

    #[gpui_kit::test]
    fn glass_panel_closes_with_owner_and_rejects_invalid_geometry(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, owner_cx) = cx.add_window_view(|_, cx| Owner {
            focus: cx.focus_handle(),
        });
        owner_cx.update(|owner, cx| {
            let window_count = cx.windows().len();
            let mut invalid = options();
            invalid.bounds.size.width = px(f32::NAN);
            assert!(
                open_glass_panel(
                    invalid,
                    owner,
                    cx,
                    |_, cx| cx.new(|_| Content { commands: 0 }),
                    |_, _| {},
                )
                .is_err()
            );
            assert_eq!(cx.windows().len(), window_count);
        });
        let panel = owner_cx.update(|owner, cx| {
            open_glass_panel(
                options(),
                owner,
                cx,
                |_, cx| cx.new(|_| Content { commands: 0 }),
                |_, _| {},
            )
            .unwrap()
        });
        owner_cx.update(|window, _| window.remove_window());
        owner_cx.run_until_parked();
        assert!(!panel.is_open(), "Owner closure releases its popup root");
    }

    #[gpui_kit::test]
    fn glass_panel_deactivation_and_handle_drop_close_once(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, owner_cx) = cx.add_window_view(|_, cx| Owner {
            focus: cx.focus_handle(),
        });
        let dismissed = Rc::new(Cell::new(0));
        let observed = dismissed.clone();
        let panel = owner_cx.update(|owner, cx| {
            open_glass_panel(
                options(),
                owner,
                cx,
                |_, cx| cx.new(|_| Content { commands: 0 }),
                move |_, _| observed.set(observed.get() + 1),
            )
            .unwrap()
        });
        let mut panel_cx = VisualTestContext::from_window(panel.window_handle(), owner_cx);
        owner_cx.update(|_, cx| panel.focus(cx).unwrap());
        owner_cx.run_until_parked();
        panel_cx.deactivate_window();
        assert!(!panel.is_open());
        assert_eq!(dismissed.get(), 1);
        drop(panel);
        owner_cx.run_until_parked();
        assert_eq!(dismissed.get(), 1);
        let observed = dismissed.clone();
        let panel = owner_cx.update(|owner, cx| {
            let mut options = options();
            options.focus = false;
            open_glass_panel(
                options,
                owner,
                cx,
                |_, cx| cx.new(|_| Content { commands: 0 }),
                move |_, _| observed.set(observed.get() + 1),
            )
            .unwrap()
        });
        assert!(panel.is_open());
        drop(panel);
        owner_cx.run_until_parked();
        assert_eq!(dismissed.get(), 2);
        assert_eq!(
            owner_cx.windows().len(),
            1,
            "No unreachable panel window remains"
        );
    }

    #[gpui_kit::test]
    fn glass_panel_dismissal_preserves_later_owner_tab_and_intentional_blur(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (owner, owner_cx) = cx.add_window_view(|window, cx| {
            let first = cx.focus_handle();
            let second = cx.focus_handle();
            first.focus(window, cx);
            FocusOwner { first, second }
        });
        for (focus, blur) in [(false, false), (true, false), (true, true)] {
            owner_cx.update(|window, cx| {
                owner.read(cx).first.clone().focus(window, cx);
                window.draw(cx).clear(cx);
            });
            let dismissed = Rc::new(Cell::new(0));
            let observed = dismissed.clone();
            let restore = owner.read_with(owner_cx, |owner, _| owner.first.clone());
            let panel = owner_cx.update(|window, cx| {
                let mut options = options();
                options.focus = focus;
                options.dismiss_on_deactivate = false;
                open_glass_panel(
                    options,
                    window,
                    cx,
                    |_, cx| cx.new(|_| Content { commands: 0 }),
                    move |window, cx| {
                        // Existing adapters restore their trigger here. A newer
                        // owner choice must survive that restoration too.
                        restore.focus(window, cx);
                        observed.set(observed.get() + 1);
                    },
                )
                .unwrap()
            });
            owner_cx.update(|window, cx| {
                if blur {
                    window.blur(cx);
                } else {
                    window.focus_next(cx);
                    assert!(owner.read(cx).second.is_focused(window));
                }
                panel.dismiss(cx).unwrap();
            });
            owner_cx.run_until_parked();
            assert!(!panel.is_open());
            assert_eq!(dismissed.get(), 1);
            owner_cx.update(|window, cx| {
                if blur {
                    assert!(window.focused(cx).is_none());
                } else {
                    assert!(owner.read(cx).second.is_focused(window));
                }
            });
        }
    }
}
