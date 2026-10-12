# Changelog

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
