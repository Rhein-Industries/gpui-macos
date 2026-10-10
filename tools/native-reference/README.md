# Native macOS Reference

This small AppKit application provides a real system-control baseline for the GPUI components. Buttons, editors, switches, segments, pickers, sliders, progress indicators, the sidebar, and the unified toolbar use standard AppKit controls without custom control drawing.

Build it from the repository root:

```sh
scripts/build-native-reference.sh
```

The script compiles with the installed macOS 26.2 SDK, signs and registers `target/native-reference/Native macOS Reference.app`, and exports `target/native-reference/metrics.json`. It does not launch an application window or change the system's Xcode configuration. Quit the reference app before rebuilding its bundle.

The app supports `--light`, `--dark`, and `--page controls|workspace`. Its Appearance menu can also switch between system, light, and dark appearances. Use the session's RiWork Cua driver to launch and compare visible windows.

Headless measurement does not enter the application event loop:

```sh
target/native-reference/Native\ macOS\ Reference.app/Contents/MacOS/native-reference \
  --measure --metrics target/native-reference/metrics.json
```

Without `--metrics`, measurement writes JSON to stdout. It includes actual intrinsic/fitting sizes and semantic NSColor values for both appearances, plus the host OS and SDK. Measurements describe the installed macOS version; they are not evidence of an unavailable newer OS. The checked-in baseline is [docs/native-metrics.json](../../docs/native-metrics.json).
