#!/usr/bin/env python3
"""Print dependency-ordered crates.io release commands; never upload anything."""
from pathlib import Path
import shlex
import tomllib

ROOT = Path(__file__).resolve().parent.parent
ORDER = (
    "vendor/gpui-component-macros/Cargo.toml",
    "vendor/gpui-kit-assets/Cargo.toml",
    "vendor/gpui-base/Cargo.toml",
    "vendor/gpui-component/Cargo.toml",
    "vendor/gpui-kit/Cargo.toml",
    "Cargo.toml",
)

print("# Run scripts/check-packages.py first. These commands are printed only.")
print("# Commit the release. Publish one crate at a time and wait for registry indexing.")
for path in ORDER:
    package = tomllib.loads((ROOT / path).read_text())["package"]
    print(f"\n# {package['name']} {package['version']}")
    print(f"cargo publish --manifest-path {shlex.quote(path)} --locked --registry crates-io")
