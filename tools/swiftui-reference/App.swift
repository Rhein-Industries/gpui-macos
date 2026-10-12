import AppKit
import SwiftUI

// A native SwiftUI mirror of the GPUI gallery (examples/gallery) for pixel
// comparison. Every pilot page is built from real SwiftUI system controls.

/// Command-line contract shared with the GPUI gallery: page, appearance, display, open.
struct LaunchOptions: Sendable {
    enum Appearance: Sendable { case system, light, dark }

    let page: String
    let appearance: Appearance
    let displayID: UInt32?
    /// `--open`: the page presents its primary popup on launch.
    let open: Bool

    static let current = LaunchOptions(arguments: Array(CommandLine.arguments.dropFirst()))

    init(arguments: [String]) {
        var page: String?
        var appearance = Appearance.system
        var displayID: UInt32?
        var open = false
        var index = 0
        while index < arguments.count {
            let argument = arguments[index]
            switch argument {
            case "--light": appearance = .light
            case "--dark": appearance = .dark
            case "--open": open = true
            case "--display":
                displayID = index + 1 < arguments.count ? UInt32(arguments[index + 1]) : nil
                if index + 1 < arguments.count { index += 1 }
            case "--page":
                if index + 1 < arguments.count, Catalog.page(arguments[index + 1]) != nil {
                    page = arguments[index + 1]
                    index += 1
                }
            default:
                // Like the gallery: the first argument naming a page selects it.
                // Anything else (e.g. -NSDocumentRevisionsDebugMode YES) is ignored.
                if page == nil, Catalog.page(argument) != nil { page = argument }
            }
            index += 1
        }
        self.page = page ?? "overview"
        self.appearance = appearance
        self.displayID = displayID
        self.open = open
    }
}

/// Full gallery window frame (including titlebar): 1060 x 760 pt.
enum WindowMetrics {
    static let size = CGSize(width: 1060, height: 760)
    static let minimumSize = CGSize(width: 680, height: 520)
}

/// Cooperative `NSApp.activate()` is ignored when the process is started
/// directly by a script; the gallery forces activation, so this does too.
/// Calling through a protocol avoids the deprecation warning.
@MainActor
private protocol LegacyActivation {
    func activate(ignoringOtherApps flag: Bool)
}

extension NSApplication: LegacyActivation {}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationWillFinishLaunching(_ notification: Notification) {
        switch LaunchOptions.current.appearance {
        case .system: NSApp.appearance = nil
        case .light: NSApp.appearance = NSAppearance(named: .aqua)
        case .dark: NSApp.appearance = NSAppearance(named: .darkAqua)
        }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Like the gallery (cx.activate(true)): come to the front even when
        // launched directly from a terminal or a capture script.
        NSApp.setActivationPolicy(.regular)
        (NSApp as LegacyActivation).activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
}

/// Entry point for the reference gallery.
@main
enum ReferenceEntry {
    @MainActor
    static func main() {
        #if SWIFTUI_PROBE
        if CommandLine.arguments.contains("--rt-watchdog") {
            exit(ReduceTransparency.watchdog(arguments: CommandLine.arguments))
        }
        if CommandLine.arguments.contains("--probe-pass2") || CommandLine.arguments.contains("--probe-merge") {
            exit(Pass2.main())
        }
        if CommandLine.arguments.contains("--probe") {
            exit(Probe.main())
        }
        #endif
        SwiftUIReferenceApp.main()
    }
}

struct SwiftUIReferenceApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate

    init() {
        UserDefaults.standard.register(defaults: [
            // AppKit otherwise treats `<slug>` as a document to open and then
            // skips opening the main window at launch.
            "NSTreatUnknownArgumentsAsOpen": "NO",
            // Never restore a previous session's windows, frames or state;
            // restored state without windows would suppress the main window.
            "NSQuitAlwaysKeepsWindows": false,
            "ApplePersistenceIgnoreState": true,
        ])
        // Split view autosave would otherwise carry a resized sidebar into the next launch.
        for key in UserDefaults.standard.dictionaryRepresentation().keys
        where key.hasPrefix("NSSplitView Subview Frames") || key.hasPrefix("NSWindow Frame") {
            UserDefaults.standard.removeObject(forKey: key)
        }
    }

    var body: some Scene {
        Window("SwiftUI macOS Reference", id: "gallery") {
            GalleryView(initialPage: LaunchOptions.current.page)
                .frame(minWidth: WindowMetrics.minimumSize.width, minHeight: WindowMetrics.minimumSize.height)
                .background(WindowConfigurator())
        }
        .defaultSize(width: WindowMetrics.size.width, height: WindowMetrics.size.height)
        .defaultPosition(.center)
        .windowToolbarStyle(.unified)
        .restorationBehavior(.disabled)
        .commands {
            CommandGroup(replacing: .newItem) {}
        }
    }
}

/// Pins the window frame to exactly the gallery's size and disables the
/// opening animation so captures are deterministic.
private struct WindowConfigurator: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView { ConfiguringView() }
    func updateNSView(_ nsView: NSView, context: Context) {}

    final class ConfiguringView: NSView {
        private var configured = false

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard let window, !configured else { return }
            configured = true
            window.animationBehavior = .none
            window.isRestorable = false
            window.setFrameAutosaveName("")
            window.setFrame(NSRect(origin: window.frame.origin, size: WindowMetrics.size), display: false)
            if let displayID = LaunchOptions.current.displayID,
               let screen = NSScreen.screens.first(where: {
                   ($0.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value == displayID
               }) {
                let visible = screen.visibleFrame
                let origin = NSPoint(
                    x: (visible.midX - WindowMetrics.size.width / 2).rounded(),
                    y: (visible.midY - WindowMetrics.size.height / 2).rounded()
                )
                window.setFrame(NSRect(origin: origin, size: WindowMetrics.size), display: false)
            } else {
                window.center()
            }
        }
    }
}
