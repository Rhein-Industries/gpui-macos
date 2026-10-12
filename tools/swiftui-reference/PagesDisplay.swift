import AppKit
import SwiftUI

// Display pages (table lives in Pages.swift).
// Each page mirrors its gallery render function (examples/gallery/).

extension PageContent {
    func displayPage(_ slug: String) -> AnyView? {
        switch slug {
        case "icon": AnyView(IconPage())
        case "group_box": AnyView(GroupBoxPage())
        case "list": AnyView(ListPage())
        case "virtual_list": AnyView(ActivityLogPage(showsTimeline: false))
        case "scrollbar": AnyView(ActivityLogPage(showsTimeline: true))
        case "scrollable": AnyView(ScrollablePage())
        case "data_table": AnyView(DataTablePage())
        case "tree": AnyView(TreePage())
        case "resizable": AnyView(ResizablePage())
        case "separator": AnyView(SeparatorPage())
        case "empty_state": AnyView(EmptyStatePage())
        case "progress": AnyView(ProgressPage())
        case "spinner": AnyView(SpinnerPage())
        case "rating": AnyView(RatingPage())
        case "image": AnyView(ImagePage())
        default: nil
        }
    }
}

/// Gallery theme roles mapped to the AppKit colours they approximate.
private enum DisplayColors {
    /// `t.border` (black 12 % / white 13 %).
    static let border = Color(nsColor: .separatorColor)
    /// `t.divider()` (foreground at 25/255).
    static let divider = Color(nsColor: .separatorColor)
    /// `t.surface` (white / #1e1e1e).
    static let surface = Color(nsColor: .controlBackgroundColor)
    /// `t.normal_fill()` (foreground at 4 %).
    static let normalFill = Color(nsColor: .quinarySystemFill)
}

/// extended.rs `note`: secondary text at 0.8125 rem.
private struct Note: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View {
        Text(text).foregroundStyle(.secondary)
    }
}

// MARK: - Icon (render_icon_page)

private struct IconPage: View {
    /// SF Symbol names from src/native_symbols.rs.
    private let icons: [(String, String)] = [
        ("Check", "checkmark"),
        ("Minus", "minus"),
        ("Plus", "plus"),
        ("Chevron down", "chevron.down"),
        ("Chevron right", "chevron.right"),
        ("Star", "star"),
        ("External link", "arrow.up.right.square"),
        ("Close", "xmark"),
        ("Search", "magnifyingglass"),
        ("Menu", "line.3.horizontal"),
        ("Settings", "gearshape"),
        ("Alert", "exclamationmark.triangle"),
    ]

    var body: some View {
        ForEach(icons, id: \.0) { name, symbol in
            HStack(spacing: 8) {
                // icon() is 1 rem square.
                Image(systemName: symbol)
                    .frame(width: 16, height: 16)
                    .accessibilityIdentifier("icon-\(symbol)")
                Text(name)
            }
        }
    }
}

// MARK: - Group box (extended.rs "group_box")

private struct GroupBoxPage: View {
    @State private var enabled = true

    var body: some View {
        GroupBox {
            HStack {
                Text("Use system appearance")
                Spacer()
                Toggle("Use system appearance", isOn: $enabled)
                    .toggleStyle(.switch)
                    .labelsHidden()
                    .accessibilityIdentifier("group-appearance")
            }
            .frame(maxWidth: .infinity)
        } label: {
            Text("Appearance")
        }
        .frame(maxWidth: .infinity)
        .accessibilityIdentifier("group-box-appearance")

        VStack(alignment: .leading, spacing: 8) {
            // SwiftUI has no outlined group box on macOS; the outline variant
            // is a GroupBox with a stroked style.
            GroupBox {
                Text("Press Tab to move keyboard focus between controls.")
                    .frame(maxWidth: .infinity, alignment: .leading)
            } label: {
                Text("Keyboard")
            }
            .groupBoxStyle(OutlinedGroupBoxStyle())
            Note("Your settings apply to this workspace.")
        }
        .frame(maxWidth: .infinity)
        .accessibilityIdentifier("group-box-keyboard")
    }
}

private struct OutlinedGroupBoxStyle: GroupBoxStyle {
    func makeBody(configuration: Configuration) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            configuration.label.fontWeight(.medium)
            configuration.content
                .padding(10)
                .frame(maxWidth: .infinity, alignment: .leading)
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .strokeBorder(DisplayColors.border)
                )
        }
    }
}

// MARK: - List (extended.rs "list")

private struct ListPage: View {
    private let rows = ["Project brief", "Meeting notes", "Design references"]
    @State private var selected: Int? = 0

    var body: some View {
        List(selection: $selected) {
            ForEach(rows.indices, id: \.self) { index in
                Label(rows[index], systemImage: "doc")
                    .listRowSeparator(.hidden)
                    .tag(index)
                    .accessibilityIdentifier("list-item-\(index)")
            }
        }
        .listStyle(.inset)
        .scrollDisabled(true)
        // 30 rem wide; three 1.75 rem rows with 0.125 rem gaps.
        .frame(width: 480, height: 96)
        .accessibilityIdentifier("list")
        Note("Selected document \((selected ?? 0) + 1)")
            .accessibilityIdentifier("list-selection")
    }
}

// MARK: - Virtual list / Scrollbar (render_activity_list, render_horizontal_scrollbar)

private struct ActivityLogPage: View {
    /// The scrollbar page shows the same log with always-visible scrollers
    /// plus a horizontal timeline.
    let showsTimeline: Bool

    var body: some View {
        ScrollViewReader { proxy in
          VStack(alignment: .leading, spacing: GalleryMetrics.contentSpacing) {
            Text("Sample activity · 1,000 events")
            HStack(spacing: 8) {
                Button("First event") { proxy.scrollTo(0, anchor: .top) }
                    .accessibilityIdentifier("activity-first")
                Button("Last event") { proxy.scrollTo(999, anchor: .top) }
                    .accessibilityIdentifier("activity-last")
            }
            .buttonStyle(.bordered)

            ScrollView(.vertical) {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(0..<1000, id: \.self) { index in
                        row(index).id(index)
                    }
                }
            }
            .scrollIndicators(showsTimeline ? .visible : .automatic)
            .frame(height: 280)
            .frame(maxWidth: .infinity)
            .background(Color(nsColor: .textBackgroundColor))
            .border(DisplayColors.border)
            .accessibilityIdentifier("activity-list")
          }
        }
        Note("Scroll through the log or jump to either end. Longer events keep their full description.")

        if showsTimeline {
            Text("Project timeline · Scroll horizontally")
            ScrollView(.horizontal) {
                HStack(spacing: 0) {
                    ForEach(Array(timeline.enumerated()), id: \.offset) { index, name in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(name)
                            Text("Week \(index + 1)").foregroundStyle(.secondary)
                        }
                        .padding(10)
                        .frame(width: 180, height: 64, alignment: .topLeading)
                        .overlay(alignment: .trailing) {
                            Rectangle().fill(DisplayColors.divider).frame(width: 1)
                        }
                    }
                }
                .frame(width: 1440, height: 64, alignment: .leading)
            }
            .scrollIndicators(.visible)
            .frame(height: 76)
            .frame(maxWidth: .infinity)
            .border(DisplayColors.border)
            .accessibilityIdentifier("timeline-scroll")
        }
    }

    private let timeline = ["Brief", "Research", "Design", "Prototype", "Review", "Build", "Test", "Launch"]

    /// Rows are 1.75 rem, or 2.75 rem when every fifth event adds a reviewer line.
    private func row(_ index: Int) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(String(format: "Event %04d · Updated project notes", index + 1))
            if index % 5 == 0 {
                Text("Review requested by Alex Lee").foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .frame(height: index % 5 == 0 ? 44 : 28)
        .accessibilityIdentifier("activity-row-\(index)")
    }
}

// MARK: - Scrollable (extended.rs "scrollable")

private struct ScrollablePage: View {
    var body: some View {
        ScrollView(.vertical) {
            LazyVStack(alignment: .leading, spacing: 0) {
                ForEach(1...60, id: \.self) { index in
                    Text(String(format: "Activity %02d — Document saved to iCloud", index))
                        .padding(.vertical, 10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .overlay(alignment: .bottom) {
                            Rectangle().fill(DisplayColors.divider).frame(height: 1)
                        }
                }
            }
            .padding(16)
        }
        // 32 x 22 rem, 0.75 rem corner radius.
        .frame(width: 512, height: 352)
        .background(DisplayColors.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(DisplayColors.border))
        .accessibilityIdentifier("scrollable-preview")
    }
}

// MARK: - Data table (extended.rs "data_table", FilesTable)

private struct DataTablePage: View {
    private struct FileRow: Identifiable {
        let id: Int
        let name: String
        let location: String
        let size: String
    }

    @State private var rows: [FileRow] = (0..<120).map { index in
        FileRow(
            id: index,
            name: String(format: "Document %03d", index + 1),
            location: index % 3 == 0 ? "iCloud" : "On My Mac",
            size: "\(24 + index * 8) KB"
        )
    }
    /// The gallery starts unsorted.
    @State private var sortOrder: [KeyPathComparator<FileRow>] = []

    var body: some View {
        Table(rows, sortOrder: $sortOrder) {
            TableColumn("Name", value: \.name).width(260)
            TableColumn("Location") { Text($0.location) }.width(160)
            // The gallery sorts sizes as strings; keep that order.
            TableColumn("Size", value: \.size).width(120)
        }
        .tableStyle(.bordered(alternatesRowBackgrounds: true))
        .onChange(of: sortOrder) { _, order in rows.sort(using: order) }
        .frame(maxWidth: .infinity)
        .frame(height: 400)
        .accessibilityIdentifier("data-table")
    }
}

// MARK: - Tree (render_tree_page)

private struct TreePage: View {
    private struct Node: Identifiable {
        let id: String
        let label: String
        var children: [Node] = []
        var disabled = false
    }

    private let nodes = [
        Node(id: "documents", label: "Documents", children: [
            Node(id: "brief", label: "Project brief.md"),
            Node(id: "notes", label: "Meeting notes.md"),
        ]),
        Node(id: "projects", label: "Projects", children: [
            Node(id: "website", label: "Website", children: [
                Node(id: "homepage", label: "Homepage.md"),
                Node(id: "assets", label: "Assets.md"),
            ]),
        ]),
        Node(id: "archive", label: "Archive (unavailable)", disabled: true),
    ]

    @State private var selection: String?
    /// Documents and Projects start expanded; Website starts collapsed.
    @State private var expanded: Set<String> = ["documents", "projects"]

    var body: some View {
        Text("Workspace files")
        List(selection: $selection) {
            ForEach(nodes) { node in
                TreeRow(node: node, expanded: $expanded)
            }
            .listRowSeparator(.hidden)
        }
        .listStyle(.inset)
        // 17.5 rem tall, at most 27.5 rem wide.
        .frame(maxWidth: 440)
        .frame(height: 280)
        .accessibilityIdentifier("tree")
        Text(selectedLabel.map { "Selected: \($0)" } ?? "Select a file or folder")
            .foregroundStyle(.secondary)
            .accessibilityIdentifier("tree-selection")
        Text("Arrow keys navigate · Left / Right collapse and expand")
            .foregroundStyle(.secondary)
    }

    private var selectedLabel: String? {
        guard let selection else { return nil }
        func find(_ nodes: [Node]) -> String? {
            for node in nodes {
                if node.id == selection { return node.label }
                if let label = find(node.children) { return label }
            }
            return nil
        }
        return find(nodes)
    }

    private struct TreeRow: View {
        let node: Node
        @Binding var expanded: Set<String>

        var body: some View {
            if node.children.isEmpty {
                Text(node.label)
                    .tag(node.id)
                    .selectionDisabled(node.disabled)
                    .disabled(node.disabled)
                    .accessibilityIdentifier("tree-\(node.id)")
            } else {
                DisclosureGroup(isExpanded: binding) {
                    ForEach(node.children) { child in
                        TreeRow(node: child, expanded: $expanded)
                    }
                    .listRowSeparator(.hidden)
                } label: {
                    Text(node.label)
                        .accessibilityIdentifier("tree-\(node.id)")
                }
                .tag(node.id)
            }
        }

        private var binding: Binding<Bool> {
            Binding(
                get: { expanded.contains(node.id) },
                set: { isExpanded in
                    if isExpanded { expanded.insert(node.id) } else { expanded.remove(node.id) }
                }
            )
        }
    }
}

// MARK: - Resizable (render_resizable_example)

private struct ResizablePage: View {
    var body: some View {
        Text("Horizontal")
        HSplitView {
            VStack(alignment: .leading, spacing: 10) {
                Text("Documents")
                Text("Project brief.md")
                Text("Meeting notes.md")
            }
            .padding(14)
            // Gallery: 13.75 rem initial, 8.75 to 26.25 rem. HSplitView ignores
            // ideal sizes, so the 13.75 rem start doubles as the minimum.
            .frame(minWidth: 220, idealWidth: 220, maxWidth: 420, maxHeight: .infinity, alignment: .topLeading)
            VStack(alignment: .leading, spacing: 14) {
                Text("Project brief").bold()
                Text("A focused desktop workspace for files, notes and project activity.")
                Text("Drag the divider to give this preview more room.").foregroundStyle(.secondary)
            }
            .padding(14)
            .frame(minWidth: 180, maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .layoutPriority(1)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 300)
        .border(DisplayColors.border)
        .accessibilityIdentifier("workspace-panes")

        Text("Vertical")
        VSplitView {
            VStack(alignment: .leading, spacing: 10) {
                Text("Preview")
                Text("The workspace is ready for review.")
            }
            .padding(14)
            // Gallery: 8.75 rem initial, 5 to 11.25 rem. VSplitView ignores
            // ideal sizes, so the 8.75 rem start doubles as the minimum.
            .frame(maxWidth: .infinity, minHeight: 140, idealHeight: 140, maxHeight: 180, alignment: .topLeading)
            VStack(alignment: .leading, spacing: 10) {
                Text("Activity")
                Text("All changes saved · No pending tasks").foregroundStyle(.secondary)
            }
            .padding(14)
            // 3.75 to 10 rem.
            .frame(maxWidth: .infinity, minHeight: 60, maxHeight: 160, alignment: .topLeading)
            .layoutPriority(1)
        }
        .frame(maxWidth: .infinity)
        .frame(height: 240)
        .border(DisplayColors.border)
        .accessibilityIdentifier("preview-console")
    }
}

// MARK: - Separator (render_separator_page)

private struct SeparatorPage: View {
    @State private var count = 0

    var body: some View {
        Text("Horizontal").foregroundStyle(.secondary)
        Text("Appearance")
        Divider()
            .accessibilityIdentifier("separator-horizontal")
        Text("Keyboard")
        Text("Vertical")
            .foregroundStyle(.secondary)
            .padding(.top, 14)
        HStack(spacing: 10) {
            Button("Add") { count += 1 }
                .accessibilityIdentifier("separator-add")
            // vertical_separator: 1.25 rem tall.
            Divider().frame(height: 20)
            Button("Reset") { count = 0 }
                .accessibilityIdentifier("separator-reset")
            Divider().frame(height: 20)
            Text("\(count) items")
                .accessibilityIdentifier("separator-count")
        }
        .buttonStyle(.bordered)
    }
}

// MARK: - Empty state (render_empty_state_page)

private struct EmptyStatePage: View {
    @State private var created = false

    var body: some View {
        if created {
            Text("Workspace created")
            Button("Reset example") { created = false }
                .buttonStyle(.bordered)
                .accessibilityIdentifier("reset-empty")
        } else {
            // The gallery's empty state has no icon: title, description, action.
            // Title and description use the gallery's 13 pt body sizes.
            ContentUnavailableView {
                Text("No workspaces")
                    .font(.system(size: GalleryMetrics.bodyFontSize, weight: .semibold))
                    .foregroundStyle(.primary)
            } description: {
                Text("Create a workspace to collect your projects.")
                    .font(.system(size: GalleryMetrics.bodyFontSize))
            } actions: {
                Button("Create workspace") { created = true }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier("create")
            }
            .accessibilityIdentifier("empty-state")
        }
    }
}

// MARK: - Progress (render_progress_page)

private struct ProgressPage: View {
    /// Gallery initial value: 30 %.
    @State private var progress = 30.0

    var body: some View {
        Text("Progress: \(Int(progress.rounded()))%")
            .accessibilityIdentifier("progress-label")
        ProgressView(value: progress, total: 100)
            .progressViewStyle(.linear)
            .frame(maxWidth: .infinity)
            .accessibilityIdentifier("progress")
        HStack(spacing: 8) {
            Button("Advance 10%") { progress = min(progress + 10, 100) }
                .buttonStyle(.borderedProminent)
                .accessibilityIdentifier("advance")
            Button("Reset") { progress = 0 }
                .buttonStyle(.bordered)
                .accessibilityIdentifier("reset")
        }
    }
}

// MARK: - Spinner (extended.rs "spinner")

private struct SpinnerPage: View {
    var body: some View {
        // Indeterminate spinners animate; the rotating glyphs are not scorable.
        HStack(spacing: 12) {
            ProgressView()
                .progressViewStyle(.circular)
                .controlSize(.large)
                .accessibilityIdentifier("spinner-large")
            Text("Syncing with iCloud…")
        }
        HStack(spacing: 12) {
            ProgressView()
                .progressViewStyle(.circular)
                .controlSize(.small)
                .accessibilityIdentifier("spinner-small")
            Text("Loading preview")
        }
    }
}

// MARK: - Rating (extended.rs "rating")

private struct RatingPage: View {
    /// Gallery initial value: 4.
    @State private var rating = 4

    var body: some View {
        RatingIndicator(value: $rating, editable: true)
            .accessibilityIdentifier("rating-edit")
        Note("Your rating: \(rating) of 5")
            .accessibilityIdentifier("rating-value")
        RatingIndicator(value: .constant(3), editable: false)
            .accessibilityIdentifier("rating-readonly")
    }
}

/// SwiftUI has no rating control; AppKit's NSLevelIndicator rating style is
/// the system equivalent.
private struct RatingIndicator: NSViewRepresentable {
    @Binding var value: Int
    let editable: Bool

    func makeCoordinator() -> Coordinator { Coordinator(value: $value) }

    func makeNSView(context: Context) -> NSLevelIndicator {
        let indicator = NSLevelIndicator()
        indicator.levelIndicatorStyle = .rating
        indicator.minValue = 0
        indicator.maxValue = 5
        indicator.isEditable = editable
        // The read-only gallery rating is disabled.
        indicator.isEnabled = editable
        indicator.target = context.coordinator
        indicator.action = #selector(Coordinator.changed(_:))
        indicator.integerValue = value
        return indicator
    }

    func updateNSView(_ indicator: NSLevelIndicator, context: Context) {
        context.coordinator.value = $value
        if indicator.integerValue != value { indicator.integerValue = value }
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSLevelIndicator, context: Context) -> CGSize? {
        nsView.fittingSize
    }

    @MainActor final class Coordinator: NSObject {
        var value: Binding<Int>
        init(value: Binding<Int>) { self.value = value }
        @objc func changed(_ sender: NSLevelIndicator) {
            value.wrappedValue = sender.integerValue
        }
    }
}

// MARK: - Image (extended.rs "image", DOCUMENT_PREVIEWS)

private struct ImagePage: View {
    private static let previews: [(title: String, detail: String, asset: String)] = [
        ("Project brief.pdf", "2 pages · Shared with Studio", "project-brief.svg"),
        ("Weekend itinerary.pdf", "1 page · On My Mac", "weekend-itinerary.svg"),
        ("Quarterly budget.pdf", "3 pages · iCloud Drive", "quarterly-budget.svg"),
    ]

    /// The gallery embeds examples/gallery/assets/*.svg; the build script
    /// copies the same files into the bundle's Resources/GalleryAssets.
    private static let assetDirectory = Bundle.main.resourceURL!
        .appendingPathComponent("GalleryAssets")

    var body: some View {
        // 19 rem cards with 1 rem gaps wrap two per row in the 47.5 rem column.
        LazyVGrid(
            columns: [GridItem(.fixed(304), spacing: 16), GridItem(.fixed(304), spacing: 16)],
            alignment: .leading,
            spacing: 16
        ) {
            ForEach(Array(Self.previews.enumerated()), id: \.offset) { index, preview in
                card(index: index, preview: preview)
            }
        }
        Note("These original document illustrations load through the real image API. Asset identifiers, local files, and remote sources use the same component.")
    }

    private func card(index: Int, preview: (title: String, detail: String, asset: String)) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Group {
                if let image = NSImage(contentsOf: Self.assetDirectory.appendingPathComponent(preview.asset)) {
                    Image(nsImage: image)
                        .resizable()
                        .scaledToFit()
                } else {
                    Image(systemName: "photo").foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity)
            .frame(height: 176)
            .accessibilityIdentifier("document-thumbnail-image-\(index)")
            .padding(8)
            .frame(width: 304, height: 208)
            .background(DisplayColors.normalFill, in: RoundedRectangle(cornerRadius: 8))
            .accessibilityIdentifier("document-thumbnail-pane-\(index)")
            Text(preview.title).fontWeight(.medium)
            Note(preview.detail)
        }
        .frame(width: 304, alignment: .leading)
    }
}
