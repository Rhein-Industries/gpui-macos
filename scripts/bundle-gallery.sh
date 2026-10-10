#!/bin/bash
set -euo pipefail

# Build a normal app bundle for native inspection or launching from Finder.
task_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$task_root"
profile="${1:-dev}"
case "$profile" in
  dev) build_folder=debug ;;
  release) build_folder=release ;;
  *) printf 'Usage: %s [dev|release]\n' "$0" >&2; exit 2 ;;
esac
cargo build --locked --example gallery --profile "$profile"

python3 - "$build_folder" <<'PY'
import json, pathlib, shutil, subprocess, sys
metadata = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--format-version', '1', '--no-deps'
]))
target = pathlib.Path(metadata['target_directory']) / sys.argv[1]
bundle = target / 'GPUI macOS Gallery.app'
contents = bundle / 'Contents'
(contents / 'MacOS').mkdir(parents=True, exist_ok=True)
(contents / 'Resources').mkdir(exist_ok=True)
notice_source = target / (bundle.stem + '.licenses')
subprocess.run(['python3', 'scripts/generate-notices.py', '--output', str(notice_source)], check=True)
notice_destination = contents / 'Resources/Licenses'
if notice_destination.exists():
    shutil.rmtree(notice_destination)
shutil.copytree(notice_source, notice_destination)
shutil.copy2(target / 'examples/gallery', contents / 'MacOS/gpui-macos-gallery')
info = {
        'CFBundleIdentifier': 'dev.gpui.macos.gallery',
        'CFBundleName': 'GPUI macOS Gallery',
        'CFBundleDisplayName': 'GPUI macOS Gallery',
        'CFBundleExecutable': 'gpui-macos-gallery',
        'CFBundlePackageType': 'APPL',
        'CFBundleShortVersionString': '0.1.0',
        'CFBundleVersion': '1',
        'NSHighResolutionCapable': True,
        'LSMinimumSystemVersion': '12.0',
    }
subprocess.run(['/usr/bin/plutil', '-convert', 'xml1', '-o',
    str(contents / 'Info.plist'), '--', '-'], input=json.dumps(info).encode(), check=True)
subprocess.run(['codesign', '--force', '--sign', '-', str(bundle)], check=True)
print(bundle)
PY
