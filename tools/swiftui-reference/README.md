# SwiftUI macOS Reference

A native SwiftUI app that mirrors the GPUI component gallery (`examples/gallery`)
page by page, so both can be captured and pixel-diffed. Pilot pages use real
SwiftUI system controls; every other page is a placeholder.

## Build and run

```sh
scripts/build-swiftui-reference.sh
open -n "target/swiftui-reference/SwiftUI macOS Reference.app" --args checkbox --dark
# or launch the executable directly (keeps the pid for capture scripts):
"target/swiftui-reference/SwiftUI macOS Reference.app/Contents/MacOS/swiftui-reference" table --light
```

The script compiles `App.swift`, `Catalog.swift`, `GalleryView.swift` and `Pages.swift` with `swiftc` against the
Xcode SDK (`arm64-apple-macosx26.0`, Swift 6, warnings as errors), writes the
bundle `dev.gpui.macos.swiftui-reference`, adds project notices, ad-hoc signs it,
registers it with Launch Services and prints the bundle path.

## CLI contract (same as `cargo run --example gallery -- …`)

- The first argument equal to a page slug selects that page (default `overview`).
  `--page <slug>` also works.
- `--light` / `--dark` force the appearance; the last one wins. Default follows the system.
- `--display <CGDirectDisplayID>` centers the full window on that active display.
  SwiftUI matches `NSScreenNumber`; GPUI sets `WindowOptions.display_id` and
  centers its bounds on that display. Missing, invalid or unknown ids retain
  each app's default centering behavior. Both frames are exactly 1060 x 760 pt,
  including the titlebar; the centered origin is aligned to whole points so
  WindowServer does not enclose fractional edges in a 1061 pt frame.
- Anything else is ignored (for example `-NSDocumentRevisionsDebugMode YES`).
  AppKit's `NSTreatUnknownArgumentsAsOpen` is turned off, otherwise a slug
  argument is treated as a document and the main window never opens.

Launches are deterministic: no restored state (`ApplePersistenceIgnoreState`,
`.restorationBehavior(.disabled)`, split-view and window frame autosave keys
are cleared), no window-open animation, no focused text field, no
time-dependent content. The app activates itself like the gallery does, and
keyboard focus starts in the sidebar, so the selected row shows the accent fill.
The system accent color is used unchanged.

## Window and chrome (values from `examples/gallery/app.rs`, 1 rem = 16 pt)

| Item | Gallery | Here |
|---|---|---|
| Window | 1060 x 760 pt, centered, min 680 x 520 | same (frame pinned after creation) |
| Toolbar | unified AppKit toolbar, title = page name; items: Toggle Sidebar, flexible space, Change appearance, Zoom Out, Actual Size (disabled at 100 %), Zoom In | `.windowToolbarStyle(.unified)`, `.navigationTitle`; same items and symbols; system sidebar toggle removed. Zoom buttons are inert. |
| Sidebar | full height, 13.75 rem = 220 pt (9.25 rem below 43.75 rem window width), source-list rows from `NSOutlineView` metrics, collapsible group headers | `NavigationSplitView`, `List(.sidebar)` with `Section(isExpanded:)`, fixed 220 pt column |
| Content | padding 1.5 rem = 24 pt, gap 0.875 rem = 14 pt, max width 47.5 rem = 760 pt, 13 pt text, secondary description above every page except Overview | same |
| Status bar | min height 1.75 rem, 0.875 rem horizontal padding, 11 pt secondary text: "`<Page>` · 99 components" and a "Documentation" link, top divider | same |

Sidebar titles, order and groups come from `src/catalog.rs`. Symbols match the
SF Symbols the gallery resolves (`src/native_symbols.rs`). Where the gallery
falls back to a Lucide SVG there is no SF Symbol, so the closest substitute is
used: Focus trap, Button, Toggle/Switch, Textarea/Text view/Description list,
Number input/Stepper/OTP input, Radio, Table, Tree, Resizable, Carousel, Diff,
Radar/Candlestick/Sankey chart (`symbolMatchesGallery: false` in `Catalog.swift`).

`Catalog.swift` is generated from `src/catalog.rs`, the categories in
`scripts/compare-swiftui/pages.json` and the gallery's `navigation_icon()`. Regenerate it if those change.

## Pages

Accessibility identifiers: the page root is `page-<slug>`; controls use the
gallery's element ids (`check`, `switch`, `primary`, `workspace-choice`, `projects`, …).

Implemented with system controls:

| Slug | Content |
|---|---|
| `checkbox` | `Toggle(.checkbox)` "Include hidden files" (on; mixed via `Toggle(sources:)`), disabled checked "Managed by policy", "Set mixed state" button |
| `switch` | `Toggle(.switch)` "Show metadata" (on; label after the switch as in the gallery), "Metadata visible" |
| `button` | Appearance rows Primary/Outline/Secondary/Danger with "Apply" + disabled "Unavailable"; "Add workspace" label button, icon-only and disabled icon-only plus buttons, "Reset"; "Activations: 0". Primary = `.borderedProminent`, Danger = prominent red, Outline and Secondary = `.bordered` (no system outline button) |
| `select` | "Default workspace" `Picker(.menu)` (Personal, Team, Archived (disabled), Sandbox) in a 360 pt column, help text, "Selected: Personal workspace", divider, disabled "Managed workspace" picker on Team with help text |
| `table` | "Workspace projects" / "8 projects · Sample data" header; `Table` with Project, Status (green/orange/secondary), Owner, Updated, Files; 8 rows; inset style with alternating rows |

All other pages show the title, inventory category and "Not yet implemented"
(category 3 pages: "No native equivalent").

## Retina comparison capture

```sh
uv run scripts/compare-swiftui.py --pages checkbox --appearances light
uv run scripts/compare-swiftui.py --display main
uv run scripts/compare-swiftui.py --display 1 --skip-build
```

The comparison defaults to `--display auto`: the active display with the
highest AppKit backing scale (main wins ties). On the DELL + built-in Color LCD
setup this selects the secondary Retina display at 2x, rather than the 1x main
display. `main` or an active numeric Core Graphics display id overrides it;
inactive ids are errors in the comparison script. The selected display's name,
id, bounds and backing scale are recorded in `results.json`.

Each launch receives the selected numeric `--display` id. Before capture,
CGWindowList must report a 1060 x 760 pt full frame wholly on that display.
Captured pixel dimensions and scale must match its backing scale (2120 x 1520
px at 2x). Mismatches fail the pair and produce no comparison metrics; images
are never resized or truncated to hide a mismatch. `regions.json` offsets are
window points and are multiplied by the capture scale for the content crop.
Screen Recording permission is required for capture, but CGWindowList frame
inspection can verify placement and point sizes without capturing pixels:

```sh
python3 scripts/compare-swiftui.py --skip-build --bounds-only \
  --pages checkbox --appearances light --display auto --out target/swiftui-compare-bounds
```

`--bounds-only` launches and terminates each app by its retained PID, records
CGWindowList bounds and the selected screen's backing scale, and skips pixel
metrics. Actual capture scale is verified only when capturing pixels.

Use `--wait-idle 120` to wait for two minutes of user inactivity before each
launch that takes focus. Input during settling discards that capture, restores
focus and waits again. Idle gating is off by default. A non-empty `--out`
directory requires `--overwrite`.

Pass `--baseline <previous-run>/results.json` to print and report per-page,
per-appearance content deltas. Mismatch deltas are percentage points (positive
means worse); SSIM deltas are score differences (positive means better).
Missing comparison metrics are reported as unavailable.

Offline pipeline checks: `python3 scripts/compare-swiftui/test_pipeline.py`
(with the comparison script's Python dependencies installed).
