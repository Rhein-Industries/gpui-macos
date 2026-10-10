# Complete macOS coverage contract

The catalog has **100 pages**: the overview plus 99 component and foundation previews. It covers all **80 documentation topics** and all **69 public component modules** in GPUI Kit at `0bbd9870ce420af9feacbf948b21d3824d82dc0b`. The captured [upstream inventory](upstream-inventory.json) records the original documentation names, public modules, and root reexport groups. Tests compare that snapshot with the vendored component source, compile imports of every public module, and require one explicit contract for every catalog page.

This is a presentation and API coverage statement, not a claim of pixel identity with an untested operating system. The host reference is macOS 26.2. The design target is [Apple's macOS 27 resources](https://developer.apple.com/design/resources/) and [current AppKit design guidance](https://developer.apple.com/videos/play/wwdc2026/289/). Common control dimensions and colors are measured in the included Swift/AppKit reference. Advanced workflow layouts below are this library's macOS interpretations.

Three rendering routes are explicit:

- **AppKit backed** uses real host chrome, glass, or SF artwork together with GPUI content. The native API and fallback route are distinct; this does not turn every child into an AppKit control.
- **GPUI replica** follows an existing macOS control using GPUI rendering and behavior. A counterpart such as `NSButton` describes the visual reference, not the instantiated class.
- **macOS interpretation** applies desktop typography, spacing, grouped content and commands to a workflow without one corresponding AppKit primitive. No invented system control is claimed.

All original Kit state, builder, delegate, validation, navigation, animation and composition APIs remain reachable through `gpui_macos::kit`. The component index supplies [public entry points](components.md); [`interpretation::INTERPRETATIONS`](../src/interpretation.rs) supplies the same typed contract used by applications. `ComponentInfo::interpretation()` connects catalog metadata to its contract.

Native integrations are recorded independently of the foreground rendering route. On compatible macOS hosts, selection lists, menus and submenus, date and color pickers, popovers and hover cards retain their GPUI controls inside separate AppKit glass panels. Each submenu has its own panel, display-bounded placement and owner cleanup; selected commands dispatch to the original application window. These entries remain GPUI replicas or macOS interpretations. Unsupported hosts and Reduce Transparency preserve the same state and callbacks through the ordinary GPUI presentation.

Reusable macOS factories in `gpui_macos::interpretation` return the complete Kit types: `form`, `description_list`, `questionnaire`, `questionnaire_choice`, `questionnaire_actions`, `message`, `bubble_content`, `attachment`, and `carousel`. Inline `carousel_previous` and `carousel_next` return the library's native-density buttons while using the original CarouselState. They preserve retained answers, validation, selection and callbacks. The gallery consumes these public factories, rather than displaying static substitutes.

`questionnaire_previous`, `questionnaire_next`, `questionnaire_submit`, and `questionnaire_skip` return `Option<gpui_macos::Button>` according to Kit's navigation visibility. They retain the original navigation, validation and submission handlers, expose the accessible names Back, Continue, Done and Skip, and use 13-point labels and 24-point native button density. Filled actions become neutral when their window is inactive. `questionnaire_actions` remains the complete Kit layout container.

## Every catalog entry

| Catalog ID | Rendering route | macOS counterpart or composition | Presentation and retained capabilities |
| --- | --- | --- | --- |
| `overview` | macOS interpretation | System Settings preference pane | Quiet grouped rows, aligned labels and trailing save actions. Editable workspace; retained selections; save and cancel. |
| `theme` | macOS interpretation | macOS semantic appearance | System font, label hierarchy, accent and appearance tokens. Light and dark; system following; accessibility preferences. |
| `root` | macOS interpretation | macOS application content host | Transparent Root with native chrome and opaque content panes. Overlay hosting; focus; selectable text; notifications. |
| `focus_trap` | macOS interpretation | Modal keyboard focus scope | Tab traversal remains inside a presented control group. Forward and reverse focus; stable trigger restoration. |
| `button` | GPUI replica | NSButton | Compact system bezel and activation-aware accent. Primary and secondary; disabled; pointer and keyboard activation. |
| `button_group` | GPUI replica | NSSegmentedControl | Joined 24-point segments with selected bezel. Single selection; disabled segments; keyboard navigation. |
| `dropdown_button` | GPUI replica | NSPopUpButton command menu | Compact command bezel, disclosure and menu alignment. Separators; disabled commands; action callbacks. |
| `link` | macOS interpretation | macOS linked text and URL action | System accent text with focus and pointer feedback. URL opening; keyboard activation; disabled state. |
| `toggle` | GPUI replica | NSButton toggle mode | Retained selected bezel, compact spacing and focus ring. Controlled selection; keyboard activation; disabled state. |
| `toggle_group` | GPUI replica | NSSegmentedControl multiple-selection mode | Joined command segments with independent selection. Multiple selection; disabled commands; keyboard traversal. |
| `clipboard` | macOS interpretation | Copy command with inline confirmation | Compact SF copy symbol and temporary success feedback. System clipboard; value ownership; accessible activation. |
| `command` | macOS interpretation | Spotlight-style command search | Searchable compact command list with sections and shortcuts. Filtering; grouped commands; disabled entries; Enter confirmation. |
| `input` | GPUI replica | NSTextField | 24-point inset content well and native label density. Selection; undo and redo; clipboard; IME; validation. |
| `textarea` | GPUI replica | NSTextView | Multiline content well with ordinary document typography. Multiline editing; selection; clipboard; IME; scrolling. |
| `editor` | macOS interpretation | Xcode-style source editor | Monospaced editing pane, line gutters and restrained chrome. Find; multiple cursors; folding; diagnostics; optional syntax grammars. |
| `number_input` | GPUI replica | NSTextField with NSStepper | Compact numeric editor and increment/decrement commands. Numeric editing; stepping; bounds; controlled updates. |
| `input_group` | macOS interpretation | Composite macOS text-field accessory row | Field-aligned prefix, suffix and accessory commands. Shared editor; affixes; accessory alignment; keyboard input. |
| `select` | GPUI replica | NSPopUpButton selection mode | Compact popup bezel with selected label and checkmark. Controlled selection; grouped values; disabled options; native glass popup; opaque fallback. |
| `combobox` | GPUI replica | NSComboBox | Inset search field and compact matching-value list. Filtering; keyboard selection; focus; retained value; native glass popup; opaque fallback. |
| `calendar` | GPUI replica | NSDatePicker calendar style | Compact calendar grid, system labels and accent date. Month navigation; date selection; focus; disabled dates. |
| `date_picker` | GPUI replica | NSDatePicker | Compact date field with calendar disclosure. Retained date; date formatting; popup calendar; keyboard input; native glass calendar popup; opaque fallback. |
| `time_field` | GPUI replica | NSDatePicker hour/minute style | Compact segmented time editing with native label size. Time value; segment focus; increment and decrement. |
| `color_picker` | GPUI replica | NSColorWell with NSColorPanel | Small color well opening an editable color inspector. Hex and channel editing; commit and cancel; retained color; native glass inspector popup; opaque fallback. |
| `otp_input` | macOS interpretation | macOS verification-code entry | Compact aligned fields in a verification form. Digit filtering; paste distribution; completion; focus movement. |
| `slider` | GPUI replica | NSSlider | Thin track and horizontal pill thumb, with inactive colors. Single value; range values; keyboard changes; disabled state. |
| `checkbox` | GPUI replica | NSButton checkbox type | 16-point rounded indicator and SF check/minus marks. Checked and mixed; controlled updates; disabled; keyboard activation. |
| `switch` | GPUI replica | NSSwitch | 54-by-24 track, elongated thumb and activation-aware color. Controlled value; animated thumb; reduced motion; disabled. |
| `radio` | GPUI replica | NSButton radio type | 16-point circular indicator and compact option rows. Exclusive selection; arrow navigation; focus; disabled. |
| `form` | macOS interpretation | System Settings editable preference group | Aligned labels, secondary help and trailing save action. Fields; validation ownership; horizontal and multicolumn layout; submission. |
| `questionnaire` | macOS interpretation | Setup Assistant guided preferences | Quiet grouped radio rows, progress and Back/Continue actions. Retained answers; validation; shortcuts; previous/next; submission. |
| `stepper` | macOS interpretation | Setup Assistant stage navigation | Compact stages with current-step emphasis. Selected stage; pointer and keyboard stage activation. |
| `search_field` | GPUI replica | NSSearchField | Inset field with SF magnifier and native editing density. Search text; selection; undo; clipboard; IME. |
| `secure_field` | GPUI replica | NSSecureTextField | Inset field with masked display and secure copy behavior. Masked editing; secure clipboard policy; IME; retained value. |
| `menu` | GPUI replica | NSMenu | Compact command rows, checks, separators and submenus. GPUI command foreground; full native-menu API; disabled commands; shortcuts; separate native glass submenu panels; owner-window command dispatch; opaque fallback. |
| `tabs` | GPUI replica | NSTabView with segmented tab selector | Compact joined tab labels above retained content. Controlled tab; keyboard traversal; disabled tabs. |
| `accordion` | macOS interpretation | macOS disclosure preference sections | Disclosure symbol and plain expandable content rows. Open state; exclusive or multiple expansion; keyboard activation. |
| `collapsible` | macOS interpretation | macOS disclosure group | Quiet disclosure row with measured expansion. Retained open state; accessible expansion; animation. |
| `nav_stack` | macOS interpretation | macOS drill-down navigation | Back/Forward commands and retained detail panes. Push and pop; forward history; retained page state; motion. |
| `pagination` | macOS interpretation | macOS paged document navigation | Compact previous/next controls and current-page label. Controlled page; disabled boundaries; keyboard activation. |
| `breadcrumb` | macOS interpretation | Finder-style path navigation | Compact path segments and disclosure separators. Clickable ancestors; disabled current location; overflow composition. |
| `sidebar` | AppKit backed | NSOutlineView-style source list on NSGlassEffectView | Edge-aligned glass source list with semibold selected labels. Selection; keyboard traversal; native glass; deterministic fallback. |
| `sheet` | GPUI replica | NSWindow sheet presentation | Attached task pane with content, actions and focus containment. Open and close; Escape; focus trap; trigger restoration. |
| `dialog` | GPUI replica | NSPanel task dialog | Compact modal task layout with trailing actions. Controlled open state; focus trap; Escape; trigger restoration. |
| `alert_dialog` | GPUI replica | NSAlert | Clear warning hierarchy and safe default action. Destructive confirmation; cancellation; keyboard focus. |
| `popover` | GPUI replica | NSPopover | Compact anchored content well with rounded edges. Anchor placement; outside dismiss; focus; collision handling; native glass foreground panel; opaque fallback. |
| `tooltip` | GPUI replica | NSView toolTip | Small supplementary label with delayed appearance. Hover and focus; delay; placement; accessible label. |
| `hover_card` | macOS interpretation | macOS contextual preview popover | Compact preview content with secondary metadata. Delayed hover; focus retention; dismiss and placement; native glass preview panel; opaque fallback. |
| `toast` | macOS interpretation | macOS transient in-app status | Compact status row with symbol, text and optional undo. Independent messages; timeout; pause on hover/focus; actions. |
| `notification` | macOS interpretation | macOS in-app notification banner | Compact semantic notification with close and optional action. Success/warning/error; stacking; dismissal; timeout. |
| `icon` | AppKit backed | NSImage system symbols | Host SF geometry, optical weight and inherited template color. SF lookup; tint; cached masks; full Lucide fallback. |
| `avatar` | macOS interpretation | macOS account/contact image | Small circular contact image with initials fallback. Image content; initials; loading fallback; sizing. |
| `text_view` | GPUI replica | NSTextView document presentation | Readable document typography and restrained link styling. HTML and Markdown; selection; links; text layout. |
| `label` | macOS interpretation | macOS primary and secondary labels | 13-point primary text and semantic secondary help. Highlights; masking; secondary text; content composition. |
| `panel` | macOS interpretation | macOS grouped content section | Quiet section heading and natural-height content. Heading; content; configurable layout; grouped controls. |
| `group_box` | GPUI replica | NSBox | Quiet filled or outlined group with an overlapping title. Title; footer; filled/outlined variants; arbitrary content. |
| `description_list` | macOS interpretation | Finder Get Info property rows | Aligned secondary labels and primary values. Horizontal/vertical; columns; spans; structured values. |
| `list` | GPUI replica | NSTableView single-column list | Flat selectable rows with SF file symbols. Selection; pointer callbacks; custom row content. |
| `virtual_list` | GPUI replica | NSTableView virtualized source rows | Compact lazy rows measured at the current zoom. Virtualization; scroll handle; visible range; retained selection. |
| `scrollbar` | GPUI replica | NSScroller | Thin overlay scroller with native density. Horizontal and vertical; scroll position; hover/drag. |
| `scrollable` | GPUI replica | NSScrollView | Bounded document pane with ordinary row separators. Overflow; wheel and trackpad; scroll position; clipping. |
| `table` | GPUI replica | NSTableView | Compact header labels and alternating document rows. Headers; cells; selection; custom rendering. |
| `data_table` | GPUI replica | NSTableView sortable data view | Compact sortable headers, document metadata and row stripes. Virtual rows; selection; column sorting; resizing; delegate callbacks. |
| `tree` | GPUI replica | NSOutlineView | Indented disclosure rows with compact selection. Expand/collapse; hierarchy; selection; keyboard traversal. |
| `resizable` | GPUI replica | NSSplitView | Thin divider separating ordinary desktop panes. Drag resize; minimum sizes; orientation; retained state. |
| `dock` | macOS interpretation | Xcode-style docked workbench | Pane titles, compact tabs and resizable tool/content regions. Dock layout; tabs; split panes; serialization; panel state. |
| `separator` | GPUI replica | NSBox separator type | One-pixel semantic divider between related content. Horizontal and vertical; semantic color; zoom-safe hairline. |
| `keycap` | macOS interpretation | macOS keyboard shortcut annotation | Compact system key names and unobtrusive key bezel. Modifier symbols; key combinations; readable command hints. |
| `badge` | macOS interpretation | macOS compact status annotation | Small semantic capsule with system label density. Status variants; color; text; sizing. |
| `tag` | macOS interpretation | Finder-style document tag | Compact semantic document label with restrained outline. Semantic variants; outline; custom content. |
| `alert` | macOS interpretation | macOS inline validation/status message | Compact symbol, explanation and semantic accent. Status variants; inline content; custom actions. |
| `empty_state` | macOS interpretation | macOS empty document/library pane | Centered SF symbol, short explanation and optional command. Empty content; primary action; layout customization. |
| `progress` | GPUI replica | NSProgressIndicator bar style | Thin activation-aware track and progress fill. Controlled progress; bounds; semantic status. |
| `spinner` | AppKit backed | NSImage progress.indicator with GPUI animation | Host radial artwork, system tint and linear activity motion. Size; color; easing; custom artwork; reduced motion. |
| `skeleton` | macOS interpretation | macOS loading content placeholders | Quiet neutral blocks matching the eventual document rows. Shapes; primary/secondary placeholders; reduced motion. |
| `shimmer` | macOS interpretation | macOS subtle loading text | Secondary text with restrained animated emphasis. Text content; tint; animation; reduced motion. |
| `rating` | GPUI replica | NSLevelIndicator rating style | Compact SF star row with a retained score. Editable rating; read-only rating; disabled; size and color. |
| `carousel` | macOS interpretation | Quick Look document preview navigation | Document thumbnails, filename metadata and compact previous/next controls. Retained selection; arrows; pagination; keyboard; reduced motion. |
| `image` | GPUI replica | NSImageView | Actual document illustrations in bounded image previews. Asset/local/remote sources; contain/cover; loading and fallback. |
| `diff` | macOS interpretation | Xcode-style source comparison | Monospaced code panes and compact comparison-mode command. Unified/split modes; file hunks; synchronized scroll; source styles. |
| `message` | macOS interpretation | Messages-style conversation row | Compact contact label, aligned bubble and delivery metadata. Avatar/header/content/footer slots; alignment; accessible identity. |
| `bubble` | macOS interpretation | Messages-style conversation surface | Neutral incoming and accent outgoing bubbles with compact typography. Variants; alignment; rich content; reactions; grouped bubbles. |
| `marker` | macOS interpretation | Conversation date and activity separator | Centered secondary timestamp or compact activity label. Text markers; loading marker; arbitrary content. |
| `attachment` | macOS interpretation | Mail-style attached document row | SF file symbol, filename, size and trailing retry/remove commands. Pending/success/error; media; retry; remove; action slots. |
| `message_scroller` | macOS interpretation | Messages-style retained transcript | Bounded virtual conversation pane that follows new messages. Virtualization; append/prepend; jump to bottom; retained scroll. |
| `speech` | macOS interpretation | macOS dictation command and level feedback | Compact microphone command and waveform in a transcript pane. System recognition; partial/final text; errors; supported/disabled state. |
| `title_bar` | GPUI replica | NSWindow titlebar and traffic-light controls | Clear current-view title with space for real system window controls. Drag region; native traffic lights; custom title content. |
| `toolbar` | AppKit backed | NSToolbar | Unified AppKit commands and search, with a composable GPUI fallback. Native commands; native search; enable state; title updates; canvas groups. |
| `status_bar` | macOS interpretation | Finder-style document status row | Small secondary status labels at the bottom of the content. Leading/trailing content; selection count; saved state. |
| `settings` | macOS interpretation | System Settings grouped preferences | Natural-height gray groups and aligned labeled control rows. Editable values; help text; live updates; save actions. |
| `inspector` | macOS interpretation | Finder Get Info/Xcode inspector pane | Narrow property pane with secondary labels and editable values. Property rows; editable content; debug inspection API. |
| `inset_list` | macOS interpretation | System Settings inset navigation rows | Quiet inset source rows with trailing disclosure. Selectable rows; custom controls; disclosure; callbacks. |
| `material` | AppKit backed | NSGlassEffectView navigation material | Opt-in native system material plus deterministic canvas fills. Native underlay; separate system glass foreground panels; semantic roles; tint; clipping; accessibility fallback. |
| `plot` | macOS interpretation | macOS analytical chart canvas | Ordinary content pane with readable axes and precise data labels. Scales; axes; points; tooltip; custom marks. |
| `line_chart` | macOS interpretation | macOS time-series chart | Thin series, restrained grid and compact legend. Series; scales; axes; points; tooltip. |
| `bar_chart` | macOS interpretation | macOS comparison chart | Clear category bars and readable baseline labels. Categories; multiple series; axes; tooltip. |
| `area_chart` | macOS interpretation | macOS cumulative/time-series chart | Translucent series fill and restrained plot chrome. Multiple series; scales; axes; tooltip. |
| `pie_chart` | macOS interpretation | macOS proportion chart | Color-coded sectors, readable legend and proportion labels. Values; sectors; colors; tooltip. |
| `radar_chart` | macOS interpretation | macOS multi-attribute chart | Restrained radial grid and labeled attribute axes. Multiple series; attributes; scales; tooltip. |
| `candlestick_chart` | macOS interpretation | macOS financial chart | Precise open/high/low/close marks with readable axes. OHLC values; time scale; rising/falling colors; tooltip. |
| `sankey_chart` | macOS interpretation | macOS flow-analysis chart | Readable node labels and restrained weighted links. Nodes; links; flow widths; colors; tooltip. |

## Public module coverage

Each namespace below remains fully exposed as `gpui_macos::kit::<module>`. Modules with a different gallery name or a supporting role are explicitly mapped; they are not omitted from the facade.

| Public Kit module | Gallery pages |
| --- | --- |
| `global_state` | `root` |
| `accordion` | `accordion` |
| `alert` | `alert` |
| `attachment` | `attachment` |
| `avatar` | `avatar` |
| `badge` | `badge` |
| `breadcrumb` | `breadcrumb` |
| `bubble` | `bubble` |
| `button` | `button` |
| `carousel` | `carousel` |
| `chart` | `line_chart`, `bar_chart`, `area_chart`, `pie_chart`, `radar_chart`, `candlestick_chart`, `sankey_chart` |
| `checkbox` | `checkbox` |
| `clipboard` | `clipboard` |
| `collapsible` | `collapsible` |
| `color_picker` | `color_picker` |
| `combobox` | `combobox` |
| `command` | `command` |
| `description_list` | `description_list` |
| `dialog` | `dialog` |
| `diff` | `diff` |
| `dock` | `dock` |
| `empty` | `empty_state` |
| `form` | `form` |
| `group_box` | `group_box` |
| `highlighter` | `editor`, `diff` |
| `history` | `input`, `textarea`, `editor` |
| `hover_card` | `hover_card` |
| `input` | `input` |
| `kbd` | `keycap` |
| `label` | `label` |
| `link` | `link` |
| `list` | `list` |
| `marker` | `marker` |
| `menu` | `menu` |
| `message` | `message` |
| `message_scroller` | `message_scroller` |
| `native_menu` | `menu` |
| `notification` | `notification` |
| `pagination` | `pagination` |
| `plot` | `plot` |
| `popover` | `popover` |
| `progress` | `progress` |
| `questionnaire` | `questionnaire` |
| `radio` | `radio` |
| `rating` | `rating` |
| `resizable` | `resizable` |
| `scroll` | `scrollable`, `scrollbar` |
| `searchable_list` | `command`, `combobox`, `list` |
| `select` | `select` |
| `separator` | `separator` |
| `setting` | `settings` |
| `sheet` | `sheet` |
| `shimmer` | `shimmer` |
| `sidebar` | `sidebar` |
| `skeleton` | `skeleton` |
| `slider` | `slider` |
| `speech` | `speech` |
| `spinner` | `spinner` |
| `status_bar` | `status_bar` |
| `stepper` | `stepper` |
| `switch` | `switch` |
| `tab` | `tabs` |
| `table` | `table` |
| `tag` | `tag` |
| `text` | `text_view`, `label` |
| `theme` | `theme` |
| `toolbar` | `toolbar` |
| `tooltip` | `tooltip` |
| `tree` | `tree` |

The root reexports also remain complete, including Root, focus and element traits, sizing, Icon, IndexPath, ropes, resizable/virtual-list helpers, time controls, TitleBar, window borders and WindowExt. The debug DivInspector API remains accessible through Kit in debug or inspector-enabled builds. Native popup menus remain available as `kit::native_menu::NativeMenu`; the ordinary `menu` constructor is the GPUI-rendered route. Highlighter registries and language bundles keep their upstream feature requirements. History and searchable-list delegates are support APIs used by the visible editing and selection controls.

## Optional behavior and validation boundary

`tree-sitter-languages` supplies syntax grammars to the editor/diff APIs; editing, gutters, find, selections and folding remain available without it. `speech` enables host recognition and retains its explicit microphone/recognizer permission flow. The gallery does not request microphone access at startup. Image previews use three original, repository-owned SVG document illustrations through the actual image loader; local and remote image APIs are preserved.

The catalog render sweep draws all 100 pages in light and dark appearance at three actual rem sizes. Focused tests cover the guided-setup answer flow, document navigation and other interactive examples. The inventory tests guard against losing upstream topics, module exports, root export groups, or a macOS contract when pages change. Desktop-native glass and toolbar behavior are separately reviewed through RiWork's Cua.ai Driver; headless rendering cannot validate native compositing. Native panel routes and their live validation are documented in the [design](design.md) and [comparison](comparison.md) notes as they are integrated.
