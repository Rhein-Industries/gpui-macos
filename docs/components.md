# Component inventory

The native gallery contains **99 component and foundation pages plus an overview**. Every catalog identifier has a working preview. Run `cargo run --example gallery`, or open a page directly with `cargo run --example gallery -- editor`. Add `--light` or `--dark` to choose an initial appearance, for example `cargo run --example gallery -- button --light`.

The [complete macOS coverage contract](coverage.md) maps every page to a real system integration, a GPUI replica, or a macOS interpretation. It also accounts for all 69 public Kit modules, including supporting APIs without dedicated documentation pages.

GPUI macOS supplies native presentation adapters for the common controls and window layouts. `gpui_macos::kit` exposes the complete GPUI Kit component API, including advanced editors, forms, data tables, and messaging parts. Initialization projects the same macOS colors, typography, geometry, and motion into both layers.

The overview is a working workspace preferences pane, with grouped rows, editable fields, retained selections, and save actions. The window uses source-list navigation and a toolbar titled for the selected component. These compositions follow [Apple's macOS 27 UI kit](https://developer.apple.com/design/resources/#macos) and the toolbar and sidebar guidance in [Build an AppKit app with the new design](https://developer.apple.com/videos/play/wwdc2025/310/). Regular controls are calibrated against the included [AppKit reference app](../tools/native-reference/main.swift) running on macOS 26.2: 13 px system labels and 24 px buttons and text fields at 100% zoom. See [design and architecture](design.md) for material behavior and platform limits.

```rust
gpui_macos::application()
    .with_assets(gpui_macos::NativeAssets::new(
        gpui_macos::gpui_kit::assets::AllAssets,
    ))
    .run(|cx| {
        gpui_macos::init(cx);
        gpui_macos::open_window(
            gpui_macos::window_options("Workspace", cx),
            cx,
            |window, cx| cx.new(|cx| Workspace::new(window, cx)),
        ).expect("open window");
    });
```

`open_window` installs the Root overlay host and system appearance observation. Retain input, selection, editor, carousel, and table state in your view; controls retain GPUI Kit behavior and accept ordinary GPUI style builders.

| Gallery page | Public entry point | Purpose |
| --- | --- | --- |
| `theme` — Theme | `gpui_macos::Theme` | System appearance, macOS accent colors, and semantic light and dark tokens. |
| `root` — Root | `gpui_macos::kit::Root` | The native window host coordinates overlays, focus, and text selection. |
| `focus_trap` — Focus trap | `gpui_macos::kit::FocusTrapElement` | Keep keyboard focus within the current group of related controls. |
| `button` — Button | `gpui_macos::button` | Content-sized actions with quiet hover, focus and pressed states. |
| `button_group` — Button group | `gpui_macos::button_group` / `gpui_macos::segmented_control` | Choose one setting from a row of mutually exclusive options. |
| `dropdown_button` — Dropdown button | `gpui_macos::menu` + `gpui_macos::button` | Combine a primary command and its related actions in one compact control. |
| `link` — Link | `gpui_macos::link` | Open a named destination. |
| `toggle` — Toggle | `gpui_macos::toggle` | Keep a command active until it is pressed again. |
| `toggle_group` — Toggle group | `gpui_macos::toggle_group` | Combine independent filters to show more than one status. |
| `clipboard` — Clipboard | `gpui_macos::kit::clipboard::Clipboard` | Copy a value with keyboard-accessible confirmation. |
| `command` — Command | `gpui_macos::kit::command::Command` | Find and invoke commands with a searchable, keyboard-controlled palette. |
| `input` — Input | `gpui_macos::input` | Single-line editing with selection, clipboard and IME support. |
| `textarea` — Textarea | `gpui_macos::textarea` | Multi-line notes with native text editing. |
| `editor` — Editor | `gpui_macos::kit::input::Editor` | Edit source with line numbers, folding, search, and multiple cursors. |
| `number_input` — Number input | `gpui_macos::number_input` | A compact numeric field with keyboard and button stepping. |
| `input_group` — Input group | `gpui_macos::kit::input::InputGroup` | Compose a text field with leading and trailing accessories. |
| `select` — Select | `gpui_macos::select` | Choose one value from a fixed set of options. |
| `combobox` — Combobox | `gpui_macos::combobox` | Search a collection, then choose a matching option. |
| `calendar` — Calendar | `gpui_macos::calendar` | Choose a date with month and year navigation. |
| `date_picker` — Date picker | `gpui_macos::date_picker` | Choose a date from a calendar anchored to a field. |
| `time_field` — Time field | `gpui_macos::kit::time_field::TimeField` | Edit hours and minutes as keyboard-navigable time segments. |
| `color_picker` — Color picker | `gpui_macos::color_picker` | Choose a label color with Hex and HSLA controls. |
| `otp_input` — OTP input | `gpui_macos::otp_input` | Enter a six-digit verification code. |
| `slider` — Slider | `gpui_macos::slider` | Adjust one value or a range with the pointer or keyboard. |
| `checkbox` — Checkbox | `gpui_macos::checkbox` | Independent choices, including a mixed selection. |
| `switch` — Switch | `gpui_macos::switch` | An immediate on/off choice with a visible track and thumb. |
| `radio` — Radio | `gpui_macos::radio` | Choose one option from a group. |
| `form` — Form | `gpui_macos::interpretation::form` / `gpui_macos::kit::form::Form` | Labeled, validated fields with consistent alignment and save actions. |
| `questionnaire` — Questionnaire | `gpui_macos::interpretation::questionnaire` / `gpui_macos::kit::questionnaire::*` | Move through questions with retained answers, validation, and progress. |
| `stepper` — Stepper | `gpui_macos::kit::stepper::Stepper` | Walk through the steps of a task with visible completion state. |
| `search_field` — Search field | `gpui_macos::search_field` | Search using a native inset field with an inline search symbol. |
| `secure_field` — Secure field | `gpui_macos::secure_field` | Enter a password in a masked text field. |
| `menu` — Menu | `gpui_macos::menu` / `gpui_macos::kit::native_menu::NativeMenu` | An anchored action menu with one pointer and keyboard cursor. |
| `tabs` — Tabs | `gpui_macos::tabs` / `gpui_macos::tab_list` | Switch between related views while keeping context. |
| `accordion` — Accordion | `gpui_macos::accordion` | Reveal supporting content when it is needed. |
| `collapsible` — Collapsible | `gpui_macos::collapsible` | Expand a single region for additional settings. |
| `nav_stack` — Navigation stack | `gpui_macos::nav_stack` | Navigate between persistent pages and return to where you left off. |
| `pagination` — Pagination | `gpui_macos::pagination` | Move through a paged collection. |
| `breadcrumb` — Breadcrumb | `gpui_macos::kit::breadcrumb::Breadcrumb` | Return to an ancestor in a document or folder hierarchy. |
| `sidebar` — Sidebar | `gpui_macos::sidebar` / `gpui_macos::sidebar_item` | Source-list navigation with icons and rounded selected rows. |
| `sheet` — Sheet | `gpui_macos::sheet` | Inspect project details in an edge-attached panel. |
| `dialog` — Dialog | `gpui_macos::dialog` | A focused task with confirmation, cancellation and focus return. |
| `alert_dialog` — Alert dialog | `gpui_macos::alert_dialog` | An explicit decision that cannot be dismissed by clicking the backdrop. |
| `popover` — Popover | `gpui_macos::popover` | Adjust contextual settings while keeping the workspace in view. |
| `tooltip` — Tooltip | `gpui_macos::tooltip` | A short explanation for an action, shown on hover. |
| `hover_card` — Hover card | `gpui_macos::hover_card` | Preview supporting details without leaving the page. |
| `toast` — Toast | `gpui_macos::toast` / `gpui_kit::base::ToastManager` | Brief feedback that leaves your current task in place. |
| `notification` — Notification | `gpui_macos::kit::notification::Notification` | Window-hosted notifications with semantic status and dismissal. |
| `icon` — Icon | `gpui_macos::icon` | Host SF Symbols with bundled SVG fallback, inheriting the surrounding text color. |
| `avatar` — Avatar | `gpui_macos::avatar` | Identify people and workspaces with an image or initials. |
| `text_view` — Text view | `gpui_macos::markdown` / `gpui_macos::html` | Read structured documents with selectable text and links. |
| `label` — Label | `gpui_macos::kit::label::Label` | Readable labels with secondary text, masking, and search highlights. |
| `panel` — Panel | `gpui_macos::panel` | A surface for a related group of settings or information. |
| `group_box` — Group box | `gpui_macos::kit::group_box::GroupBox` | Group preferences in a titled, rounded container. |
| `description_list` — Description list | `gpui_macos::interpretation::description_list` / `gpui_macos::kit::description_list::DescriptionList` | Compare labels and values in a structured inspector. |
| `list` — List | `gpui_macos::interpretation::list_item` in `gpui_macos::list_rows` / `gpui_macos::kit::list::ListItem` | Selectable rows with icons, secondary content, and keyboard navigation. |
| `virtual_list` — Virtual list | `gpui_macos::virtual_list` | Browse a large activity log with variable-height rows. |
| `scrollbar` — Scrollbar | `gpui_macos::scrollbar` | Drag the scroll thumb to move through a long activity log. |
| `scrollable` — Scrollable | `gpui_kit::StatefulInteractiveElement` scroll builders | Scroll long content in a bounded native pane. |
| `table` — Table | `gpui_macos::table` | Aligned columns for comparing records. |
| `data_table` — Data table | `gpui_macos::kit::table::DataTable` | Use the full table delegate for selection, columns, and large datasets. |
| `tree` — Tree | `gpui_macos::tree` | Explore nested folders and select a workspace document. |
| `resizable` — Resizable | `gpui_macos::resizable` | Drag the divider to adjust space between panes. |
| `dock` — Dock | `gpui_macos::dock_area` | Rearrange document panels by dragging their tabs. |
| `separator` — Separator | `gpui_macos::separator` | A quiet boundary between distinct sections. |
| `keycap` — Keycap | `gpui_macos::keycap` | Compact, readable keyboard hints. |
| `badge` — Badge | `gpui_macos::badge` | Short labels for neutral and semantic status. |
| `tag` — Tag | `gpui_macos::kit::tag::Tag` | Compact semantic labels with filled and outline presentations. |
| `alert` — Alert | `gpui_macos::alert` | Inline feedback banners for neutral and semantic conditions. |
| `empty_state` — Empty state | `gpui_macos::empty_state` | Explain an empty collection and its next step. |
| `progress` — Progress | `gpui_macos::progress` | Show how much of a known task is complete. |
| `spinner` — Spinner | `gpui_macos::native_spinner` / `kit::spinner::Spinner` | Show an ongoing task with an animated progress indicator. |
| `skeleton` — Skeleton | `gpui_macos::kit::skeleton::Skeleton` | Reserve stable layout while content is loading. |
| `shimmer` — Shimmer | `gpui_macos::kit::shimmer::ShimmerText` | Animate a subtle highlight over text during background work. |
| `rating` — Rating | `gpui_macos::kit::rating::Rating` | Choose a star rating or display a read-only score. |
| `carousel` — Carousel | `gpui_macos::interpretation::carousel` / `gpui_macos::kit::carousel::*` | Browse a collection with previous, next, and pagination controls. |
| `image` — Image | `gpui_kit::img` | Draw bundled or local images with preserved aspect ratio and cropping. |
| `diff` — Diff | `gpui_macos::kit::diff::Diff` | Review source changes in unified or side-by-side views. |
| `message` — Message | `gpui_macos::interpretation::message` / `gpui_macos::kit::message::*` | Compose avatars, headers, bodies, and footers in a conversation. |
| `bubble` — Bubble | `gpui_macos::interpretation::bubble_content` / `gpui_macos::kit::bubble::*` | Display incoming and outgoing message bubbles. |
| `marker` — Marker | `gpui_macos::kit::marker::Marker` | Mark dates and events in a conversation timeline. |
| `attachment` — Attachment | `gpui_macos::interpretation::attachment` / `gpui_macos::kit::attachment::*` | Show files with progress, retry, and remove actions. |
| `message_scroller` — Message scroller | `gpui_macos::kit::message_scroller::MessageScroller` | Follow new messages while preserving a manually scrolled position. |
| `speech` — Speech | `gpui_macos::kit::speech::{SpeechButton, SpeechWaveform}` | Speech capture and waveform controls; system recognition is an optional feature. |
| `title_bar` — Title bar | `gpui_macos::title_bar` / `gpui_macos::native_window_options` / `gpui_macos::window_options` | A native title-bar surface leaves room for the system window controls. |
| `toolbar` — Toolbar | `gpui_macos::toolbar` / `gpui_macos::toolbar_group` / `gpui_macos::toolbar_button` / `gpui_macos::native_toolbar_group` / `gpui_macos::native::install_native_toolbar` | Related commands share a glass group, with flat symbol buttons and an unbacked view title. |
| `status_bar` — Status bar | `gpui_macos::status_bar` | Quiet document status and contextual controls along a window edge. |
| `settings` — Settings | `gpui_macos::settings_group` / `gpui_macos::settings_row` | Grouped preference rows with descriptive labels and trailing controls. |
| `inspector` — Inspector | `gpui_macos::inspector` | A compact details pane with clear field alignment. |
| `inset_list` — Inset list | `gpui_macos::inset_list` / `gpui_macos::inset_list_row` | Rounded inset rows for grouped preferences and collections. |
| `material` — Material | `gpui_macos::material` / `gpui_macos::native_material` / `gpui_macos::native::open_glass_panel` | Semantic canvas fills, AppKit glass underlays, and separate system glass foreground panels. |
| `plot` — Plot | `gpui_macos::plot` | Compose axes, marks, scales, and grids for custom visualizations. |
| `line_chart` — Line chart | `gpui_macos::LineChart` | Follow one measure over time. Hover a point for its value. |
| `bar_chart` — Bar chart | `gpui_macos::BarChart` | Compare totals across categories. |
| `area_chart` — Area chart | `gpui_macos::AreaChart` | Compare related measures over time, such as download and upload. |
| `pie_chart` — Pie chart | `gpui_macos::PieChart` | Show how a whole divides into parts. |
| `radar_chart` — Radar chart | `gpui_macos::RadarChart` | Compare profiles across several dimensions at once. |
| `candlestick_chart` — Candlestick chart | `gpui_macos::CandlestickChart` | Read the open, high, low and close of each trading session. |
| `sankey_chart` — Sankey chart | `gpui_macos::SankeyChart` | Trace how an amount flows from its sources to where it goes. |


## Feature-dependent behavior

Code editing, gutters, search, selections, folding, and language editing rules work in the default build. Enable `tree-sitter-languages` to bundle syntax grammars. Enable `speech` to connect the speech controls to the macOS or Windows system recognizer; capture starts when the user presses the microphone. Without that feature, the gallery shows the actual disabled speech control.

The gallery uses a real AppKit toolbar and `NSGlassEffectView` sidebar underlays, backed by an opaque window-background layer, on macOS 26 and newer. `native_material` and `native_toolbar_group` expose system glass for application layouts; leave the GPUI renderer transparent with `native_window_options` where these surfaces should show. Menus and submenus, popovers, selection lists, date and color pickers, and hover cards use separate native glass windows with GPUI foreground content. Their glass can sample the owning window's rendered scene. `native::open_glass_panel` exposes this retained foreground route for application content. `material` and ordinary canvas surfaces use themed fills. Unsupported and headless hosts, and Reduce Transparency, use deterministic fallbacks. System Reduce Transparency and Increase Contrast preferences project into the theme.

## Verification

The gallery test suite retains the original control interaction tests and draws every catalog page in light and dark appearance at 16, 20, and 24 pixel rem sizes. Catalog tests verify unique identifiers, complete sidebar coverage, and nonempty descriptions.

The completed audit maps all 80 pinned documentation topics and all 69 public module namespaces to 100 explicit presentation contracts: 5 AppKit-backed entries, 42 GPUI replicas, and 53 macOS interpretations. These contracts identify actual system integrations separately from GPUI foreground controls. Advanced builders retain their full upstream API. The complete test run passes 165 tests, including 600 gallery draws; optional speech and syntax grammar features compile, and all-target Clippy passes with warnings denied. Live Cua checks verify native glass refraction, accessible popup controls, submenu selection in the original owner, and panel state after moving. See the [comparison evidence](comparison.md).

## Upstream coverage

This inventory covers every published GPUI Kit component documentation page at the pinned revision, including the foundation (`root`, `theme`, `focus-trap`), primitives (`plot`, `image`, `scrollable`), advanced workflows (`editor`, `diff`, `questionnaire`), and messaging components. Chart documentation is represented by seven dedicated chart previews; `kbd` is named `keycap`, `empty` is named `empty_state`, and `settings` uses the native grouped settings composition.

The complete upstream API remains available under `gpui_macos::kit`; [GPUI Kit documentation sources](https://github.com/longbridge/gpui-kit/tree/0bbd9870ce420af9feacbf948b21d3824d82dc0b/website/component) explain each builder in detail.
