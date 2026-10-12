#!/bin/bash
# Build the SwiftUI mirror of the component gallery as an ad-hoc signed app.
# Usage: scripts/build-swiftui-reference.sh
# Then:  open -n "target/swiftui-reference/SwiftUI macOS Reference.app" --args <slug> [--light|--dark]
set -euo pipefail

task_root="$(cd "$(dirname "$0")/.." && pwd)"
bundle="$task_root/target/swiftui-reference/SwiftUI macOS Reference.app"
sdk=""
for candidate in \
    "$(xcrun --sdk macosx --show-sdk-path 2>/dev/null || true)" \
    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk" \
    "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk"; do
    if [[ -n "$candidate" && -f "$candidate/SDKSettings.plist" ]]; then sdk="$candidate"; break; fi
done
if [[ -z "$sdk" ]]; then
    printf 'No usable macOS SDK found. Set up Xcode or Command Line Tools.\n' >&2
    exit 1
fi

rm -rf "$bundle"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
mkdir -p "$bundle/Contents/Resources/GalleryAssets"
cp "$task_root"/examples/gallery/assets/*.svg "$bundle/Contents/Resources/GalleryAssets/"
swiftc -sdk "$sdk" -target arm64-apple-macosx26.0 -swift-version 6 -O -parse-as-library \
    -warnings-as-errors \
    "$task_root/tools/swiftui-reference/App.swift" \
    "$task_root/tools/swiftui-reference/Catalog.swift" \
    "$task_root/tools/swiftui-reference/GalleryView.swift" \
    "$task_root/tools/swiftui-reference/Pages.swift" \
    "$task_root/tools/swiftui-reference/PagesActions.swift" \
    "$task_root/tools/swiftui-reference/PagesForms.swift" \
    "$task_root/tools/swiftui-reference/PagesOverlays.swift" \
    "$task_root/tools/swiftui-reference/PagesDisplay.swift" \
    -o "$bundle/Contents/MacOS/swiftui-reference"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>dev.gpui.macos.swiftui-reference</string>
<key>CFBundleName</key><string>SwiftUI macOS Reference</string>
<key>CFBundleDisplayName</key><string>SwiftUI macOS Reference</string>
<key>CFBundleExecutable</key><string>swiftui-reference</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>26.0</string>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>NSQuitAlwaysKeepsWindows</key><false/>
</dict></plist>
PLIST

python3 "$task_root/scripts/generate-notices.py" --project-only --output "$bundle/Contents/Resources/Licenses"

codesign --force --sign - "$bundle"
"/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" -f "$bundle"
printf '%s\n' "$bundle"
