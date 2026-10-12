# Design and architecture

The design target is Apple's current macOS 27 design, using the public
[Apple Design Resources](https://developer.apple.com/design/resources/),
[materials guidance](https://developer.apple.com/design/human-interface-guidelines/materials),
and [Modernize your AppKit app](https://developer.apple.com/videos/play/wwdc2026/289/).
The implementation uses system typography, plain content backgrounds, quiet
gray preference groups, accent-colored controls, and glass reserved for
navigation and command groups. The macOS 27 sidebar extends to the window edge;
its selected label uses semibold weight. Toolbars are clear layouts containing
shared command groups, rather than a solid band of individually outlined buttons.

The architectural reference is
[gpui-omarchy](https://github.com/huacnlee/gpui-omarchy) at
`e63ff32b73299d4f8abd8401af5d04924b2a9d18`. Its behavioral adapters and gallery
are retained under MIT and extensively restyled. GPUI Kit is pinned to
`0bbd9870ce420af9feacbf948b21d3824d82dc0b` so the base/facade and renderer APIs
cannot drift independently. Cargo.lock records the transitive renderer versions.

## Layers

1. GPUI Kit provides the runtime, native windows, GPU rendering and assets.
2. GPUI Base provides controlled-state primitives, editors, focus, keyboard
   actions, accessibility, overlays, lists, dock layout and motion samplers.
3. gpui-macos adapters supply native geometry, colors and interaction feedback.
4. The advanced Kit facade preserves editor, diff, form, data table, command,
   questionnaire, media and message APIs with synchronized macOS tokens.
5. The AppKit bridge hosts genuine unified toolbars, native search, glass
   navigation underlays and separate glass popup windows. `NativeAssets` supplies host SF Symbols
   to both the adapters and nested Kit icons. Applications own navigation state.

`Theme::apply_palette` sets the macOS global, synchronizes Kit, then projects
native Base tokens, appearance and plot motion synchronously on the UI thread.
One retained Base-global observer repairs later Kit overwrites. It compares
tokens and appearance immutably before writing, so a repair schedules one equal
pass and terminates. Base is repaired before the next ordinary frame; an explicit
`window.draw` inside the same Kit-update callback can still see Kit's projection.
Scrollbar and resizable settings are never changed by the native projection. Each window receives
the same global palette. Explicit appearance also updates native window chrome;
following the system clears that override. Appearance subscriptions are scoped
to the lifetime of each window.

## Typography

`src/tokens.rs` holds native values copied by hand from `docs/native-spec.md`,
with section citations. `TextStyle` and `NativeTextStyle::native_text_style`
apply the system family, role weight, size and absolute line advance in rems.
Roots inherit Body (13/16 points); Title3 uses the measured 20 point advance.
Base typography is xs 10/13, sm 11/14, md 13/16, lg 15/20, xl 17/22;
Base mono_md maps to Callout (12/15 points). Base radii are 5, 6, 12, 16 points.
Raw Kit controls retain Kit typography.
The token lint ratchets production literals independently of examples and tests.

Checkbox, Radio and Switch build state-dependent children at render time through
`child_with_state(ControlState)`. The final disabled flag is available even when
set after children are added. `child_with_disabled` adapts the same hook to bool;
`focus_visible` is reserved and currently false, preserving existing focus behavior.

## Geometry

The default rem size is 16 pixels. Native labels are 13 pixels, small auxiliary
labels 11 pixels, and regular buttons, fields, segments, and switches are
24 pixels tall. These dimensions come from the Swift reference app's actual
AppKit intrinsic sizes on macOS 26.2. Regular controls use 6px radii; groups use
10px and large surfaces 16px. Switches use a 54px track and an elongated pill
thumb; checkbox and radio indicators are 16px. Physical
hairline borders remain one pixel. Virtual-list measurements convert rem
units using the current window so zoom does not desynchronize row sizes.

## Materials and accessibility

`window_options` asks the macOS backend for a blurred native window backdrop.
Applications installing AppKit chrome use `native_window_options` instead;
its transparent renderer avoids the separate selection-material blur layer.
Sidebars and opt-in `native_material` surfaces host actual `NSGlassEffectView`
underlays on macOS 26 and newer. The view follows GPUI layout and clipping,
ignores pointer hit testing, and leaves the renderer and window delegate intact.
Opaque content areas sit above it. `native_material` underlays sample the
window backdrop. A sidebar's glass sits on its own opaque backdrop view, as in
SwiftUI's `NavigationSplitView`, whose opaque window gives its sidebar glass
that backdrop rather than the desktop. The earlier `.contentBackground` layer
was effective (our glass measured exactly free regular glass over the window
colour), but the native sidebar glass is darker than that; the backdrop is now
a custom approximation calibrated so regular glass matches the native colour
in an active window (verified by capture: within 1/255 light, exact dark). Anchored popup factories use separate GPUI windows, whose
original content views are embedded in `NSGlassEffectView.contentView`.
Their real material samples the owning GPUI canvas underneath, while the
unchanged renderer draws interactive foreground content. Keeping the original
content view preserves AccessKit's dynamically installed accessibility adapter.
Retained factories create elements in each presenting window's current frame;
`AnyElement` values are never transferred between frame arenas. Scoped leases
remove popup windows when their triggers unmount or their owners close.
Reduced transparency and unsupported hosts use opaque fallback surfaces.

`native::install_native_toolbar` installs a real unified `NSToolbar` with native
SF Symbol commands and optional `NSSearchField` items. AppKit draws its window
corners and toolbar groups. The retained handle restores prior chrome when
dropped; actions defer into GPUI and safely ignore closed windows. Native
examples reserve the measured `native_toolbar_inset` and let AppKit place the
traffic lights. Native controls resolve activation independently for each window.

For a sidebar that spans the window height, call the toolbar handle's
`set_sidebar_width` with its current logical width. This makes the native
titlebar transparent and reserves a left titlebar accessory so the window
title starts after navigation. Zero removes that accessory and restores
ordinary toolbar treatment. Fullscreen exit preserves the current policy;
dropping the handle removes its accessory and restores the preceding chrome.
Only the detail pane should reserve `native_toolbar_inset` in this layout.

Sidebar rows use a cached AppKit source list to read the system row height.
Inactive material selection follows measured source-list paint; reduced
transparency and increased contrast use opaque semantic selection colors.
Category disclosure controls preserve the selected component. The gallery
reveals a collapsed group when keyboard navigation enters it.

The theme provides clear-to-tinted material control and an opaque accessibility
fallback. It reads system Reduce transparency and Increase contrast when
following appearance; `refresh_accessibility` allows an application to re-read
runtime changes. Base motion samplers honor Reduce motion. Native traffic
lights belong to the operating system; no replacements are painted.

Controls preserve base accessible roles/names and activation semantics.
Disabled buttons withhold hover, pressed and focus-visible styles. Secure
editors mask both visual values and clipboard behavior. Sheets and dialogs
own focus traps; close handlers restore a stable trigger handle. Hover cards
should contain supplementary information only.

The local GPUI Base button extension writes AccessKit's disabled flag on the
original button node, preserving its identity, label, role and child structure.
Disabled commands leave focus traversal and suppress pointer and keyboard
activation. Node metadata and those interaction guarantees are tested.
Sidebar commands explicitly expose their selected state.

## Coverage and validation

The typed catalog is the inventory for documentation and gallery navigation.
It includes all upstream component documentation topics, the seven chart types,
and additional macOS compositions. Catalog tests reject duplicate or missing
identifiers. The gallery sweep renders every page in light and dark mode, while
focused interaction tests exercise real pointer/keyboard/focus behavior.

Visual verification uses the gallery and a real Swift/AppKit comparison app
through RiWork's Cua.ai Driver. The reference exports intrinsic control sizes
and semantic colors in both appearances. The host and SDK run macOS 26.2;
macOS 27 refinements use Apple's public component preview and WWDC guidance.
The complete Figma/Sketch UI Kit has not been downloaded. Background captures
show inactive AppKit controls, which can suppress their accent tint.
macOS-specific blur and window decorations are not covered by headless rendering.

Optional speech integration remains platform-dependent and requires explicit
system permission when used. Editor highlighting depends on enabled Tree-sitter
features. Components retain upstream API/maturity constraints; the library does
not replace them with reduced facsimiles.
