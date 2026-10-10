#!/bin/bash
set -euo pipefail

task_root="$(cd "$(dirname "$0")/.." && pwd)"
bundle="$task_root/target/native-reference/Native macOS Reference.app"
sdk=""
for candidate in "/Library/Developer/CommandLineTools/SDKs/MacOSX26.2.sdk" "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"; do
    if [[ -f "$candidate/SDKSettings.plist" ]]; then sdk="$candidate"; break; fi
done
if [[ -z "$sdk" ]]; then
    printf 'No usable macOS SDK found. Set up Xcode or Command Line Tools.\n' >&2
    exit 1
fi
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
swiftc -sdk "$sdk" -target arm64-apple-macosx26.0 -swift-version 6 -O \
    "$task_root/tools/native-reference/main.swift" -o "$bundle/Contents/MacOS/native-reference"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>dev.gpui.macos.native-reference</string>
<key>CFBundleName</key><string>Native macOS Reference</string>
<key>CFBundleDisplayName</key><string>Native macOS Reference</string>
<key>CFBundleExecutable</key><string>native-reference</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
<key>LSMinimumSystemVersion</key><string>26.0</string>
<key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST

python3 "$task_root/scripts/generate-notices.py" --project-only --output "$bundle/Contents/Resources/Licenses"

codesign --force --sign - "$bundle"
"/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" -f "$bundle"
"$bundle/Contents/MacOS/native-reference" --measure --metrics "$task_root/target/native-reference/metrics.json"
printf '%s\n' "$bundle" "$task_root/target/native-reference/metrics.json"
