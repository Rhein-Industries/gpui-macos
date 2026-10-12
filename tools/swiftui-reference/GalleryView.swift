import AppKit
import SwiftUI

/// Values read from examples/gallery/app.rs (1 rem = 16 pt).
enum GalleryMetrics {
    /// Sidebar width: 13.75 rem (9.25 rem when the window is narrower than 43.75 rem).
    static let sidebarWidth: CGFloat = 220
    static let compactSidebarWidth: CGFloat = 148
    /// Detail content padding: 1.5 rem.
    static let contentPadding: CGFloat = 24
    /// Gap between the description and each top-level child of a page: 0.875 rem.
    static let contentSpacing: CGFloat = 14
    /// Maximum content width: 47.5 rem.
    static let contentMaxWidth: CGFloat = 760
    /// Status bar: min height 1.75 rem, horizontal padding 0.875 rem, 0.6875 rem text.
    static let statusBarHeight: CGFloat = 28
    static let statusBarPadding: CGFloat = 14
    static let statusBarFontSize: CGFloat = 11
    /// Gallery body text: 0.8125 rem.
    static let bodyFontSize: CGFloat = 13
}

struct GalleryView: View {
    @State private var selection: String?
    @State private var columnVisibility = NavigationSplitViewVisibility.all

    init(initialPage: String) {
        _selection = State(initialValue: initialPage)
    }

    private var page: PageInfo {
        Catalog.page(selection ?? "overview") ?? Catalog.pages[0]
    }

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            Sidebar(selection: $selection)
                .toolbar(removing: .sidebarToggle)
                // The gallery sidebar has a fixed width (not user-resizable).
                .navigationSplitViewColumnWidth(GalleryMetrics.sidebarWidth)
        } detail: {
            DetailView(page: page)
                .navigationTitle(page.title)
        }
        .toolbar {
            // Same native toolbar items as the gallery, in the same order.
            ToolbarItem(placement: .primaryAction) {
                Button {
                    columnVisibility = columnVisibility == .detailOnly ? .all : .detailOnly
                } label: {
                    Label("Toggle Sidebar", systemImage: "sidebar.left")
                }
                .help("Toggle Sidebar")
                .accessibilityIdentifier("toggle-sidebar")
            }
            ToolbarSpacer(.fixed, placement: .primaryAction)
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    cycleAppearance()
                } label: {
                    Label("Change appearance", systemImage: "circle.lefthalf.filled")
                }
                .help("Change appearance")
                .accessibilityIdentifier("theme")
                // Zoom is not mirrored; the buttons keep the gallery's 100 % enabled states.
                Button {} label: { Label("Zoom Out", systemImage: "minus.magnifyingglass") }
                    .help("Zoom Out")
                    .accessibilityIdentifier("zoom-out")
                Button {} label: { Label("Actual Size", systemImage: "1.magnifyingglass") }
                    .help("Actual Size")
                    .disabled(true)
                    .accessibilityIdentifier("actual-size")
                Button {} label: { Label("Zoom In", systemImage: "plus.magnifyingglass") }
                    .help("Zoom In")
                    .accessibilityIdentifier("zoom-in")
            }
        }
    }

    /// Mirrors the gallery's theme command: System -> Dark -> Light -> System.
    private func cycleAppearance() {
        switch NSApp.appearance?.name {
        case nil: NSApp.appearance = NSAppearance(named: .darkAqua)
        case .some(.darkAqua): NSApp.appearance = NSAppearance(named: .aqua)
        default: NSApp.appearance = nil
        }
    }
}

private struct Sidebar: View {
    @Binding var selection: String?
    @State private var expanded: Set<String> = Set(Catalog.groups.map(\.title))
    /// The gallery starts with keyboard focus in its navigation list, so the
    /// selected row shows the active accent fill rather than a focused control.
    @FocusState private var focused: Bool

    var body: some View {
        List(selection: $selection) {
            ForEach(Catalog.groups) { group in
                Section(group.title, isExpanded: expandedBinding(group.title)) {
                    ForEach(group.pages) { page in
                        Label(page.title, systemImage: page.symbol)
                            .tag(page.slug)
                            .accessibilityIdentifier("navigation-row-\(page.slug)")
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .focused($focused)
        .onAppear { focused = true }
        .accessibilityIdentifier("component-sidebar")
    }

    private func expandedBinding(_ title: String) -> Binding<Bool> {
        Binding(
            get: { expanded.contains(title) },
            set: { isExpanded in
                if isExpanded { expanded.insert(title) } else { expanded.remove(title) }
            }
        )
    }
}

private struct DetailView: View {
    let page: PageInfo

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: GalleryMetrics.contentSpacing) {
                if page.slug != "overview" {
                    Text(page.description)
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier("page-description")
                }
                PageContent(page: page)
            }
            .frame(maxWidth: GalleryMetrics.contentMaxWidth, alignment: .leading)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(GalleryMetrics.contentPadding)
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("page-\(page.slug)")
        }
        .font(.system(size: GalleryMetrics.bodyFontSize))
        .safeAreaInset(edge: .bottom, spacing: 0) {
            StatusBar(page: page)
        }
        .id(page.slug)
    }
}

private struct StatusBar: View {
    let page: PageInfo

    var body: some View {
        VStack(spacing: 0) {
            Divider()
            HStack(spacing: 10) {
                Text("\(page.title) · \(Catalog.pages.count - 1) components")
                    .lineLimit(1)
                    .truncationMode(.tail)
                Spacer(minLength: 0)
                Link("Documentation", destination: URL(string: "https://gpui-kit.com/component/")!)
                    .foregroundStyle(.secondary)
                    .tint(.secondary)
                    .accessibilityIdentifier("gallery-repository")
            }
            .font(.system(size: GalleryMetrics.statusBarFontSize))
            .foregroundStyle(.secondary)
            .padding(.horizontal, GalleryMetrics.statusBarPadding)
            .padding(.vertical, 4)
            .frame(minHeight: GalleryMetrics.statusBarHeight)
        }
        .background(.background)
        .accessibilityIdentifier("status-bar")
    }
}
