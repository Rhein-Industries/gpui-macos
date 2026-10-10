import AppKit
import Foundation

// Every control in this comparison app is a real system AppKit control.
// Run --measure without entering the application loop to export native metrics.

@MainActor
final class FlippedDocumentView: NSView {
    override var isFlipped: Bool { true }
}

@MainActor
final class SourceListNode: NSObject {
    let title: String
    let symbol: String?
    let children: [SourceListNode]
    init(_ title: String, symbol: String? = nil, children: [SourceListNode] = []) {
        self.title = title
        self.symbol = symbol
        self.children = children
    }
}

/// A standard AppKit outline source list. Its row size and cell outlet metrics
/// come from AppKit and the user's sidebar icon size preference.
@MainActor
final class SourceListController: NSViewController, NSOutlineViewDataSource, NSOutlineViewDelegate {
    let outline = NSOutlineView()
    var rowSize: NSTableView.RowSizeStyle = .default
    let groups = [
        SourceListNode("Explore", children: [
            SourceListNode("Overview", symbol: "square.grid.2x2"),
            SourceListNode("Theme", symbol: "paintpalette"),
            SourceListNode("Root", symbol: "macwindow"),
            SourceListNode("Focus trap", symbol: "scope")
        ]),
        SourceListNode("Actions", children: [
            SourceListNode("Button", symbol: "rectangle"),
            SourceListNode("Button group", symbol: "rectangle.split.3x1"),
            SourceListNode("Dropdown button", symbol: "chevron.down.square"),
            SourceListNode("Link", symbol: "link"),
            SourceListNode("Toggle", symbol: "button.horizontal"),
            SourceListNode("Toggle group", symbol: "rectangle.split.3x1"),
            SourceListNode("Clipboard", symbol: "doc.on.clipboard"),
            SourceListNode("Command", symbol: "command")
        ]),
        SourceListNode("Forms", children: [
            SourceListNode("Input", symbol: "character.cursor.ibeam"),
            SourceListNode("Textarea", symbol: "text.alignleft"),
            SourceListNode("Checkbox", symbol: "checkmark.square"),
            SourceListNode("Switch", symbol: "switch.2"),
            SourceListNode("Radio group", symbol: "circle.inset.filled"),
            SourceListNode("Slider", symbol: "slider.horizontal.3"),
            SourceListNode("Calendar", symbol: "calendar")
        ])
    ]
    var on_selection: ((String) -> Void)?

    override func loadView() {
        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("Component"))
        column.title = "Components"
        outline.addTableColumn(column)
        outline.outlineTableColumn = column
        outline.headerView = nil
        outline.style = .sourceList
        outline.rowSizeStyle = rowSize
        outline.dataSource = self
        outline.delegate = self
        let scroll = NSScrollView()
        scroll.documentView = outline
        scroll.hasVerticalScroller = true
        scroll.autohidesScrollers = true
        self.view = scroll
        outline.reloadData()
        outline.expandItem(nil, expandChildren: true)
        outline.selectRowIndexes(IndexSet(integer: 1), byExtendingSelection: false)
    }

    func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        (item as? SourceListNode)?.children.count ?? groups.count
    }
    func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        (item as? SourceListNode)?.children[index] ?? groups[index]
    }
    func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        !(item as! SourceListNode).children.isEmpty
    }
    func outlineView(_ outlineView: NSOutlineView, isGroupItem item: Any) -> Bool {
        !(item as! SourceListNode).children.isEmpty
    }
    func outlineView(_ outlineView: NSOutlineView, shouldSelectItem item: Any) -> Bool {
        (item as! SourceListNode).children.isEmpty
    }
    func outlineView(_ outlineView: NSOutlineView, viewFor tableColumn: NSTableColumn?, item: Any) -> NSView? {
        let node = item as! SourceListNode
        let cell = NSTableCellView()
        let text = NSTextField(labelWithString: node.title)
        cell.textField = text
        cell.addSubview(text)
        if let symbol = node.symbol {
            let image = NSImageView()
            image.image = NSImage(systemSymbolName: symbol, accessibilityDescription: nil)
            image.contentTintColor = .controlAccentColor
            cell.imageView = image
            cell.addSubview(image)
        }
        return cell
    }
    func outlineViewSelectionDidChange(_ notification: Notification) {
        if let node = outline.item(atRow: outline.selectedRow) as? SourceListNode {
            on_selection?(node.title)
        }
    }

    func metrics(rowSize: NSTableView.RowSizeStyle = .default) -> [String: Any] {
        self.rowSize = rowSize
        view.frame = NSRect(x: 0, y: 0, width: 220, height: 600)
        view.layoutSubtreeIfNeeded()
        outline.reloadData()
        outline.expandItem(nil, expandChildren: true)
        outline.layoutSubtreeIfNeeded()
        var rows: [[String: Any]] = []
        for row in 0..<min(outline.numberOfRows, 7) {
            guard let node = outline.item(atRow: row) as? SourceListNode else { continue }
            let cell = outline.view(atColumn: 0, row: row, makeIfNecessary: true) as? NSTableCellView
            cell?.layoutSubtreeIfNeeded()
            var entry: [String: Any] = [
                "title": node.title,
                "group": !node.children.isEmpty,
                "row": NSStringFromRect(outline.rect(ofRow: row)),
                "disclosure": NSStringFromRect(outline.frameOfOutlineCell(atRow: row)),
                "cell": NSStringFromRect(cell?.frame ?? .zero),
                "text": NSStringFromRect(cell?.textField?.frame ?? .zero),
                "font_points": cell?.textField?.font?.pointSize ?? NSNull()
            ]
            if let image = cell?.imageView { entry["image"] = NSStringFromRect(image.frame) }
            rows.append(entry)
        }
        return [
            "row_height": outline.rowHeight,
            "row_size_style": outline.rowSizeStyle.rawValue,
            "intercell_spacing": NSStringFromSize(outline.intercellSpacing),
            "indentation_per_level": outline.indentationPerLevel,
            "rows": rows
        ]
    }
}

@MainActor
func label(_ title: String, secondary: Bool = false, bold: Bool = false) -> NSTextField {
    let view = NSTextField(labelWithString: title)
    view.font = .systemFont(ofSize: 13, weight: bold ? .semibold : .regular)
    view.textColor = secondary ? .secondaryLabelColor : .labelColor
    return view
}

@MainActor
func nativeButton(_ title: String, primary: Bool = false, disabled: Bool = false) -> NSButton {
    let button = NSButton(title: title, target: nil, action: nil)
    button.bezelStyle = .rounded
    button.controlSize = .regular
    if primary { button.keyEquivalent = "\r" }
    button.isEnabled = !disabled
    return button
}

@MainActor
func checkbox(_ title: String, state: NSControl.StateValue = .off) -> NSButton {
    let button = NSButton(checkboxWithTitle: title, target: nil, action: nil)
    button.allowsMixedState = true
    button.state = state
    return button
}

@MainActor
func radio(_ title: String, selected: Bool = false) -> NSButton {
    let button = NSButton(radioButtonWithTitle: title, target: nil, action: nil)
    button.state = selected ? .on : .off
    return button
}

@MainActor
func field(_ placeholder: String, value: String = "") -> NSTextField {
    let view = NSTextField(string: value)
    view.placeholderString = placeholder
    view.controlSize = .regular
    return view
}

@MainActor
func segment() -> NSSegmentedControl {
    let view = NSSegmentedControl(labels: ["General", "Network", "Advanced"], trackingMode: .selectOne, target: nil, action: nil)
    view.selectedSegment = 0
    view.segmentStyle = .automatic
    return view
}

@MainActor
func popup() -> NSPopUpButton {
    let view = NSPopUpButton(frame: .zero, pullsDown: false)
    view.addItems(withTitles: ["Standard", "High", "Best"])
    return view
}

@MainActor
func stack(_ views: [NSView], vertical: Bool = false, spacing: CGFloat = 12) -> NSStackView {
    let view = NSStackView(views: views)
    view.orientation = vertical ? .vertical : .horizontal
    view.alignment = vertical ? .leading : .centerY
    view.spacing = spacing
    view.distribution = .fill
    return view
}

@MainActor
func fixedWidth(_ view: NSView, _ width: CGFloat) -> NSView {
    view.translatesAutoresizingMaskIntoConstraints = false
    view.widthAnchor.constraint(equalToConstant: width).isActive = true
    return view
}

@MainActor
func row(_ title: String, _ views: [NSView]) -> NSStackView {
    let titleView = fixedWidth(label(title, secondary: true), 116)
    return stack([titleView] + views)
}

@MainActor
func group(_ title: String, _ rows: [NSView]) -> NSBox {
    let box = NSBox()
    box.title = title
    box.boxType = .primary
    box.titlePosition = .atTop
    box.contentViewMargins = NSSize(width: 16, height: 12)
    let content = stack(rows, vertical: true, spacing: 14)
    // NSBox does not propagate its content's Auto Layout intrinsic height.
    // Size the group from the real row fitting sizes, native title cell, and
    // its own content margins. Individual system controls remain unconstrained.
    let rowsHeight = rows.reduce(CGFloat(0)) { $0 + $1.fittingSize.height }
    let titleHeight = (box.titleCell as! NSCell).cellSize.height
    let height = rowsHeight + CGFloat(max(0, rows.count - 1)) * content.spacing
        + box.contentViewMargins.height * 2 + titleHeight + 4
    content.translatesAutoresizingMaskIntoConstraints = true
    content.autoresizingMask = [.width, .height]
    box.contentView = content
    box.heightAnchor.constraint(equalToConstant: height).isActive = true
    return box
}

@MainActor
final class ReferenceDelegate: NSObject, NSApplicationDelegate, NSToolbarDelegate, NSTableViewDataSource, NSTableViewDelegate {
    var window: NSWindow!
    var sidebar: NSTableView!
    var sourceListSidebar: SourceListController?
    var detail: NSViewController!
    var actionStatus: NSTextField?
    var numberField: NSTextField!
    var stepper: NSStepper!
    var radioButtons: [NSButton] = []
    let pages = ["Controls", "Workspace"]
    var initialPage = "controls"
    let files = [("Design system", "Folder", "Today"), ("Components.swift", "Swift source", "Today"), ("Interface.sketch", "Design document", "Yesterday"), ("Research", "Folder", "Yesterday"), ("Release notes.md", "Markdown", "Monday")]

    func applicationDidFinishLaunching(_ notification: Notification) {
        installMenu()
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1060, height: 820), styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView], backing: .buffered, defer: false)
        window.title = "Native macOS Reference"
        window.minSize = NSSize(width: 840, height: 680)
        window.toolbarStyle = .unified
        window.titleVisibility = .visible
        let toolbar = NSToolbar(identifier: "ReferenceToolbar")
        toolbar.delegate = self
        toolbar.displayMode = .iconOnly
        window.toolbar = toolbar

        let split = NSSplitViewController()
        let sidebarController: NSViewController
        if initialPage == "sidebar" {
            let controller = SourceListController()
            sourceListSidebar = controller
            sidebarController = controller
        } else {
            sidebarController = NSViewController()
            sidebar = NSTableView()
            let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("Page"))
            column.title = "Library"
            sidebar.addTableColumn(column)
            sidebar.headerView = nil
            sidebar.dataSource = self
            sidebar.delegate = self
            sidebar.rowHeight = 30
            sidebar.style = .sourceList
            let sidebarScroll = NSScrollView()
            sidebarScroll.documentView = sidebar
            sidebarScroll.hasVerticalScroller = false
            sidebarController.view = sidebarScroll
        }
        let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebarController)
        sidebarItem.minimumThickness = initialPage == "sidebar" ? 220 : 190
        sidebarItem.maximumThickness = initialPage == "sidebar" ? 360 : 230
        split.addSplitViewItem(sidebarItem)
        detail = NSViewController()
        detail.view = NSView()
        split.addSplitViewItem(NSSplitViewItem(viewController: detail))
        window.contentViewController = split
        let index = initialPage == "sidebar" ? 2 : (initialPage == "workspace" ? 1 : 0)
        if index < 2 { sidebar.selectRowIndexes(IndexSet(integer: index), byExtendingSelection: false) }
        showPage(index)
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }

    func installMenu() {
        let menu = NSMenu()
        let appItem = NSMenuItem()
        menu.addItem(appItem)
        let appMenu = NSMenu()
        appMenu.addItem(withTitle: "About Native macOS Reference", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: "")
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: "Quit Native macOS Reference", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appItem.submenu = appMenu
        let appearanceItem = NSMenuItem(title: "Appearance", action: nil, keyEquivalent: "")
        let appearanceMenu = NSMenu()
        for (title, tag) in [("System", 0), ("Light", 1), ("Dark", 2)] {
            let item = NSMenuItem(title: title, action: #selector(changeAppearance(_:)), keyEquivalent: "")
            item.target = self
            item.tag = tag
            appearanceMenu.addItem(item)
        }
        appearanceItem.submenu = appearanceMenu
        menu.addItem(appearanceItem)
        NSApplication.shared.mainMenu = menu
    }

    @objc func changeAppearance(_ sender: NSMenuItem) {
        NSApplication.shared.appearance = sender.tag == 0 ? nil : NSAppearance(named: sender.tag == 1 ? .aqua : .darkAqua)
    }

    @objc func clickedButton(_ sender: NSButton) {
        actionStatus?.stringValue = "Last action: \(sender.title)"
    }
    @objc func updateNumber(_ sender: NSStepper) { numberField.doubleValue = sender.doubleValue }
    @objc func updateStepper(_ sender: NSTextField) { stepper.doubleValue = sender.doubleValue }
    @objc func chooseRadio(_ sender: NSButton) {
        for button in radioButtons { button.state = button === sender ? .on : .off }
    }

    func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] { [.toggleSidebar, .flexibleSpace, .init("search"), .init("add")] }
    func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] { [.toggleSidebar, .flexibleSpace, .init("search"), .init("add")] }
    func toolbar(_ toolbar: NSToolbar, itemForItemIdentifier identifier: NSToolbarItem.Identifier, willBeInsertedIntoToolbar: Bool) -> NSToolbarItem? {
        if identifier == .toggleSidebar { return nil }
        let item = NSToolbarItem(itemIdentifier: identifier)
        if identifier.rawValue == "search" {
            let search = NSSearchField()
            search.placeholderString = "Search components"
            search.sizeToFit()
            search.frame.size.width = 200
            item.view = search
            item.label = "Search"
        } else {
            item.image = NSImage(systemSymbolName: "plus", accessibilityDescription: "Add")
            item.label = "Add"
        }
        return item
    }

    func numberOfRows(in tableView: NSTableView) -> Int { tableView === sidebar ? pages.count : files.count }
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row index: Int) -> NSView? {
        let identifier = tableColumn?.identifier.rawValue ?? "Page"
        if tableView === sidebar {
            let cell = NSTableCellView()
            let image = NSImageView()
            image.image = NSImage(systemSymbolName: index == 0 ? "slider.horizontal.3" : "folder", accessibilityDescription: nil)
            let text = label(pages[index])
            let contents = stack([image, text], spacing: 8)
            image.translatesAutoresizingMaskIntoConstraints = false
            NSLayoutConstraint.activate([image.widthAnchor.constraint(equalToConstant: 16), image.heightAnchor.constraint(equalToConstant: 16)])
            contents.translatesAutoresizingMaskIntoConstraints = false
            cell.addSubview(contents)
            NSLayoutConstraint.activate([contents.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 10), contents.trailingAnchor.constraint(lessThanOrEqualTo: cell.trailingAnchor, constant: -10), contents.centerYAnchor.constraint(equalTo: cell.centerYAnchor)])
            return cell
        }
        let file = files[index]
        return label(identifier == "Name" ? file.0 : identifier == "Kind" ? file.1 : file.2)
    }
    func tableViewSelectionDidChange(_ notification: Notification) {
        if notification.object as? NSTableView === sidebar && sidebar.selectedRow >= 0 { showPage(sidebar.selectedRow) }
    }

    func showPage(_ index: Int) {
        let content = index == 0 ? controlsPage() : (index == 1 ? workspacePage() : sidebarPage())
        let scroll = NSScrollView()
        scroll.drawsBackground = true
        scroll.backgroundColor = .windowBackgroundColor
        scroll.hasVerticalScroller = true
        scroll.autohidesScrollers = true
        let document = FlippedDocumentView()
        document.translatesAutoresizingMaskIntoConstraints = false
        content.translatesAutoresizingMaskIntoConstraints = false
        document.addSubview(content)
        scroll.documentView = document
        NSLayoutConstraint.activate([
            document.widthAnchor.constraint(equalTo: scroll.contentView.widthAnchor),
            content.leadingAnchor.constraint(equalTo: document.leadingAnchor, constant: 28),
            content.trailingAnchor.constraint(equalTo: document.trailingAnchor, constant: -28),
            content.topAnchor.constraint(equalTo: document.topAnchor, constant: 24),
            content.bottomAnchor.constraint(equalTo: document.bottomAnchor, constant: -24)
        ])
        let container = detail.view
        container.subviews.forEach { $0.removeFromSuperview() }
        scroll.translatesAutoresizingMaskIntoConstraints = false
        container.addSubview(scroll)
        NSLayoutConstraint.activate([
            scroll.leadingAnchor.constraint(equalTo: container.leadingAnchor),
            scroll.trailingAnchor.constraint(equalTo: container.trailingAnchor),
            scroll.topAnchor.constraint(equalTo: container.topAnchor),
            scroll.bottomAnchor.constraint(equalTo: container.bottomAnchor)
        ])
    }

    func controlsPage() -> NSStackView {
        let heading = label("Native controls", bold: true)
        heading.font = .systemFont(ofSize: 24, weight: .semibold)
        let caption = label("Actual AppKit controls on the current macOS. No custom control drawing.", secondary: true)
        actionStatus = caption
        let primary = nativeButton("Continue", primary: true)
        let secondary = nativeButton("Cancel")
        for button in [primary, secondary] { button.target = self; button.action = #selector(clickedButton(_:)) }
        let buttons = row("Buttons", [primary, secondary, nativeButton("Disabled", disabled: true)])
        let checkboxes = row("Checkboxes", [checkbox("Remember me", state: .on), checkbox("Mixed selection", state: .mixed)])
        radioButtons = [radio("Automatic", selected: true), radio("Manual")]
        for button in radioButtons { button.target = self; button.action = #selector(chooseRadio(_:)) }
        let radios = row("Radio buttons", radioButtons)
        let on = NSSwitch(); on.state = .on
        let off = NSSwitch(); off.state = .off
        let switches = row("Switches", [on, label("Wi-Fi"), off, label("AirDrop")])
        let regular = fixedWidth(field("Name", value: "Taylor"), 170)
        let search = NSSearchField(); search.placeholderString = "Search"
        let secure = NSSecureTextField(string: "macOS"); secure.placeholderString = "Password"
        let fields = row("Text fields", [regular, fixedWidth(search, 170), fixedWidth(secure, 170)])
        let segments = row("Segmented", [segment()])
        numberField = field("", value: "42")
        numberField.alignment = .right
        numberField.target = self; numberField.action = #selector(updateStepper(_:))
        stepper = NSStepper(); stepper.minValue = 0; stepper.maxValue = 100; stepper.doubleValue = 42
        stepper.target = self; stepper.action = #selector(updateNumber(_:))
        let choices = row("Pickers", [popup(), label("Number", secondary: true), fixedWidth(numberField, 70), stepper])
        let slider = NSSlider(value: 65, minValue: 0, maxValue: 100, target: nil, action: nil)
        let ticks = NSSlider(value: 50, minValue: 0, maxValue: 100, target: nil, action: nil)
        ticks.numberOfTickMarks = 5; ticks.allowsTickMarkValuesOnly = true
        let sliders = row("Sliders", [fixedWidth(slider, 220), fixedWidth(ticks, 220)])
        let progress = NSProgressIndicator(); progress.style = .bar; progress.isIndeterminate = false; progress.doubleValue = 62
        let spinner = NSProgressIndicator(); spinner.style = .spinning; spinner.controlSize = .small; spinner.startAnimation(nil)
        let progresses = row("Progress", [fixedWidth(progress, 220), spinner, label("Loading…", secondary: true)])
        let automatic = NSSwitch(); automatic.state = .on
        let notifications = checkbox("Allow notifications", state: .on)
        let preferences = group("Preferences", [row("Appearance", [popup()]), row("Updates", [label("Download automatically"), automatic]), row("Notifications", [notifications])])
        let controls = group("Standard system controls", [buttons, checkboxes, radios, switches, fields, segments, choices, sliders, progresses])
        let page = stack([heading, caption, controls, preferences], vertical: true, spacing: 16)
        for box in [controls, preferences] { box.widthAnchor.constraint(equalTo: page.widthAnchor).isActive = true }
        return page
    }

    func workspacePage() -> NSStackView {
        let heading = label("Workspace", bold: true); heading.font = .systemFont(ofSize: 24, weight: .semibold)
        let table = NSTableView()
        for (title, width) in [("Name", CGFloat(300)), ("Kind", 180), ("Modified", 120)] {
            let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier(title)); column.title = title; column.width = width; table.addTableColumn(column)
        }
        table.dataSource = self; table.delegate = self; table.rowHeight = 28; table.usesAlternatingRowBackgroundColors = true; table.style = .fullWidth
        let scroll = NSScrollView(); scroll.documentView = table; scroll.hasVerticalScroller = true; scroll.borderType = .bezelBorder
        scroll.heightAnchor.constraint(equalToConstant: 340).isActive = true
        let page = stack([heading, label("Native table, sidebar and unified toolbar.", secondary: true), scroll], vertical: true, spacing: 16)
        scroll.widthAnchor.constraint(equalTo: page.widthAnchor).isActive = true
        return page
    }

    func sidebarPage() -> NSStackView {
        let heading = label("Native sidebar", bold: true)
        heading.font = .systemFont(ofSize: 24, weight: .semibold)
        let selection = label("Selected: Overview", secondary: true)
        sourceListSidebar?.on_selection = { selection.stringValue = "Selected: \($0)" }
        return stack([
            heading,
            label("NSOutlineView.sourceList with system sidebar size and standard cell outlets.", secondary: true),
            selection,
            label("Disclosure groups, SF Symbols, selection and scrolling are drawn by AppKit.", secondary: true)
        ], vertical: true, spacing: 16)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
}

@MainActor
func measurements() throws -> Data {
    var results: [String: Any] = [:]
    for (name, appearanceName) in [("light", NSAppearance.Name.aqua), ("dark", .darkAqua)] {
        let appearance = NSAppearance(named: appearanceName)!
        var sample: [String: Any] = [:]
        appearance.performAsCurrentDrawingAppearance {
            let on = NSSwitch(); on.state = .on
            let off = NSSwitch(); off.state = .off
            let search = NSSearchField(); search.placeholderString = "Search"
            let secure = NSSecureTextField(string: "macOS")
            let slider = NSSlider(value: 65, minValue: 0, maxValue: 100, target: nil, action: nil)
            let progress = NSProgressIndicator(); progress.style = .bar; progress.isIndeterminate = false
            let controls: [(String, NSView)] = [
                ("button_primary", nativeButton("Continue", primary: true)), ("button_secondary", nativeButton("Cancel")), ("button_disabled", nativeButton("Disabled", disabled: true)),
                ("checkbox", checkbox("Remember me", state: .on)), ("checkbox_mixed", checkbox("Mixed selection", state: .mixed)), ("radio", radio("Automatic", selected: true)),
                ("switch_on", on), ("switch_off", off), ("text_field", field("Name", value: "Taylor")), ("search_field", search), ("secure_field", secure),
                ("segmented", segment()), ("popup", popup()), ("stepper", NSStepper()), ("slider", slider), ("progress", progress)
            ]
            var sizes: [String: Any] = [:]
            for (key, control) in controls {
                control.appearance = appearance
                if let control = control as? NSControl { control.sizeToFit() }
                let intrinsic = control.intrinsicContentSize
                let fitting = control.fittingSize
                var entry: [String: Any] = [
                    "intrinsic_width": intrinsic.width >= 0 ? intrinsic.width as Any : NSNull(), "intrinsic_height": intrinsic.height >= 0 ? intrinsic.height as Any : NSNull(),
                    "fitting_width": fitting.width, "fitting_height": fitting.height, "frame_width": control.frame.width, "frame_height": control.frame.height
                ]
                if let control = control as? NSControl { entry["font_points"] = control.font?.pointSize ?? NSNull() }
                if let button = control as? NSButton {
                    entry["bezel_style"] = button.bezelStyle.rawValue
                    entry["bezel_drawing_rect"] = NSStringFromRect(button.cell?.drawingRect(forBounds: button.bounds) ?? .zero)
                }
                sizes[key] = entry
            }
            let colors: [(String, NSColor)] = [
                ("window_background", .windowBackgroundColor), ("control_background", .controlBackgroundColor), ("control", .controlColor), ("under_page_background", .underPageBackgroundColor), ("text_background", .textBackgroundColor), ("control_text", .controlTextColor), ("disabled_control_text", .disabledControlTextColor),
                ("label", .labelColor), ("secondary_label", .secondaryLabelColor), ("tertiary_label", .tertiaryLabelColor), ("separator", .separatorColor),
                ("accent", .controlAccentColor), ("selected_control", .selectedControlColor), ("selected_control_text", .selectedControlTextColor),
                ("selected_content_background", .selectedContentBackgroundColor), ("unemphasized_selected_content_background", .unemphasizedSelectedContentBackgroundColor), ("unemphasized_selected_text", .unemphasizedSelectedTextColor), ("keyboard_focus", .keyboardFocusIndicatorColor)
            ]
            var rgba: [String: Any] = [:]
            for (key, color) in colors {
                if let color = color.usingColorSpace(.sRGB) { rgba[key] = ["r": color.redComponent, "g": color.greenComponent, "b": color.blueComponent, "a": color.alphaComponent] }
            }
            sample = ["controls": sizes, "colors": rgba, "sidebar": SourceListController().metrics(), "sidebar_sizes": ["small": SourceListController().metrics(rowSize: .small), "medium": SourceListController().metrics(rowSize: .medium), "large": SourceListController().metrics(rowSize: .large)]]
        }
        results[name] = sample
    }
    results["environment"] = ["os": ProcessInfo.processInfo.operatingSystemVersionString, "sdk": "macOS 26.2", "implementation": "Standard AppKit controls; intrinsic/fitting dimensions are sampled, not inferred", "font": NSFont.systemFont(ofSize: NSFont.systemFontSize).fontName, "system_font_size": NSFont.systemFontSize]
    return try JSONSerialization.data(withJSONObject: results, options: [.prettyPrinted, .sortedKeys])
}

let arguments = CommandLine.arguments
let app = NSApplication.shared
if arguments.contains("--measure") {
    do {
        let data = try measurements()
        if let index = arguments.firstIndex(of: "--metrics"), arguments.count > index + 1 {
            try data.write(to: URL(fileURLWithPath: arguments[index + 1]))
        } else {
            FileHandle.standardOutput.write(data)
            FileHandle.standardOutput.write(Data("\n".utf8))
        }
    } catch {
        FileHandle.standardError.write(Data("Measurement failed: \(error)\n".utf8))
        exit(1)
    }
} else {
    app.setActivationPolicy(.regular)
    if arguments.contains("--light") { app.appearance = NSAppearance(named: .aqua) }
    if arguments.contains("--dark") { app.appearance = NSAppearance(named: .darkAqua) }
    let delegate = ReferenceDelegate()
    if let index = arguments.firstIndex(of: "--page"), arguments.count > index + 1 { delegate.initialPage = arguments[index + 1] }
    app.delegate = delegate
    app.run()
}
