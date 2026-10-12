import AppKit
import SwiftUI

// Overlays, menu, material and text pages (text_view, label, keycap, panel).
// Each page mirrors its gallery render function (examples/gallery/).
// With `--open` (LaunchOptions.open) popup pages present their primary popup.

extension PageContent {
    func overlaysTextPage(_ slug: String) -> AnyView? {
        switch slug {
        case "menu": AnyView(MenuPage())
        case "popover": AnyView(PopoverPage())
        case "tooltip": AnyView(TooltipPage())
        case "dialog": AnyView(DialogPage(alert: false))
        case "alert_dialog": AnyView(DialogPage(alert: true))
        case "sheet": AnyView(SheetPage())
        case "material": AnyView(MaterialPage())
        case "text_view": AnyView(TextViewPage())
        case "label": AnyView(LabelPage())
        case "keycap": AnyView(KeycapPage())
        case "panel": AnyView(PanelPage())
        default: nil
        }
    }
}

// MARK: - Shared

private enum OverlayMetrics {
    /// Preview cards: 22.5 rem wide, 0.875 rem padding.
    static let cardWidth: CGFloat = 360
    static let cardPadding: CGFloat = 14
    /// Popover surface: 17.5 rem including its padding.
    static let popoverWidth: CGFloat = 280
    /// Popover padding: SwiftUI's `.padding()` (spec C26).
    static let popoverPadding: CGFloat = 16
    /// Dialog popup: 28.75 rem wide, 1.5 rem padding, 1.25 rem radius.
    static let dialogWidth: CGFloat = 460
    static let dialogPadding: CGFloat = 24
    /// Dialog footer buttons: min width 5 rem.
    static let dialogButtonWidth: CGFloat = 80
    /// Settle time before presenting a launch popup, so the window is key.
    static let launchDelay: Duration = .milliseconds(400)
}

/// The gallery's `normal_fill`: a faint foreground wash.
private let normalFill = Color.primary.opacity(0.05)

/// ToolbarTitle text style: 15 pt semibold (dialog and panel titles).
private let titleFont = Font.system(size: 15, weight: .semibold)

/// Presents `isPresented` once after launch when `--open` was passed.
private struct LaunchPresentation: ViewModifier {
    @Binding var isPresented: Bool

    func body(content: Content) -> some View {
        content.task {
            guard LaunchOptions.current.open, !isPresented else { return }
            try? await Task.sleep(for: OverlayMetrics.launchDelay)
            var transaction = Transaction()
            transaction.disablesAnimations = true
            withTransaction(transaction) { isPresented = true }
        }
    }
}

private extension View {
    func presentsOnLaunch(_ isPresented: Binding<Bool>) -> some View {
        modifier(LaunchPresentation(isPresented: isPresented))
    }

    /// A gallery dialog button: at least 5 rem wide.
    func dialogButtonWidth() -> some View {
        frame(minWidth: OverlayMetrics.dialogButtonWidth - 16)
    }
}

/// AppKit help tags cannot be shown programmatically, so this mirrors the
/// measured one (spec C27): 11 pt tooltip font in the label colour, 20 pt
/// tall, radius 5 and no border, on the tooltip's own material (opaque
/// under Reduce Transparency, like the library).
private struct HelpTag: View {
    let text: String
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

    /// The tooltip's own material; the same tint opaque under Reduce Transparency.
    private var fill: Color {
        let dark = colorScheme == .dark
        let tint: Double = dark ? 0x3A / 255 : 0xE7 / 255
        let alpha = reduceTransparency ? 1 : (dark ? 0.828 : 0.898)
        return Color(red: tint, green: tint, blue: tint).opacity(alpha)
    }

    var body: some View {
        Text(text)
            .font(Font(NSFont.toolTipsFont(ofSize: 0)))
            .foregroundStyle(Color(nsColor: .labelColor))
            .padding(.horizontal, 10)
            .frame(height: 20)
            .background(fill, in: RoundedRectangle(cornerRadius: 5, style: .circular))
            .shadow(color: .black.opacity(0.12), radius: 2, y: 1)
            .fixedSize()
    }
}

// MARK: - Menu (render_menu_example)

private struct MenuPage: View {
    var body: some View {
        Menu {
            Button {} label: { Label("New workspace", systemImage: "plus") }
            Button {} label: { Label("Favorite workspace", systemImage: "star") }
            Menu("Export") {
                Button("Export as PDF") {}
                Button("Export as Markdown") {}
            }
            Button("Unavailable action") {}.disabled(true)
        } label: {
            Text("Workspace actions")
        }
        .menuStyle(.button)
        .fixedSize()
        .background(MenuLauncher(open: LaunchOptions.current.open))
        .accessibilityIdentifier("menu-trigger")
        Text("No action selected")
            .accessibilityIdentifier("menu-result")
    }
}

/// SwiftUI cannot open a `Menu` programmatically. With `--open` this pops up
/// the same items as an `NSMenu` (what SwiftUI's pull-down uses) just below
/// the trigger it backs.
private struct MenuLauncher: NSViewRepresentable {
    let open: Bool

    func makeNSView(context: Context) -> LauncherView { LauncherView(open: open) }
    func updateNSView(_ nsView: LauncherView, context: Context) {}

    final class LauncherView: NSView {
        private var pending: Bool

        init(open: Bool) {
            pending = open
            super.init(frame: .zero)
        }

        required init?(coder: NSCoder) { nil }

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard pending, window != nil else { return }
            pending = false
            Task { @MainActor [weak self] in
                try? await Task.sleep(for: OverlayMetrics.launchDelay)
                self?.popUp()
            }
        }

        private func popUp() {
            guard window != nil else { return }
            let menu = NSMenu()
            menu.autoenablesItems = false
            func item(_ title: String, symbol: String? = nil) -> NSMenuItem {
                let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                if let symbol {
                    item.image = NSImage(systemSymbolName: symbol, accessibilityDescription: nil)
                }
                return item
            }
            menu.addItem(item("New workspace", symbol: "plus"))
            menu.addItem(item("Favorite workspace", symbol: "star"))
            let export = item("Export")
            let submenu = NSMenu(title: "Export")
            submenu.addItem(item("Export as PDF"))
            submenu.addItem(item("Export as Markdown"))
            export.submenu = submenu
            menu.addItem(export)
            let unavailable = item("Unavailable action")
            unavailable.isEnabled = false
            menu.addItem(unavailable)
            let below = NSPoint(x: 0, y: isFlipped ? bounds.height + 4 : -4)
            menu.popUp(positioning: nil, at: below, in: self)
        }
    }
}

// MARK: - Popover (render_popover_example)

private struct PopoverPage: View {
    @State private var showHidden = true
    @State private var sync = true
    @State private var shown = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Personal workspace")
            Text("Documents")
            Text("Projects")
            if showHidden {
                Text(".config").foregroundStyle(.secondary)
            }
        }
        .padding(OverlayMetrics.cardPadding)
        .frame(width: OverlayMetrics.cardWidth, alignment: .leading)
        .background(normalFill)

        Button { shown.toggle() } label: {
            HStack(spacing: 4) {
                Text("Display options")
                Image(systemName: "chevron.down").imageScale(.small)
            }
        }
        .popover(isPresented: $shown, arrowEdge: .bottom) {
            VStack(alignment: .leading, spacing: 14) {
                Text("Display options").bold()
                Toggle("Show hidden files", isOn: $showHidden)
                    .toggleStyle(.checkbox)
                    .accessibilityIdentifier("popover-hidden")
                HStack(spacing: 8) {
                    Toggle("Sync workspace", isOn: $sync)
                        .toggleStyle(.switch)
                        .labelsHidden()
                        .accessibilityIdentifier("popover-sync")
                    Text("Sync workspace")
                }
            }
            .padding(OverlayMetrics.popoverPadding)
            .frame(width: OverlayMetrics.popoverWidth, alignment: .leading)
        }
        .presentsOnLaunch($shown)
        .accessibilityIdentifier("display-options-trigger")

        Text(sync ? "Workspace sync is enabled." : "Workspace sync is paused.")
            .foregroundStyle(.secondary)
    }
}

// MARK: - Tooltip (render_tooltip_example)

private struct TooltipPage: View {
    @State private var pressed = false
    @State private var buttonHeight: CGFloat = 0

    private var action: String { pressed ? "Remove from favorites" : "Add to favorites" }

    var body: some View {
        HStack(spacing: 14) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Personal workspace")
                Text(pressed ? "In favorites" : "Not in favorites").foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Button { pressed.toggle() } label: {
                Image(systemName: pressed ? "star.fill" : "star")
            }
            .help(action)
            .accessibilityLabel(action)
            .accessibilityIdentifier("tooltip-favorite")
            // `--open`: the help tag the star shows on hover, pinned below it.
            .onGeometryChange(for: CGFloat.self) { $0.size.height } action: { buttonHeight = $0 }
            .overlay(alignment: .topLeading) {
                if LaunchOptions.current.open {
                    HelpTag(text: action)
                        .offset(y: buttonHeight + 4)
                        .accessibilityIdentifier("launch-tooltip")
                }
            }
        }
        .padding(OverlayMetrics.cardPadding)
        .frame(width: OverlayMetrics.cardWidth)
        .background(normalFill)
        .zIndex(1)

        Text("Hover the star for its action. Tab and Return also activate it.")
            .foregroundStyle(.secondary)
        Text("Tooltip appearance")
            .padding(.top, 14)
        HelpTag(text: "Add to favorites")
    }
}

// MARK: - Dialog and alert dialog (render_dialog_page)

private struct DialogPage: View {
    let alert: Bool
    @State private var presented = false
    @State private var draft = "Personal workspace"

    var body: some View {
        Text(alert ? "Destructive confirmation" : "Form dialog")
            .font(.system(size: 13, weight: .bold))
        Text(alert
            ? "A named consequence, a clear way back, and a distinct destructive action."
            : "A short task with visible labels and outline actions.")
            .foregroundStyle(.secondary)

        DialogCard {
            if alert { ResetForm {} } else { WorkspaceForm(name: $draft) {} }
        }

        HStack(spacing: 12) {
            Button(alert ? "Open alert dialog…" : "Open dialog…") { presented = true }
                .accessibilityIdentifier("open-modal")
                .dialogButtonWidth()
            Text(alert
                ? "Escape cancels · Backdrop keeps the dialog open"
                : "Escape cancels · Tab moves between controls")
                .foregroundStyle(.secondary)
        }
        .padding(.top, 6)
        .modifier(DialogPresentation(alert: alert, presented: $presented, draft: $draft))
        .presentsOnLaunch($presented)
    }
}

/// Form dialog: a window-modal sheet. Alert dialog: a system alert with a
/// destructive action (SwiftUI shows it as an alert sheet on the window).
private struct DialogPresentation: ViewModifier {
    let alert: Bool
    @Binding var presented: Bool
    @Binding var draft: String

    func body(content: Content) -> some View {
        if alert {
            content.alert("Reset workspace settings?", isPresented: $presented) {
                Button("Reset", role: .destructive) {}
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("The custom name “Personal workspace” will be replaced with “Personal workspace”. Your files will stay in place.")
            }
        } else {
            content.sheet(isPresented: $presented) {
                WorkspaceForm(name: $draft) { presented = false }
                    .padding(OverlayMetrics.dialogPadding)
                    .frame(width: OverlayMetrics.dialogWidth)
            }
        }
    }
}

/// The gallery's inline dialog specimen surface (dialog_popup).
private struct DialogCard<Content: View>: View {
    @ViewBuilder let content: Content

    var body: some View {
        content
            .padding(OverlayMetrics.dialogPadding)
            .frame(width: OverlayMetrics.dialogWidth, alignment: .leading)
            .background(Color(nsColor: .windowBackgroundColor), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 20, style: .continuous)
                    .strokeBorder(Color(nsColor: .separatorColor), lineWidth: 1)
            )
            .shadow(color: .black.opacity(0.12), radius: 12, y: 4)
    }
}

private struct WorkspaceForm: View {
    @Binding var name: String
    let close: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Workspace settings").font(titleFont)
                Text("Give this workspace a name you can recognize.").foregroundStyle(.secondary)
            }
            VStack(alignment: .leading, spacing: 6) {
                Text("Name")
                TextField("Name", text: $name)
                    .labelsHidden()
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("specimen-name")
                Text("Shown in the workspace switcher.")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
            HStack(spacing: 8) {
                Image(systemName: "gearshape")
                Text("Current workspace: Personal workspace")
            }
            .foregroundStyle(.secondary)
            .padding(.vertical, 8)
            Divider()
            HStack(spacing: 8) {
                Spacer()
                Button("Cancel", action: close).dialogButtonWidth()
                Button("Save", action: close)
                    .buttonStyle(.borderedProminent)
                    .dialogButtonWidth()
            }
        }
    }
}

private struct ResetForm: View {
    let close: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack(spacing: 10) {
                Image(systemName: "exclamationmark.triangle")
                    .font(.system(size: 20))
                    .foregroundStyle(Color(nsColor: .systemRed))
                Text("Reset workspace settings?").font(titleFont)
            }
            Text("The custom name “Personal workspace” will be replaced with “Personal workspace”. Your files will stay in place.")
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            HStack(spacing: 8) {
                Spacer()
                Button("Cancel", action: close).dialogButtonWidth()
                Button("Reset", role: .destructive, action: close)
                    .buttonStyle(.borderedProminent)
                    .tint(.red)
                    .dialogButtonWidth()
            }
        }
    }
}

// MARK: - Sheet (Gallery::render "sheet", render_project_sheet)

private struct SheetPage: View {
    @State private var presented = false

    var body: some View {
        Text("Website refresh · In review")
        Button("Project details…") { presented = true }
            .accessibilityIdentifier("open-project-sheet")
            .sheet(isPresented: $presented) {
                ProjectSheet { presented = false }
            }
            .presentsOnLaunch($presented)
        Text("Inspect the project without leaving this page. Close the panel or press Escape to return.")
            .foregroundStyle(.secondary)
    }
}

/// The gallery's sheet is a 22.5 rem panel inset 0.75 rem from the window's
/// right edge; a SwiftUI sheet attaches below the titlebar instead.
private struct ProjectSheet: View {
    let close: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Website refresh").font(titleFont)
            Text("Project details").foregroundStyle(.secondary)
            Divider()
            Text("In review · Due Friday")
            Text("Owner: Alex Lee")
            Text("Review the homepage layout, navigation and images before the autumn release.")
                .fixedSize(horizontal: false, vertical: true)
            Divider()
            Text("Next steps")
            Text("Check narrow windows")
            Text("Review keyboard navigation")
            Text("Approve release notes")
            Spacer(minLength: 0)
            Button("Close", action: close)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier("close-sheet")
        }
        .padding(18)
        .frame(width: 360, height: 560, alignment: .topLeading)
        .accessibilityIdentifier("project-sheet")
    }
}

// MARK: - Material (extended.rs "material")

private struct MaterialPage: View {
    @State private var shown = false

    private let tiles: [(NSVisualEffectView.Material, String)] = [
        (.sidebar, "Sidebar"),
        (.headerView, "Toolbar"),
        (.popover, "Popover"),
        (.hudWindow, "HUD"),
    ]

    var body: some View {
        // 13 x 8 rem tiles with a 1 rem gap wrap after three in the 47.5 rem column.
        VStack(alignment: .leading, spacing: 16) {
            HStack(spacing: 16) {
                ForEach(tiles.prefix(3), id: \.1) { tile($0.0, $0.1) }
            }
            HStack(spacing: 16) {
                ForEach(tiles.suffix(1), id: \.1) { tile($0.0, $0.1) }
            }
        }

        Button("Show Glass Panel") { shown.toggle() }
            .popover(isPresented: $shown, arrowEdge: .bottom) {
                VStack(alignment: .leading, spacing: 10) {
                    Text("System glass").fontWeight(.semibold)
                    Text("The foreground stays interactive while AppKit draws the material behind it.")
                    Text("The native route is available on compatible macOS hosts. Appearance and transparency preferences determine the fallback.")
                        .foregroundStyle(.secondary)
                }
                .fixedSize(horizontal: false, vertical: true)
                .padding(OverlayMetrics.popoverPadding)
                .frame(width: OverlayMetrics.popoverWidth, alignment: .leading)
            }
            .presentsOnLaunch($shown)
            .accessibilityIdentifier("material-glass-trigger")

        Text("These samples show semantic canvas fills. Native sidebar and toolbar materials use AppKit glass; menus, popovers and picker panels use separate system glass windows above the GPUI scene. Reduced transparency uses opaque fills.")
            .foregroundStyle(.secondary)
            .fixedSize(horizontal: false, vertical: true)
    }

    private func tile(_ material: NSVisualEffectView.Material, _ label: String) -> some View {
        let shape = RoundedRectangle(cornerRadius: 16, style: .continuous)
        return VStack(spacing: 8) {
            Image(systemName: "square.3.layers.3d")
                .font(.system(size: 28))
                .foregroundStyle(Color.accentColor)
            Text(label)
        }
        .frame(width: 208, height: 128)
        .background(EffectView(material: material).clipShape(shape))
        .overlay(shape.strokeBorder(Color(nsColor: .separatorColor), lineWidth: 1))
        .shadow(color: .black.opacity(0.1), radius: 4, y: 2)
        .accessibilityIdentifier("material-\(label.lowercased())")
    }
}

/// System material sampled within the window, so captures stay independent
/// of the desktop behind it.
private struct EffectView: NSViewRepresentable {
    let material: NSVisualEffectView.Material

    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = .withinWindow
        view.state = .active
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
        nsView.material = material
    }
}

// MARK: - Text view (render_text_view)

private struct TextViewPage: View {
    private let items = [
        "Review the introduction and installation steps.",
        "Verify keyboard navigation in both themes.",
        "Publish the release notes.",
    ]

    var body: some View {
        // Markdown blocks, 0.75 rem paragraph gap; headings use Title2 / Title3.
        VStack(alignment: .leading, spacing: 12) {
            Text("Website refresh").font(.title2)
            Text("A clearer home for our **project documentation**. Keep navigation simple and preserve the reading experience.")
            Text("Before launch").font(.title3)
            VStack(alignment: .leading, spacing: 4) {
                ForEach(items, id: \.self) { item in
                    HStack(alignment: .firstTextBaseline, spacing: 6) {
                        Text("•")
                        Text(item)
                    }
                }
            }
            HStack(spacing: 10) {
                Rectangle()
                    .fill(Color(nsColor: .separatorColor))
                    .frame(width: 3)
                Text("Changes should make the next step easier to understand.")
                    .foregroundStyle(.secondary)
            }
            .fixedSize(horizontal: false, vertical: true)
            Text("Run the gallery locally:")
            Text(verbatim: "cargo run --example gallery")
                .font(.system(size: 12, design: .monospaced))
                .padding(10)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(normalFill, in: RoundedRectangle(cornerRadius: 8, style: .continuous))
            Text("Read the [project source](https://github.com/longbridge/gpui-kit) for usage examples.")
        }
        .textSelection(.enabled)
        .accessibilityIdentifier("project-brief")

        Divider()

        VStack(alignment: .leading, spacing: 12) {
            Text("Release summary").font(.title3)
            Text("**Ready for review.** The documentation is complete; launch follows the final keyboard check.")
            Text("*Updated by Alex Lee*")
        }
        .textSelection(.enabled)
        .accessibilityIdentifier("release-summary")
    }
}

// MARK: - Label (extended.rs "label")

private struct LabelPage: View {
    var body: some View {
        // Label line height: 1.25 rem.
        Group {
            Text("Workspace preferences \(Text("Saved locally").foregroundStyle(.secondary))")
            Text("Find \(Text("documents").foregroundStyle(Color(nsColor: .systemBlue))) by name")
            // Masked: one bullet per character of the full text.
            Text(String(repeating: "•", count: "Password: correct-horse-battery".count))
                .accessibilityLabel("Masked password")
        }
        .frame(minHeight: 20)
    }
}

// MARK: - Keycap (render_keycap_page)

private struct KeycapPage: View {
    var body: some View {
        HStack(spacing: 8) {
            Keycap(key: "tab")
            Text("Next control")
            Keycap(key: "enter")
            Text("Activate")
        }
    }
}

/// Callout text (12 pt) in a bordered inset cap: 0.25 rem / 0.125 rem padding.
private struct Keycap: View {
    let key: String

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 4, style: .continuous)
        Text(key)
            .font(.system(size: 12))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 4)
            .padding(.vertical, 2)
            .background(Color(nsColor: .controlBackgroundColor), in: shape)
            .overlay(shape.strokeBorder(Color(nsColor: .separatorColor), lineWidth: 1))
            .accessibilityIdentifier("keycap-\(key)")
    }
}

// MARK: - Panel (render_panel_page)

private struct PanelPage: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Workspace").font(titleFont)
            Text("Theme-aware surface with a title and composable children")
            // The badge stretches across the panel's column, as in the gallery.
            Text("Ready")
                .font(.system(size: 11))
                .foregroundStyle(Color(nsColor: .systemGreen))
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(nsColor: .systemGreen).opacity(0.12), in: Capsule())
                .accessibilityIdentifier("panel-badge")
        }
        .padding(18)
        .fixedSize()
        .background(normalFill, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        .accessibilityIdentifier("panel")
    }
}
