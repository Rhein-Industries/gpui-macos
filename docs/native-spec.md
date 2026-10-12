# Native design spec

Reference for the native macOS look that `src/tokens.rs` and the role x state
resolvers implement. Source code, README and `docs/design.md` cite sections by
number ("spec §6.2.3"); section numbers are stable.

## 1. Purpose, sources, scope

**Purpose.** State the macOS native look once, as tokens plus centrally
resolved role x state tables, so components never patch colors, sizes or
opacities individually.

**Precedence:** SwiftUI on macOS 27 first, then AppKit, then HIG and the design
kit. Deliberate deviations are marked **policy**.

**Measurement environment.** Values were measured with an internal tool from
native SwiftUI renders on macOS 27.0.1 (26A434), SDK 27.0, Xcode 27: built-in
Retina display at 2x (Display P3 profile), wallpaper tinting off, Multicolor
accent (`controlAccentColor` #007AFF), keyboard navigation on. AppKit reference
data is `docs/native-metrics.json` (macOS 26.2).

**Scope.** Light and dark; active and inactive window; control sizes regular
and small (mini / large / xl kept where measured as reference). Increase
Contrast (IC) and Reduce Transparency (RT) values come from in-process
simulation of the system flag, not the real setting.

**Confidence markers**

| Mark | Meaning |
|---|---|
| **M** | measured |
| **D** | derived by arithmetic from measurements |
| **E** | estimated: indirect measurement or known bias |
| **Sim** | measured under a simulated system setting (IC / RT) |
| **Policy** | library decision, not a measurement |
| **O** | open: not measured |

**Caveats**
- Window "active" captures were taken with the window forced active; "inactive" captures use a genuinely inactive app. Absences seen only in forced-active captures (no ring, no hover) are scoped to that technique: the switch focus ring was missing when forced and present with a genuinely key window. Hover absences are scoped to synthetic mouse-move delivery.
- Token values are sRGB. Opaque colors match on-screen captures within 0 to 2/255. Translucent fills differ by 13 to 20/255 between offscreen and on-screen captures (hypothesis: display-space blending in P3, not isolated). Pixel-diff targets should use per-region calibrated tolerances, not one blanket value.

---

## 2. Colors: semantic system colors vs rendered control tokens

- **2A Semantic system colors** are `NSColor` values, read at runtime in the window's effective appearance. IC variants come from the accessibility appearance.
- **2B Rendered control tokens** are what SwiftUI controls actually paint. Several cannot be derived from `NSColor`, especially under IC; they are library tokens with measured values for standard and IC.

### 2A.1 Labels and text (M; IC Sim)

| Token | NSColor | SwiftUI | Light | Dark | IC Light | IC Dark |
|---|---|---|---|---|---|---|
| `label` | labelColor | `.primary` | #000 @0.847 | #FFF @0.847 | #000 @1.0 | #FFF @1.0 |
| `secondary_label` | secondaryLabelColor | `.secondary` | @0.498 | @0.549 | @0.757 | @0.698 |
| `tertiary_label` | tertiaryLabelColor | `.tertiary` | @0.259 | @0.247 | @0.557 | @0.498 |
| `quaternary_label` | quaternaryLabelColor | `.quaternary` | @0.098 | @0.098 | @0.349 | @0.298 |
| `quinary_label` | quinaryLabelColor | `.quinary` | @0.047 | @0.047 | rendered @0.149 | @0.149 |
| `text` | textColor | n/a | #000 | #FFF | #000 | #FFF |
| `placeholder` | placeholderTextColor | `.placeholder` | @0.498 | @0.549 | @0.757 | @0.698 |
| `disabled_control_text` | disabledControlTextColor | n/a | @0.247 | @0.247 | not used by measured controls | |
| `link` | linkColor | `.link` | #0068DA | #419CFF | #0068DA | #419CFF (rendered links: #6AC5FF, 2B) |
| `on_accent` | alternateSelectedControlTextColor | n/a | #FFF | #FFF | #FFF | #FFF |

### 2A.2 Fills, separators, backgrounds (M)

| Token | NSColor | SwiftUI | Light | Dark | IC L / D | Notes |
|---|---|---|---|---|---|---|
| `system_fill` | systemFill | `.fill` | @0.098 | @0.098 | **0.2 / 0.2** semantic, but `.fill` renders **0.498** | see 2B |
| `secondary_system_fill` | secondarySystemFill | `.fill.secondary` | @0.078 | @0.078 | 0.149 / 0.149 | the bezel renders 0.176 under IC (2B) |
| `tertiary_system_fill` | tertiarySystemFill | `.fill.tertiary` | @0.047 | @0.047 | rendered 0.098 | |
| `quaternary_system_fill` | quaternarySystemFill | `.fill.quaternary` | @0.0275 | @0.0275 | unchanged | |
| `separator` | separatorColor | `.separator` | @0.098 | @0.098 | 0.349 / 0.298 | `Divider()` 1 pt |
| `grid` | gridColor | n/a | #E6E6E6 | #1A1A1A | | |
| `window_background` | windowBackgroundColor | `.windowBackground` | #FFFFFF | #1E1E1E | unchanged | |
| `text_background` | textBackgroundColor | `.background` | #FFFFFF | #1E1E1E | #FFFFFF / #202020 | |
| `control_background` | controlBackgroundColor | n/a | #FFFFFF | #1E1E1E | | |
| `under_page_background` | underPageBackgroundColor | n/a | #F6F6F6 | #282828 | | AppKit 26.2 light differs |
| `alternating_row` | alternatingContentBackgroundColors[1] | n/a | #F4F5F5 | #FFF @0.047 | | |

### 2A.3 Accent, selection, focus (semantic, M)

| Token | NSColor | Light | Dark | IC L / D | Notes |
|---|---|---|---|---|---|
| `control_accent` | controlAccentColor | #007AFF | #007AFF | **#0040DD / #1774DA** | equals rendered `Color.accentColor` / `.tint` in standard mode |
| `keyboard_focus_indicator` | keyboardFocusIndicatorColor | #0067F4 @0.498 | #1AA9FF @0.498 | **#0022AF / #469AF5 opaque** | rendered rings differ under IC (2B) |
| `selected_content` | selectedContentBackgroundColor | #0064E1 | #0059D1 | #0040DD / #1774DA | |
| `unemphasized_selected_content` | unemphasizedSelectedContentBackgroundColor | #DCDCDC | #464646 | unchanged | |
| `text_selection` | selectedTextBackgroundColor | #B3D7FF | #3F638B | | |
| `text_selection_unemphasized` | unemphasizedSelectedTextBackgroundColor | #DCDCDC | #464646 | | |
| systemBlue | `Color.blue` | #0088FF | #0091FF | | not the control accent |

### 2A.4 System hues (M)

IC red: #E9152D / #FF6165.

| Hue | Light | Dark | | Hue | Light | Dark |
|---|---|---|---|---|---|---|
| systemRed | #FF383C | #FF4245 | | systemTeal | #00C3D0 | #00D2E0 |
| systemOrange | #FF8D28 | #FF9230 | | systemCyan | #00C0E8 | #3CD3FE |
| systemYellow | #FFCC00 | #FFD600 | | systemBlue | #0088FF | #0091FF |
| systemGreen | #34C759 | #30D158 | | systemIndigo | #6155F5 | #6D7CFF |
| systemMint | #00C8B3 | #00DAC3 | | systemPurple | #CB30E0 | #DB34F2 |
| systemPink | #FF2D55 | #FF375F | | systemBrown | #AC7F5E | #B78A66 |
| systemGray | #8E8E93 | #98989D | | | | |

The library `warning` #A46100 is a library-only text color (**Policy**).

### 2B Rendered control-role tokens (not derivable from NSColor)

Standard values are M; IC values are Sim (normal state, active window).

| Token | Light | Dark | IC Light | IC Dark | Differs from 2A because |
|---|---|---|---|---|---|
| `bezel_fill` | ink @0.078 | @0.078 | **@0.176** | @0.176 | `secondarySystemFill` IC is 0.149 |
| `prominent_tint` | #007AFF | #007AFF | **#007AFF (unchanged)** | #007AFF | `controlAccentColor` IC is #0040DD |
| `prominent_red` | #FF383C | #FF4245 | #E9152D | #FF6165 | = systemRed |
| `indicator_accent` (checkbox, radio, toggle on, slider fill) | #0078F9 (sRGB; seen #0277F8) | #067EFF (seen #117DFF) | #0070E9 | #1385FF | accent plus translucent overlay |
| `selected_segment` / `switch_on_track` | #0078F9 | #067EFF | **#003BCA** | **#287EDD** (record #2678D1, seen) | |
| `progress_linear` | #007AFF | #007AFF | unchanged | unchanged | |
| `progress_circular` | #0A81FE | #1769E6 | #004FC8 | #0A74FF | |
| `check_well_off` / `slider_track` | ink @0.098 | @0.098 | @0.349 | @0.298 | |
| `switch_track_off` | ink @0.098 | **#E4E4E4 @0.11** (about white @0.10; recovered RGB is not white) | O | O | |
| `switch_knob` | 2D table (6.2.7) | | | | state-aware |
| `fill_shape` (`.fill`) | @0.098 | @0.098 | **@0.498** | @0.498 | `systemFill` IC is 0.2 |
| `text_field_fill` | #FFF @0.977 | #161616 @0.98 | unchanged | **#0D0D0D** | |
| `text_field_border` (1 pt) | about ink @0.15 (seen #D9D9D9) | about white @0.09 (seen #333333) | O | O | D |
| `link_rendered` | #0068DA | #419CFF | #0068DA | **#6AC5FF** | |
| `focus_ring_control` (buttons, checkbox, radio, slider, switch) | about #0C6BFB @0.49 | about #25AFFF @0.49 | **unchanged (translucent)** | unchanged | |
| `focus_ring_field` (text, search, segmented) | about #0B63E8 @0.53 | about #33B2FF @0.51 | **opaque #0122AF** | **#479AF5** | |
| `group_box_fill` | ink @0.031 | @0.031 | | | |
| `progress_track` | ink @0.059 | @0.078 | | | |
| `slider_knob` | #FFF | seen #DDDDDD | | | |

---

## 3. Typography

### 3.1 Text styles (M)

Size and weight come from AppKit preferred fonts; line advance is the
`NSLayoutManager` default (equal to the SwiftUI line advance).

| Style | Size | Weight | Line advance | SwiftUI 1-line | Asc / desc / leading |
|---|---|---|---|---|---|
| largeTitle | 26 | regular | 32 | 31 | 25.14 / 5.48 / 1.38 |
| title | 22 | regular | 26 | 26 | 21.27 / 4.64 / 0.09 |
| title2 | 17 | regular | 22 | 20.5 | 16.44 / 3.59 / 1.98 |
| title3 | 15 | regular | 20 | 19 | 14.50 / 3.16 / 2.33 |
| headline | 13 | **bold** | 16 | 16 | 12.57 / 2.74 / 0.69 |
| body | 13 | regular | 16 | 16 | 12.57 / 2.74 / 0.69 |
| callout | 12 | regular | 15 | 15 | 11.60 / 2.53 / 0.87 |
| subheadline | 11 | regular | 14 | 14 | 10.63 / 2.32 / 1.04 |
| footnote | 10 | regular | 13 | 13 | 9.67 / 2.11 / 1.22 |
| caption | 10 | regular | 13 | 13 | same |
| caption2 | 10 | medium (E; HIG) | 13 | 13 | same |

| Measured role | Font | Conf. |
|---|---|---|
| Unified toolbar title | Semibold 15 (AppKit unifiedCompact: Bold 13) | M |
| Expanded / no-toolbar title | Headline 13 | M |
| Menu items | system 13 | M |
| Tooltip | system 11, label color | M |

- Tracking: SwiftUI and AppKit differ by 0.007 pt per character or less. No explicit tracking (**O**: GPUI parity).
- Emphasized weights: **O**.

### 3.2 Line height rule

`line_height(style) = round(ascender + |descender| + leading)`, exact for all
11 styles (**D**). Store it in points, zoom-scaled; **never a multiplier**.
`Text` line spacing defaults to 0. Title sizes use the advance column (C11).

### 3.3 Control font sizes (M)

mini 9, small 11, regular 13, large and xl 13.

---

## 4. Control metrics

The **nominal** value is what the library uses; the **measured fit** keeps the
shape ambiguity. Text-advance widths used for padding derivations: "Button"
40.29 / 34.86 / 29.21 at 13 / 11 / 9 pt; "Label" 32.87 at 13 pt.

### 4.1 Corner radii

Fits are against SwiftUI's own `.circular` / `.continuous` paths (light,
forced-active); "ambiguous" means both shapes fit about equally.

| Control | Nominal (library) regular / small | Measured fit regular / small (shape) | Notes |
|---|---|---|---|
| Bordered / automatic button, pop-up, pull-down | 6 / 5 | 6.00 / 5.00, **ambiguous** (continuous slightly better) | mini 4; large and xl capsule 14 / 18 |
| Prominent, toggle button on | 6 / 5 | 5.88 / 4.88 **circular** (toggle on regular: ambiguous) | |
| Segmented track | 6 / 5 | 5.88 / 5.00 **ambiguous** (circular slightly better) | selected segment about 5.75 (E) |
| Stepper | 6 / 5 | 6.00 / 5.13 circular | |
| Text, secure field | 6 / 5 | 5.88 (ambiguous) / 4.88 (circular) | large 7, xl 9 |
| Search field | capsule | 11.88 / 9.75 ambiguous | |
| Date field | 6 | 6.00 continuous (both sizes) | |
| Checkbox off | 5.5 / 4.5 | 5.50 continuous / 4.50 ambiguous | state-specific |
| Checkbox on | 5.5 / 4.5 | 5.38 / 4.38 circular | keep one nominal (delta 0.12 or less) |
| Color well | capsule | 8.75 / 6.75 ambiguous | |
| Linear progress | capsule | 4 / 3 | |
| GroupBox | 12 | 12.13 continuous | |
| Glass / glassProminent | 6.5 | 6.38 / 6.50 continuous | small 5.38 / 5.5 |
| List, table, sidebar selection | 5 | diagonal 5.2 to 5.5 only | **E** |
| Menu panel / highlight | 13 / 6.5 | panel 13.0 (L) / 12.4 (D) continuous; highlight renders about 6 to 7 | highlight **E** |
| Popover | 16 | fit RMS about 1 pt | **E** |
| Tooltip | 5 | 5.13 (L) / 5.5 (D) circular | |
| Window | 16 continuous | 16.5 (L) / 15.9 (D) | M |
| Focus ring outer | control radius + about 2.6 to 3 | bordered and field 8.63; checkbox 8 | not an exact +3 (**E**) |

### 4.2 Bezel family (M unless noted)

| Metric | small | regular | (mini / large / xl) | Conf. |
|---|---|---|---|---|
| Height | 20 | 24 | 13 / 28 / 36 | M (AppKit 24) |
| Pop-up, pull-down, segmented height | 20 | 24 | 16 / 28 / 36 | M |
| Horizontal padding | 10 | 12 | 8 / 14 / 18 | D: (frame - text advance) / 2; AppKit 12 |
| Radius | 5 | 6 | 4.1 | nominal |
| Font / weight | 11 regular | 13 regular | 9 / 13 / 13 | M (3.3) |
| Min width | not measured | | | **O** |
| Icon-only (bordered, `plus`) | 32 x 17 | **37.5 x 20.5** visible (layout frame 39 x 21) | | M (C7) |
| Stepper | 17 x 20 | 20 x 24 | 13x16 / 23x28 / 30x36 | M |
| Glass button | layout 24; visible 24 | layout 24; visible 24 (L) / 24 (D) | | M |
| GlassProminent button | | layout **24**; visible **25 (L) / 24 (D)** | | M |

### 4.3 Text inputs (M unless noted)

| Metric | small | regular |
|---|---|---|
| Text field height | 21 | 24 (mini 19; large and xl 24) |
| Secure field | 22 | 24 |
| Search field | 20 | 24 |
| Toolbar `.searchable` | 24 (compact) | 36 tall, 262 to 325 wide (unified); 28 (expanded) |
| Date field | 19 | 21 |
| Text inset (D) | about 6 | about 6 |

### 4.4 Indicators

| Metric | small | regular | (mini / large) | Conf. |
|---|---|---|---|---|
| Checkbox box | 14 | 16 | 12 / 18 | M (AppKit 16) |
| Checkbox label gap | about 5 | 5 | | D (AppKit 6) |
| Labeled checkbox row | 15 | **17** (box at y 0.5) | | M (C2; AppKit 16) |
| Check mark ink box | 9 x 8.5 | 10 x 9.5 | | M |
| Mixed dash | 7 x 2 | 7 x 2 | | D |
| Radio | not measured | 16, dot about 5, group pitch 22.5 | | M |

### 4.5 Switch, slider, progress

| Metric | small | regular | (mini / large / xl) | Conf. |
|---|---|---|---|---|
| Switch track | 44 x 20 | 54 x 24 | 36x16 / 64x28 / 80x36 | M |
| Switch knob | 26 x 16 | 32 x 20 | 21x13 / 38x24 / 47x30 | M |
| Knob inset | 2 | 2 | 1.5 / 2 / 3 | M |
| Labeled switch gap | n/a | 8 | | D |
| Slider track | 4 | 6 | | M |
| Slider knob | about 19 x 14 | about 20 x 16 | | E |
| Linear progress | 6 | 8 | | M |
| Circular progress | 16 | 32 | | M |

---

## 5. Layout

### 5.1 Spacing (M unless noted)

| Token | Value |
|---|---|
| `spacing.default` | 8 (HStack pairs, VStack controls, LabeledContent, button pairs at all sizes) |
| `spacing.toggle_stack` | 6 |
| text to control (VStack) | 8.5 (D) |
| text to text / image to text (VStack) | 0 |
| `padding.default` | 16 |

Steps 2 / 4 / 12 / 20 / 24 are library defaults, not measured (**E**).

### 5.2 Rows and selection

| Item | Value | Conf. |
|---|---|---|
| Sidebar row (default) | **32** (pitch 32; 31.75 inactive) | M |
| Sidebar small / large preference | 24 / 40; label 11 / 15; icon column 20 / 26 | M (AppKit) |
| Sidebar symbol (SwiftUI, regular) | label size (13) at large scale, medium weight, natural size; centred 13 after the row content, title 30 after it (26 slot) | M (capture ink boxes within 1 px) |
| Sidebar group header | 19, 11 pt, `tertiary_label`; disclosure chevron only under the pointer | M (AppKit; m5 capture) |
| Sidebar group spacing | 13 before every group but the first | M (AppKit) |
| List / table row | **24** (selection rect height; pitch 24.25 to 24.5) | M |
| Table header | 27, then a 1.5 pt separator | M |
| Selection inset | 11.5 each side | M (0.5) |
| Selection radius | about 5 (5.26 to 5.36 measured on the `.inset` list) | M (radius fit) |
| `.inset` list content inset | 10 above the first row (second row's selection at y 34 = 10 + 24) and below the last | M (top); bottom E |

Descriptive only (not row tokens): median text-line pitch of Form panels
(mixed content, not row frames). Form row tokens need frame measurements (**O**).

| Panel | L active | L focused | L inactive | D active | D focused | D inactive |
|---|---|---|---|---|---|---|
| formGrouped | 38.5 | 38.5 | 38 | 37.25 | 37.5 | 37.5 |
| formColumns | 26.5 | 26.5 | 27 | 26.5 | 26.5 | 27 |

### 5.3 Window chrome and toolbar (M)

| Item | Value |
|---|---|
| Titled window, no toolbar | 32 |
| Titlebar + toolbar: unified / unifiedCompact / expanded | **52 / 40 / 68** (expanded 48 with an empty toolbar) |
| Traffic lights (close origin) | unified (19, 19); compact (12, 13); expanded and none (9, 9) |
| Toolbar items (unified) | glass capsule 36 tall at y 8 (r = h / 2). Icon: 36 x 36 circle. Text item: 52 x 36. Group of three icons: one 109 x 36 capsule |
| Toolbar items (compact / expanded) | 24 tall at y 8 / 28 tall at y 32 |
| Sidebar toggle | inside the sidebar, 36 x 36 at x 176 |
| Detail title (split view) | origin 20 after the sidebar's trailing edge (240 for a 220 sidebar; glyph ink at 240.5 to 241) |
| Window corners | continuous 16 |
| Split divider | 1 |
| Sidebar default width | 144 (SwiftUI default, probably content-driven; not a design value; C3) |

---

## 6. Role x state tables

States do **not** compose by multiplication (for example the stepper, toggle
on and the switch on track in an inactive disabled window). Each role has an
explicit table; the resolver looks up `(role, window, state, contrast)`.

Cell notation: "ink @a" = label RGB (black in light, white in dark) at alpha a;
"x k" = alpha relative to the active-normal value of the same part; "L / D"
when appearance differs; "=" same as the cell to its left; "O" not measured;
"seen" = sampled from a render.

Columns: **AN** active normal, **AD** active disabled, **IN** inactive normal,
**ID** inactive disabled, **P** pressed (active, on-screen), **IC-xx** Increase
Contrast (Sim). IC pressed is not measured. All values are M unless marked.

### 6.1 Roles (part level)

| Role | Members |
|---|---|
| `Bezel` | bordered and automatic buttons, pop-up, pull-down, toggle button off, icon-only and label buttons, stepper body |
| `StepperGlyph` | stepper arrows |
| `Prominent` | `.borderedProminent` (accent), `.tint(.red)`; `DefaultAction` differs only in AD |
| `ToggleOn` | toggle button on |
| `SegTrack` / `SegSelected` / `SegLabel` | segmented and palette pickers |
| `CheckOff` / `CheckOn` (well + mark) | checkbox and radio off; on and mixed |
| `SwitchTrackOff` / `SwitchTrackOn` / `SwitchKnob` | switch parts |
| `SliderTrack` / `SliderFill` / `SliderKnob` | slider parts |
| `ProgressLinear` / `ProgressCircular` / `ProgressLinearIndeterminate` / `Spinner` | progress views |
| `BorderlessLabel` / `PlainLabel` / `LinkLabel` / `MenuBorderlessLabel` | text-only buttons and links |
| `FieldText` / `FieldPlaceholder` / `FieldChrome` | rounded, secure and search fields |
| `DateFieldText` / `PlainFieldText` | date field, plain text field |
| `ControlLabel` | the label beside a checkbox, radio or switch |
| `StaticText` | Text, Label |
| `Glass` / `GlassProminent` | glass button styles |
| `Row` (6.4), `ToolbarItem` / `AccessoryBar` (6.5), `MenuItem` (7.3) | |

### 6.2 Control tables

**6.2.1 Bezel** (fill and label)

| Part | AN | AD | IN | ID | P | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| fill | ink @0.078 | @0.039 | = AN | = AD | **@0.165** | @0.176 | @0.090 | = IC-AN | = IC-AD |
| label | `label` | @0.257 (L) / 0.290 (D) | = AN | = AD | = AN | ink @1.0 | **@0.596 (L) / 0.545 (D)** | = IC-AN | = IC-AD |

**6.2.2 StepperGlyph** (arrows; the body is a Bezel)

| AN | AD | IN | ID | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|
| `label` | x0.30 (L) / x0.27 (D) | x0.59 (L) / x0.61 (D) | **= IN** (ratio 1.00) | ink @1.0 | @0.74 | @0.83 | @0.78 (L) / 0.74 (D) |

**6.2.3 Prominent** (accent; red where noted) and DefaultAction

| Part | AN | AD | IN | ID | P (on-screen) | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| fill | `prominent_tint` / `prominent_red` | tint x0.69 | **Bezel AN** | **Bezel AD** | accent: #036EE6 (L) / **#2D87FF (D)**; red: O | accent unchanged; red #E9152D / #FF6165 | x0.69 | Bezel IC-AN | Bezel IC-AD |
| label | white | white @0.50 | Bezel label | Bezel label AD | white | white | white @0.50 (accent); red L about #F7A0AA @0.84 seen | Bezel | Bezel |
| DefaultAction AD | | **Bezel AD** | | | | | **Bezel IC-AD** | | |

**6.2.4 ToggleOn**

| Part | AN | AD | IN | ID | P | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| fill | `indicator_accent` | x0.70 (#0074F3 @0.71) | ink @0.125 | **ink @0.067** (x0.53 of IN) | #036CE1 (L) / #338AFF (D), seen | #0070E9 / #1385FF | about x0.74 (#0068D9 @0.74 / #218AFF @0.74) | **ink @0.369 / 0.322, white label (L)** | @0.188 / 0.161 |
| label | white | white @0.50 | `label` | x0.31 of IN | white | white | seen tinted @0.87 | white (L) / label (D) | D @0.58; L O |

**6.2.5 Segmented**

| Part | AN | AD | IN | ID | P | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| SegTrack | ink @0.078 | **= AN** (ratio 1.0001) | = AN | = AN | = AN | @0.176 | = | = | = |
| SegSelected | `indicator_accent` | about x0.71 (seen #4597F1 over track) | ink @0.125 over track (seen #CDCDCD / #4A4A4A) | seen #DCDCDC / #3E3E3E | pressing an unselected segment turns it accent | **#003BCA / #287EDD** | L #0033B2 @0.79; D O | O | O |
| SegLabel (selected / unselected) | white / `label` | O / O (the aggregate label ratio 0.71 is the selected region, not text) | `label` / `label` | aggregate x0.35 | | white / label | O | label | label x0.63 (L) |

**6.2.6 CheckOff / CheckOn** (checkbox and radio; mixed = on)

| Part | AN | AD | IN | ID | P | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| CheckOff well | ink @0.098 | @0.051 | = AN | = AD | @0.188 | @0.349 / 0.298 | @0.176 / 0.149 | = IC-AN | = IC-AD |
| CheckOn well | `indicator_accent` | ink @0.075 | ink @0.141 | **ink @0.075** (x0.53 of IN) | #036CE1 / #338AFF seen | #0070E9 / #1385FF | @0.24 / 0.21 | @0.447 / 0.404 | @0.24 / 0.21 |
| CheckOn mark | white | ink @0.31 / 0.30 | `label` | x0.30 of IN | white | white | @0.66 / 0.60 | ink @1.0 | @0.66 / 0.60 |

**6.2.7 Switch** (offscreen values, sRGB straight alpha)

| Part | AN | AD | IN | ID | P | IC-AN | IC-AD | IC-IN | IC-ID |
|---|---|---|---|---|---|---|---|---|---|
| TrackOff | ink @0.098 (L); **#E4E4E4 @0.11 (D)** | @0.051 (L); #CFCFCF @0.063 (D) | = AN | = AD | @0.188 | O | O | O | O |
| TrackOn | #0078F9 (L) / #067DFC (D) | **x0.71** (#0074F3 @0.71 / #0B7EFB @0.71) | ink @0.141 (L); #EBEBEB @0.153 (D) | **@0.075 (L); #DCDCDC @0.086 (D)** | #036CE0 / #338AFF seen | #003BCA / #2678D1 (seen; alt #287EDD) | O | O | O |
| Knob (off) | #FFF (L); **#FFF @0.867 (D)** | #FFF (L); @0.859 (D) | = AN | = AD | O | L #FFF; D #FFF @0.894 | D @0.875 | = | = |
| Knob (on) | #FFF (L); **#DAECFF (D)** | #FFF (L); **#E4F1FF @0.954 (D)** | #FFF (L); #FFF @0.871 (D) | #FFF (L); @0.863 (D) | O | L #FFF; D #DFECFA | D #E8F1FB @0.96 | D @0.91 | D @0.88 |
| Knob shadow | about -6/255 below the knob, on-screen | | | | | | | | |

**6.2.8 Slider**

| Part | AN | AD | IN | ID | IC-AN | IC-AD / IN / ID |
|---|---|---|---|---|---|---|
| SliderTrack | ink @0.098 | **= AN** | = AN | = AN | @0.349 / 0.298 | = IC-AN |
| SliderFill | `indicator_accent` | neutral ink about 0.15 (seen #DADADA / #3F3F3F) | ink about 0.22 (seen #C5C5C5 / #515151) | ink about 0.165 (seen #D5D5D5 / #434343) | #0070E9 / #1385FF | O (fill not isolated) |
| SliderKnob | #FFF / seen #DDDDDD | = AN | = AN | = AN | | |

**6.2.9 Progress**

| Subrole | Active | Inactive | IC |
|---|---|---|---|
| ProgressLinear (determinate) | `progress_linear` #007AFF on `progress_track` | **opaque #AAAAAA** (L and D) | unchanged |
| ProgressCircular (determinate) | #0A81FE (L) / #1769E6 (D) | **#B8B8B8 (L) / #FFF @0.33 (D)** | #004FC8 / #0A74FF |
| ProgressLinearIndeterminate | accent on track | accent x0.33 (L) / x0.56 (D) | unchanged |
| Spinner (circular indeterminate) | system spinner | uniform x0.50 | ink about 0.80 (#CB) |

**6.2.10 Text roles**

| Role | AN | AD | IN / ID | P | IC-AN | IC-AD |
|---|---|---|---|---|---|---|
| BorderlessLabel (`button.borderless`, icon-only borderless) | `secondary_label` | **@0.25 in both** (L x0.50; D x0.457) | = active | to ink @0.85 | @0.76 (L) / 0.70 (D) | @0.50 / 0.50 |
| PlainLabel (`button.plain`) | `label` | uniform x0.50 | = active | O | ink @1.0 | O |
| LinkLabel (`button.link`, `Link`) | `link` | uniform x0.50 | = active | #004CB4 (L) / #6FCAFF (D) | L #0068DA; **D #6AC5FF** | D x0.60 |
| MenuBorderlessLabel | `label` | uniform about x0.42, poor fit | = active | O | ink @1.0 | O |
| FieldText (rounded, secure, search) | `label` (ink-biased; E) | **x0.32** | = active | n/a | ink @1.0 | seen #666666 (L) / #909090 (D) |
| FieldPlaceholder | `placeholder` | **unchanged** | = active | | seen #373737 / #BEBEBE | unchanged |
| FieldChrome (fill, border) | 2B | **unchanged** | = active | | 2B | unchanged |
| DateFieldText | `label` | **x0.29** | = active | | | |
| PlainFieldText | `label` | uniform **x0.43** | = active | | ink @1.0 | O |
| ControlLabel | `label` | **`tertiary_label`** (0.259 / 0.247) | = active | about AN | ink @1.0 | **`tertiary_label` IC** (0.557 / 0.498) |
| StaticText | `label` | **unchanged** | = active | | ink @1.0 | unchanged |

**6.2.11 Glass / GlassProminent**

| Role | AN | AD | IN | ID | Hover | P |
|---|---|---|---|---|---|---|
| Glass fill | #F5F5F5 @0.53 (L) / #4C4C4C @0.41 (D) | = AN | **#E5E5E5 @0.58 (L) / #363636 @0.37 (D)** | = IN | yes (6.5) | O |
| Glass label | ink @1.0 | x0.31 | x0.32 (L) / x0.29 (D), **E** | x0.84 / x0.79 of IN, E | | |
| GlassProminent fill | about accent @0.91 | **becomes Glass AN** (neutral glass) | **becomes Glass IN** | = Glass ID | **none** | O |
| GlassProminent label | white | about Glass AD label | Glass IN label | | | |

All glass values are backdrop dependent; RT turns them opaque (7.6).

### 6.3 Focus ring

| Property | Value | Conf. |
|---|---|---|
| Families | `focus_ring_control` (buttons, checkbox, radio, slider, switch) and `focus_ring_field` (text, search, segmented); they differ under IC (2B) | M / Sim |
| Width / outset | 3 to 3.5 / 3 | M |
| Outer radius | measured 8.63 (bordered, field), 8 (checkbox); capsule for the switch; nominal about r + 3 (4.1) | E |
| Scope | whole control (buttons, pop-ups, fields); checkbox box only (22 x 22); selected radio only; selected segment only (65.5 x 30); slider knob only (about 26 x 22); switch capsule 60 x 30 | M |
| Fill / border while focused | unchanged; no accent border | M |
| Inactive window | none | M |

### 6.4 Rows and selection

| Context | Focused (key, list is first responder) | Unfocused (key) | Inactive | Conf. |
|---|---|---|---|---|
| List / table | `selected_content` (seen #0064E1 L; #0058D1 to #025AD1 D) | `unemphasized_selected_content` #DCDCDC / #464646 | same as unfocused | M |
| Sidebar | opaque, seen #0070F5 (L) / #007AFF (D) | **black @0.085 / white @0.088 over the material** | **opaque #E5E5E5 / #363636** on opaque #F0F0F0 / #282828 | M |
| Sidebar, RT | same | #DFDFDF / #545454 | #E1E1E1 / #232323 | Sim |
| Text selection | `text_selection` | `text_selection_unemphasized` | same | semantic M; not panel-measured (E) |
| Geometry | inset 11.5, radius about 5, full row height (5.2) | | | |
| Hover | none | | | M (scoped) |
| IC | O | | | |

### 6.5 Hover

Measured with synthetic mouse-move delivery; positive controls responded in
all combinations. Results are scoped to that delivery method.

| Role | Hover | Light | Dark | Conf. |
|---|---|---|---|---|
| Bezel, Prominent, DefaultAction, ToggleOn, Seg*, Check*, Switch*, Slider*, Borderless, Plain, Link, MenuBorderless, fields, color well, compact date picker, stepper, `.accessoryBarAction`, **GlassProminent** | none | | | M (scoped) |
| Row (inset list, sidebar list, table) | none | | | M (scoped) |
| `AccessoryBar` (`.accessoryBar`; AppKit `showsBorderOnlyWhileMouseInside`) | bezel ink @0.075, bordered shape | #00000013 | #FFFFFF13 | M |
| `Glass` | darker (L) / brighter (D) | seen #F9F9F9 to #EFEFEF | #3B3B3B to #4F4F4F | M (backdrop dependent) |
| `ToolbarItem` (SwiftUI glass toolbar) | circular disc ink about 0.05 (L) / 0.04 (D). In an inactive window the icon also goes from dimmed (#B2B2B2 L) to full `label` | #FAF9FA to #EEEDEE | #434343 to #4C4C4C | M |
| AppKit `NSToolbar` bordered image item | none | | | M |
| `MenuItem` | accent highlight (keyboard-measured; mouse hover E) | | | M / E |

### 6.6 Pressed

Targets are measured on-screen values; formulas are provisional token rules.

| Role | Target L | Target D | Provisional formula | Validated? |
|---|---|---|---|---|
| Bezel (and toggle off) | ink @0.165 | white @0.165 | alpha x2.2 | yes |
| CheckOff / SwitchTrackOff | @0.188 | @0.188 | alpha x1.92 | yes |
| Prominent accent | #007AFF to #036EE6 | #007AFF to **#2D87FF** | L: mix 10% toward black (3/255 error or less); D: **none** (HSL L +9 gives #2E92FF; a white overlay needs 0.18 R vs 0.10 G) | L yes; **D no** |
| Indicator accent (checkbox on, toggle on, switch on) | #0277F8 to #036CE1 | #117DFF to #338AFF | as above | L yes; D no |
| Prominent red, glass, segmented, menus, pickers | O | | | |
| BorderlessLabel | to ink @0.85 | to @0.85 | set alpha 0.85 | yes |
| LinkLabel | #0068DA to #004CB4 | #419CFF to #6FCAFF | none | no |
| Pull-down while open | bezel #ECECEC to #D5D5D5 (seen, L) | O | = Bezel P (D) | partly |

The dark formula must be fitted in a single color space before implementation
(**O**). Until then the dark resolver uses the measured target colors for the
default accent and falls back to "no change" for custom tints.

### 6.7 Precedence

Within a window state: disabled > pressed / open > focused (additive ring) >
selected > hover (6.5 roles only) > normal. RT replaces material colors only
(7.6).

---

## 7. Materials, glass, menus, popovers, tooltips

### 7.1 Behind-window materials (M)

Fit is `seen = c + k * backdrop` (sRGB) over black, white, gray and red
backdrops. Blur is the 10 to 90% edge width, a lower bound. **Each specimen
keeps its own token.**

| Specimen | Light tint @op | Dark tint @op | Blur L / D (pt) |
|---|---|---|---|
| `visualEffect.sidebar` / `toolTip` / `underWindowBackground` | #E7E7E7 @0.898 | #2C2C2C @0.816 | 48 / 66 |
| `visualEffect.menu` | #E7E7E7 @0.765 | #2D2D2D @0.718 | 59 / 66 |
| `visualEffect.popover` | #E9E9E9 @0.635 | #2D2D2D @0.616 | 65 / 70 |
| `visualEffect.titlebar` | #FDFDFD @0.80 | #3C3C3C @0.80 | 72 |
| `visualEffect.headerView` | #FFFFFF @0.80 | #1E1E1E @0.80 | 72 |
| `visualEffect.hudWindow` | #EBEBEB @0.502 | **#303030 @0.416** | 71 |
| `visualEffect.fullScreenUI` | #EBEBEB @0.502 | #2F2F2F @0.514 | 71 |
| `visualEffect.selection` | black @0.09 | white @0.129 | none |
| `visualEffect.sheet` | opaque #FFFFFF | **#1E1E1E @0.898** | none |
| `visualEffect.windowBackground` / `contentBackground` | opaque window color | opaque | none |
| `visualEffect.underPageBackground` | **#F6F6F6** | **#141414** | none |
| SwiftUI `.regularMaterial` | #E9E9E9 @0.635 | #2D2D2D @0.616 | **128 / 137** |
| SwiftUI `.thinMaterial` | #EBEBEB @0.502 | **#2F2F2F @0.514** | **137 / 142** |
| SwiftUI `.bar` | #FFFFFF @0.80 | #1E1E1E @0.80 | **139** |
| Liquid Glass regular (`NSGlassEffectView`, `.glassEffect`) | #FCFCFC @0.527 | #404040 @0.50, **non-linear** | 35 / 28 |
| Liquid Glass clear | #FFFFFF @0.126 | same | 3.7 |
| Liquid Glass regular, accent tint | #006AEA @0.908 | #007DFF @0.984 | |

A SwiftUI `NavigationSplitView` sidebar (macOS 27 view dump) is Liquid Glass
regular (no tint, radius 0, full height including behind the toolbar) in an
opaque window whose theme frame holds a full-window `.contentBackground`
effect (within-window, active), so the sidebar glass samples the window
background, not the desktop (M, view hierarchy).
The window colour is #FFFFFF / #1E1E1E there, yet the sidebar shows #EDEDED /
#262626 (polish-check captures), while free regular glass over the same colour
shows #FEFEFE / #373737 (our captures, matching the response above). The
native sidebar glass therefore uses a configuration beyond the public style,
tint and radius (O; the local view dump prints its private state). The library
puts an opaque backdrop under regular glass instead, a custom approximation:
#C9C9C9 light and #080808 dark, calibrated through the response above
(#D9D9D9 measured 242, #FFFFFF 254, about 0.316 output per backdrop step) and
verified by capture in an active window (polish3-check: 236 against the native
237 light, 38 against 38 dark; M for active windows, O for inactive windows and
Increase Contrast).

When the pointer is over the detail column's titlebar band
(`NSTitlebarBackgroundView`, 52 pt), SwiftUI shows the titlebar background: the
sidebar's surface (#262626 dark in the reference hover screenshot) with a
0.5 pt hairline at its bottom (white @0.07 dark, M; black @0.05 light, the
scroll pocket's hard fill in the view dump, E). The sidebar list keeps a
52 pt scroll pocket (`NSScrollPocket`, soft edge effect) under the titlebar
that blurs rows scrolled under it.

Active materials boost saturation (red-backdrop R gain 1.04 to 1.19; dark
SwiftUI materials 1.5 to 1.7). A plain alpha blend is a known fallback deviation.

### 7.2 Material inactive behavior (scoped to what was measured)

| Condition | Specimens | Result |
|---|---|---|
| Genuinely inactive window | sidebar (`followsWindowActiveState`) | **opaque #F0F0F0 / #282828** (M) |
| Genuinely inactive window | SwiftUI `.bar` | opaque #FFFFFF / #1E1E1E (M) |
| Genuinely inactive window | SwiftUI `.regularMaterial`, `.thinMaterial` | **unchanged (translucent)** (M) |
| Genuinely inactive window | Liquid Glass regular / clear | L #E5E5E5 @0.58 / #DDDDDD @0.32; D #2A2A2A @0.47 (non-linear) / #575757 @0.29 (M) |
| `state = .inactive` forced in an active window | sidebar, menu, popover, titlebar, hudWindow, fullScreenUI, toolTip, underWindowBackground | opaque #F0F0F0 / #282828 |
| same | headerView | opaque #FFFFFF / #1E1E1E |
| same | selection | black @0.043 / white @0.063 |
| same | sheet, windowBackground, contentBackground, underPageBackground | unchanged |

Rule (**Policy**): a material token carries `follows_window_state: bool`. Only
the specimens shown opaque above switch to their inactive color; other
`followsWindowActiveState` cases are inferred from the forced results (**E**).

### 7.3 Menus (M, genuinely key window)

| Property | Value | Conf. |
|---|---|---|
| Panel | Liquid Glass, continuous 13 (L) / 12.4 (D) | M |
| Row height / vertical padding / separator row | 24 / 5 / about 11 to 12 | M / M / D |
| Font | system 13 | M |
| Highlight | accent #007AFF (exact), 24 tall, inset 5 (L) / 5.5 (R) from the panel edges, radius about 6.5, white text | M (radius E) |
| Shadow | alpha about 0.18 to 0.26, extent 9 to 18 pt | M |
| Light material | about #F4F4F4 @0.54, saturation boost; dark adaptive | M / E |
| Placement | pop-up opens over its button (selected item aligned); pull-down opens below and its bezel darkens | M |
| Destructive items | not red | M |
| Disabled items, mouse hover | not measured | O |

### 7.4 Popovers

| Property | Value | Conf. |
|---|---|---|
| Surface | Liquid Glass. L interior about #F8F8F8 @0.93 (active), #EAEAEA @0.80 (inactive). D adaptive | M / E |
| Arrow | about 24 x 10.5, tip at the anchor | E |
| Corner | about 16 continuous | E |
| Built-in padding | none (content is clipped by the corner) | M |
| Shadow | about 13 pt window margin; about 18 pt below, max alpha about 0.36 | M |

### 7.5 Tooltips

Measured from a programmatically shown tooltip; genuine hover did not show one.

| Property | Light | Dark | Conf. |
|---|---|---|---|
| Body (bbox) | 71 x 20 | 72 x 21 (bbox at -0.5 pt; layout likely 20, **E**) | M |
| Corner | 5.13 circular | 5.5 circular | M |
| Font | system 11, label color | same | M |
| Material (tooltip's own fit) | #E7E7E7 @0.898 (= `visualEffect.toolTip`) | **#3A3A3A @0.828** (differs from the `toolTip` specimen #2C2C2C @0.816) | M |

### 7.6 Reduce Transparency (Sim)

The system toggle could not be set; these are in-process simulation values with
no cross-check against the real setting. Treat them as simulated.

| Surface | Light | Dark |
|---|---|---|
| sidebar (active / inactive) | #F6F6F6 / #ECECEC | #383838 / #141414 |
| sidebar selection: focused / unfocused / inactive | #0070F5 / #DFDFDF / #E1E1E1 | #007AFF / #545454 / #232323 |
| menu, popover, HUD | #ECECEC, #EAEAEA, #E4E4E4 | #323232, #2C2C2C, #262626 |
| titlebar, headerView, `.bar` | #FFFFFF | #262626, #1E1E1E, #1E1E1E |
| `.regularMaterial` | #EAEAEA | #2C2C2C |
| Liquid Glass regular | about #FDFDFD | #141414 |
| SwiftUI popover | #FFFFFF | #151515 |

---

## 8. How the library uses this

`src/tokens.rs` holds the numeric constants cited from this spec; `Theme`
resolves 2A colors at runtime and carries 2B tokens (standard and IC). Role x
state paint (6) is resolved centrally instead of per-component opacity
literals; hover (6.5) exists only for `AccessoryBar`, `Glass`, `ToolbarItem`
and `MenuItem`. Materials (7.1 to 7.2) are per-specimen tokens with a
`follows_window_state` flag. Lint tests guard against ad hoc literals.

### 8.2 Disabled-state value changes applied

| Item | Value |
|---|---|
| 15 pt line height | 19 to 20 (3.2) |
| Light disabled control label | tertiary label: 66/255 L, 63/255 D; IC 0.557 / 0.498 (2A.1) |
| Switch disabled on track | 0.70 to 0.71 (6.2.7) |
| Dark disabled check mark | 0.31 to 0.30 (6.2.6) |
| Dark disabled off track | #CFCFCF @0.063 (6.2.7) |
| Inactive + disabled prominent | Bezel AD (6.2.3) |
| Kept as is | Prominent AD x0.69 and label 0.50; light off track 0.051; disabled check wells 0.051 / 0.075 |

---

## 9. Decisions

Final outcomes. Decisions marked **Policy** are library choices where measurement left room.

| # | Topic | Decision |
|---|---|---|
| C1 | Prominent pressed in dark | lightens (target #2D87FF); token formula unvalidated (6.6) |
| C2 | Labeled checkbox row | 17 (SwiftUI over AppKit 16) |
| C3 | Sidebar width | min / ideal / max with ideal 220 (**Policy**); SwiftUI default 144 is not a design value |
| C4 | Borderless button | add a `Borderless` variant: `secondary_label`, disabled @0.25, pressed @0.85, no hover |
| C5 | Destructive | role alone stays accent (buttons), not red (menus); `.tint(.red)` = systemRed |
| C6 | Segmented | selected = `indicator_accent` with white label; track unchanged when disabled; IC #003BCA / #287EDD |
| C7 | Icon-only bordered button | follow SwiftUI geometry (37.5 x 20.5 visible, 39 x 21 layout); no 24 pt hit-target exception |
| C9 | Checkbox radius | nominal 5.5 / 4.5 |
| C10 | Accent | rendered accent = controlAccentColor in standard mode; IC prominent stays #007AFF |
| C11 | Title-size line height | use the advance (3.2) |
| C12 | Tinted disabled alpha | keep per-role measured values (Prominent 0.69, ToggleOn 0.70, TrackOn 0.71, SegSelected about 0.71) |
| C13 | Text field | nominal 6 / 5 |
| C14 | Bezel fill | translucent `bezel_fill` 0.078 (IC 0.176) |
| C15 | Focus ring | two families (control / field), 3 outset, key window only |
| C16 | Row heights | list/table 24, table header 27 |
| C17 | Hover | none on bezel controls and rows; keep for toolbar items, accessory-bar and glass (not glassProminent) buttons; menu highlight = accent |
| C18 | Slider disabled | fill neutralized, track and knob unchanged |
| C19 | Switch knob | state-aware knob (6.2.7); tune the existing shadow |
| C20 | Sidebar selection | inactive opaque #E5E5E5 / #363636 on #F0F0F0 / #282828; unfocused key black @0.085 / white @0.088 over the material |
| C21 to C24 | label gap 5; tinted disabled label 0.50; `link` color; toolbar heights 52 / 40 / 68 / 32 | as specified |
| C25 | Menus | row 24, padding 5, panel 13, highlight inset 5 radius about 6.5 |
| C26 | Popover | radius 16, arrow 24 x 10.5, padding 16 (SwiftUI `.padding()`); arrow and corner estimated |
| C27 | Tooltip | 20 tall, radius 5, 11 pt; dark uses its own #3A3A3A @0.83 material |
| C28 | Composed toolbar | 36 glass items, Semibold 15 title, hover disc |
| C29 | Inactive materials | per-specimen `follows_window_state` (7.2) |
| C30 | IC coverage | 2A at runtime plus 2B / 6.2 tables |
| C31 | Pixel-diff color tolerance | per-region calibrated tolerances |
| C32 | Dark pressed accent | use the measured target for the default accent; refit before supporting custom tints |
| C33 | Glass / glassProminent | adopt the 6.2.11 tables for any library glass-button fallback |

---

## 10. Verification

Tests assert nominal metrics within 0.5 pt (shape is not asserted where the
radius fit is ambiguous), colors within 2/255 (opaque) or 0.01 alpha
(translucent), and that cells marked **O** are explicit fallbacks. Pixel-diff
regression against SwiftUI uses per-region calibrated tolerances; Liquid
Glass, dark adaptive materials and popover or menu interiors are excluded from
scoring.

---

## 11. Not measured / estimates

- Real Reduce Transparency and Increase Contrast system settings (both simulated); IC pressed and IC panels/rows.
- Compact DatePicker popup.
- Menu mouse hover, context-menu and pull-down highlight, disabled items; menu highlight radius (fit 0.5 contradicts the render of about 6 to 7).
- Popover corner and arrow (estimates); material blur radius (lower bounds); dark Liquid Glass, popover and menu materials (adaptive; per-backdrop values only).
- Tooltip via genuine hover.
- Pressed: red tint, glass, segmented, pickers, toolbar; dark pressed formula.
- Button min width; small radio; emphasized weights; table header weight; GPUI tracking parity; Form row frames; selection radii (not refitted); disabled SegLabel; slider fill and switch track under IC.
- Glyph-ink label colors are off by 0.02 to 0.03 alpha (anti-aliasing); `menu.borderlessButton` and plain text field disabled are uniform fits with large residuals; "label" ratios for segmented and switch measure the selected or track region, not text (use the part tables); secure field small 22 vs text field 21; mini push button 13 vs mini pop-up 16; caption2 weight is regular in AppKit but matches medium by width; `sidebar_width_default` 144 is probably a minimum or content-driven width.
