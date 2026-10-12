// Generated from src/catalog.rs, scripts/compare-swiftui/pages.json and the gallery's
// navigation_icon(); regenerate if those change. Do not edit by hand.

struct PageInfo: Identifiable, Hashable, Sendable {
    let slug: String
    let title: String
    let description: String
    /// SwiftUI inventory category: 1 direct equivalent, 2 composed, 3 no native equivalent.
    let category: Int
    /// SF Symbol shown in the sidebar.
    let symbol: String
    /// True when the gallery itself draws this SF Symbol; false when the gallery
    /// falls back to a Lucide SVG and `symbol` is the closest SF substitute.
    let symbolMatchesGallery: Bool
    var id: String { slug }
}

struct PageGroup: Identifiable, Sendable {
    let title: String
    let pages: [PageInfo]
    var id: String { title }
}

enum Catalog {
    static let groups: [PageGroup] = [
        PageGroup(title: "Explore", pages: [
            PageInfo(slug: "overview", title: "Overview", description: "Explore the components together, then use the sidebar to inspect each one.", category: 2, symbol: "square.grid.2x2", symbolMatchesGallery: true),
            PageInfo(slug: "theme", title: "Theme", description: "System appearance, macOS accent colors, and semantic light and dark tokens.", category: 2, symbol: "paintpalette", symbolMatchesGallery: true),
            PageInfo(slug: "root", title: "Root", description: "The native window host coordinates overlays, focus, and text selection.", category: 2, symbol: "macwindow", symbolMatchesGallery: true),
            PageInfo(slug: "focus_trap", title: "Focus trap", description: "Keep keyboard focus within the current group of related controls.", category: 2, symbol: "viewfinder", symbolMatchesGallery: false),
        ]),
        PageGroup(title: "Actions", pages: [
            PageInfo(slug: "button", title: "Button", description: "Content-sized actions with quiet hover, focus and pressed states.", category: 1, symbol: "rectangle", symbolMatchesGallery: false),
            PageInfo(slug: "button_group", title: "Button group", description: "Choose one setting from a row of mutually exclusive options.", category: 1, symbol: "rectangle.split.2x1", symbolMatchesGallery: true),
            PageInfo(slug: "dropdown_button", title: "Dropdown button", description: "Combine a primary command and its related actions in one compact control.", category: 1, symbol: "chevron.up.chevron.down", symbolMatchesGallery: true),
            PageInfo(slug: "link", title: "Link", description: "Open a named destination.", category: 1, symbol: "link", symbolMatchesGallery: true),
            PageInfo(slug: "toggle", title: "Toggle", description: "Keep a command active until it is pressed again.", category: 1, symbol: "switch.2", symbolMatchesGallery: false),
            PageInfo(slug: "toggle_group", title: "Toggle group", description: "Combine independent filters to show more than one status.", category: 1, symbol: "rectangle.split.2x1", symbolMatchesGallery: true),
            PageInfo(slug: "clipboard", title: "Clipboard", description: "Copy a value with keyboard-accessible confirmation.", category: 2, symbol: "doc.on.clipboard", symbolMatchesGallery: true),
            PageInfo(slug: "command", title: "Command", description: "Find and invoke commands with a searchable, keyboard-controlled palette.", category: 2, symbol: "command", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "Forms", pages: [
            PageInfo(slug: "input", title: "Input", description: "Single-line editing with selection, clipboard and IME support.", category: 1, symbol: "textbox", symbolMatchesGallery: true),
            PageInfo(slug: "textarea", title: "Textarea", description: "Multi-line notes with native text editing.", category: 1, symbol: "text.alignleft", symbolMatchesGallery: false),
            PageInfo(slug: "editor", title: "Editor", description: "Edit source with line numbers, folding, search, and multiple cursors.", category: 3, symbol: "square.and.pencil", symbolMatchesGallery: true),
            PageInfo(slug: "number_input", title: "Number input", description: "A compact numeric field with keyboard and button stepping.", category: 1, symbol: "number", symbolMatchesGallery: false),
            PageInfo(slug: "input_group", title: "Input group", description: "Compose a text field with leading and trailing accessories.", category: 2, symbol: "textbox", symbolMatchesGallery: true),
            PageInfo(slug: "select", title: "Select", description: "Choose one value from a fixed set of options.", category: 1, symbol: "chevron.up.chevron.down", symbolMatchesGallery: true),
            PageInfo(slug: "combobox", title: "Combobox", description: "Search a collection, then choose a matching option.", category: 1, symbol: "magnifyingglass", symbolMatchesGallery: true),
            PageInfo(slug: "calendar", title: "Calendar", description: "Choose a date with month and year navigation.", category: 1, symbol: "calendar", symbolMatchesGallery: true),
            PageInfo(slug: "date_picker", title: "Date picker", description: "Choose a date from a calendar anchored to a field.", category: 1, symbol: "calendar", symbolMatchesGallery: true),
            PageInfo(slug: "time_field", title: "Time field", description: "Edit hours and minutes as keyboard-navigable time segments.", category: 1, symbol: "clock", symbolMatchesGallery: true),
            PageInfo(slug: "color_picker", title: "Color picker", description: "Choose a label color with Hex and HSLA controls.", category: 1, symbol: "paintpalette", symbolMatchesGallery: true),
            PageInfo(slug: "otp_input", title: "OTP input", description: "Enter a six-digit verification code.", category: 2, symbol: "number", symbolMatchesGallery: false),
            PageInfo(slug: "slider", title: "Slider", description: "Adjust one value or a range with the pointer or keyboard.", category: 1, symbol: "slider.horizontal.3", symbolMatchesGallery: true),
            PageInfo(slug: "checkbox", title: "Checkbox", description: "Independent choices, including a mixed selection.", category: 1, symbol: "checkmark.square", symbolMatchesGallery: true),
            PageInfo(slug: "switch", title: "Switch", description: "An immediate on/off choice with a visible track and thumb.", category: 1, symbol: "switch.2", symbolMatchesGallery: false),
            PageInfo(slug: "radio", title: "Radio", description: "Choose one option from a group.", category: 1, symbol: "smallcircle.filled.circle", symbolMatchesGallery: false),
            PageInfo(slug: "form", title: "Form", description: "Labeled, validated fields with consistent alignment and save actions.", category: 2, symbol: "list.clipboard", symbolMatchesGallery: true),
            PageInfo(slug: "questionnaire", title: "Questionnaire", description: "Move through questions with retained answers, validation, and progress.", category: 2, symbol: "list.clipboard", symbolMatchesGallery: true),
            PageInfo(slug: "stepper", title: "Stepper", description: "Walk through the steps of a task with visible completion state.", category: 2, symbol: "number", symbolMatchesGallery: false),
            PageInfo(slug: "search_field", title: "Search field", description: "Search using a native inset field with an inline search symbol.", category: 1, symbol: "magnifyingglass", symbolMatchesGallery: true),
            PageInfo(slug: "secure_field", title: "Secure field", description: "Enter a password in a masked text field.", category: 1, symbol: "lock", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "Navigation", pages: [
            PageInfo(slug: "menu", title: "Menu", description: "An anchored action menu with one pointer and keyboard cursor.", category: 1, symbol: "line.3.horizontal", symbolMatchesGallery: true),
            PageInfo(slug: "tabs", title: "Tabs", description: "Switch between related views while keeping context.", category: 1, symbol: "rectangle.split.2x2", symbolMatchesGallery: true),
            PageInfo(slug: "accordion", title: "Accordion", description: "Reveal supporting content when it is needed.", category: 2, symbol: "list.bullet", symbolMatchesGallery: true),
            PageInfo(slug: "collapsible", title: "Collapsible", description: "Expand a single region for additional settings.", category: 1, symbol: "list.bullet", symbolMatchesGallery: true),
            PageInfo(slug: "nav_stack", title: "Navigation stack", description: "Navigate between persistent pages and return to where you left off.", category: 2, symbol: "folder", symbolMatchesGallery: true),
            PageInfo(slug: "pagination", title: "Pagination", description: "Move through a paged collection.", category: 2, symbol: "ellipsis", symbolMatchesGallery: true),
            PageInfo(slug: "breadcrumb", title: "Breadcrumb", description: "Return to an ancestor in a document or folder hierarchy.", category: 2, symbol: "folder", symbolMatchesGallery: true),
            PageInfo(slug: "sidebar", title: "Sidebar", description: "Source-list navigation with icons and rounded selected rows.", category: 1, symbol: "sidebar.left", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "Overlays", pages: [
            PageInfo(slug: "sheet", title: "Sheet", description: "Inspect project details in an edge-attached panel.", category: 1, symbol: "sidebar.right", symbolMatchesGallery: true),
            PageInfo(slug: "dialog", title: "Dialog", description: "A focused task with confirmation, cancellation and focus return.", category: 1, symbol: "macwindow", symbolMatchesGallery: true),
            PageInfo(slug: "alert_dialog", title: "Alert dialog", description: "An explicit decision that cannot be dismissed by clicking the backdrop.", category: 1, symbol: "macwindow", symbolMatchesGallery: true),
            PageInfo(slug: "popover", title: "Popover", description: "Adjust contextual settings while keeping the workspace in view.", category: 1, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "tooltip", title: "Tooltip", description: "A short explanation for an action, shown on hover.", category: 1, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "hover_card", title: "Hover card", description: "Preview supporting details without leaving the page.", category: 2, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "toast", title: "Toast", description: "Brief feedback that leaves your current task in place.", category: 2, symbol: "bell", symbolMatchesGallery: true),
            PageInfo(slug: "notification", title: "Notification", description: "Window-hosted notifications with semantic status and dismissal.", category: 2, symbol: "bell", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "Display", pages: [
            PageInfo(slug: "icon", title: "Icon", description: "Native SF Symbols on macOS with Lucide fallbacks that inherit text color.", category: 1, symbol: "sparkles", symbolMatchesGallery: true),
            PageInfo(slug: "avatar", title: "Avatar", description: "Identify people and workspaces with an image or initials.", category: 2, symbol: "person", symbolMatchesGallery: true),
            PageInfo(slug: "text_view", title: "Text view", description: "Read structured documents with selectable text and links.", category: 2, symbol: "text.alignleft", symbolMatchesGallery: false),
            PageInfo(slug: "label", title: "Label", description: "Readable labels with secondary text, masking, and search highlights.", category: 2, symbol: "textformat", symbolMatchesGallery: true),
            PageInfo(slug: "panel", title: "Panel", description: "A surface for a related group of settings or information.", category: 2, symbol: "sidebar.right", symbolMatchesGallery: true),
            PageInfo(slug: "group_box", title: "Group box", description: "Group preferences in a titled, rounded container.", category: 1, symbol: "sidebar.right", symbolMatchesGallery: true),
            PageInfo(slug: "description_list", title: "Description list", description: "Compare labels and values in a structured inspector.", category: 2, symbol: "text.alignleft", symbolMatchesGallery: false),
            PageInfo(slug: "list", title: "List", description: "Selectable rows with icons, secondary content, and keyboard navigation.", category: 1, symbol: "list.bullet", symbolMatchesGallery: true),
            PageInfo(slug: "virtual_list", title: "Virtual list", description: "Browse a large activity log with variable-height rows.", category: 1, symbol: "list.bullet", symbolMatchesGallery: true),
            PageInfo(slug: "scrollbar", title: "Scrollbar", description: "Drag the scroll thumb to move through a long activity log.", category: 1, symbol: "arrow.up.arrow.down", symbolMatchesGallery: true),
            PageInfo(slug: "scrollable", title: "Scrollable", description: "Scroll long content in a bounded native pane.", category: 1, symbol: "arrow.up.arrow.down", symbolMatchesGallery: true),
            PageInfo(slug: "table", title: "Table", description: "Aligned columns for comparing records.", category: 1, symbol: "tablecells", symbolMatchesGallery: false),
            PageInfo(slug: "data_table", title: "Data table", description: "Use the full table delegate for selection, columns, and large datasets.", category: 1, symbol: "externaldrive", symbolMatchesGallery: true),
            PageInfo(slug: "tree", title: "Tree", description: "Explore nested folders and select a workspace document.", category: 1, symbol: "list.bullet.indent", symbolMatchesGallery: false),
            PageInfo(slug: "resizable", title: "Resizable", description: "Drag the divider to adjust space between panes.", category: 1, symbol: "square.split.2x1", symbolMatchesGallery: false),
            PageInfo(slug: "dock", title: "Dock", description: "Rearrange document panels by dragging their tabs.", category: 3, symbol: "square.grid.2x2", symbolMatchesGallery: true),
            PageInfo(slug: "separator", title: "Separator", description: "A quiet boundary between distinct sections.", category: 1, symbol: "minus", symbolMatchesGallery: true),
            PageInfo(slug: "keycap", title: "Keycap", description: "Compact, readable keyboard hints.", category: 2, symbol: "command", symbolMatchesGallery: true),
            PageInfo(slug: "badge", title: "Badge", description: "Short labels for neutral and semantic status.", category: 2, symbol: "tag", symbolMatchesGallery: true),
            PageInfo(slug: "tag", title: "Tag", description: "Compact semantic labels with filled and outline presentations.", category: 2, symbol: "tag", symbolMatchesGallery: true),
            PageInfo(slug: "alert", title: "Alert", description: "Inline feedback banners for neutral and semantic conditions.", category: 2, symbol: "exclamationmark.triangle", symbolMatchesGallery: true),
            PageInfo(slug: "empty_state", title: "Empty state", description: "Explain an empty collection and its next step.", category: 1, symbol: "tray", symbolMatchesGallery: true),
            PageInfo(slug: "progress", title: "Progress", description: "Show how much of a known task is complete.", category: 1, symbol: "gauge", symbolMatchesGallery: true),
            PageInfo(slug: "spinner", title: "Spinner", description: "Show an ongoing task with an animated progress indicator.", category: 1, symbol: "progress.indicator", symbolMatchesGallery: true),
            PageInfo(slug: "skeleton", title: "Skeleton", description: "Reserve stable layout while content is loading.", category: 2, symbol: "rectangle.split.1x2", symbolMatchesGallery: true),
            PageInfo(slug: "shimmer", title: "Shimmer", description: "Animate a subtle highlight over text during background work.", category: 2, symbol: "sparkles", symbolMatchesGallery: true),
            PageInfo(slug: "rating", title: "Rating", description: "Choose a star rating or display a read-only score.", category: 1, symbol: "star", symbolMatchesGallery: true),
            PageInfo(slug: "carousel", title: "Carousel", description: "Browse a collection with previous, next, and pagination controls.", category: 2, symbol: "rectangle.stack", symbolMatchesGallery: false),
            PageInfo(slug: "image", title: "Image", description: "Draw bundled or local images with preserved aspect ratio and cropping.", category: 1, symbol: "photo", symbolMatchesGallery: true),
            PageInfo(slug: "diff", title: "Diff", description: "Review source changes in unified or side-by-side views.", category: 3, symbol: "plus.forwardslash.minus", symbolMatchesGallery: false),
        ]),
        PageGroup(title: "Messages", pages: [
            PageInfo(slug: "message", title: "Message", description: "Compose avatars, headers, bodies, and footers in a conversation.", category: 2, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "bubble", title: "Bubble", description: "Display incoming and outgoing message bubbles.", category: 2, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "marker", title: "Marker", description: "Mark dates and events in a conversation timeline.", category: 2, symbol: "mappin", symbolMatchesGallery: true),
            PageInfo(slug: "attachment", title: "Attachment", description: "Show files with progress, retry, and remove actions.", category: 2, symbol: "paperclip", symbolMatchesGallery: true),
            PageInfo(slug: "message_scroller", title: "Message scroller", description: "Follow new messages while preserving a manually scrolled position.", category: 2, symbol: "bubble.left", symbolMatchesGallery: true),
            PageInfo(slug: "speech", title: "Speech", description: "Speech capture and waveform controls; system recognition is an optional feature.", category: 3, symbol: "mic", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "macOS layouts", pages: [
            PageInfo(slug: "title_bar", title: "Title bar", description: "A native title-bar surface leaves room for the system window controls.", category: 3, symbol: "macwindow", symbolMatchesGallery: true),
            PageInfo(slug: "toolbar", title: "Toolbar", description: "Compact command groups on a translucent macOS toolbar surface.", category: 2, symbol: "rectangle.split.2x1", symbolMatchesGallery: true),
            PageInfo(slug: "status_bar", title: "Status bar", description: "Quiet document status and contextual controls along a window edge.", category: 2, symbol: "minus", symbolMatchesGallery: true),
            PageInfo(slug: "settings", title: "Settings", description: "Grouped preference rows with descriptive labels and trailing controls.", category: 2, symbol: "gearshape", symbolMatchesGallery: true),
            PageInfo(slug: "inspector", title: "Inspector", description: "A compact details pane with clear field alignment.", category: 2, symbol: "sidebar.right", symbolMatchesGallery: true),
            PageInfo(slug: "inset_list", title: "Inset list", description: "Rounded inset rows for grouped preferences and collections.", category: 2, symbol: "list.bullet", symbolMatchesGallery: true),
            PageInfo(slug: "material", title: "Material", description: "Semantic surface fills and opt-in native system glass underlays.", category: 1, symbol: "paintpalette", symbolMatchesGallery: true),
        ]),
        PageGroup(title: "Charts", pages: [
            PageInfo(slug: "plot", title: "Plot", description: "Compose axes, marks, scales, and grids for custom visualizations.", category: 2, symbol: "chart.xyaxis.line", symbolMatchesGallery: true),
            PageInfo(slug: "line_chart", title: "Line chart", description: "Follow one measure over time. Hover a point for its value.", category: 1, symbol: "chart.xyaxis.line", symbolMatchesGallery: true),
            PageInfo(slug: "bar_chart", title: "Bar chart", description: "Compare totals across categories.", category: 1, symbol: "chart.bar", symbolMatchesGallery: true),
            PageInfo(slug: "area_chart", title: "Area chart", description: "Compare related measures over time, such as download and upload.", category: 1, symbol: "chart.xyaxis.line", symbolMatchesGallery: true),
            PageInfo(slug: "pie_chart", title: "Pie chart", description: "Show how a whole divides into parts.", category: 1, symbol: "chart.pie", symbolMatchesGallery: true),
            PageInfo(slug: "radar_chart", title: "Radar chart", description: "Compare profiles across several dimensions at once.", category: 3, symbol: "chart.dots.scatter", symbolMatchesGallery: false),
            PageInfo(slug: "candlestick_chart", title: "Candlestick chart", description: "Read the open, high, low and close of each trading session.", category: 2, symbol: "chart.bar.xaxis", symbolMatchesGallery: false),
            PageInfo(slug: "sankey_chart", title: "Sankey chart", description: "Trace how an amount flows from its sources to where it goes.", category: 3, symbol: "point.3.connected.trianglepath.dotted", symbolMatchesGallery: false),
        ]),
    ]

    static let pages: [PageInfo] = groups.flatMap(\.pages)

    static func page(_ slug: String) -> PageInfo? { pages.first { $0.slug == slug } }
}
