// Run with: swift scripts/diagnose-native-symbols.swift [output-directory]
// Headless AppKit and SwiftUI SF Symbol raster probe; never orders a window.
import AppKit
import SwiftUI
import Foundation
import Dispatch

func measure(_ bitmap: NSBitmapImageRep) -> [String: Any] {
    var left = bitmap.pixelsWide, right = -1
    var top = bitmap.pixelsHigh, bottom = -1
    var nonzero = 0
    for y in 0..<bitmap.pixelsHigh {
        for x in 0..<bitmap.pixelsWide {
            var pixel = [Int](repeating: 0, count: bitmap.samplesPerPixel)
            bitmap.getPixel(&pixel, atX: x, y: y)
            let alphaIndex = bitmap.bitmapFormat.contains(.alphaFirst) ? 0 : bitmap.samplesPerPixel - 1
            let alpha = bitmap.hasAlpha ? pixel[alphaIndex] : 255
            if alpha > 128 {
                left = min(left, x); right = max(right, x)
                top = min(top, y); bottom = max(bottom, y)
                nonzero += 1
            }
        }
    }
    return ["canvas": [bitmap.pixelsWide, bitmap.pixelsHigh],
            "point_size": [bitmap.size.width, bitmap.size.height],
            "ink": [max(0, right + 1 - left), max(0, bottom + 1 - top)],
            "bounds": [left, top, right, bottom], "pixels_over_128": nonzero]
}

func blankBitmap() -> NSBitmapImageRep {
    let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 52, pixelsHigh: 52,
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
        colorSpaceName: .deviceRGB, bitmapFormat: [], bytesPerRow: 52 * 4, bitsPerPixel: 32)!
    memset(bitmap.bitmapData!, 0, bitmap.bytesPerRow * bitmap.pixelsHigh)
    bitmap.size = NSSize(width: 26, height: 26)
    return bitmap
}

func rect(_ symbol: NSImage) -> NSRect {
    let intrinsic = symbol.size
    let fit = min(1, 26 / max(intrinsic.width, intrinsic.height))
    let size = NSSize(width: intrinsic.width * fit, height: intrinsic.height * fit)
    return NSRect(x: (26 - size.width) / 2, y: (26 - size.height) / 2,
                  width: size.width, height: size.height)
}

func appkitBitmap(_ symbol: NSImage) -> NSBitmapImageRep {
    let bitmap = blankBitmap()
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)!
    symbol.draw(in: rect(symbol), from: .zero, operation: .sourceOver, fraction: 1)
    NSGraphicsContext.restoreGraphicsState()
    return bitmap
}

func directCGContext(_ symbol: NSImage) -> NSBitmapImageRep {
    let bitmap = blankBitmap()
    let context = CGContext(data: bitmap.bitmapData!, width: 52, height: 52,
        bitsPerComponent: 8, bytesPerRow: bitmap.bytesPerRow,
        space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    context.scaleBy(x: 2, y: 2)
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(cgContext: context, flipped: false)
    symbol.draw(in: rect(symbol), from: .zero, operation: .sourceOver, fraction: 1)
    NSGraphicsContext.restoreGraphicsState()
    return bitmap
}

final class BackgroundRender: @unchecked Sendable {
    var bitmap: NSBitmapImageRep?
    var isMainThread = true
}

func backgroundBitmap(_ name: String) -> BackgroundRender {
    let result = BackgroundRender()
    let ready = DispatchSemaphore(value: 0)
    DispatchQueue.global().async {
        autoreleasepool {
            let base = NSImage(systemSymbolName: name, accessibilityDescription: nil)!
            let configuration = NSImage.SymbolConfiguration(pointSize: 13, weight: .medium, scale: .large)
                .applying(.preferringMonochrome())
            result.isMainThread = Thread.isMainThread
            result.bitmap = appkitBitmap(base.withSymbolConfiguration(configuration)!)
        }
        ready.signal()
    }
    ready.wait()
    return result
}

@MainActor func swiftUIBitmap(_ name: String) -> NSBitmapImageRep? {
    let renderer = ImageRenderer(content: Image(systemName: name)
        .font(.system(size: 13, weight: .medium)).imageScale(.large)
        .symbolRenderingMode(.monochrome).foregroundStyle(.white)
        .frame(width: 26, height: 26))
    renderer.scale = 2
    renderer.isOpaque = false
    guard let image = renderer.cgImage else { return nil }
    return NSBitmapImageRep(cgImage: image)
}

@MainActor func run() {
    let inCI = ProcessInfo.processInfo.environment["GITHUB_ACTIONS"] != nil
    let output = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "/tmp/gpui-macos26-investigation/probe-output"
    try! FileManager.default.createDirectory(atPath: output, withIntermediateDirectories: true)
    var results: [[String: Any]] = []
    for name in ["square.grid.2x2", "paintpalette", "macwindow"] {
        let base = NSImage(systemSymbolName: name, accessibilityDescription: nil)!
        let configuration = NSImage.SymbolConfiguration(pointSize: 13, weight: .medium, scale: .large)
            .applying(.preferringMonochrome())
        let symbol = base.withSymbolConfiguration(configuration)!
        let diagnosticContext = NSGraphicsContext(bitmapImageRep: blankBitmap())!
        let ctm = diagnosticContext.cgContext.ctm
        var entry: [String: Any] = ["name": name, "intrinsic_points": [symbol.size.width, symbol.size.height],
                                  "draw_rect_points": [rect(symbol).origin.x, rect(symbol).origin.y,
                                                       rect(symbol).width, rect(symbol).height]]
        entry["context_ctm"] = [ctm.a, ctm.b, ctm.c, ctm.d, ctm.tx, ctm.ty]
        entry["context_is_drawing_to_screen"] = diagnosticContext.isDrawingToScreen
        var renditions = [("appkit_matching_rust", appkitBitmap(symbol)),
                          ("independent_cgcontext", directCGContext(symbol))]
        let best = symbol.bestRepresentation(for: rect(symbol), context: diagnosticContext, hints: nil)
        entry["best_representation_class"] = best.map { String(describing: type(of: $0)) } ?? "none"
        entry["representations"] = symbol.representations.map { String(describing: type(of: $0)) }
        let background = backgroundBitmap(name)
        entry["background_is_main_thread"] = background.isMainThread
        renditions.append(("appkit_background", background.bitmap!))
        if let bitmap = swiftUIBitmap(name) { renditions.append(("swiftui_image_renderer", bitmap)) }
        if let tiff = symbol.tiffRepresentation {
            for (index, rep) in NSBitmapImageRep.imageReps(with: tiff).enumerated() {
                if let bitmap = rep as? NSBitmapImageRep { renditions.append(("nsimage_tiff_\(index)", bitmap)) }
            }
        }
        var proposed = rect(symbol)
        let ctx = NSGraphicsContext(bitmapImageRep: blankBitmap())!
        if let cg = symbol.cgImage(forProposedRect: &proposed, context: ctx, hints: nil) {
            renditions.append(("nsimage_cgimage", NSBitmapImageRep(cgImage: cg)))
        }
        for (method, bitmap) in renditions {
            let data = bitmap.representation(using: .png, properties: [:])!
            var measurement = measure(bitmap)
            if inCI && ["appkit_matching_rust", "swiftui_image_renderer"].contains(method) {
                measurement["png_base64"] = data.base64EncodedString()
            }
            entry[method] = measurement
            try! data.write(to: URL(fileURLWithPath: output).appendingPathComponent("\(name).\(method).png"))
        }
        results.append(entry)
    }
    let os = ProcessInfo.processInfo.operatingSystemVersion
    let report: [String: Any] = ["os": ProcessInfo.processInfo.operatingSystemVersionString,
        "os_version": [os.majorVersion, os.minorVersion, os.patchVersion],
        "screen_backing_scales": NSScreen.screens.map { $0.backingScaleFactor },
        "method": "Alpha > 128, 13 pt medium / large scale, natural size centered in 26 pt box at 2x.",
        "results": results]
    let options: JSONSerialization.WritingOptions = inCI ? [.sortedKeys] : [.prettyPrinted, .sortedKeys]
    let data = try! JSONSerialization.data(withJSONObject: report, options: options)
    try! data.write(to: URL(fileURLWithPath: output).appendingPathComponent("results.json"))
    let prefix = inCI ? "GPUI_NATIVE_SYMBOL_DIAGNOSTICS " : ""
    print(prefix + String(data: data, encoding: .utf8)!)
}

MainActor.assumeIsolated { run() }
