//! All raw-handle conversion and Objective-C subclassing is isolated here.
use super::{Command, NativeToolbarItem, SidebarMetrics};
use anyhow::{Context as _, bail};
use gpui_kit::{App, Bounds, Hsla, Pixels, SharedString, Window, px};
use objc2::runtime::{AnyClass, AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::{
    NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSAutoresizingMaskOptions, NSBezierPath, NSColor, NSControlTextDidChangeNotification,
    NSControlTextEditingDelegate, NSGlassEffectView, NSGlassEffectViewStyle, NSGradient, NSImage,
    NSImageResizingMode, NSLayoutAttribute, NSOutlineView, NSPanel, NSSearchField,
    NSSearchFieldDelegate, NSTableViewRowSizeStyle, NSTableViewStyle, NSTextFieldDelegate,
    NSTitlebarAccessoryViewController, NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode,
    NSToolbarItem, NSUserInterfaceItemIdentification, NSView, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView, NSWindow,
    NSWindowDidExitFullScreenNotification, NSWindowOrderingMode, NSWindowStyleMask,
    NSWindowTitleVisibility, NSWindowToolbarStyle,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSEdgeInsets, NSNotification, NSNotificationCenter, NSObject,
    NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
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

/// Update the glass of the popup hosting `window`, if it is one. The glass
/// view is the window's content view (see [`GlassPopup::new`]) and clips the
/// GPUI foreground, so one setter changes both the material and the clip.
/// Repeated declarations of the same radius do no AppKit work.
pub(super) fn set_popup_corner_radius(window: &Window, radius: Pixels) {
    if AnyClass::get(c"NSGlassEffectView").is_none() {
        return;
    }
    let Some(view) = native_window(window)
        .ok()
        .and_then(|window| window.contentView())
    else {
        return;
    };
    let Some(glass) = view.downcast_ref::<NSGlassEffectView>() else {
        return;
    };
    let radius = f64::from(f32::from(radius));
    if glass.cornerRadius() != radius {
        glass.setCornerRadius(radius);
    }
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

define_class!(
    // SAFETY: NSView has no extra subclassing invariants. All methods run on
    // the main thread and match AppKit's selectors and signatures exactly.
    #[unsafe(super = NSView)]
    #[name = "GPUIMacosSidebarBackdropView"]
    #[thread_kind = MainThreadOnly]
    struct SidebarBackdropView;

    unsafe impl NSObjectProtocol for SidebarBackdropView {}

    impl SidebarBackdropView {
        #[unsafe(method(isOpaque))]
        fn opaque(&self) -> bool { true }

        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> { None }

        /// Fill with the calibrated backdrop for the view's own effective
        /// appearance, the one the glass above it renders with. AppKit
        /// redisplays drawing views when that appearance changes.
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, dirty: NSRect) {
            let [red, green, blue] = sidebar_backdrop_rgb(appearance_is_dark(self));
            NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, 1.).setFill();
            NSBezierPath::fillRect(dirty);
        }

        #[unsafe(method(viewDidChangeEffectiveAppearance))]
        fn appearance_changed(&self) {
            self.setNeedsDisplay(true);
        }
    }
);

fn appearance_is_dark(view: &NSView) -> bool {
    // SAFETY: AppKit's appearance-name constants are immutable NSStrings.
    let (aqua, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    view.effectiveAppearance()
        .bestMatchFromAppearancesWithNames(&NSArray::from_slice(&[aqua, dark]))
        .is_some_and(|name| &*name == dark)
}

/// sRGB components of the sidebar glass backdrop (tokens::SIDEBAR_GLASS_BACKDROP).
fn sidebar_backdrop_rgb(dark: bool) -> [f64; 3] {
    let rgb = crate::tokens::SIDEBAR_GLASS_BACKDROP[usize::from(dark)];
    [16, 8, 0].map(|shift| f64::from((rgb >> shift) & 0xff) / 255.)
}

/// A clipped native underlay, owned by one consecutive-frame GPUI element.
pub(super) struct GlassUnderlay {
    host: Retained<MaterialHostView>,
    glass: Retained<NSGlassEffectView>,
    /// The sidebar's opaque in-window backdrop, under the glass. See `new`.
    backdrop: Option<Retained<SidebarBackdropView>>,
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
    /// `sidebar` puts an opaque window-background layer under the glass.
    pub(super) fn new(window: &Window, sidebar: bool) -> anyhow::Result<Self> {
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
        // A SwiftUI `NavigationSplitView` sidebar is regular Liquid Glass
        // (no tint, radius 0) over an opaque window-colour backdrop
        // (#FFFFFF / #1E1E1E on macOS 26 and 27). Our glass already sampled
        // the same backdrop through a `.contentBackground` effect: captures
        // measured #FEFEFE / #373737, exactly free regular glass over
        // #FFFFFF / #1E1E1E (spec §7.1 glass response). The native sidebar's
        // #EDEDED / #262626 comes from a sidebar glass configuration that
        // AppKit does not expose publicly. Because the glass only ever
        // samples this opaque view (the GPUI plane is above it), a backdrop
        // calibrated through the measured glass response approximates the
        // native colour with public API (verified by capture in an active
        // window; inactive and accessibility states are not calibrated).
        let backdrop = sidebar.then(|| {
            // SAFETY: SidebarBackdropView inherits NSView's initWithFrame:
            // contract; the zero frame is valid on the main thread.
            let backdrop: Retained<SidebarBackdropView> =
                unsafe { msg_send![SidebarBackdropView::alloc(mtm), initWithFrame: NSRect::ZERO] };
            host.addSubview(&backdrop);
            backdrop
        });
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
            backdrop,
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
        let Some(frame) = parent_frame(&renderer, &parent, visible) else {
            self.hide();
            return;
        };
        // Keep the full glass geometry within a clipping host. Scrolling does
        // not create artificial rounded corners at the viewport's cut edge.
        let glass_frame = inner_frame(bounds, visible);
        let radius = f64::from(f32::from(radius));
        let previous = self.presentation.get();
        // Reapplying material properties on every GPUI repaint causes needless
        // AppKit layout and compositing work, even when only a control animates.
        if previous.is_none_or(|previous| previous.host_frame != frame) {
            self.host.setFrame(frame);
        }
        if previous.is_none_or(|previous| previous.glass_frame != glass_frame) {
            self.glass.setFrame(glass_frame);
            if let Some(backdrop) = &self.backdrop {
                backdrop.setFrame(glass_frame);
            }
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

define_class!(
    // SAFETY: NSVisualEffectView has no extra subclassing invariants. All
    // methods run on the main thread and match AppKit's selectors exactly;
    // updateLayer calls its superclass implementation first.
    #[unsafe(super(NSVisualEffectView, NSView))]
    #[name = "GPUIMacosScrollEdgeBlurView"]
    #[thread_kind = MainThreadOnly]
    #[ivars = BlurIvars]
    struct ScrollEdgeBlurView;

    unsafe impl NSObjectProtocol for ScrollEdgeBlurView {}

    impl ScrollEdgeBlurView {
        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> { None }

        #[unsafe(method(updateLayer))]
        fn update_layer(&self) {
            // SAFETY: updateLayer takes no arguments and returns nothing.
            let _: () = unsafe { msg_send![super(self), updateLayer] };
            // Once unsupported, the material stays untouched.
            if !self.ivars().support.supported() {
                return;
            }
            // SAFETY: -[NSView layer] returns the view's (optional) CALayer.
            let layer: Option<Retained<AnyObject>> = unsafe { msg_send![self, layer] };
            let Some(layer) = layer else {
                return;
            };
            if self.ivars().support.record(strip_material_tint(&layer)) {
                // Rejected while possibly on screen: hide the untouched
                // material now (its host is the clipping MaterialHostView)
                // and ask the element to repaint with its fallback.
                if let Some(host) = parent_view(self) {
                    host.setHidden(true);
                }
                if let Some(on_reject) = self.ivars().on_reject.borrow().as_ref() {
                    on_reject();
                }
            }
        }
    }
);

struct BlurIvars {
    support: LayerSupport,
    /// Schedules a GPUI repaint (outside of drawing) after a rejection.
    on_reject: RefCell<Option<Box<dyn Fn()>>>,
}

/// Whether a scroll-edge material's layer tree was recognised. Unknown until
/// AppKit first builds the layers; a rejection is final.
#[derive(Default)]
struct LayerSupport(Cell<Option<bool>>);

impl LayerSupport {
    fn supported(&self) -> bool {
        self.0.get() != Some(false)
    }

    /// Record a recognition result; true only for the transition into
    /// rejection, when the caller must hide the material and repaint.
    fn record(&self, recognised: bool) -> bool {
        if !self.supported() {
            return false;
        }
        self.0.set(Some(recognised));
        !recognised
    }
}

/// A CGColorRef argument, encoded as AppKit declares it.
#[repr(transparent)]
struct CgColorRef(*const std::ffi::c_void);
// SAFETY: A CGColorRef is a pointer to the opaque `CGColor` struct.
unsafe impl objc2::Encode for CgColorRef {
    const ENCODING: objc2::Encoding =
        objc2::Encoding::Pointer(&objc2::Encoding::Struct("CGColor", &[]));
}

/// What a material's layer tree must contain before its tint is stripped.
#[derive(Default, Debug, PartialEq)]
struct MaterialLayers {
    /// A backdrop (blur) layer: `CABackdropLayer` or a subclass.
    backdrop: bool,
    /// The tint: a `CAChameleonLayer` or a saturation filter.
    tint: bool,
}

impl MaterialLayers {
    /// The structure GPUI's own `Blurred` backdrop strips: a blur and the
    /// tint layer or filter that is removed. Anything else stays untouched.
    fn supported(&self) -> bool {
        self.backdrop && self.tint
    }

    fn note(&mut self, class_name: &str, filters: &[String]) {
        if class_name.contains("Backdrop") {
            self.backdrop = true;
        }
        if class_name == "CAChameleonLayer" || filters.iter().any(|f| f.contains("Saturat")) {
            self.tint = true;
        }
    }
}

/// Visit a layer tree: (class name, filter descriptions) per layer.
fn visit_layers(layer: &AnyObject, visit: &mut dyn FnMut(&AnyObject, &str, &[String])) {
    // SAFETY: className, filters, description and sublayers are public
    // NSObject/CALayer getters, sent on the main thread to our own view's
    // layer tree during its updateLayer pass.
    unsafe {
        let class_name: Retained<NSString> = msg_send![layer, className];
        let filters: Option<Retained<NSArray<AnyObject>>> = msg_send![layer, filters];
        let filters: Vec<String> = filters
            .iter()
            .flat_map(|filters| filters.iter())
            .map(|filter| {
                let description: Retained<NSString> = msg_send![&*filter, description];
                description.to_string()
            })
            .collect();
        visit(layer, &class_name.to_string(), &filters);
        let sublayers: Option<Retained<NSArray<AnyObject>>> = msg_send![layer, sublayers];
        if let Some(sublayers) = sublayers {
            for sublayer in sublayers.iter() {
                visit_layers(&sublayer, visit);
            }
        }
    }
}

/// Leave only the blur of a material: the treatment GPUI's own `Blurred`
/// window backdrop applies (clear the fills of non-backdrop layers, hide
/// the tinting layer, drop the saturation filter). A scroll-edge effect blurs content without tinting
/// it, so it is invisible over a uniform background. The tree is mutated only
/// when it has exactly the recognised structure; otherwise it is left
/// untouched and this returns false.
fn strip_material_tint(layer: &AnyObject) -> bool {
    let mut layers = MaterialLayers::default();
    visit_layers(layer, &mut |_, class_name, filters| {
        layers.note(class_name, filters)
    });
    if !layers.supported() {
        return false;
    }
    visit_layers(layer, &mut |layer, class_name, filters| {
        // SAFETY: Public CALayer setters with their declared argument types,
        // on the main thread, for layers of the recognised structure.
        unsafe {
            if class_name == "CAChameleonLayer" {
                let _: () = msg_send![layer, setHidden: true];
                return;
            }
            // The blur itself keeps its own configuration.
            if !class_name.contains("Backdrop") {
                let _: () = msg_send![layer, setBackgroundColor: CgColorRef(std::ptr::null())];
            }
            if filters.iter().any(|filter| filter.contains("Saturat")) {
                let current: Option<Retained<NSArray<AnyObject>>> = msg_send![layer, filters];
                if let Some(current) = current {
                    let kept: Vec<Retained<AnyObject>> = current
                        .iter()
                        .filter(|filter| {
                            let description: Retained<NSString> = msg_send![&**filter, description];
                            !description.to_string().contains("Saturat")
                        })
                        .collect();
                    let kept = NSArray::from_retained_slice(&kept);
                    let _: () = msg_send![layer, setFilters: &*kept];
                }
            }
        }
    });
    true
}

/// A vertical alpha mask, opaque over the top half and fading out toward
/// the bottom, stretched to the effect's bounds.
fn scroll_edge_mask() -> Retained<NSImage> {
    let draw = block2::RcBlock::new(|rect: NSRect| -> objc2::runtime::Bool {
        let colors = NSArray::from_retained_slice(&[
            NSColor::colorWithSRGBRed_green_blue_alpha(0., 0., 0., 0.),
            NSColor::colorWithSRGBRed_green_blue_alpha(0., 0., 0., 1.),
            NSColor::colorWithSRGBRed_green_blue_alpha(0., 0., 0., 1.),
        ]);
        if let Some(gradient) =
            NSGradient::initWithColors(<NSGradient as objc2::AnyThread>::alloc(), &colors)
        {
            // 90 degrees runs from the bottom (clear) to the top (opaque) in
            // the unflipped drawing context.
            gradient.drawInRect_angle(rect, 90.);
        }
        objc2::runtime::Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(NSSize::new(1., 64.), false, &draw);
    image.setCapInsets(NSEdgeInsets {
        top: 0.,
        left: 0.,
        bottom: 0.,
        right: 0.,
    });
    image.setResizingMode(NSImageResizingMode::Stretch);
    image
}

/// The rectangle of `visible` (renderer coordinates) in the renderer's parent.
fn parent_frame(renderer: &NSView, parent: &NSView, visible: Bounds<Pixels>) -> Option<NSRect> {
    let w = f64::from(f32::from(visible.size.width));
    let h = f64::from(f32::from(visible.size.height));
    if w <= 0. || h <= 0. {
        return None;
    }
    let x = f64::from(f32::from(visible.origin.x));
    let top = f64::from(f32::from(visible.origin.y));
    let y = if renderer.isFlipped() {
        top
    } else {
        renderer.bounds().size.height - top - h
    };
    Some(parent.convertRect_fromView(
        NSRect::new(NSPoint::new(x, y), NSSize::new(w, h)),
        Some(renderer),
    ))
}

/// An in-window blur above the GPUI renderer, owned by one consecutive-frame
/// GPUI element: the counterpart of AppKit's soft scroll-edge effect (the
/// scroll pocket under a titlebar). It ignores hit testing, so pointer
/// events reach the renderer below it.
pub(super) struct ScrollEdgeOverlay {
    host: Retained<MaterialHostView>,
    blur: Retained<ScrollEdgeBlurView>,
    frame: Cell<Option<(NSRect, NSRect)>>,
    visible: Cell<bool>,
}

impl ScrollEdgeOverlay {
    /// `on_reject` runs if AppKit's material layers are not recognised; it
    /// must schedule a GPUI repaint, which then paints the fallback.
    pub(super) fn new(window: &Window, on_reject: Box<dyn Fn()>) -> anyhow::Result<Self> {
        let mtm = main_thread()?;
        let renderer = renderer_view(window)?;
        let parent = parent_view(&renderer).context("Renderer has no native parent view")?;
        // SAFETY: MaterialHostView inherits NSView's initWithFrame: contract.
        let host: Retained<MaterialHostView> =
            unsafe { msg_send![MaterialHostView::alloc(mtm), initWithFrame: NSRect::ZERO] };
        host.setClipsToBounds(true);
        host.setHidden(true);
        // SAFETY: ScrollEdgeBlurView inherits NSVisualEffectView's
        // initWithFrame: contract; the zero frame is valid on the main thread.
        let blur = ScrollEdgeBlurView::alloc(mtm).set_ivars(BlurIvars {
            support: LayerSupport::default(),
            on_reject: RefCell::new(Some(on_reject)),
        });
        let blur: Retained<ScrollEdgeBlurView> =
            unsafe { msg_send![super(blur), initWithFrame: NSRect::ZERO] };
        blur.setMaterial(NSVisualEffectMaterial::HeaderView);
        blur.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
        blur.setState(NSVisualEffectState::Active);
        blur.setMaskImage(Some(&scroll_edge_mask()));
        host.addSubview(&blur);
        parent.addSubview_positioned_relativeTo(
            &host,
            NSWindowOrderingMode::Above,
            Some(&renderer),
        );
        Ok(Self {
            host,
            blur,
            frame: Cell::new(None),
            visible: Cell::new(false),
        })
    }

    pub(super) fn hide(&self) {
        if self.visible.replace(false) {
            self.host.setHidden(true);
        }
    }

    /// False once AppKit's material layers turned out not to have the
    /// structure the tint stripping recognises; callers paint their
    /// fallback instead (the view is then hidden and left untouched).
    pub(super) fn supported(&self) -> bool {
        self.blur.ivars().support.supported()
    }

    /// Show the effect over `bounds`, clipped to `visible`. The blur keeps
    /// the full bounds inside a clipping host, so a partly clipped band
    /// clips its fade rather than rescaling it.
    pub(super) fn update(&self, bounds: Bounds<Pixels>, visible: Bounds<Pixels>, window: &Window) {
        if !self.supported() {
            self.hide();
            return;
        }
        let Some(frame) = renderer_view(window).ok().and_then(|renderer| {
            parent_view(&renderer).and_then(|parent| parent_frame(&renderer, &parent, visible))
        }) else {
            self.hide();
            return;
        };
        let blur_frame = inner_frame(bounds, visible);
        if self.frame.replace(Some((frame, blur_frame))) != Some((frame, blur_frame)) {
            self.host.setFrame(frame);
            self.blur.setFrame(blur_frame);
        }
        if !self.visible.replace(true) {
            self.host.setHidden(false);
        }
    }
}

/// `bounds` relative to `visible`, in a flipped host whose frame is `visible`.
fn inner_frame(bounds: Bounds<Pixels>, visible: Bounds<Pixels>) -> NSRect {
    NSRect::new(
        NSPoint::new(
            f64::from(f32::from(bounds.origin.x - visible.origin.x)),
            f64::from(f32::from(bounds.origin.y - visible.origin.y)),
        ),
        NSSize::new(
            f64::from(f32::from(bounds.size.width)),
            f64::from(f32::from(bounds.size.height)),
        ),
    )
}

impl Drop for ScrollEdgeOverlay {
    fn drop(&mut self) {
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
        // This accessory reserves horizontal space without adding a title row.
        accessory.view().setFrame(NSRect::new(
            NSPoint::ZERO,
            NSSize::new(f64::from(title_accessory_width(width)), 28.),
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

/// AppKit's unified title begins at 108 pt without an accessory. SwiftUI's
/// `NavigationSplitView` draws its detail title 20 pt after the sidebar
/// (spec §5.3), so a 220 pt sidebar needs a 132 pt left accessory to put the
/// title at 240 pt.
fn title_accessory_width(sidebar_width: f32) -> f32 {
    const UNIFIED_TITLE_ORIGIN: f32 = 108.;
    (sidebar_width + crate::tokens::TOOLBAR_TITLE_SIDEBAR_GAP - UNIFIED_TITLE_ORIGIN).max(0.)
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

#[cfg(test)]
mod tests {
    /// Tint stripping only proceeds for a tree with a backdrop and a tint
    /// (a chameleon layer or a saturation filter); anything else is left
    /// untouched and reported unsupported.
    #[test]
    fn material_tint_is_stripped_only_from_a_recognised_layer_tree() {
        use super::MaterialLayers;
        let tree = |layers: &[(&str, &[&str])]| {
            let mut found = MaterialLayers::default();
            for (class_name, filters) in layers {
                let filters: Vec<String> = filters.iter().map(|f| f.to_string()).collect();
                found.note(class_name, &filters);
            }
            found.supported()
        };
        assert!(tree(&[
            ("CALayer", &[]),
            ("CABackdropLayer", &[]),
            ("CAChameleonLayer", &[])
        ]));
        assert!(tree(&[
            ("CALayer", &[]),
            (
                "CABackdropLayer",
                &["<CAFilter: colorSaturate inputAmount = 1.4>"]
            )
        ]));
        assert!(
            !tree(&[("CALayer", &[]), ("CAChameleonLayer", &[])]),
            "no blur"
        );
        assert!(
            !tree(&[("CALayer", &[]), ("CABackdropLayer", &[])]),
            "no tint"
        );
        assert!(!tree(&[("SomeFutureLayer", &[])]), "unknown");
    }

    /// A rejection is reported once, as the transition from supported (or
    /// unknown) to unsupported, including for an overlay that was already
    /// shown with a recognised tree; it is final.
    #[test]
    fn layer_support_reports_the_visible_to_unsupported_transition_once() {
        let support = super::LayerSupport::default();
        assert!(support.supported(), "unknown until AppKit builds layers");
        assert!(!support.record(true), "recognised: shown, nothing to do");
        assert!(support.supported());
        assert!(
            support.record(false),
            "visible -> unsupported: hide and repaint"
        );
        assert!(!support.supported());
        assert!(!support.record(false), "reported once");
        assert!(!support.record(true), "final");
        assert!(!support.supported());
        let fresh = super::LayerSupport::default();
        assert!(
            fresh.record(false),
            "unknown -> unsupported: hide and repaint"
        );
    }

    /// The sidebar glass backdrop resolves its light and dark token.
    #[test]
    fn sidebar_backdrop_resolves_the_calibrated_token_per_appearance() {
        assert_eq!(super::sidebar_backdrop_rgb(false), [201. / 255.; 3]);
        assert_eq!(super::sidebar_backdrop_rgb(true), [8. / 255.; 3]);
    }

    /// Spec §5.3 arithmetic only: the accessory width that puts the title
    /// origin 20 pt after the sidebar, given AppKit's assumed 108 pt unified
    /// title origin. Narrow sidebars clamp at zero (no negative accessory).
    /// Native placement and accessory removal need an on-screen check.
    #[test]
    fn title_accessory_width_offsets_the_title_and_clamps() {
        use super::title_accessory_width as width;
        for sidebar in [88., 144., 220., 275., 440.] {
            assert_eq!(108. + width(sidebar), sidebar + 20., "{sidebar}");
        }
        assert_eq!(width(220.), 132.);
        for sidebar in [0., 40., 87.5, 88.] {
            assert_eq!(width(sidebar), 0., "{sidebar}");
        }
    }
}
