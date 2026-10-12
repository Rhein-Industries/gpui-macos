//! Optional AppKit chrome and material hosting for macOS windows.
//!
//! Navigation underlays expose system material through transparent GPUI regions.
//! Anchored popup factories use separate GPUI windows with an AppKit glass
//! content view, so the material can sample the owning window's rendered canvas.
//!
//! For native chrome, set `WindowOptions::window_background` to
//! [`gpui_kit::WindowBackgroundAppearance::Transparent`]. GPUI's `Blurred`
//! mode adds a separate selection-material backdrop that changes the appearance
//! of native glass layered above it.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use gpui_kit::{
    AnyElement, App, Context, Div, ElementId, Hsla, InteractiveElement, Interactivity, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Stateful, StatefulInteractiveElement,
    StyleRefinement, Styled, Subscription, Window, canvas, div, px, rems,
};
use std::rc::Rc;

#[cfg(target_os = "macos")]
mod macos;
mod panel;
mod portal;
pub use panel::{
    GlassPanel, GlassPanelHandle, GlassPanelOptions, NativeGlassPopup, install_glass_popup,
    open_glass_panel,
};
pub use portal::install_glass_popup_presenter;

/// Whether a real NSGlassEffectView can be hosted in this desktop window.
/// Returns false off macOS, in headless windows, and before macOS 26.
pub fn glass_available(window: &Window) -> bool {
    #[cfg(target_os = "macos")]
    return macos::glass_available(window);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        false
    }
}

/// Whether this window's foreground renderer is hosted in a real glass popup.
/// Surface adapters can leave their backgrounds clear in this presentation.
pub fn is_glass_popup(window: &Window) -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_glass_popup(window);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        false
    }
}

/// Declare the corner radius, in points, of the glass popup presenting this
/// content: 13 for menus and 16 for popovers (spec §7.3–7.4, plan D13).
///
/// Only a declaration made while the portal renders a popup's own content
/// counts, and it applies to that popup's panel alone (see
/// [`popup_radius_scope`]). Base also runs content factories in the owner
/// window to measure them, and a nested popup's factory runs inside its
/// outer popup's window; those calls fall outside the scope and change
/// nothing. Panels opened directly with [`open_glass_panel`] keep their
/// caller's `corner_radius`. Inline chrome can therefore call this
/// unconditionally. The radius is points at the window's rem size, which
/// the portal sets to the owner's, so it follows the owner's zoom.
pub(crate) fn set_popup_corner_radius(window: &Window, points: f32) {
    let radius = native_corner_radius(
        crate::tokens::pt(points).into(),
        window.rem_size(),
        window.viewport_size(),
    );
    #[cfg(test)]
    POPUP_RADII.with(|radii| radii.borrow_mut().push(radius));
    let id = window.window_handle().window_id();
    POPUP_SCOPE.with(|scope| {
        if let Some((scope_window, declared)) = scope.borrow_mut().last_mut()
            && *scope_window == id
        {
            *declared = Some(radius);
        }
    });
}

thread_local! {
    /// The popup windows whose own content is being built, innermost last,
    /// with the radius their content declared.
    static POPUP_SCOPE: std::cell::RefCell<Vec<(gpui_kit::WindowId, Option<Pixels>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Run `build` as the content of the popup in `window` and return the radius
/// it declared through [`set_popup_corner_radius`], if any.
pub(crate) fn popup_radius_scope<R>(
    window: gpui_kit::WindowId,
    build: impl FnOnce() -> R,
) -> (R, Option<Pixels>) {
    POPUP_SCOPE.with(|scope| scope.borrow_mut().push((window, None)));
    let result = build();
    let declared = POPUP_SCOPE
        .with(|scope| scope.borrow_mut().pop())
        .and_then(|(_, r)| r);
    (result, declared)
}

/// Give the glass of the popup hosting `window` the declared `radius`. Does
/// nothing unless the window's content view is the popup glass.
pub(crate) fn apply_popup_corner_radius(window: &Window, radius: Pixels) {
    #[cfg(test)]
    APPLIED_RADII.with(|radii| {
        radii
            .borrow_mut()
            .push((window.window_handle().window_id(), radius))
    });
    #[cfg(target_os = "macos")]
    macos::set_popup_corner_radius(window, radius);
    #[cfg(not(target_os = "macos"))]
    let _ = (window, radius);
}

#[cfg(test)]
thread_local! {
    /// Every radius [`set_popup_corner_radius`] resolved on this test thread.
    static POPUP_RADII: std::cell::RefCell<Vec<Pixels>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Every radius applied to a popup window on this test thread.
    static APPLIED_RADII: std::cell::RefCell<Vec<(gpui_kit::WindowId, Pixels)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Drain the radii declared since the last call (tests only).
#[cfg(test)]
pub(crate) fn take_popup_radii() -> Vec<Pixels> {
    POPUP_RADII.with(|radii| std::mem::take(&mut *radii.borrow_mut()))
}

/// Drain the radii applied to popup windows since the last call (tests only).
#[cfg(test)]
pub(crate) fn take_applied_radii() -> Vec<(gpui_kit::WindowId, Pixels)> {
    APPLIED_RADII.with(|radii| std::mem::take(&mut *radii.borrow_mut()))
}

/// AppKit source-list dimensions in logical points. Native rows follow the
/// system sidebar size preference; portable/headless views use the regular size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarMetrics {
    pub row_height: Pixels,
    pub icon_column_width: Pixels,
    pub label_size: Pixels,
}

impl Default for SidebarMetrics {
    fn default() -> Self {
        Self {
            row_height: px(32.),
            icon_column_width: px(24.),
            label_size: px(13.),
        }
    }
}

/// Read the system's default source-list row size without allocating a view
/// for every row. Call [`crate::refresh_accessibility`] after changing settings.
pub fn sidebar_metrics() -> SidebarMetrics {
    #[cfg(target_os = "macos")]
    return macos::sidebar_metrics();
    #[cfg(not(target_os = "macos"))]
    SidebarMetrics::default()
}

/// The selected sidebar row in an inactive window: opaque #E5E5E5 / #363636
/// on the opaque #F0F0F0 / #282828 material (spec §6.4, C20). Under Reduce
/// Transparency it is #E1E1E1 / #232323. [`crate::sidebar_item`] paints it
/// itself; this is the same value for custom sidebar rows.
pub fn sidebar_inactive_selection(cx: &App) -> Hsla {
    use crate::state::{PaintRole, Part, RowKind, States, paint};
    let row = PaintRole::Row {
        kind: RowKind::Sidebar,
        selected: true,
        emphasized: false,
    };
    paint(row, Part::Fill, States::inactive(), cx.macos())
}

pub(crate) fn refresh_sidebar_metrics() {
    #[cfg(target_os = "macos")]
    macos::refresh_sidebar_metrics();
}

/// A system glass underlay with GPUI-owned content, layout and interactions.
///
/// Leave ancestor fills transparent where the native material should show.
/// Opaque GPUI content hides the underlay normally. The system view refracts
/// content behind the window; it cannot refract the GPUI Metal plane above it.
/// The underlay follows layout and clipping, ignores pointer hit testing, and
/// is removed when the element stops rendering. Older systems and reduced
/// transparency use the theme's opaque/material fallback.
/// Native corners are uniform; use `corner_radius` or uniform `rounded` builders
/// to keep the AppKit effect and GPUI border geometry aligned.
#[derive(IntoElement)]
pub struct NativeGlass {
    id: ElementId,
    base: Stateful<Div>,
    clear: bool,
    tint: Option<Hsla>,
    sidebar: bool,
}

/// Start a real system glass surface where supported, with a themed fallback.
pub fn native_glass(id: impl Into<ElementId>, cx: &App) -> NativeGlass {
    let id = id.into();
    let t = cx.macos();
    NativeGlass {
        base: div()
            .id(id.clone())
            .relative()
            .rounded(rems(0.75))
            .border_color(t.glass_border())
            .font_family(t.font.clone())
            .native_text_style(crate::TextStyle::Body, cx.macos())
            .text_color(t.foreground),
        id,
        clear: false,
        tint: None,
        sidebar: false,
    }
}

impl NativeGlass {
    /// Match both the native glass and GPUI border corner geometry.
    pub fn corner_radius(mut self, radius: Pixels) -> Self {
        let value = f32::from(radius);
        let radius = px(if value.is_finite() { value.max(0.) } else { 0. });
        self.base = self.base.rounded(radius);
        self
    }

    /// Use AppKit's clear glass style instead of its regular style.
    pub fn clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Back the real glass with an opaque backdrop, as the glass of SwiftUI's
    /// `NavigationSplitView` sidebar samples its opaque window rather than
    /// the desktop. The backdrop is a custom approximation, calibrated so
    /// regular glass matches the native sidebar colour in an active window
    /// (`tokens::SIDEBAR_GLASS_BACKDROP`, verified by capture).
    /// Without real glass (older systems, headless, Reduce Transparency), paint [`crate::Theme::sidebar_material`]:
    /// the measured tint while the window is active, opaque #F0F0F0 /
    /// #282828 while it is inactive.
    pub(crate) fn sidebar_material(mut self) -> Self {
        self.sidebar = true;
        self
    }

    /// Tint the real glass using a system NSColor.
    pub fn tint(mut self, tint: Hsla) -> Self {
        self.tint = Some(tint);
        self
    }
}

impl Styled for NativeGlass {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}
impl ParentElement for NativeGlass {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}
impl InteractiveElement for NativeGlass {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}
impl StatefulInteractiveElement for NativeGlass {}

impl RenderOnce for NativeGlass {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !cx.macos().reduce_transparency && is_glass_popup(window) {
            // The popup's own glass already surrounds this renderer. Avoid
            // adding arbitrary siblings inside AppKit's glass content tree.
            return self.base;
        }
        #[cfg(target_os = "macos")]
        if !cx.macos().reduce_transparency && glass_available(window) {
            let sidebar = self.sidebar;
            let state = window.use_keyed_state((self.id, "native-glass-host"), cx, |window, _| {
                macos::GlassUnderlay::new(window, sidebar).ok()
            });
            if state.read(cx).is_some() {
                let radius = self
                    .base
                    .style()
                    .corner_radii
                    .top_left
                    .unwrap_or(px(0.).into());
                let rem_size = window.rem_size();
                let clear = self.clear;
                let tint = self.tint;
                return self.base.child(
                    canvas(
                        |bounds, window, _| bounds.intersect(&window.content_mask().bounds),
                        move |bounds, visible, window, cx| {
                            if let Some(host) = state.read(cx) {
                                host.update(
                                    bounds,
                                    visible,
                                    native_corner_radius(radius, rem_size, bounds.size),
                                    clear,
                                    tint,
                                    window,
                                );
                            }
                        },
                    )
                    .absolute()
                    .inset_0(),
                );
            }
        }
        let fallback = if self.sidebar {
            let active = crate::state::States::in_window(window).window_active;
            cx.macos().sidebar_material(active)
        } else {
            cx.macos().glass_fill()
        };
        self.base = self.base.bg(fallback);
        self.base
    }
}

/// AppKit's soft scroll-edge effect for content that scrolls under the
/// titlebar: the scroll pocket a SwiftUI sidebar list keeps in the top 52 pt
/// band under the window controls (spec §7.1 view dump). The effect is
/// full strength over the top half of the band and fades out toward the
/// band's bottom edge, where scrolled content becomes sharp again.
///
/// Position it over the band (for example `.absolute().top_0().left_0()
/// .w_full().h(native_toolbar_inset(window))`) as the last child of the
/// scrolling region, and set [`ScrollEdgeEffect::active`] only while content
/// is scrolled underneath. With real system materials it is an AppKit blur
/// above the GPUI renderer that ignores pointer hit testing; elsewhere
/// (older systems, headless, Reduce Transparency, or a host whose material
/// layers are not recognised) it paints a fade from
/// [`ScrollEdgeEffect::fallback_color`], the surface colour behind the band.
///
/// Restriction: the native blur sits above the whole GPUI renderer, so any
/// GPUI content painted over the band — inline sheets and dialogs, toasts,
/// tooltips, selection overlays — is blurred with it. Deactivate the effect
/// while such an overlay is open (the gallery does so for its inline sheet
/// and dialog). Popups that open as their own windows are unaffected.
#[derive(IntoElement)]
pub struct ScrollEdgeEffect {
    id: ElementId,
    base: Div,
    active: bool,
    fallback: Option<Hsla>,
}

/// Start a scroll-edge effect; see [`ScrollEdgeEffect`].
pub fn scroll_edge_effect(id: impl Into<ElementId>) -> ScrollEdgeEffect {
    ScrollEdgeEffect {
        id: id.into(),
        base: div(),
        active: false,
        fallback: None,
    }
}

impl ScrollEdgeEffect {
    /// Show the effect: content is scrolled under the band. Inactive, it
    /// renders nothing and releases its native view.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// The opaque surface behind the band, which the fallback fades from.
    /// Defaults to the theme's window background.
    pub fn fallback_color(mut self, color: Hsla) -> Self {
        self.fallback = Some(color);
        self
    }
}

impl Styled for ScrollEdgeEffect {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ScrollEdgeEffect {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.active {
            return self.base;
        }
        #[cfg(target_os = "macos")]
        if !cx.macos().reduce_transparency && glass_available(window) && !is_glass_popup(window) {
            let state =
                window.use_keyed_state((self.id.clone(), "scroll-edge"), cx, |window, cx| {
                    // A rejected material layer tree is found by AppKit after this
                    // frame; repaint then (outside drawing) to show the fallback.
                    let handle = window.window_handle();
                    let async_cx = cx.to_async();
                    let on_reject = Box::new(move || {
                        async_cx
                            .spawn(async move |cx| {
                                let _ = handle.update(cx, |_, window, _| window.refresh());
                            })
                            .detach();
                    });
                    macos::ScrollEdgeOverlay::new(window, on_reject).ok()
                });
            if let Some(overlay) = state.read(cx).as_ref()
                && !overlay.supported()
            {
                // Never leave an untouched material above the fallback.
                overlay.hide();
            }
            if state
                .read(cx)
                .as_ref()
                .is_some_and(|overlay| overlay.supported())
            {
                return self.base.child(
                    canvas(
                        |bounds, window, _| {
                            (bounds, bounds.intersect(&window.content_mask().bounds))
                        },
                        move |_, (bounds, visible), window, cx| {
                            if let Some(overlay) = state.read(cx) {
                                overlay.update(bounds, visible, window);
                            }
                        },
                    )
                    .absolute()
                    .inset_0(),
                );
            }
        }
        let _ = &self.id;
        let color = self.fallback.unwrap_or(cx.macos().background);
        self.base.bg(scroll_edge_fallback(color))
    }
}

/// The fallback fade: `color` over the top half of the band, then fading to
/// clear at its bottom edge (CSS angle 180 runs top to bottom).
pub(crate) fn scroll_edge_fallback(color: Hsla) -> gpui_kit::Background {
    gpui_kit::linear_gradient(
        180.,
        gpui_kit::linear_color_stop(color, 0.5),
        gpui_kit::linear_color_stop(color.opacity(0.), 1.),
    )
}

fn native_corner_radius(
    radius: gpui_kit::AbsoluteLength,
    rem_size: Pixels,
    size: gpui_kit::Size<Pixels>,
) -> Pixels {
    let radius = f32::from(radius.to_pixels(rem_size));
    px(if radius.is_finite() {
        radius
            .max(0.)
            .min(f32::from(size.width).max(0.) / 2.)
            .min(f32::from(size.height).max(0.) / 2.)
    } else {
        0.
    })
}

/// A real NSToolbar command with native SF Symbols artwork and accessibility.
#[derive(Clone, Debug)]
pub enum NativeToolbarItem {
    Button {
        id: SharedString,
        label: SharedString,
        symbol: SharedString,
        enabled: bool,
    },
    Search {
        id: SharedString,
        label: SharedString,
        placeholder: SharedString,
        width: Pixels,
        enabled: bool,
    },
    FlexibleSpace,
}

impl NativeToolbarItem {
    pub fn button(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        sf_symbol: impl Into<SharedString>,
    ) -> Self {
        Self::Button {
            id: id.into(),
            label: label.into(),
            symbol: sf_symbol.into(),
            enabled: true,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Button { enabled, .. } | Self::Search { enabled, .. } => *enabled = !disabled,
            Self::FlexibleSpace => {}
        }
        self
    }

    /// An actual NSSearchField, sized in logical points inside the toolbar.
    pub fn search(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        placeholder: impl Into<SharedString>,
        width: Pixels,
    ) -> Self {
        Self::Search {
            id: id.into(),
            label: label.into(),
            placeholder: placeholder.into(),
            width,
            enabled: true,
        }
    }
}

/// Keep this handle in the application's window view for the toolbar lifetime.
/// Dropping it restores the window's preceding toolbar and title treatment.
pub struct NativeToolbarHandle {
    #[cfg(target_os = "macos")]
    inner: macos::ToolbarHost,
}

impl NativeToolbarHandle {
    /// Update an existing command's enabled state and its native accessibility.
    pub fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        #[cfg(target_os = "macos")]
        return self.inner.set_enabled(id, enabled);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (id, enabled);
            false
        }
    }

    /// Read an actual toolbar search field. Its command callback fires whenever
    /// native search text changes, allowing the application to read this value.
    pub fn search_text(&self, id: &str) -> Option<SharedString> {
        #[cfg(target_os = "macos")]
        return self.inner.search_text(id);
        #[cfg(not(target_os = "macos"))]
        {
            let _ = id;
            None
        }
    }

    pub fn set_title(&self, title: impl Into<SharedString>) {
        let title = title.into();
        #[cfg(target_os = "macos")]
        self.inner.set_title(&title);
        #[cfg(not(target_os = "macos"))]
        let _ = title;
    }

    /// Extend a full-height sidebar behind the native titlebar and reserve its
    /// width before the native window title. Zero restores ordinary chrome.
    /// Update this when the sidebar is resized or hidden.
    pub fn set_sidebar_width(&self, width: Pixels) {
        #[cfg(target_os = "macos")]
        self.inner.set_sidebar_width(width);
        #[cfg(not(target_os = "macos"))]
        let _ = width;
    }
}

type Command = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// Install an actual unified AppKit toolbar on the existing GPUI NSWindow.
///
/// Use [`native_toolbar_inset`] to reserve any native toolbar overlap in a
/// full-size content window. Avoid drawing a second GPUI title/toolbar row.
/// Command callbacks run on GPUI's foreground executor after the native event,
/// so they cannot reenter an App borrow. The NSWindow delegate stays with GPUI.
/// Use a transparent GPUI window background rather than its separate `Blurred`
/// backdrop. The bridge supplies AppKit's semantic window/titlebar background.
pub fn install_native_toolbar(
    id: impl Into<SharedString>,
    title: impl Into<SharedString>,
    items: Vec<NativeToolbarItem>,
    window: &mut Window,
    cx: &mut App,
    on_command: impl Fn(SharedString, &mut Window, &mut App) + 'static,
) -> anyhow::Result<NativeToolbarHandle> {
    let id = id.into();
    let title = title.into();
    let command: Command = Rc::new(on_command);
    #[cfg(target_os = "macos")]
    return macos::install_toolbar(id, title, items, window, cx, command)
        .map(|inner| NativeToolbarHandle { inner });
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (id, title, items, window, cx, command);
        anyhow::bail!("Native AppKit toolbars require macOS")
    }
}

/// The area at the top of the GPUI renderer covered by native window chrome.
/// Returns zero when AppKit already excludes that area or no handle exists.
pub fn native_toolbar_inset(window: &Window) -> Pixels {
    #[cfg(target_os = "macos")]
    return macos::toolbar_inset(window);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        Pixels::ZERO
    }
}

/// Observe this window's real activation state without replacing its delegate.
/// Keep the subscription on the view, or detach it for that view's lifetime.
pub fn observe_activation<V: 'static>(
    window: &mut Window,
    cx: &mut Context<V>,
    mut callback: impl FnMut(&mut V, bool, &mut Window, &mut Context<V>) + 'static,
) -> Subscription {
    cx.observe_window_activation(window, move |view, window, cx| {
        callback(view, window.is_window_active(), window, cx);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonVariant, Theme, button};
    use gpui_kit::{Render, TestAppContext, size};
    use std::cell::Cell;

    struct FallbackHarness {
        clicks: usize,
    }

    #[test]
    fn native_glass_corner_geometry_scales_and_matches_capsule_bounds() {
        let bounds = size(px(200.), px(40.));
        assert_eq!(
            native_corner_radius(rems(0.75).into(), px(16.), bounds),
            px(12.)
        );
        assert_eq!(
            native_corner_radius(rems(0.75).into(), px(24.), bounds),
            px(18.)
        );
        assert_eq!(
            native_corner_radius(rems(999.).into(), px(16.), bounds),
            px(20.)
        );
        assert_eq!(
            native_corner_radius(px(f32::NAN).into(), px(16.), bounds),
            Pixels::ZERO
        );
    }

    struct RadiusHarness(f32);

    impl Render for RadiusHarness {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            set_popup_corner_radius(window, self.0);
            div()
        }
    }

    /// Plan D13: the declared radius is points at the window's zoom, clamped
    /// to a capsule for a panel smaller than twice the radius.
    #[gpui_kit::test]
    fn popup_radius_is_zoom_scaled_and_clamped(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| RadiusHarness(16.));
        for (zoom, height, expected) in [
            (1f32, 200f32, 16f32),
            (2., 200., 32.),
            (1.5, 200., 24.),
            (2., 40., 20.),
        ] {
            cx.simulate_resize(size(px(300.), px(height)));
            take_popup_radii();
            cx.update(|window, cx| {
                window.set_rem_size(px(16. * zoom));
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(take_popup_radii().last(), Some(&px(expected)), "{zoom}x");
        }
    }

    impl Render for FallbackHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            native_glass("fallback-surface", cx)
                .w(px(240.))
                .h(px(96.))
                .p(px(12.))
                .corner_radius(px(8.))
                .child(
                    button("native-content", "Continue", ButtonVariant::Secondary, cx)
                        .debug_selector(|| "native-content".into())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.clicks += 1;
                            cx.notify();
                        })),
                )
        }
    }

    #[gpui_kit::test]
    fn headless_glass_paints_accessible_fallback_and_preserves_child_interaction(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| FallbackHarness { clicks: 0 });
        for dark in [false, true] {
            for reduce_transparency in [false, true] {
                cx.update(|window, cx| {
                    let theme = if dark { Theme::dark() } else { Theme::light() };
                    theme
                        .with_reduce_transparency(reduce_transparency)
                        .apply(cx);
                    assert!(!glass_available(window));
                    assert_eq!(native_toolbar_inset(window), Pixels::ZERO);
                    window.draw(cx).clear(cx);
                    let fill = cx.macos().glass_fill();
                    let surface = window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| {
                            quad.bounds.size == size(px(240.), px(96.)).scale(window.scale_factor())
                        })
                        .expect("Fallback must paint the laid-out material surface");
                    assert_eq!(surface.background.as_solid(), Some(fill));
                    if reduce_transparency {
                        assert_eq!(fill.a, 1., "Reduced transparency must be fully opaque");
                    }
                });
            }
        }
        let command = cx.debug_bounds("native-content").unwrap().center();
        cx.simulate_click(command, Default::default());
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(view.read(cx).clicks, 1);
        });
    }

    #[gpui_kit::test]
    fn headless_toolbar_install_is_rejected_without_emitting_commands(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| FallbackHarness { clicks: 0 });
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        cx.update(|window, cx| {
            let result = install_native_toolbar(
                "headless-toolbar",
                "Test",
                vec![
                    NativeToolbarItem::button("add", "Add", "plus"),
                    NativeToolbarItem::search("search", "Search", "Search", px(200.)),
                ],
                window,
                cx,
                move |_, _, _| observed.set(observed.get() + 1),
            );
            assert!(
                result.is_err(),
                "Headless windows have no AppKit toolbar host"
            );
            assert_eq!(native_toolbar_inset(window), Pixels::ZERO);
            window.draw(cx).clear(cx);
        });
        assert_eq!(calls.get(), 0);
    }
}
