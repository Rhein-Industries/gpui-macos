// Headless AppKit/CoreText measurement. Creates no window and never runs an app loop.
import AppKit
import CoreText

let app = NSApplication.shared
app.appearance = NSAppearance(named: .aqua)
let field = NSSecureTextField(string: "macOS")
field.frame = NSRect(x: 0, y: 0, width: 170, height: 24)
let scale = 2
let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: 340, pixelsHigh: 48,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true,
    isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
)!
let graphics = NSGraphicsContext(bitmapImageRep: bitmap)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = graphics
graphics.cgContext.scaleBy(x: CGFloat(scale), y: CGFloat(scale))
NSColor.white.setFill()
field.bounds.fill()
field.cell!.draw(withFrame: field.bounds, in: field)
NSGraphicsContext.restoreGraphicsState()

let bytes = bitmap.bitmapData!
var spans: [[String: Double]] = []
var firstX: Int? = nil
var inkYs: [Int] = []
for x in 0..<bitmap.pixelsWide {
    var column: [Int] = []
    for y in 0..<bitmap.pixelsHigh {
        let offset = y * bitmap.bytesPerRow + x * 4
        if bytes[offset] < 100 && bytes[offset + 1] < 100 && bytes[offset + 2] < 100 {
            column.append(y)
        }
    }
    if !column.isEmpty {
        if firstX == nil { firstX = x }
        inkYs.append(contentsOf: column)
    } else if let first = firstX {
        spans.append([
            "left": Double(first) / Double(scale),
            "top": Double(inkYs.min()!) / Double(scale),
            "ink_width": Double(x - first) / Double(scale),
            "ink_height": Double(inkYs.max()! - inkYs.min()! + 1) / Double(scale)
        ])
        firstX = nil
        inkYs.removeAll(keepingCapacity: true)
    }
}

let font = NSFont(name: "HelveticaNeue", size: 12.8)!
let ctFont = font as CTFont
var character = Array("●".utf16)
var glyph: [CGGlyph] = [0]
CTFontGetGlyphsForCharacters(ctFont, &character, &glyph, 1)
var inkBounds = CGRect.zero
var advance = CGSize.zero
CTFontGetBoundingRectsForGlyphs(ctFont, .default, &glyph, &inkBounds, 1)
CTFontGetAdvancesForGlyphs(ctFont, .default, &glyph, &advance, 1)
let advances = zip(spans, spans.dropFirst()).map { $1["left"]! - $0["left"]! }
let results: [String: Any] = [
    "environment": ["os": ProcessInfo.processInfo.operatingSystemVersionString, "scale": scale],
    "native": [
        "font_points": field.font!.pointSize,
        "ink_boxes": spans,
        "advances": advances
    ],
    "adapter": [
        "font_family": font.familyName!,
        "font_points": font.pointSize,
        "character": "●",
        "ink_width": inkBounds.width,
        "ink_height": inkBounds.height,
        "advance": advance.width
    ]
]
let data = try JSONSerialization.data(withJSONObject: results, options: [.prettyPrinted, .sortedKeys])
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write(Data("\n".utf8))
if let index = CommandLine.arguments.firstIndex(of: "--png"), CommandLine.arguments.count > index + 1 {
    try bitmap.representation(using: .png, properties: [:])!
        .write(to: URL(fileURLWithPath: CommandLine.arguments[index + 1]))
}
