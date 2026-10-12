# Changelog

## Unreleased — M5 menus, tooltips, popovers

### Polish

- Added `list_rows(cx)`: the `.inset` list layout for `list_item` rows
  (spec §5.2, §6.4). Rows follow each other at the 24 pt pitch with no gap,
  10 pt below the top and above the bottom, inset 11.5 pt from each side.
- `interpretation::list_item` rows have the 5 pt selection radius and a 4 pt
  content inset (was Kit's 12 pt), and set their 4 pt vertical padding
  explicitly. The gallery list uses `list_rows`; its row pitch is now 24 pt
  (was 26: the gallery added a 2 pt gap between rows), its first row starts
  10 pt into the list and its selection floats inside the list with rounded
  corners. A test asserts the pitch between consecutive rows on 1x and 2x
  displays at 100% and 200% zoom. `inset_list_row` (40 pt) is unchanged.
- Added `sidebar_section_header(id, title, expanded, cx)`: a 19 pt
  collapsible group header with an 11 pt semibold label in the tertiary
  label color (was the secondary label in the gallery). Its disclosure
  chevron appears only while the pointer is over the header or it has
  keyboard focus; the header stays a focusable button that reports its
  expanded state. Added `sidebar_group_spacing()`, the 13 pt space above
  every group but the first. Long titles truncate on one line and the
  chevron keeps a fixed slot, so revealing it never moves the label.
- `sidebar_section` uses the tertiary label color (was the secondary label).
- Gallery sidebar: later groups now have 13 pt of leading space. The header
  used a top margin, which GPUI's virtualized list does not measure, so each
  later group header overlapped the row above it and the rows below drifted
  13 pt upward per group.
- The native toolbar title starts 20 pt after the sidebar (240 pt for the
  220 pt sidebar; was 228), as in a SwiftUI `NavigationSplitView` (spec
  §5.3).
- `sidebar` keeps its regular Liquid Glass (`NSGlassEffectView`, no tint,
  radius 0) but now puts an opaque `.contentBackground` layer
  (`NSVisualEffectView`, within-window, active) under it. A SwiftUI
  `NavigationSplitView` sidebar is the same glass in an opaque window with
  that layer behind everything, so its glass samples the window background.
  Ours sat in a transparent window and sampled the desktop, so it was
  lighter (about #FEFEFE light and #373737 dark against the native #EDEDED
  and #262626 in the comparison captures). Known issue: a capture after
  this change still measured the lighter values, so this layer does not
  yet give the glass an opaque backdrop behind our transparent window;
  matching the native sidebar background is open. Fallback paint (older systems,
  headless, Reduce Transparency) is unchanged, and so are `native_material`
  and `native_toolbar_group`, which still sample behind the window.

### Changed

- Menus and submenus use native metrics (spec §7.3): rows are 24 pt (was
  28 pt plus a 2 pt gap). Rows sit 5 pt from the leading edge, 5.5 pt from
  the trailing edge and 5 pt from the top and bottom (was 6 pt inside the
  border). The panel radius is 13 (was 12), the highlight radius 6.5 (was 6)
  and a separator row 11 pt (was 9). Text is 13 pt. The highlight is the
  accent with a white label, under the pointer and on the keyboard cursor.
  Disabled items use the tertiary label color instead of dimming the whole
  row to 45%. Destructive items are not red. The inline submenu is now
  opaque, like the menu.
- Select and combobox popups follow the same rule (decision D10): 24 pt rows,
  the same 5 / 5.5 / 5 pt insets, radius 13, highlight radius 6.5, an 8 pt
  label inset (was 6) and 13 pt menu text. They use the accent highlight for
  the pointer and the keyboard cursor. Disabled options use the tertiary
  label color instead of 45% dimming.
- Tooltips are 20 pt tall (was 28) with radius 5 (was 6) and no border.
  Text is 11 pt in the label color. They use the tooltip's own material:
  #E7E7E7 @0.898 in light and #3A3A3A @0.828 in dark (was the generic glass
  fill). Under Reduce Transparency the same tint is opaque.
- Popovers have radius 16 (was 14) and padding 16 (was 14), with no arrow.
  This shifts the layout of content inside `popover`, `popover_surface`,
  hover cards and the color picker popup by 2 pt per side.
- Popups that the library presents in native glass (menus, submenus,
  select and combobox popups, popovers, hover cards and the color picker)
  now get their corner radius from their own content, at the owner's zoom:
  13 for menus, submenus and select or combobox popups, and 16 for popovers,
  hover cards and the color picker. Before, these popups all used the
  `GlassPanelOptions` default of 14. The declared radius is passed to the
  glass when it is installed, after the panel's first content frame; this
  is covered by tests but not yet confirmed on screen. A popup opened from inside another popup changes only its
  own panel. Panels that applications open with `open_glass_panel` keep the
  `corner_radius` they pass. The `GlassPanelOptions` API and its default of
  14 are unchanged. The public `popover_surface(cx)` only sets the drawn
  radius and padding; it does not declare a glass radius. The date picker's
  calendar popup keeps 14.
- Menu, submenu, select and combobox rows have one highlight: the row
  cursor. The keyboard and the pointer both move it, so a resting pointer no
  longer leaves a second highlighted row after an arrow key.
- SwiftUI reference app: popovers use 16 pt padding (`.padding()`), and the
  mirrored help tag uses the measured tooltip (20 pt, radius 5, no border,
  tooltip material, opaque under Reduce Transparency), so both apps compare
  the same targets.

## Unreleased — M4 focus ring, hover, selection

### Added

- `Theme::focus_ring` (`focus_ring_control`, about #0C6BFB / #25AFFF @0.49;
  a custom accent at the same alpha), `Theme::selected_content` (#0064E1 /
  #0059D1), `Theme::unemphasized_selected_content` (#DCDCDC / #464646),
  `Theme::alternating_row` (#F4F5F5 / white @0.047) and
  `Theme::tertiary_fill` (ink @0.047).
- `row_selection(emphasized, window, cx) -> (fill, foreground)` for
  application-owned list and table selection: `selected_content` with white
  text while emphasized in a key window, otherwise
  `unemphasized_selected_content` with the label color.
- `Button::selection_emphasis(bool)` (default `true`) for sidebar rows: pass
  whether the sidebar holds keyboard focus.
- Base `ControlState::focus_visible` is now produced by Checkbox, Radio,
  Switch, Toggle, Tab and AccordionTrigger: enabled, focused, last input from
  the keyboard and an active window. Tab and AccordionTrigger take the handle
  passed to `track_focus`. Those controls also gain
  `child_with_resolved_style` (state plus the final resolved style), and
  `Radio::focus_indicator(handle)` reports a group's keyboard focus on its
  current option without tracking the handle twice. `DatePicker`,
  `ColorPicker` and `InputBase` also gain `child_with_resolved_style`.
- `Theme::sidebar_material(window_active)`: the deterministic sidebar
  material for sidebars over opaque content, where real glass cannot show.
- `scripts/compare-swiftui.py` measures several named regions per page from
  one capture. `regions.json` gains `regions` with a `sidebar` region
  (x 0–220 pt below the toolbar), reported for every page next to `content`;
  baseline deltas and the report include it. `metrics.content` keeps its
  shape.

### Changed

- Keyboard focus draws one ring: 3 pt wide and outset 3 pt (both scale with
  zoom) from the border box, with the control's top-left corner radius plus
  3, in `focus_ring` for controls and fields alike. Buttons, toggles, tabs,
  pickers, disclosure triggers and fields read their final resolved base
  radius and top border width (instance and semantic styles; hover, active
  and focus refinements are not considered), so caller overrides are
  followed; the ring is uniform,
  so asymmetric borders or corners get the top-left values. Indicator,
  switch, slider and segment rings use the library's own geometry. It shows
  only for keyboard focus in an active window on an enabled control (fields,
  including one-time-code cells: while focused in an active window and
  enabled, also after a click), and the control's border and fill no longer
  change while focused. The ring surrounds the whole button, pop-up, toggle,
  date and color picker, disclosure trigger and field; the checkbox box
  (22 × 22); the focused radio's circle; the switch track (60 × 30); the
  focused segment; and the slider knob. `focus_shadow`, Base `ring` and Kit
  `ring` use the same color (was accent @0.35 / @0.4 with an accent border).
  Exceptions drawn with `focus_shadow` (same color and width, but GPUI keeps
  the control's own corner radius instead of radius plus 3): links,
  calendar days, dock tabs and the current one-time-code cell.
- Hover follows one rule: bordered, prominent and borderless buttons, toggles,
  tabs, segments, checkboxes, fields, number-input steppers, calendar days,
  disclosure rows, sidebar, tree and table rows, and dock tabs no longer
  change under the pointer. Links keep the pointer cursor but no longer
  underline on hover; the slider knob no longer grows on hover (it still
  grows while dragged). Kit button hover colors (primary, secondary, danger,
  success, warning, info, link) equal rest. Kit list items, table rows and
  diff gutters keep their resting background (selection, stripes) under the
  pointer through a local Kit patch that treats the transparent
  `list_hover` / `table_hover` token as no hover; other raw Kit components
  keep their own hover styles.
- Toolbar buttons are toolbar items: a hover disc of ink @0.05 (light) /
  @0.04 (dark) and the focus ring. Pressed is twice the hover ink
  (@0.10 / @0.08), a library choice: the spec has no measurement for it.
- Sidebar rows (`sidebar_item`) use native selection: opaque accent with a
  white label and symbol while emphasized in a key window; black @0.085 /
  white @0.088 over the material with the label color while unemphasized;
  opaque #E5E5E5 / #363636 with the label color in an inactive window
  (Reduce Transparency: #DFDFDF / #545454 and #E1E1E1 / #232323). Symbols
  no longer fade in an inactive window. `native::sidebar_inactive_selection`
  returns the new inactive value. Extra children added to a sidebar row
  follow its symbol and label.
- Where real glass is unavailable (older systems, headless windows, Reduce
  Transparency), `sidebar()` paints `Theme::sidebar_material` instead of the
  generic glass fallback: #E7E7E7 @0.898 / #2C2C2C @0.816 in an active
  window, opaque #F0F0F0 / #282828 in an inactive one, and #F6F6F6 / #383838
  (active), #ECECEC / #141414 (inactive) under Reduce Transparency. Real
  glass is left to AppKit, including its inactive appearance. The fallback
  layer keeps square corners; a rounded sidebar does not clip it.
- Tree selection uses `row_selection`: emphasized while the window is
  active (#0064E1 / #0059D1, white text), otherwise #DCDCDC / #464646.
- Table rows alternate with `alternating_row` (was foreground @0.04).
- Menu, select and combobox popup rows take their highlight from the menu
  item role; the colors are unchanged until M5.
- Avatars and solid chart cross lines use `tertiary_fill` instead of the
  hover fill.
- Gallery: the component sidebar is emphasized while it holds keyboard
  focus; the sidebar demo paints `sidebar_material` because it sits on opaque
  page content; the sidebar and list pages show unemphasized native selection; the
  radio page rings the keyboard-focused option's circle instead of tinting
  the row; the sidebar group headers have no hover.

## Unreleased — M3 control and row metrics (regular size)

### Added

- `interpretation::list_item`: a Kit `ListItem` with Body text and a 24 pt
  minimum height. Raw `kit::list::ListItem` keeps Kit's 16 pt text.
- `tokens::SIDEBAR_IDEAL_WIDTH` (220 pt), the width `sidebar()` uses.

### Changed

- Icon-only buttons (`icon_button`) now have a 37.5 × 20.5 pt bezel inside a
  39 × 21 pt layout frame and a 10.5 pt glyph. Before, they were a 24 × 24
  square. The hit area is the visible bezel; there is no larger hit target.
  On the device-pixel grid the frame stays exact and the margins absorb the
  bezel's half-point rounding (for example 39 × 21 with a 37 × 20 bezel on a
  1x display at 100%). A caller `.w()`, `.h()`, `.size()` or margin still
  wins on that axis.
- Checkboxes: the box has a 5.5 pt corner radius (was 4) and a labeled row
  is 17 pt tall (was 16), with the box 0.5 pt from the top. Radio buttons
  have a 5 pt dot (was 4). The 5 pt label gap is unchanged.
- Table body rows are 24 pt (cell padding 4 instead of 6; was 28). Header
  cells are 27 pt plus a separator, set as one fixed 28 pt height that
  rounds once on the device grid and that content cannot exceed (was 28 + 1).
  The label is centered on a 27 pt line instead of vertical padding. The separator is a one-point
  hairline at 100% zoom (two device pixels on Retina) that now scales with
  zoom, an approximation of the measured 1.5 pt. Tree rows are 24 pt
  (was 28).
- Native glass corners in `native_material` (12 pt) and
  `native_toolbar_group` (18 pt) now scale with zoom. Before, they stayed
  fixed in pixels.
- Bezel, pop-up, segmented, field, switch, slider and progress sizes now come
  from shared tokens. Their values are unchanged: 24 pt tall controls,
  12 pt padding, radius 6, search field capsule, switch 54 × 24, slider track
  6, linear progress 8.
- Gallery: the sidebar is always 220 pt wide; the narrower sidebar for small
  windows is gone. The table summary row and the toggle-group article rows
  now span the page width. The list page uses `list_item`, so its rows show
  13 pt text instead of 16 pt.

## Unreleased — M2 state rules

### Added

- `ButtonVariant::Borderless` (`.borderless`): secondary label ink, no bezel
  or hover, label ink @0.25 when disabled and @0.85 while pressed. Its frame
  is the label (16 pt tall at regular size). **Breaking:** downstream code
  that matches `ButtonVariant` exhaustively needs a new arm.
- `Theme::bezel_fill`, `Theme::indicator_accent` and `Theme::link` expose the
  rendered control tokens (spec 2B).
- Base `Toggle` and `Tab` gain `child_with_state` / `child_with_disabled` and
  render-time `default_styles` (`StateStyles`) that sit beneath caller styles;
  a caller `hover` or `active` still replaces the default one. Base
  `ControlState` gains `checked` (the final value) and `pressed` (left button
  held on a Checkbox, Radio or Switch).

### Changed

- The native control roles (buttons, toggles, segments, checkboxes, radios,
  switches, sliders, progress, links, tree rows, calendar days, one-time-code
  cells and questionnaire choices) take their default disabled and
  inactive-window colors, and where wired their pressed colors, from one
  role × state table (spec 6.2) instead of a 45% opacity.
  Inactive-and-disabled states have their own values rather than combining
  the two. Adapters that still paint their own look (toolbar, sidebar, menu,
  select and dock rows) and buttons whose caller replaces the background keep
  the 45% disabled dimming for now.
- Bordered buttons, pop-up triggers and toggles that are off use a translucent
  bezel (ink @0.078; disabled @0.039, pressed @0.165) that no longer changes
  in an inactive window. Disabled labels use ink @0.257 (light) / 0.290 (dark).
  A caller label color is kept while the role still paints the bezel.
- Prominent buttons in an inactive window show the plain bezel; pressed
  default-accent buttons go to #036EE6 (light) / #2D87FF (dark).
- Checkbox, radio, switch, slider, toggle-on and selected-segment fills use
  `indicator_accent` (#0078F9 / #067EFF with the default accent). In an
  inactive window they become translucent ink (checkbox and radio @0.141,
  toggle-on and segment @0.125, switch @0.141 / #EBEBEB @0.153) instead of an
  opaque gray; unchecked wells and off tracks are translucent ink @0.098
  (dark switch track #E4E4E4 @0.11).
- Pressing anywhere on a checkbox, radio or switch (including its label) with
  the primary button shows the pressed well or track, also when the caller
  sets its own `group`; pressing an unselected segment turns it accent.
  Toggles and tabs follow their final pressed / selected value and no longer
  show their default hover or pressed paint while disabled or in an inactive
  window; caller-supplied `hover` / `active` styles behave as before.
- Switch knobs are state-aware in dark mode (tinted #DAECFF when on) and
  their shadow is lighter in light mode.
- Segmented controls (`button_group`) and segmented tabs (`tab`, `tab_list`)
  show the selected segment in the accent with a white label.
- Sliders keep their track and knob when disabled; only the fill turns
  neutral. The track is ink @0.098 and the dark knob #DDDDDD.
- Links use the rendered link color (#0068DA / #419CFF with the default
  accent), change to #004CB4 (light, darker) / #6FCAFF (dark, brighter) while
  pressed and fade to 50% when disabled.
- Disabled tree rows, calendar days, one-time-code cells and questionnaire
  choices use tertiary label ink instead of a faded row.
- Kit receives button, link, progress, slider and switch colors from the same
  rules for an active window; Kit components do not resolve per-window
  inactive states.

### Not yet covered

- Slider knob, stepper and questionnaire-choice pressed states, Increase
  Contrast columns and circular progress are not wired to the role table.

## Unreleased — M1 native foundation

- Add spec-cited native text roles and `NativeTextStyle::native_text_style`,
  with absolute, zoom-scaled line advances; roots use Body and Title3 advances 20 pt.
- Project the native Base type/radius scales and repair later Kit overwrites with
  one retained, idempotent observer, preserving Kit scrollbar and resize settings.
- Consolidate disabled values into native tokens, including tertiary labels,
  switch tracks, check marks, and Bezel disabled paint for prominent buttons that are both inactive and disabled.
- Add Base `ControlState` / `child_with_state` to Checkbox, Radio and Switch,
  retaining `child_with_disabled` as an adapter; focus visibility remains reserved.
- Add a production token-literal ratchet and native text/projection/state tests.
- Visible typography changes: title bar 13 → 15 pt semibold; markdown headings
  20/16/14/12 → 22/17/15/13 pt (the 13 pt heading is bold); textarea and chat
  bubbles use 16 pt advance instead of 1.4× their font size.
- Light disabled control labels use 66/255 ink alpha instead of 63/255.
- Keycaps retain their tight font-size advance (12 pt on macOS); dialog titles
  change from 16 to 15 pt semibold and panel titles from 14 to 15 pt semibold,
  both with a 20 pt advance.
