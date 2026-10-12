import AppKit
import SwiftUI

/// Routes a page to its SwiftUI mirror. Content mirrors the gallery render
/// function named in each comment (examples/gallery/app.rs).
struct PageContent: View {
    let page: PageInfo

    var body: some View {
        switch page.slug {
        case "checkbox": CheckboxPage()
        case "switch": SwitchPage()
        case "button": ButtonPage()
        case "select": SelectPage()
        case "table": TablePage()
        default: PlaceholderPage(page: page)
        }
    }
}

// MARK: - Placeholder

private struct PlaceholderPage: View {
    let page: PageInfo

    private var categoryText: String {
        switch page.category {
        case 1: "Category 1: direct SwiftUI or system equivalent"
        case 2: "Category 2: composed from SwiftUI pieces"
        default: "Category 3: no meaningful native equivalent"
        }
    }

    var body: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 6) {
                Text(page.title).font(.headline)
                Text(categoryText).foregroundStyle(.secondary)
                Text(page.category == 3 ? "No native equivalent" : "Not yet implemented")
                    .foregroundStyle(.secondary)
                    .accessibilityIdentifier("placeholder-status")
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(6)
        }
        .accessibilityIdentifier("placeholder")
    }
}

// MARK: - Checkbox (render_checkbox_page)

private struct CheckboxPage: View {
    /// Gallery state: checked = true, mixed = false. A mixed checkbox is a
    /// SwiftUI Toggle over sources that disagree.
    @State private var sources = [true, true]

    var body: some View {
        Toggle(sources: $sources, isOn: \.self) {
            Text("Include hidden files")
        }
        .toggleStyle(.checkbox)
        .accessibilityIdentifier("check")

        Toggle("Managed by policy", isOn: .constant(true))
            .toggleStyle(.checkbox)
            .disabled(true)
            .accessibilityIdentifier("check-disabled")

        Button("Set mixed state") { sources = [true, false] }
            .accessibilityIdentifier("mixed")
    }
}

// MARK: - Switch (render_switch_page)

private struct SwitchPage: View {
    @State private var enabled = true

    var body: some View {
        // The gallery places the label after the switch; keep that order.
        HStack(spacing: 8) {
            Toggle("Show metadata", isOn: $enabled)
                .toggleStyle(.switch)
                .labelsHidden()
                .accessibilityIdentifier("switch")
            Text("Show metadata")
        }
        Text(enabled ? "Metadata visible" : "Metadata hidden")
            .accessibilityIdentifier("switch-status")
    }
}

// MARK: - Button (render_button_example)

private struct ButtonPage: View {
    @State private var count = 0

    private enum Variant: String, CaseIterable {
        case primary = "Primary", outline = "Outline", secondary = "Secondary", danger = "Danger"
        var key: String { rawValue.lowercased() }
    }

    var body: some View {
        Text("Appearance").foregroundStyle(.secondary)
        ForEach(Variant.allCases, id: \.self) { variant in
            HStack(spacing: 8) {
                // 5.625 rem label column.
                Text(variant.rawValue)
                    .foregroundStyle(.secondary)
                    .frame(width: 90, alignment: .leading)
                styled(Button("Apply") { count += 1 }, variant)
                    .accessibilityIdentifier(variant.key)
                styled(Button("Unavailable") {}, variant)
                    .disabled(true)
                    .accessibilityIdentifier("\(variant.key)-disabled")
            }
        }
        Text("Icons and actions").foregroundStyle(.secondary)
        HStack(spacing: 8) {
            Button { count += 1 } label: {
                Label("Add workspace", systemImage: "plus")
            }
            .accessibilityIdentifier("icon-action")
            Button { count += 1 } label: {
                Image(systemName: "plus")
            }
            .help("Add workspace")
            .accessibilityLabel("Add workspace")
            .accessibilityIdentifier("icon-only")
            Button {} label: {
                Image(systemName: "plus")
            }
            .disabled(true)
            .accessibilityLabel("Add workspace unavailable")
            .accessibilityIdentifier("icon-disabled")
            Button("Reset") { count = 0 }
                .accessibilityIdentifier("reset-count")
        }
        .buttonStyle(.bordered)
        Text("Activations: \(count)")
            .foregroundStyle(.secondary)
            .accessibilityIdentifier("activations")
    }

    /// Primary maps to the prominent (accent) style, Danger to a destructive
    /// prominent button. AppKit has no outline push button, so Outline and
    /// Secondary both use the standard bordered style.
    @ViewBuilder
    private func styled(_ button: Button<Text>, _ variant: Variant) -> some View {
        switch variant {
        case .primary:
            button.buttonStyle(.borderedProminent)
        case .outline, .secondary:
            button.buttonStyle(.bordered)
        case .danger:
            button.buttonStyle(.borderedProminent).tint(.red)
        }
    }
}

// MARK: - Select (render_select_page)

private struct SelectPage: View {
    private struct Choice: Identifiable {
        let id: String
        let label: String
        var disabled = false
    }

    private let options = [
        Choice(id: "personal", label: "Personal workspace"),
        Choice(id: "team", label: "Team workspace"),
        Choice(id: "archive", label: "Archived workspace", disabled: true),
        Choice(id: "sandbox", label: "Sandbox"),
    ]

    @State private var selected = "personal"
    @State private var managed = "team"

    /// 22.5 rem column.
    private let columnWidth: CGFloat = 360

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Default workspace")
            picker("Default workspace", selection: $selected)
                .accessibilityIdentifier("workspace-choice")
            Text("Archived workspaces cannot be selected.").foregroundStyle(.secondary)
            Text("Selected: \(label(selected))")
                .padding(.top, 12)
                .accessibilityIdentifier("selected-choice")
        }
        .frame(width: columnWidth, alignment: .leading)

        Divider()
            .padding(.top, 18)

        VStack(alignment: .leading, spacing: 8) {
            Text("Managed workspace")
            picker("Managed workspace", selection: $managed)
                .disabled(true)
                .accessibilityIdentifier("managed-choice")
            Text("This setting is managed by your organization.").foregroundStyle(.secondary)
        }
        .frame(width: columnWidth, alignment: .leading)
    }

    private func picker(_ title: String, selection: Binding<String>) -> some View {
        Picker(title, selection: selection) {
            ForEach(options) { option in
                Text(option.label).tag(option.id).selectionDisabled(option.disabled)
            }
        }
        .pickerStyle(.menu)
        .labelsHidden()
        // The gallery's select fills its 22.5 rem column.
        .buttonSizing(.flexible)
        .frame(maxWidth: .infinity)
    }

    private func label(_ id: String) -> String {
        options.first { $0.id == id }?.label ?? "None"
    }
}

// MARK: - Table (render_table_example)

private struct TablePage: View {
    private enum Status { case neutral, success, warning }

    private struct Project: Identifiable {
        let name: String
        let label: String
        let status: Status
        let owner: String
        let updated: String
        let files: String
        var id: String { name }
    }

    private let records = [
        Project(name: "Website refresh", label: "Active", status: .success, owner: "Alex Lee", updated: "Sep 7", files: "12 files"),
        Project(name: "Design system", label: "Active", status: .success, owner: "Morgan Kim", updated: "Sep 6", files: "28 files"),
        Project(name: "Release notes", label: "Review", status: .warning, owner: "Sam Rivera", updated: "Sep 5", files: "4 files"),
        Project(name: "Desktop client", label: "Active", status: .success, owner: "Alex Lee", updated: "Sep 4", files: "36 files"),
        Project(name: "Onboarding", label: "Review", status: .warning, owner: "Morgan Kim", updated: "Sep 3", files: "9 files"),
        Project(name: "Research archive", label: "Archived", status: .neutral, owner: "Sam Rivera", updated: "Aug 28", files: "42 files"),
        Project(name: "API reference", label: "Active", status: .success, owner: "Alex Lee", updated: "Aug 26", files: "17 files"),
        Project(name: "Brand assets", label: "Archived", status: .neutral, owner: "Morgan Kim", updated: "Aug 21", files: "24 files"),
    ]

    var body: some View {
        HStack {
            Text("Workspace projects")
            Spacer()
            Text("8 projects · Sample data").foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity)

        Table(records) {
            TableColumn("Project") { Text($0.name).lineLimit(1) }
            TableColumn("Status") { project in
                Text(project.label).foregroundStyle(color(project.status))
            }
            TableColumn("Owner", value: \.owner)
            TableColumn("Updated", value: \.updated)
            TableColumn("Files", value: \.files)
        }
        .tableStyle(.inset(alternatesRowBackgrounds: true))
        .scrollDisabled(true)
        // Table scrolls itself; inside the page ScrollView it needs a fixed
        // height that fits the header and all eight rows. 40 rem minimum width.
        .frame(minWidth: 640)
        .frame(height: 230)
        .accessibilityIdentifier("projects")
    }

    private func color(_ status: Status) -> Color {
        switch status {
        case .neutral: .secondary
        case .success: Color(nsColor: .systemGreen)
        case .warning: Color(nsColor: .systemOrange)
        }
    }
}
