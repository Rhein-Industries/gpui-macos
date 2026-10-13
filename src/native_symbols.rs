//! SF Symbols supplied by the running macOS, rasterized into GPUI's image atlas.
//!
//! AppKit objects never escape this module. Only owned alpha masks and tinted
//! BGRA pixels are retained; no system artwork is exported or bundled.
use crate::IconName;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use gpui_kit::{FontWeight, Hsla, RenderImage, Rgba};
use image::{Frame, ImageEncoder as _, RgbaImage, codecs::png::PngEncoder};
use objc2::{AnyThread, rc::autoreleasepool};
use objc2_app_kit::{
    NSBitmapFormat, NSBitmapImageRep, NSCompositingOperation, NSDeviceRGBColorSpace,
    NSGraphicsContext, NSImage, NSImageSymbolConfiguration, NSImageSymbolScale,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, LazyLock},
};

#[derive(Clone, Hash, PartialEq, Eq)]
struct MaskKey {
    symbol: Arc<str>,
    points: u16,
    pixels: u16,
    weight: u8,
    /// A natural-size symbol: its configured point size (x64) and whether it
    /// uses the large symbol scale. `None` fits the symbol to the box.
    natural: Option<(u16, bool)>,
}

/// An SF Symbol drawn at its natural size for a configured point size and
/// symbol scale, centred in the icon box, as AppKit and SwiftUI lay out
/// symbols beside text. Every symbol keeps its own width and height, rather
/// than being fitted to the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NaturalSymbol {
    pub(crate) points: f32,
    pub(crate) large: bool,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct ImageKey {
    mask: MaskKey,
    color: [u8; 4],
}

struct AlphaMask {
    pixels: u16,
    alpha: Vec<u8>,
}

struct SymbolCache {
    names: HashMap<IconName, Option<Arc<str>>>,
    masks: SymbolLru<MaskKey, Arc<AlphaMask>>,
    images: SymbolLru<ImageKey, Arc<RenderImage>>,
    svg: HashMap<IconName, Option<Arc<[u8]>>>,
}

impl Default for SymbolCache {
    fn default() -> Self {
        Self {
            names: HashMap::new(),
            masks: SymbolLru::new(256, 16 * 1024 * 1024),
            images: SymbolLru::new(512, 64 * 1024 * 1024),
            svg: HashMap::new(),
        }
    }
}

struct SymbolEntry<V> {
    value: V,
    used_at: u64,
    bytes: usize,
}

/// Evict cold entries individually, keeping hot symbols through color and zoom
/// changes. Byte limits also bound large rasterizations, not just entry counts.
struct SymbolLru<K, V> {
    entries: HashMap<K, SymbolEntry<V>>,
    clock: u64,
    bytes: usize,
    max_entries: usize,
    max_bytes: usize,
}

impl<K: Clone + Eq + Hash, V: Clone> SymbolLru<K, V> {
    fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            clock: 0,
            bytes: 0,
            max_entries,
            max_bytes,
        }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        let entry = self.entries.get_mut(key)?;
        self.clock += 1;
        entry.used_at = self.clock;
        Some(entry.value.clone())
    }

    fn insert(&mut self, key: K, value: V, bytes: usize) {
        if let Some(previous) = self.entries.remove(&key) {
            self.bytes -= previous.bytes;
        }
        if self.max_entries == 0 || bytes > self.max_bytes {
            return;
        }
        while self.entries.len() >= self.max_entries || self.bytes + bytes > self.max_bytes {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used_at)
                .map(|(key, _)| key.clone());
            let Some(oldest) = oldest else { break };
            if let Some(entry) = self.entries.remove(&oldest) {
                self.bytes -= entry.bytes;
            }
        }
        self.clock += 1;
        self.bytes += bytes;
        self.entries.insert(
            key,
            SymbolEntry {
                value,
                used_at: self.clock,
                bytes,
            },
        );
    }
}

static PATH_NAMES: LazyLock<HashMap<gpui_kit::SharedString, IconName>> = LazyLock::new(|| {
    IconName::ALL
        .iter()
        .copied()
        .map(|name| (name.path(), name))
        .collect()
});

thread_local! {
    // Drawing contexts and AppKit symbol lookup are confined to the caller's
    // thread. Production uses GPUI's UI thread; independent headless windows
    // can render on their own threads without sharing AppKit objects.
    static CACHE: RefCell<SymbolCache> = RefCell::new(SymbolCache::default());
}

/// Native artwork at the asset boundary also reaches upstream Kit SVG icons.
pub(crate) fn svg_for_path(path: &str) -> Option<Arc<[u8]>> {
    let name = *PATH_NAMES.get(path)?;
    if let Some(cached) = CACHE.with_borrow(|cache| cache.svg.get(&name).cloned()) {
        return cached;
    }
    let svg = (|| {
        // Oversample a regular 16 pt symbol, preserving its optical design when
        // upstream SVG elements ask the GPU atlas for different logical sizes.
        let image = image(
            name,
            16.,
            8.,
            FontWeight::NORMAL,
            gpui_kit::rgb(0xffffff).into(),
        )?;
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(
                image.as_bytes(0)?,
                128,
                128,
                image::ExtendedColorType::Rgba8,
            )
            .ok()?;
        let png = STANDARD.encode(png);
        Some(Arc::from(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="128" height="128" viewBox="0 0 128 128"><image width="128" height="128" xlink:href="data:image/png;base64,{png}"/></svg>"#).into_bytes()))
    })();
    CACHE.with_borrow_mut(|cache| {
        cache.svg.insert(name, svg.clone());
    });
    svg
}

/// Resolve and tint a native symbol. A missing host symbol selects Lucide fallback.
pub(crate) fn image(
    name: IconName,
    points: f32,
    scale: f32,
    weight: FontWeight,
    color: Hsla,
) -> Option<Arc<RenderImage>> {
    image_in_box(name, points, scale, weight, color, None)
}

/// Like [`image`] in a `points` box; a `natural` symbol keeps its own size
/// (see [`NaturalSymbol`]) and is only scaled down if it overflows the box.
pub(crate) fn image_in_box(
    name: IconName,
    points: f32,
    scale: f32,
    weight: FontWeight,
    color: Hsla,
    natural: Option<NaturalSymbol>,
) -> Option<Arc<RenderImage>> {
    let natural = match natural {
        Some(symbol) if symbol.points.is_finite() && symbol.points > 0. => Some((
            (symbol.points.clamp(1., 256.) * 64.).round() as u16,
            symbol.large,
        )),
        Some(_) => return None,
        None => None,
    };
    if !points.is_finite()
        || points <= 0.
        || !scale.is_finite()
        || scale <= 0.
        || !weight.0.is_finite()
    {
        return None;
    }
    let points = (points.clamp(1., 256.) * 64.).round() as u16;
    let pixels = ((points as f32 / 64. * scale).ceil() as u16).clamp(1, 1024);
    let weight = (weight.0 / 100.).round().clamp(1., 9.) as u8;
    let rgba: Rgba = color.into();
    let color = [rgba.r, rgba.g, rgba.b, rgba.a].map(|c| (c.clamp(0., 1.) * 255.).round() as u8);
    CACHE.with_borrow_mut(|cache| {
        let symbol = cache
            .names
            .entry(name)
            .or_insert_with(|| resolve_name(name).map(Arc::from))
            .clone()?;
        let mask_key = MaskKey {
            symbol,
            points,
            pixels,
            weight,
            natural,
        };
        let image_key = ImageKey {
            mask: mask_key.clone(),
            color,
        };
        if let Some(image) = cache.images.get(&image_key) {
            // A hot tinted image also keeps its reusable alpha mask hot.
            cache.masks.get(&mask_key);
            return Some(image);
        }
        let mask = match cache.masks.get(&mask_key) {
            Some(mask) => mask,
            None => {
                let mask = Arc::new(rasterize(&mask_key)?);
                cache.masks.insert(mask_key, mask.clone(), mask.alpha.len());
                mask
            }
        };
        let bytes = tint(&mask.alpha, color);
        let buffer = RgbaImage::from_raw(mask.pixels.into(), mask.pixels.into(), bytes)?;
        // RenderImage deliberately consumes BGRA, despite image::RgbaImage's name.
        let image = Arc::new(RenderImage::new(vec![Frame::new(buffer)]));
        cache.images.insert(
            image_key,
            image.clone(),
            usize::from(mask.pixels).pow(2) * 4,
        );
        Some(image)
    })
}

fn tint(alpha: &[u8], color: [u8; 4]) -> Vec<u8> {
    alpha
        .iter()
        .flat_map(|coverage| {
            [
                color[2],
                color[1],
                color[0],
                ((*coverage as u16 * color[3] as u16 + 127) / 255) as u8,
            ]
        })
        .collect()
}

fn resolve_name(name: IconName) -> Option<String> {
    autoreleasepool(|_| {
        let preferred = preferred_symbol(name);
        let path = name.path();
        let conventional = path
            .rsplit('/')
            .next()?
            .strip_suffix(".svg")?
            .replace('-', ".");
        // Common names (star, bell, arrow.left, etc.) resolve directly, while
        // explicit translations cover Lucide terminology that differs from SF.
        for candidate in preferred
            .into_iter()
            .chain(std::iter::once(conventional.as_str()))
        {
            if NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str(candidate),
                None,
            )
            .is_some()
            {
                return Some(candidate.to_owned());
            }
        }
        None
    })
}

fn rasterize(key: &MaskKey) -> Option<AlphaMask> {
    autoreleasepool(|_| {
        let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(&key.symbol),
            None,
        )?;
        // SF symbols use the same nine optical weights as system typography.
        let weight = [-0.8, -0.6, -0.4, 0., 0.23, 0.3, 0.4, 0.56, 0.62][key.weight as usize - 1];
        let (config_points, symbol_scale) = match key.natural {
            Some((points, true)) => (points, NSImageSymbolScale::Large),
            Some((points, false)) => (points, NSImageSymbolScale::Medium),
            None => (key.points, NSImageSymbolScale::Medium),
        };
        let config = NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
            config_points as f64 / 64.,
            weight,
            symbol_scale,
        )
        .configurationByApplyingConfiguration(
            &NSImageSymbolConfiguration::configurationPreferringMonochrome(),
        );
        let symbol = symbol.imageWithSymbolConfiguration(&config)?;
        let pixels = usize::from(key.pixels);
        // SAFETY: null planes ask AppKit to allocate and own one interleaved
        // four-channel, eight-bit buffer. Explicit row length excludes padding.
        let bitmap = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bitmapFormat_bytesPerRow_bitsPerPixel(
                NSBitmapImageRep::alloc(), std::ptr::null_mut(), pixels as isize, pixels as isize,
                8, 4, true, false, NSDeviceRGBColorSpace, NSBitmapFormat::empty(),
                (pixels * 4) as isize, 32,
            )
        }?;
        let data = bitmap.bitmapData();
        if data.is_null() {
            return None;
        }
        // SAFETY: the owned bitmap above allocates pixels * pixels * 4 bytes;
        // it remains alive through drawing and the final copy of alpha values.
        unsafe { std::ptr::write_bytes(data, 0, pixels * pixels * 4) };
        // A natural symbol draws in a point-sized context at the box's pixel
        // density, so AppKit renders it at its true size. NSImage rounds a
        // symbol's size to whole points; stretching that size over pixels
        // would enlarge it (paintpalette at 13 pt large: 44 x 36 px instead
        // of AppKit's 40 x 34 at 2x).
        let box_points = key.points as f64 / 64.;
        if key.natural.is_some() {
            bitmap.setSize(NSSize::new(box_points, box_points));
        }
        let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(&bitmap)?;
        let intrinsic = symbol.size();
        if intrinsic.width <= 0. || intrinsic.height <= 0. {
            return None;
        }
        let (extent, fit) = match key.natural {
            // Natural size in points, shrunk only if it would overflow the box.
            Some(_) => (
                box_points,
                (box_points / intrinsic.width.max(intrinsic.height)).min(1.),
            ),
            None => (
                pixels as f64,
                pixels as f64 / intrinsic.width.max(intrinsic.height),
            ),
        };
        let size = NSSize::new(intrinsic.width * fit, intrinsic.height * fit);
        let rect = NSRect::new(
            NSPoint::new((extent - size.width) / 2., (extent - size.height) / 2.),
            size,
        );
        NSGraphicsContext::saveGraphicsState_class();
        NSGraphicsContext::setCurrentContext(Some(&context));
        symbol.drawInRect_fromRect_operation_fraction(
            rect,
            NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 0.)),
            NSCompositingOperation::SourceOver,
            1.,
        );
        NSGraphicsContext::restoreGraphicsState_class();
        // SAFETY: identical owned allocation bounds as the initialized buffer.
        let bytes = unsafe { std::slice::from_raw_parts(data, pixels * pixels * 4) };
        let alpha = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[3])
            .collect::<Vec<_>>();
        if !alpha.iter().any(|alpha| *alpha > 0) {
            return None;
        }
        Some(AlphaMask {
            pixels: key.pixels,
            alpha,
        })
    })
}

fn preferred_symbol(name: IconName) -> Option<&'static str> {
    Some(match name {
        IconName::Check => "checkmark",
        IconName::Minus => "minus",
        IconName::Plus => "plus",
        IconName::ChevronDown => "chevron.down",
        IconName::ChevronUp => "chevron.up",
        IconName::ChevronLeft => "chevron.left",
        IconName::ChevronRight => "chevron.right",
        IconName::ChevronsUpDown => "chevron.up.chevron.down",
        IconName::Calendar => "calendar",
        IconName::Clock => "clock",
        IconName::Star => "star",
        IconName::ExternalLink => "arrow.up.right.square",
        IconName::Close => "xmark",
        IconName::Search => "magnifyingglass",
        IconName::Menu => "line.3.horizontal",
        IconName::Settings => "gearshape",
        IconName::SlidersHorizontal => "slider.horizontal.3",
        IconName::TriangleAlert => "exclamationmark.triangle",
        IconName::CircleCheck => "checkmark.circle",
        IconName::AppWindow => "macwindow",
        IconName::ChartLine => "chart.xyaxis.line",
        IconName::Command => "command",
        IconName::File => "doc",
        IconName::FileText => "doc.text",
        IconName::Folder => "folder",
        IconName::Layers => "square.3.layers.3d",
        IconName::MessageSquare => "bubble.left",
        IconName::PanelLeft => "sidebar.left",
        IconName::PanelsTopLeft => "rectangle.split.2x2",
        IconName::TextCursorInput => "textbox",
        IconName::Users => "person.2",
        _ => return translated_symbol(name.path().as_ref()),
    })
}

fn translated_symbol(path: &str) -> Option<&'static str> {
    let stem = path.rsplit('/').next()?.strip_suffix(".svg")?;
    Some(match stem {
        "arrow-down-left" => "arrow.down.left",
        "arrow-down-right" => "arrow.down.right",
        "arrow-up-left" => "arrow.up.left",
        "arrow-up-right" => "arrow.up.right",
        "arrow-left-right" => "arrow.left.arrow.right",
        "arrow-up-down" => "arrow.up.arrow.down",
        "arrow-big-down" => "arrow.down",
        "arrow-big-left" => "arrow.left",
        "arrow-big-right" => "arrow.right",
        "arrow-big-up" => "arrow.up",
        "arrow-down-to-line" | "download" => "arrow.down.to.line",
        "arrow-up-from-line" | "upload" => "arrow.up.to.line",
        "arrow-down-to-dot" => "arrow.down.to.line.compact",
        "arrow-up-to-line" => "arrow.up.to.line",
        "chevrons-down" => "chevron.down.2",
        "chevrons-left" => "chevron.left.2",
        "chevrons-right" => "chevron.right.2",
        "chevrons-up" => "chevron.up.2",
        "corner-down-left" => "arrow.turn.down.left",
        "corner-down-right" => "arrow.turn.down.right",
        "corner-up-left" => "arrow.turn.up.left",
        "corner-up-right" => "arrow.turn.up.right",
        "move" => "arrow.up.and.down.and.arrow.left.and.right",
        "move-horizontal" => "arrow.left.and.right",
        "move-vertical" => "arrow.up.and.down",
        "maximize" | "maximize-2" => "arrow.up.left.and.arrow.down.right",
        "minimize" | "minimize-2" => "arrow.down.right.and.arrow.up.left",
        "refresh-cw" | "rotate-cw" => "arrow.clockwise",
        "refresh-ccw" | "rotate-ccw" => "arrow.counterclockwise",
        "undo" | "undo-2" => "arrow.uturn.backward",
        "redo" | "redo-2" => "arrow.uturn.forward",
        "play" => "play.fill",
        "pause" => "pause.fill",
        "stop" | "square" => "stop",
        "skip-back" => "backward.end.fill",
        "skip-forward" => "forward.end.fill",
        "fast-forward" => "forward.fill",
        "rewind" => "backward.fill",
        "volume" | "volume-1" => "speaker.wave.1",
        "volume-2" => "speaker.wave.2",
        "volume-x" => "speaker.slash",
        "mic" => "mic",
        "mic-off" => "mic.slash",
        "video" => "video",
        "video-off" => "video.slash",
        "circle-check" | "circle-check-2" => "checkmark.circle",
        "circle-x" => "xmark.circle",
        "circle-plus" => "plus.circle",
        "circle-minus" => "minus.circle",
        "circle-alert" => "exclamationmark.circle",
        "circle-help" => "questionmark.circle",
        "square-check" | "square-check-big" => "checkmark.square",
        "square-x" => "xmark.square",
        "square-plus" => "plus.square",
        "square-minus" => "minus.square",
        "info" => "info.circle",
        "eye-off" => "eye.slash",
        "lock" | "lock-keyhole" => "lock",
        "lock-open" | "lock-keyhole-open" => "lock.open",
        "shield-check" => "checkmark.shield",
        "shield-alert" => "exclamationmark.shield",
        "user" | "user-round" => "person",
        "users" | "users-round" => "person.2",
        "user-plus" | "user-round-plus" => "person.badge.plus",
        "user-minus" | "user-round-minus" => "person.badge.minus",
        "user-check" | "user-round-check" => "person.crop.circle.badge.checkmark",
        "user-x" | "user-round-x" => "person.crop.circle.badge.xmark",
        "circle-user" | "circle-user-round" => "person.crop.circle",
        "contact" => "person.crop.square",
        "log-in" => "rectangle.portrait.and.arrow.right",
        "log-out" => "rectangle.portrait.and.arrow.forward",
        "home" | "house" => "house",
        "building" | "building-2" => "building.2",
        "file" => "doc",
        "file-text" => "doc.text",
        "file-code" | "file-code-2" => "doc.text",
        "file-plus" | "file-plus-2" => "doc.badge.plus",
        "file-check" | "file-check-2" => "checkmark.rectangle",
        "file-x" | "file-x-2" => "doc.badge.xmark",
        "files" | "copy" => "doc.on.doc",
        "folder-open" => "folder",
        "folder-plus" => "folder.badge.plus",
        "folder-minus" => "folder.badge.minus",
        "archive" => "archivebox",
        "clipboard" | "clipboard-copy" => "doc.on.clipboard",
        "clipboard-check" => "checklist",
        "clipboard-list" => "list.clipboard",
        "save" => "square.and.arrow.down",
        "share" | "share-2" => "square.and.arrow.up",
        "trash" | "trash-2" => "trash",
        "pencil" | "pencil-line" => "pencil",
        "square-pen" | "notebook-pen" => "square.and.pencil",
        "image" | "image-plus" => "photo",
        "images" => "photo.on.rectangle",
        "mail" => "envelope",
        "mail-open" => "envelope.open",
        "mail-plus" => "envelope.badge",
        "send" => "paperplane",
        "message-square" => "bubble.left",
        "message-circle" => "bubble.left",
        "messages-square" => "bubble.left.and.bubble.right",
        "bell-off" => "bell.slash",
        "bell-ring" => "bell.badge",
        "ellipsis" | "more-horizontal" => "ellipsis",
        "ellipsis-vertical" | "more-vertical" => "ellipsis",
        "list" | "list-collapse" => "list.bullet",
        "list-ordered" => "list.number",
        "list-checks" | "list-todo" => "checklist",
        "align-left" => "text.alignleft",
        "align-center" => "text.aligncenter",
        "align-right" => "text.alignright",
        "align-justify" => "text.justify",
        "bold" => "bold",
        "italic" => "italic",
        "underline" => "underline",
        "strikethrough" => "strikethrough",
        "quote" => "quote.opening",
        "code" | "code-2" => "chevron.left.forwardslash.chevron.right",
        "terminal" | "square-terminal" => "terminal",
        "link" | "link-2" => "link",
        "panel-left" | "panel-left-close" | "panel-left-open" => "sidebar.left",
        "panel-right" | "panel-right-close" | "panel-right-open" => "sidebar.right",
        "panel-top" => "rectangle.topthird.inset.filled",
        "panel-bottom" | "panel-bottom-open" => "rectangle.bottomthird.inset.filled",
        "panels-top-left" | "layout-dashboard" => "rectangle.split.2x2",
        "columns-2" => "rectangle.split.2x1",
        "rows-2" => "rectangle.split.1x2",
        "grid-2x2" | "grid-3x3" | "layout-grid" => "square.grid.2x2",
        "chart-bar" | "chart-column" | "chart-no-axes-column" => "chart.bar",
        "chart-line" | "chart-area" => "chart.xyaxis.line",
        "chart-pie" => "chart.pie",
        "filter" => "line.3.horizontal.decrease",
        "sliders-horizontal" => "slider.horizontal.3",
        "settings-2" => "slider.horizontal.3",
        "search" => "magnifyingglass",
        "zoom-in" => "plus.magnifyingglass",
        "zoom-out" => "minus.magnifyingglass",
        "cloud-download" => "icloud.and.arrow.down",
        "cloud-upload" => "icloud.and.arrow.up",
        "cloud-off" => "icloud.slash",
        "map-pin" => "mappin",
        "navigation" => "location",
        "compass" => "safari",
        "monitor" => "display",
        "laptop" => "laptopcomputer",
        "smartphone" => "iphone",
        "tablet" => "ipad",
        "database" => "externaldrive",
        "hard-drive" => "internaldrive",
        "plug" => "powerplug",
        "battery" | "battery-full" => "battery.100percent",
        "battery-medium" => "battery.50percent",
        "battery-low" => "battery.25percent",
        "battery-charging" => "battery.100percent.bolt",
        "wifi-off" => "wifi.slash",
        "sun" => "sun.max",
        "moon" => "moon",
        "snowflake" => "snowflake",
        "flame" => "flame",
        "zap" => "bolt",
        "zap-off" => "bolt.slash",
        "sparkles" => "sparkles",
        "asterisk" => "asterisk",
        "braces" => "curlybraces",
        "book-open" => "book",
        "box" => "cube",
        "brain" => "brain.head.profile",
        "case-sensitive" => "textformat",
        "a-large-small" => "textformat.size",
        "ban" => "nosign",
        "dash" => "minus",
        "frame" => "viewfinder",
        "inbox" => "tray",
        "chart-no-axes-combined" => "chart.xyaxis.line",
        "check-check" => "checkmark.2",
        "circle-chevron-down" => "chevron.down.circle",
        "circle-chevron-left" => "chevron.left.circle",
        "circle-chevron-right" => "chevron.right.circle",
        "circle-chevron-up" => "chevron.up.circle",
        "construction" => "hammer",
        "delete" => "delete.left",
        "dot" => "circle.fill",
        "earth" => "globe.europe.africa",
        "file-search" | "file-search-2" => "doc.text.magnifyingglass",
        "folder-closed" => "folder",
        "gallery-vertical-end" => "square.stack",
        "heart-off" => "heart.slash",
        "inspector" => "sidebar.right",
        "loader" | "loader-circle" => "progress.indicator",
        "memory-stick" => "memorychip",
        "palette" => "paintpalette",
        "replace" => "arrow.triangle.2.circlepath",
        "resize-corner" => "arrow.up.left.and.arrow.down.right",
        "sort-ascending" | "arrow-down-wide-narrow" => "arrow.up",
        "sort-descending" | "arrow-up-wide-narrow" => "arrow.down",
        "star-fill" => "star.fill",
        "star-off" => "star.slash",
        "thumbs-up" => "hand.thumbsup",
        "thumbs-down" => "hand.thumbsdown",
        "window-close" | "x" => "xmark",
        "window-maximize" => "arrow.up.left.and.arrow.down.right",
        "window-minimize" => "minus",
        "window-restore" => "arrow.down.right.and.arrow.up.left",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_symbol_weights_fall_back_without_indexing_the_optical_weights() {
        for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(image(IconName::Check, 16., 2., FontWeight(weight), Hsla::white()).is_none());
        }
    }

    #[test]
    fn symbol_cache_evicts_cold_entries_without_clearing_hot_artwork() {
        let mut cache = SymbolLru::new(3, 100);
        for key in 0..3 {
            cache.insert(key, key, 10);
        }
        assert_eq!(cache.get(&0), Some(0));
        cache.insert(3, 3, 10);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&0), Some(0));
        assert_eq!(cache.get(&2), Some(2));
        assert_eq!(cache.get(&3), Some(3));
        assert_eq!(cache.bytes, 30);
    }

    #[test]
    fn symbol_cache_bounds_raster_bytes_and_accounts_for_replacements() {
        let mut cache = SymbolLru::new(10, 8);
        cache.insert(0, "small", 2);
        cache.insert(1, "small", 2);
        cache.insert(0, "larger", 6);
        assert_eq!(cache.bytes, 8);
        cache.insert(2, "large", 6);
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.get(&2), Some("large"));
        assert_eq!(cache.bytes, 6);
        cache.insert(3, "oversized", 9);
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.bytes, 6);
    }

    /// Sidebar symbols (tokens::SIDEBAR_SYMBOL_*): 13 pt, large scale,
    /// medium weight at their natural size in the 26 pt slot reproduce the
    /// SwiftUI sidebar's ink boxes at 2x (measured against the SwiftUI
    /// reference app, tools/swiftui-reference: 30 x 31 and 40 x 34 px, within
    /// 1 px), where fitting each symbol to a 24 pt box gave 37 x 37 and
    /// 42 x 34. Each symbol keeps its own width.
    ///
    /// Baseline: host SF Symbols artwork on macOS 26 and 27 (captured on
    /// macOS 27). Other releases may redraw the artwork, so there the test
    /// only checks that natural symbols render.
    #[test]
    fn natural_sidebar_symbols_match_the_native_ink_boxes() {
        let os_major = objc2_foundation::NSProcessInfo::processInfo()
            .operatingSystemVersion()
            .majorVersion;
        let baseline = matches!(os_major, 26 | 27);
        let natural = NaturalSymbol {
            points: 13.,
            large: true,
        };
        let ink_box = |name: IconName| {
            let image = image_in_box(
                name,
                26.,
                2.,
                FontWeight::MEDIUM,
                Hsla::white(),
                Some(natural),
            )
            .unwrap();
            let pixels = image.as_bytes(0).unwrap();
            let side = (pixels.len() / 4).isqrt();
            assert_eq!(side, 52, "a 26 pt box at 2x");
            let (mut left, mut right, mut top, mut bottom) = (side, 0, side, 0);
            for (index, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
                if pixel[3] > 128 {
                    let (x, y) = (index % side, index / side);
                    (left, right) = (left.min(x), right.max(x));
                    (top, bottom) = (top.min(y), bottom.max(y));
                }
            }
            (
                (right + 1).saturating_sub(left),
                (bottom + 1).saturating_sub(top),
            )
        };
        for (name, native) in [
            (IconName::Grid2x2, (30, 31)),
            (IconName::Palette, (40, 34)),
            (IconName::AppWindow, (39, 31)),
        ] {
            let (width, height) = ink_box(name);
            assert!(width > 0 && height > 0, "{name:?} renders");
            if baseline {
                assert!(
                    width.abs_diff(native.0) <= 1 && height.abs_diff(native.1) <= 1,
                    "{name:?}: {width} x {height} px against the macOS {os_major} native {native:?}"
                );
            }
        }
        // Natural symbols are cached apart from fitted ones of the same box.
        let fitted = image(
            IconName::Grid2x2,
            26.,
            2.,
            FontWeight::MEDIUM,
            Hsla::white(),
        );
        let natural = image_in_box(
            IconName::Grid2x2,
            26.,
            2.,
            FontWeight::MEDIUM,
            Hsla::white(),
            Some(natural),
        );
        assert!(!Arc::ptr_eq(&fitted.unwrap(), &natural.unwrap()));
        assert!(
            image_in_box(
                IconName::Grid2x2,
                26.,
                2.,
                FontWeight::MEDIUM,
                Hsla::white(),
                Some(NaturalSymbol {
                    points: f32::NAN,
                    large: true
                })
            )
            .is_none()
        );
    }

    #[test]
    fn tint_preserves_antialiasing_and_inherited_alpha_in_bgra() {
        assert_eq!(
            tint(&[0, 128, 255], [20, 40, 80, 128]),
            [80, 40, 20, 0, 80, 40, 20, 64, 80, 40, 20, 128]
        );
    }

    #[test]
    fn the_host_renders_every_symbol_used_by_native_controls_and_gallery() {
        for name in [
            IconName::AppWindow,
            IconName::Calendar,
            IconName::ChartLine,
            IconName::Check,
            IconName::ChevronDown,
            IconName::ChevronLeft,
            IconName::ChevronRight,
            IconName::ChevronUp,
            IconName::ChevronsUpDown,
            IconName::CircleCheck,
            IconName::Clock,
            IconName::Close,
            IconName::Command,
            IconName::ExternalLink,
            IconName::File,
            IconName::FileText,
            IconName::Folder,
            IconName::Layers,
            IconName::Menu,
            IconName::MessageSquare,
            IconName::Minus,
            IconName::PanelLeft,
            IconName::PanelsTopLeft,
            IconName::Plus,
            IconName::Search,
            IconName::Settings,
            IconName::SlidersHorizontal,
            IconName::Star,
            IconName::TextCursorInput,
            IconName::TriangleAlert,
            IconName::Users,
        ] {
            let rendered = image(
                name,
                16.,
                2.,
                FontWeight::NORMAL,
                gpui_kit::rgb(0x007aff).into(),
            )
            .unwrap_or_else(|| {
                panic!(
                    "the current host should supply {name:?}: {:?}",
                    preferred_symbol(name)
                )
            });
            let bytes = rendered.as_bytes(0).unwrap();
            assert_eq!(bytes.len(), 32 * 32 * 4);
            assert!(
                bytes.as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0),
                "{name:?}"
            );
            assert!(
                bytes.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 0),
                "{name:?}: preserves transparency"
            );
            let cached = image(
                name,
                16.,
                2.,
                FontWeight::NORMAL,
                gpui_kit::rgb(0x007aff).into(),
            )
            .unwrap();
            assert!(
                Arc::ptr_eq(&rendered, &cached),
                "{name:?}: reuses atlas identity"
            );
        }
    }

    #[test]
    fn catalog_coverage_counts_native_symbols_and_fallbacks_on_this_host() {
        use gpui_kit::AssetSource as _;
        let total = IconName::ALL.len();
        let native = IconName::ALL
            .iter()
            .copied()
            .filter(|name| {
                image(
                    *name,
                    16.,
                    1.,
                    FontWeight::NORMAL,
                    gpui_kit::rgb(0xffffff).into(),
                )
                .is_some()
            })
            .count();
        let cached = CACHE.with_borrow(|cache| cache.names.len());
        assert_eq!(
            cached, total,
            "both successful and failed lookups are cached"
        );
        assert!(
            native >= 31,
            "native controls are covered on the supported host"
        );
        println!(
            "SF Symbol catalog coverage on this host: {native}/{total} native; {} Lucide fallbacks; {cached} cached lookups",
            total - native
        );
        let defaults = gpui_kit::assets::Assets.list("icons/").unwrap();
        let default_fallbacks = CACHE.with_borrow(|cache| {
            defaults
                .iter()
                .filter(|path| {
                    PATH_NAMES
                        .get(path.as_str())
                        .is_none_or(|name| cache.names.get(name).is_none_or(Option::is_none))
                })
                .collect::<Vec<_>>()
        });
        println!(
            "GPUI Kit default bundle: {}/{} native; {} fallbacks: {default_fallbacks:?}",
            defaults.len() - default_fallbacks.len(),
            defaults.len(),
            default_fallbacks.len()
        );
    }
}
