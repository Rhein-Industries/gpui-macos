//! All raw-handle conversion and Objective-C subclassing is isolated here.
use super::{Command, NativeToolbarItem, SidebarMetrics};
use anyhow::{Context as _, bail};
use gpui_kit::{App, Bounds, Hsla, Pixels, SharedString, Window, px};
use objc2::runtime::{AnyClass, AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSAutoresizingMaskOptions,
    NSColor, NSColorSpace, NSControlTextDidChangeNotification, NSControlTextEditingDelegate,
    NSGlassEffectView, NSGlassEffectViewStyle, NSImage, NSLayoutAttribute, NSOutlineView, NSPanel,
    NSSearchField, NSSearchFieldDelegate, NSTableViewRowSizeStyle, NSTableViewStyle,
    NSTextFieldDelegate, NSTitlebarAccessoryViewController, NSToolbar, NSToolbarDelegate,
    NSToolbarDisplayMode, NSToolbarItem, NSUserInterfaceItemIdentification, NSView, NSWindow,
    NSWindowDidExitFullScreenNotification, NSWindowOrderingMode, NSWindowStyleMask,
    NSWindowTitleVisibility, NSWindowToolbarStyle,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSNotification, NSNotificationCenter, NSObject, NSObjectProtocol,
    NSPoint, NSRect, NSSize, NSString,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

fn main_thread() -> anyhow::Result<MainThreadMarker> {
    MainThreadMarker::new().context("AppKit bridge requires the macOS main thread")
}

thread_local! {
    static SIDEBAR_OUTLINE: RefCell<Option<Retained<NSOutlineView>>> = const { RefCell::new(None) };
}

pub(super) fn sidebar_metrics() -> SidebarMetrics {
    let Ok(mtm) = main_thread() else {
        return SidebarMetrics::default();
    };
    SIDEBAR_OUTLINE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let outline = cache.get_or_insert_with(|| {
            let outline = NSOutlineView::new(mtm);
            outline.setStyle(NSTableViewStyle::SourceList);
            outline.setRowSizeStyle(NSTableViewRowSizeStyle::Default);
            outline
        });
        let row_height = outline.rowHeight() as f32;
        // Standard NSTableCellView outlets measured under sourceList's three
        // system row sizes, including the icon column and label font.
        let (icon_column_width, label_size) = match row_height.round() as u32 {
            24 => (20., 11.),
            40 => (26., 15.),
            _ => (24., 13.),
        };
        SidebarMetrics {
            row_height: px(row_height),
            icon_column_width: px(icon_column_width),
            label_size: px(label_size),
        }
    })
}

pub(super) fn refresh_sidebar_metrics() {
    SIDEBAR_OUTLINE.with(|cache| {
        cache.borrow_mut().take();
    });
}

pub(super) fn sidebar_inactive_selection(dark: bool) -> Option<Hsla> {
    let _ = main_thread().ok()?;
    // SAFETY: These are immutable AppKit appearance-name constants.
    let name = unsafe {
        if dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        }
    };
    let appearance = NSAppearance::appearanceNamed(name)?;
    let result = Cell::new(None);
    let sample = block2::RcBlock::new(|| {
        if let Some(color) = NSColor::unemphasizedSelectedContentBackgroundColor()
            .colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())
        {
            result.set(Some(
                gpui_kit::Rgba {
                    r: color.redComponent() as f32,
                    g: color.greenComponent() as f32,
                    b: color.blueComponent() as f32,
                    a: color.alphaComponent() as f32,
                }
                .into(),
            ));
        }
    });
    appearance.performAsCurrentDrawingAppearance(&sample);
    result.get()
}

fn renderer_view(window: &Window) -> anyhow::Result<Retained<NSView>> {
    let _ = main_thread()?;
    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("Native window handle is unavailable: {error:?}"))?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        bail!("This GPUI window has no AppKit native view");
    };
    // SAFETY: HasWindowHandle ties the non-null pointer's validity to the live
    // Window borrow. AppKit raw handles identify NSView instances. We check the
    // main thread above and retain the view before that borrow ends.
    unsafe { Retained::retain(handle.ns_view.as_ptr().cast::<NSView>()) }
        .context("The AppKit renderer view could not be retained")
}

fn native_window(window: &Window) -> anyhow::Result<Retained<NSWindow>> {
    renderer_view(window)?
        .window()
        .context("The renderer view is detached from its NSWindow")
}

fn parent_view(view: &NSView) -> Option<Retained<NSView>> {
    // SAFETY: The bridge only runs on the main thread and owns the renderer
    // borrow/retain while reading its weak superview. AppKit's hierarchy is not
    // mutated concurrently; the returned parent is retained for its caller.
    unsafe { view.superview() }
}

pub(super) fn glass_available(window: &Window) -> bool {
    main_thread().is_ok()
        && AnyClass::get(c"NSGlassEffectView").is_some()
        && renderer_view(window).is_ok_and(|view| parent_view(&view).is_some())
}

pub(super) fn is_glass_popup(window: &Window) -> bool {
    let Some(class) = AnyClass::get(c"NSGlassEffectView") else {
        return false;
    };
    native_window(window)
        .ok()
        .and_then(|window| window.contentView())
        .is_some_and(|view| view.isKindOfClass(class))
}

define_class!(
    // SAFETY: NSView has no extra subclassing invariants. All methods run on
    // the main thread, and hitTest: exactly matches AppKit's method signature.
    #[unsafe(super = NSView)]
    #[name = "GPUIMacosMaterialHostView"]
    #[thread_kind = MainThreadOnly]
    struct MaterialHostView;

    unsafe impl NSObjectProtocol for MaterialHostView {}

    impl MaterialHostView {
        #[unsafe(method(isFlipped))]
        fn flipped(&self) -> bool { true }

        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> { None }
    }
);

/// A clipped native underlay, owned by one consecutive-frame GPUI element.
pub(super) struct GlassUnderlay {
    host: Retained<MaterialHostView>,
    glass: Retained<NSGlassEffectView>,
    presentation: Cell<Option<UnderlayPresentation>>,
    visible: Cell<bool>,
}

#[derive(Clone, Copy)]
struct UnderlayPresentation {
    host_frame: NSRect,
    glass_frame: NSRect,
    radius: f64,
    clear: bool,
    tint: Option<Hsla>,
}

impl GlassUnderlay {
    pub(super) fn new(window: &Window) -> anyhow::Result<Self> {
        let mtm = main_thread()?;
        if !glass_available(window) {
            bail!("NSGlassEffectView requires macOS 26 or newer");
        }
        let renderer = renderer_view(window)?;
        let parent = parent_view(&renderer).context("Renderer has no native parent view")?;
        // SAFETY: MaterialHostView inherits NSView's initWithFrame: contract.
        // The zero frame is valid, and this call occurs on the main thread.
        let host: Retained<MaterialHostView> =
            unsafe { msg_send![MaterialHostView::alloc(mtm), initWithFrame: NSRect::ZERO] };
        host.setClipsToBounds(true);
        host.setHidden(true);
        let glass = NSGlassEffectView::new(mtm);
        // Apple's documented contentView relationship is used; we never place
        // the GPUI renderer inside arbitrary NSGlassEffectView subviews.
        glass.setContentView(Some(&NSView::new(mtm)));
        host.addSubview(&glass);
        parent.addSubview_positioned_relativeTo(
            &host,
            NSWindowOrderingMode::Below,
            Some(&renderer),
        );
        Ok(Self {
            host,
            glass,
            presentation: Cell::new(None),
            visible: Cell::new(false),
        })
    }

    fn hide(&self) {
        if self.visible.replace(false) {
            self.host.setHidden(true);
        }
    }

    pub(super) fn update(
        &self,
        bounds: Bounds<Pixels>,
        visible: Bounds<Pixels>,
        radius: Pixels,
        clear: bool,
        tint: Option<Hsla>,
        window: &Window,
    ) {
        let Ok(renderer) = renderer_view(window) else {
            self.hide();
            return;
        };
        let Some(parent) = parent_view(&renderer) else {
            self.hide();
            return;
        };
        let w = f64::from(f32::from(visible.size.width));
        let h = f64::from(f32::from(visible.size.height));
        if w <= 0. || h <= 0. {
            self.hide();
            return;
        }
        let x = f64::from(f32::from(visible.origin.x));
        let top = f64::from(f32::from(visible.origin.y));
        let y = if renderer.isFlipped() {
            top
        } else {
            renderer.bounds().size.height - top - h
        };
        let frame = parent.convertRect_fromView(
            NSRect::new(NSPoint::new(x, y), NSSize::new(w, h)),
            Some(&renderer),
        );
        // Keep the full glass geometry within a clipping host. Scrolling does
        // not create artificial rounded corners at the viewport's cut edge.
        let glass_frame = NSRect::new(
            NSPoint::new(
                f64::from(f32::from(bounds.origin.x - visible.origin.x)),
                f64::from(f32::from(bounds.origin.y - visible.origin.y)),
            ),
            NSSize::new(
                f64::from(f32::from(bounds.size.width)),
                f64::from(f32::from(bounds.size.height)),
            ),
        );
        let radius = f64::from(f32::from(radius));
        let previous = self.presentation.get();
        // Reapplying material properties on every GPUI repaint causes needless
        // AppKit layout and compositing work, even when only a control animates.
        if previous.is_none_or(|previous| previous.host_frame != frame) {
            self.host.setFrame(frame);
        }
        if previous.is_none_or(|previous| previous.glass_frame != glass_frame) {
            self.glass.setFrame(glass_frame);
        }
        if previous.is_none_or(|previous| previous.radius != radius) {
            self.glass.setCornerRadius(radius);
        }
        if previous.is_none_or(|previous| previous.clear != clear) {
            self.glass.setStyle(if clear {
                NSGlassEffectViewStyle::Clear
            } else {
                NSGlassEffectViewStyle::Regular
            });
        }
        if previous.is_none_or(|previous| previous.tint != tint) {
            let color = tint.map(|tint| {
                let rgb = tint.to_rgb();
                NSColor::colorWithSRGBRed_green_blue_alpha(
                    f64::from(rgb.r),
                    f64::from(rgb.g),
                    f64::from(rgb.b),
                    f64::from(rgb.a),
                )
            });
            self.glass.setTintColor(color.as_deref());
        }
        self.presentation.set(Some(UnderlayPresentation {
            host_frame: frame,
            glass_frame,
            radius,
            clear,
            tint,
        }));
        if !self.visible.replace(true) {
            self.host.setHidden(false);
        }
    }
}

impl Drop for GlassUnderlay {
    fn drop(&mut self) {
        // Element state is created, updated and destroyed on GPUI's main thread.
        // Removing the host releases AppKit's extra retain on its glass subtree.
        self.host.removeFromSuperview();
    }
}

/// The owner is retained before opening a GPUI popup, without extending a
/// mutable GPUI Window borrow across the popup's construction.
pub(super) struct GlassPopupOwner {
    window: Retained<NSWindow>,
    renderer: Retained<NSView>,
}

impl GlassPopupOwner {
    pub(super) fn new(owner: &Window) -> anyhow::Result<Self> {
        if !glass_available(owner) {
            bail!("A native glass popup requires macOS 26 and an AppKit window");
        }
        Ok(Self {
            window: native_window(owner)?,
            renderer: renderer_view(owner)?,
        })
    }

    fn screen_rect(&self, bounds: Bounds<Pixels>) -> NSRect {
        let height = f64::from(f32::from(bounds.size.height));
        let top = f64::from(f32::from(bounds.origin.y));
        let rect = NSRect::new(
            NSPoint::new(
                f64::from(f32::from(bounds.origin.x)),
                if self.renderer.isFlipped() {
                    top
                } else {
                    self.renderer.bounds().size.height - top - height
                },
            ),
            NSSize::new(f64::from(f32::from(bounds.size.width)), height),
        );
        self.window
            .convertRectToScreen(self.renderer.convertRect_toView(rect, None))
    }
}

/// A separate GPUI NSPanel whose original GPUI container is in glass.contentView.
/// The owner's already-rendered Metal scene is behind this window, rather
/// than above the material in the same compositor plane.
pub(super) struct GlassPopup {
    owner: GlassPopupOwner,
    window: Retained<NSWindow>,
    renderer: Retained<NSView>,
    glass: Retained<NSGlassEffectView>,
    original_content: Retained<NSView>,
    original_parent: Retained<NSView>,
    original_frame: NSRect,
    original_autoresizing: NSAutoresizingMaskOptions,
    original_content_autoresizing: NSAutoresizingMaskOptions,
    original_style: NSWindowStyleMask,
    original_background: Option<Retained<NSColor>>,
    original_opaque: bool,
    original_shadow: bool,
}

impl GlassPopup {
    pub(super) fn new(
        owner: GlassPopupOwner,
        popup: &Window,
        options: &super::GlassPanelOptions,
    ) -> anyhow::Result<Self> {
        let mtm = main_thread()?;
        if !glass_available(popup) {
            bail!("This popup cannot host NSGlassEffectView");
        }
        let window = native_window(popup)?;
        anyhow::ensure!(
            window.downcast_ref::<NSPanel>().is_some(),
            "The glass foreground must be a GPUI WindowKind::PopUp NSPanel"
        );
        if window == owner.window {
            bail!("The glass foreground requires a separate GPUI popup window");
        }
        let renderer = renderer_view(popup)?;
        let original_parent = parent_view(&renderer).context("Popup renderer has no parent")?;
        let original_content = window.contentView().context("Popup has no content view")?;
        let original_frame = renderer.frame();
        let original_autoresizing = renderer.autoresizingMask();
        let original_content_autoresizing = original_content.autoresizingMask();
        let original_style = window.styleMask();
        let original_background = Some(window.backgroundColor());
        let original_opaque = window.isOpaque();
        let original_shadow = window.hasShadow();
        let glass = NSGlassEffectView::new(mtm);
        glass.setStyle(if options.clear {
            NSGlassEffectViewStyle::Clear
        } else {
            NSGlassEffectViewStyle::Regular
        });
        glass.setCornerRadius(f64::from(f32::from(options.corner_radius)));
        let tint = options.tint.map(|tint| {
            let rgb = tint.to_rgb();
            NSColor::colorWithSRGBRed_green_blue_alpha(
                f64::from(rgb.r),
                f64::from(rgb.g),
                f64::from(rgb.b),
                f64::from(rgb.a),
            )
        });
        glass.setTintColor(tint.as_deref());
        // Preserve GPUI's original container and its AccessKit adapter in the
        // same NSWindow. SubclassingAdapter::for_window attaches to this view,
        // rather than the renderer; detaching it would hide native AX children.
        // Apple's contentView relationship places this entire foreground in
        // glass; an arbitrary sibling behind the renderer would not do so.
        window.setStyleMask(NSWindowStyleMask::NonactivatingPanel);
        window.setBackgroundColor(Some(&NSColor::clearColor()));
        window.setOpaque(false);
        window.setHasShadow(true);
        window.setContentView(Some(&glass));
        glass.setContentView(Some(&original_content));
        glass.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        renderer.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        original_content.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        window.setFrame_display(owner.screen_rect(options.bounds), false);
        original_content.setFrame(glass.bounds());
        renderer.setFrame(original_content.bounds());
        // SAFETY: Both windows are retained, main-thread AppKit objects. The
        // child was created by GPUI; neither its delegate nor input handling
        // is replaced. The attachment is removed before releasing either.
        unsafe {
            owner
                .window
                .addChildWindow_ordered(&window, NSWindowOrderingMode::Above)
        };
        Ok(Self {
            owner,
            window,
            renderer,
            glass,
            original_content,
            original_parent,
            original_frame,
            original_autoresizing,
            original_content_autoresizing,
            original_style,
            original_background,
            original_opaque,
            original_shadow,
        })
    }

    pub(super) fn reposition(&self, bounds: Bounds<Pixels>, owner: &Window) -> anyhow::Result<()> {
        let owner = GlassPopupOwner::new(owner)?;
        if owner.window != self.owner.window {
            bail!("A glass popup cannot change its owning window");
        }
        self.window
            .setFrame_display(owner.screen_rect(bounds), true);
        self.original_content.setFrame(self.glass.bounds());
        self.renderer.setFrame(self.original_content.bounds());
        Ok(())
    }
}

impl Drop for GlassPopup {
    fn drop(&mut self) {
        self.owner.window.removeChildWindow(&self.window);
        // Restore only our own installation. A newer host may have replaced
        // the content view, and must keep its hierarchy and window treatment.
        if self.window.contentView().as_deref() == Some(&*self.glass) {
            self.glass.setContentView(None);
            self.window.setContentView(Some(&self.original_content));
            self.original_parent.addSubview(&self.renderer);
            self.original_content
                .setAutoresizingMask(self.original_content_autoresizing);
            self.renderer
                .setAutoresizingMask(self.original_autoresizing);
            self.renderer.setFrame(self.original_frame);
            self.window.setStyleMask(self.original_style);
            self.window
                .setBackgroundColor(self.original_background.as_deref());
            self.window.setOpaque(self.original_opaque);
            self.window.setHasShadow(self.original_shadow);
        }
    }
}

struct ToolbarIvars {
    items: RefCell<Vec<NativeToolbarItem>>,
    searches: RefCell<HashMap<String, Retained<NSSearchField>>>,
    search_values: RefCell<HashMap<String, String>>,
    command: Rc<dyn Fn(SharedString)>,
    chrome: RefCell<Option<NativeChromeState>>,
}

#[derive(Clone)]
struct NativeChromeState {
    window: Retained<NSWindow>,
    toolbar: Retained<NSToolbar>,
    sidebar_width: Rc<Cell<f32>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing invariants. The delegate is retained
    // by ToolbarHost, and all declared selectors use their AppKit signatures.
    #[unsafe(super = NSObject)]
    #[name = "GPUIMacosToolbarDelegate"]
    #[thread_kind = MainThreadOnly]
    #[ivars = ToolbarIvars]
    struct ToolbarDelegate;

    unsafe impl NSObjectProtocol for ToolbarDelegate {}

    unsafe impl NSControlTextEditingDelegate for ToolbarDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn text_did_change(&self, notification: &NSNotification) {
            if let Some(control) = notification.object()
                && let Some(field) = control.downcast_ref::<NSSearchField>()
            {
                self.emit_search_change(field);
            }
        }
    }

    unsafe impl NSTextFieldDelegate for ToolbarDelegate {}

    unsafe impl NSSearchFieldDelegate for ToolbarDelegate {
        #[unsafe(method(searchFieldDidEndSearching:))]
        fn search_did_end(&self, field: &NSSearchField) {
            self.emit_search_change(field);
        }
    }

    unsafe impl NSToolbarDelegate for ToolbarDelegate {
        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            self.identifiers()
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_identifiers(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            self.identifiers()
        }

        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn toolbar_item(&self, _toolbar: &NSToolbar, identifier: &NSString, inserted: bool) -> Option<Retained<NSToolbarItem>> {
            self.build_item(identifier, inserted)
        }
    }

    impl ToolbarDelegate {
        #[unsafe(method(windowDidExitFullscreen:))]
        fn window_did_exit_fullscreen(&self, _notification: &NSNotification) {
            let chrome = self.ivars().chrome.borrow().clone();
            if let Some(chrome) = chrome
                && chrome.window.toolbar().as_deref() == Some(&*chrome.toolbar)
            {
                // GPUI restores its original transparent-titlebar preference
                // during fullscreen exit. Reapply only while this bridge still
                // owns the toolbar; another window controller remains untouched.
                chrome.window.setTitlebarAppearsTransparent(chrome.sidebar_width.get() > 0.);
            }
        }

        #[unsafe(method(command:))]
        fn command(&self, sender: &AnyObject) {
            if let Some(item) = sender.downcast_ref::<NSToolbarItem>() {
                if item.isEnabled() {
                    (self.ivars().command)(item.itemIdentifier().to_string().into());
                }
            } else if let Some(field) = sender.downcast_ref::<NSSearchField>() {
                self.emit_search_change(field);
            }
        }
    }
);

impl ToolbarDelegate {
    fn emit_search_change(&self, field: &NSSearchField) {
        if !field.isEnabled() {
            return;
        }
        let id = self
            .ivars()
            .searches
            .borrow()
            .iter()
            .find_map(|(id, known)| std::ptr::eq(&**known, field).then(|| id.clone()));
        let Some(id) = id else {
            return;
        };
        let value = field.stringValue().to_string();
        let changed = {
            let mut values = self.ivars().search_values.borrow_mut();
            if values.get(&id) == Some(&value) {
                false
            } else {
                values.insert(id.clone(), value);
                true
            }
        };
        if changed {
            // AppKit may send both an action and a text-edit notification for
            // one edit. Emit once for the changed value and keep Return from
            // redundantly activating the same search.
            (self.ivars().command)(id.into());
        }
    }

    fn build_item(&self, identifier: &NSString, inserted: bool) -> Option<Retained<NSToolbarItem>> {
        let identifier_text = identifier.to_string();
        let spec = self
            .ivars()
            .items
            .borrow()
            .iter()
            .find(|entry| match entry {
                NativeToolbarItem::Button { id, .. } | NativeToolbarItem::Search { id, .. } => {
                    id.as_ref() == identifier_text
                }
                NativeToolbarItem::FlexibleSpace => false,
            })
            .cloned()?;
        let item =
            NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(self.mtm()), identifier);
        match spec {
            NativeToolbarItem::Button {
                label,
                symbol,
                enabled,
                ..
            } => {
                let native_label = NSString::from_str(&label);
                item.setLabel(&native_label);
                item.setToolTip(Some(&native_label));
                item.setEnabled(enabled);
                item.setAutovalidates(false);
                let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                    &NSString::from_str(&symbol),
                    Some(&native_label),
                );
                if let Some(image) = &image {
                    // Native toolbar controls choose the tint for their current
                    // appearance and activation state, including light mode.
                    image.setTemplate(true);
                }
                item.setImage(image.as_deref());
                // SAFETY: The retained delegate implements command: with an
                // object argument. ToolbarHost keeps the weak target alive.
                unsafe {
                    item.setTarget(Some(self));
                    item.setAction(Some(sel!(command:)));
                }
            }
            NativeToolbarItem::Search {
                label,
                placeholder,
                width,
                enabled,
                ..
            } => {
                let field = NSSearchField::new(self.mtm());
                field.setPlaceholderString(Some(&NSString::from_str(&placeholder)));
                field.setIdentifier(Some(identifier));
                field.setEnabled(enabled);
                field.setSendsWholeSearchString(false);
                field.setSendsSearchStringImmediately(true);
                field.sizeToFit();
                let width = f32::from(width);
                let width = if width.is_finite() {
                    width.max(44.)
                } else {
                    200.
                };
                field.setFrameSize(NSSize::new(f64::from(width), field.frame().size.height));
                // SAFETY: The target's command: accepts any Objective-C object
                // and safely checks the sender class before reading properties.
                unsafe {
                    field.setTarget(Some(self));
                    field.setAction(Some(sel!(command:)));
                }
                item.setLabel(&NSString::from_str(&label));
                item.setView(Some(&field));
                item.setEnabled(enabled);
                item.setAutovalidates(false);
                // SAFETY: ToolbarHost retains this delegate for the field's
                // lifetime. It implements the complete required protocol
                // hierarchy and controlTextDidChange:'s notification signature.
                // Set it after attaching the view so toolbar initialization
                // cannot replace the search field's editing delegate.
                unsafe { field.setDelegate(Some(ProtocolObject::from_ref(self))) };
                if inserted {
                    self.ivars()
                        .search_values
                        .borrow_mut()
                        .insert(identifier_text.clone(), field.stringValue().to_string());
                    self.ivars()
                        .searches
                        .borrow_mut()
                        .insert(identifier_text, field);
                }
            }
            NativeToolbarItem::FlexibleSpace => return None,
        }
        Some(item)
    }

    fn identifiers(&self) -> Retained<NSArray<NSString>> {
        let identifiers: Vec<_> = self
            .ivars()
            .items
            .borrow()
            .iter()
            .map(|item| match item {
                NativeToolbarItem::Button { id, .. } | NativeToolbarItem::Search { id, .. } => {
                    NSString::from_str(id)
                }
                NativeToolbarItem::FlexibleSpace => {
                    NSString::from_str("NSToolbarFlexibleSpaceItem")
                }
            })
            .collect();
        NSArray::from_retained_slice(&identifiers)
    }

    fn new(mtm: MainThreadMarker, ivars: ToolbarIvars) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ivars);
        // SAFETY: NSObject init initializes this allocated NSObject subclass.
        unsafe { msg_send![super(this), init] }
    }
}

pub(super) struct ToolbarHost {
    window: Retained<NSWindow>,
    toolbar: Retained<NSToolbar>,
    delegate: Retained<ToolbarDelegate>,
    previous: Option<Retained<NSToolbar>>,
    transparent: bool,
    visibility: NSWindowTitleVisibility,
    style: NSWindowToolbarStyle,
    background: Retained<NSColor>,
    sidebar_width: Rc<Cell<f32>>,
    sidebar_accessory: RefCell<Option<Retained<NSTitlebarAccessoryViewController>>>,
    title: RefCell<SharedString>,
}

impl ToolbarHost {
    pub(super) fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        let mut found = false;
        let mut changed = false;
        for item in self.delegate.ivars().items.borrow_mut().iter_mut() {
            if let NativeToolbarItem::Button {
                id: candidate,
                enabled: value,
                ..
            }
            | NativeToolbarItem::Search {
                id: candidate,
                enabled: value,
                ..
            } = item
                && candidate.as_ref() == id
            {
                changed |= *value != enabled;
                *value = enabled;
                found = true;
            }
        }
        if !changed {
            return found;
        }
        for item in self.toolbar.items().iter() {
            if item.itemIdentifier().to_string() == id {
                item.setEnabled(enabled);
            }
        }
        if let Some(field) = self.delegate.ivars().searches.borrow().get(id) {
            field.setEnabled(enabled);
        }
        found
    }

    pub(super) fn search_text(&self, id: &str) -> Option<SharedString> {
        self.delegate
            .ivars()
            .searches
            .borrow()
            .get(id)
            .map(|field| field.stringValue().to_string().into())
    }

    pub(super) fn set_title(&self, title: &str) {
        if self.title.borrow().as_ref() != title {
            self.window.setTitle(&NSString::from_str(title));
            *self.title.borrow_mut() = title.to_owned().into();
        }
    }

    pub(super) fn set_sidebar_width(&self, width: Pixels) {
        if self.window.toolbar().as_deref() != Some(&*self.toolbar) {
            return;
        }
        let width = f32::from(width);
        let width = if width.is_finite() { width.max(0.) } else { 0. };
        if self.sidebar_width.replace(width) == width {
            return;
        }
        self.window.setTitlebarAppearsTransparent(width > 0.);
        if width == 0. {
            self.remove_sidebar_accessory();
            return;
        }
        let Ok(mtm) = main_thread() else {
            return;
        };
        let mut accessory = self.sidebar_accessory.borrow_mut();
        let accessory = accessory.get_or_insert_with(|| {
            let accessory = NSTitlebarAccessoryViewController::new(mtm);
            accessory.setLayoutAttribute(NSLayoutAttribute::Left);
            let view = NSView::new(mtm);
            accessory.setView(&view);
            self.window.addTitlebarAccessoryViewController(&accessory);
            accessory
        });
        // AppKit's unified title begins at 108 pt. A 120 pt left accessory
        // moves it to 228 pt, matching a 220 pt NSSplitViewController sidebar.
        // This accessory reserves horizontal space without adding a title row.
        accessory.view().setFrame(NSRect::new(
            NSPoint::ZERO,
            NSSize::new(f64::from((width - 100.).max(0.)), 28.),
        ));
    }

    fn remove_sidebar_accessory(&self) {
        if let Some(accessory) = self.sidebar_accessory.borrow_mut().take()
            && let Some(index) = self
                .window
                .titlebarAccessoryViewControllers()
                .iter()
                .position(|candidate| candidate == accessory)
        {
            self.window
                .removeTitlebarAccessoryViewControllerAtIndex(index as isize);
        }
    }
}

impl Drop for ToolbarHost {
    fn drop(&mut self) {
        self.remove_sidebar_accessory();
        // SAFETY: This delegate was registered only by this bridge and remains
        // retained until after removal. The observer and all NSWindow events
        // are created, delivered and removed on AppKit's main thread.
        unsafe { NSNotificationCenter::defaultCenter().removeObserver(&self.delegate) };
        self.delegate.ivars().chrome.borrow_mut().take();
        for field in self.delegate.ivars().searches.borrow().values() {
            // SAFETY: Clear the weak delegate and action target before this
            // handle releases its retained delegate, including active editors.
            unsafe {
                field.setDelegate(None);
                field.setTarget(None);
                field.setAction(None);
            }
        }
        if self.window.toolbar().as_deref() == Some(&*self.toolbar) {
            self.window.setToolbar(self.previous.as_deref());
            self.window.setToolbarStyle(self.style);
            self.window.setTitlebarAppearsTransparent(self.transparent);
            self.window.setTitleVisibility(self.visibility);
            self.window.setBackgroundColor(Some(&self.background));
        }
        self.toolbar.setDelegate(None);
    }
}

pub(super) fn install_toolbar(
    id: SharedString,
    title: SharedString,
    items: Vec<NativeToolbarItem>,
    window: &mut Window,
    cx: &mut App,
    on_command: Command,
) -> anyhow::Result<ToolbarHost> {
    let mtm = main_thread()?;
    let ns_window = native_window(window)?;
    let handle = Window::window_handle(window);
    let async_cx = cx.to_async();
    let command = Rc::new(move |id: SharedString| {
        let on_command = on_command.clone();
        async_cx
            .spawn(async move |cx| {
                // Window updates return an error after closure; a late native action
                // therefore cannot call an application handler on a dead window.
                let _ = handle.update(cx, |_, window, cx| {
                    on_command(id, window, cx);
                    // Native controls can own first responder without sending
                    // an input event through GPUI's renderer. Request a frame
                    // explicitly after the command has updated application state.
                    window.refresh();
                });
            })
            .detach();
    });
    let delegate = ToolbarDelegate::new(
        mtm,
        ToolbarIvars {
            items: RefCell::new(items),
            searches: RefCell::new(HashMap::new()),
            search_values: RefCell::new(HashMap::new()),
            command,
            chrome: RefCell::new(None),
        },
    );
    let toolbar = NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), &NSString::from_str(&id));
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    toolbar.setAllowsUserCustomization(false);
    #[allow(deprecated)]
    toolbar.setShowsBaselineSeparator(false);
    let host = ToolbarHost {
        previous: ns_window.toolbar(),
        transparent: ns_window.titlebarAppearsTransparent(),
        visibility: ns_window.titleVisibility(),
        style: ns_window.toolbarStyle(),
        background: ns_window.backgroundColor(),
        window: ns_window,
        toolbar,
        delegate,
        sidebar_width: Rc::new(Cell::new(0.)),
        sidebar_accessory: RefCell::new(None),
        title: RefCell::new(title.clone()),
    };
    host.window.setTitle(&NSString::from_str(&title));
    host.window
        .setTitleVisibility(NSWindowTitleVisibility::Visible);
    // GPUI's transparent renderer may initialize the NSWindow background to
    // nearly clear black. AppKit's real titlebar needs its semantic background
    // rather than inheriting that compositing setup.
    host.window
        .setBackgroundColor(Some(&NSColor::windowBackgroundColor()));
    host.window.setTitlebarAppearsTransparent(false);
    host.window.setToolbarStyle(NSWindowToolbarStyle::Unified);
    host.window.setToolbar(Some(&host.toolbar));
    for field in host.delegate.ivars().searches.borrow().values() {
        // SAFETY: AppKit retains the custom control while our handle retains its
        // delegate. Reattach after installation, then observe the public control
        // notification independently of any toolbar-owned delegate rewiring.
        unsafe {
            field.setDelegate(Some(ProtocolObject::from_ref(&*host.delegate)));
            field.setTarget(Some(&*host.delegate));
            field.setAction(Some(sel!(command:)));
            NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
                &host.delegate,
                sel!(controlTextDidChange:),
                Some(NSControlTextDidChangeNotification),
                Some(field),
            );
        }
    }
    *host.delegate.ivars().chrome.borrow_mut() = Some(NativeChromeState {
        window: host.window.clone(),
        toolbar: host.toolbar.clone(),
        sidebar_width: host.sidebar_width.clone(),
    });
    // SAFETY: The retained delegate implements windowDidExitFullscreen: with
    // the NSNotification argument required by NotificationCenter. Filtering to
    // this main-thread NSWindow guarantees AppKit delivery on the main thread.
    // ToolbarHost removes the registration before releasing the delegate.
    unsafe {
        NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
            &host.delegate,
            sel!(windowDidExitFullscreen:),
            Some(NSWindowDidExitFullScreenNotification),
            Some(&host.window),
        );
    }
    window.refresh();
    Ok(host)
}

pub(super) fn toolbar_inset(window: &Window) -> Pixels {
    let Ok(renderer) = renderer_view(window) else {
        return Pixels::ZERO;
    };
    let Some(ns_window) = renderer.window() else {
        return Pixels::ZERO;
    };
    let layout = renderer.convertRect_fromView(ns_window.contentLayoutRect(), None);
    let inset = if renderer.isFlipped() {
        layout.origin.y - renderer.bounds().origin.y
    } else {
        renderer.bounds().origin.y + renderer.bounds().size.height
            - layout.origin.y
            - layout.size.height
    };
    px(inset.max(0.) as f32)
}
