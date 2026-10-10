#!/usr/bin/env python3
"""Normalize the pinned Kit manifests without requiring consumer [patch] tables.

Usage: scripts/vendor-kit.py /path/to/gpui-kit-checkout
Existing source files are kept: local editor/presentation changes are intentional.
"""
import copy
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
PIN = "0bbd9870ce420af9feacbf948b21d3824d82dc0b"
CRATES = {
    "gpui-kit": "kit",
    "gpui-base": "base",
    "gpui-component": "component",
    "gpui-component-macros": "component-macros",
    "gpui-kit-assets": "assets",
}
FORK_VERSION = "0.7.1-macos.1"
FORK_REPOSITORY = "https://github.com/Rhein-Industries/gpui-macos"
FORK_PACKAGES = {
    "gpui-kit": "gpui-macos-kit",
    "gpui-base": "gpui-macos-base",
    "gpui-component": "gpui-macos-component",
    "gpui-component-macros": "gpui-macos-component-macros",
    "gpui-kit-assets": "gpui-macos-assets",
}


def key(value):
    return value if re.fullmatch(r"[A-Za-z0-9_-]+", value) else json.dumps(value)


def scalar(value):
    if isinstance(value, bool):
        return str(value).lower()
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ", ".join(scalar(item) for item in value) + "]"
    return str(value)


def serialize(data):
    lines = ["# Pinned GPUI Kit source; workspace dependencies normalized locally."]

    def table(values, path=(), array=False):
        if path:
            name = ".".join(key(part) for part in path)
            lines.extend(["", ("[[" + name + "]]") if array else ("[" + name + "]")])
        for name, value in values.items():
            if not isinstance(value, dict) and not (
                isinstance(value, list) and value and isinstance(value[0], dict)
            ):
                lines.append(f"{key(name)} = {scalar(value)}")
        for name, value in values.items():
            if isinstance(value, dict):
                table(value, path + (name,))
            elif isinstance(value, list) and value and isinstance(value[0], dict):
                for item in value:
                    table(item, path + (name,), True)

    table(data)
    return "\n".join(lines) + "\n"


def normalize(data, workspace):
    package = data.get("package", {})
    original_name = next(
        (name for name, fork in FORK_PACKAGES.items() if package.get("name") == fork),
        package.get("name"),
    )
    if original_name == "gpui-base":
        # Only upstream's excluded native example uses this obsolete client.
        # Keep its intentional removal stable when regenerating manifests.
        data.get("dev-dependencies", {}).pop("reqwest_client", None)
    if isinstance(data.get("package", {}).get("edition"), dict):
        data["package"]["edition"] = workspace["package"]["edition"]
    if data.get("lints", {}).get("workspace"):
        data["lints"] = copy.deepcopy(workspace["lints"])

    def visit(values):
        for section, dependencies in list(values.items()):
            if section in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, spec in list(dependencies.items()):
                    if isinstance(spec, dict) and spec.get("workspace"):
                        inherited = copy.deepcopy(workspace["dependencies"][name])
                        if isinstance(inherited, str):
                            inherited = {"version": inherited}
                        local = {k: v for k, v in spec.items() if k != "workspace"}
                        if "features" in inherited and "features" in local:
                            local["features"] = list(dict.fromkeys(inherited["features"] + local["features"]))
                        inherited.update(local)
                        if name in CRATES:
                            inherited["path"] = "../" + name
                        else:
                            inherited.pop("path", None)
                        dependencies[name] = inherited
            elif isinstance(dependencies, dict):
                visit(dependencies)

    visit(data)
    if original_name in FORK_PACKAGES:
        package["name"] = FORK_PACKAGES[original_name]
        package["version"] = FORK_VERSION
        package["documentation"] = f"https://docs.rs/{package['name']}"
        package["repository"] = FORK_REPOSITORY
        package["homepage"] = FORK_REPOSITORY
        package["publish"] = ["crates-io"]
        package["description"] = f"The gpui-macos pinned fork of {original_name}."
        if original_name == "gpui-kit-assets":
            package["license"] = "Apache-2.0 AND ISC AND MIT"
        package.setdefault("metadata", {})["gpui-macos-fork"] = {
            "upstream-package": original_name,
            "upstream-version": "0.7.1",
            "upstream-revision": PIN,
            "upstream-source": "https://github.com/longbridge/gpui-kit",
        }
        package["metadata"].setdefault("docs", {})["rs"] = {
            "default-target": "x86_64-unknown-linux-gnu",
            "targets": [],
        }
        # Dependency aliases and library target names preserve every Rust path.
        data.setdefault("lib", {})["name"] = original_name.replace("-", "_")

    def fork_dependencies(values):
        for section, dependencies in values.items():
            if section in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, spec in dependencies.items():
                    if name in FORK_PACKAGES:
                        spec["package"] = FORK_PACKAGES[name]
                        spec["version"] = "=" + FORK_VERSION
                        spec["path"] = "../" + name
                    if original_name == "gpui-kit-assets" and name == "rust-embed":
                        # Debug app bundles must be independent of the source
                        # checkout, including macOS Documents privacy access.
                        spec["features"] = list(dict.fromkeys(
                            spec.get("features", []) + ["debug-embed"]
                        ))
            elif isinstance(dependencies, dict):
                fork_dependencies(dependencies)

    fork_dependencies(data)
    if data.get("package", {}).get("readme", "").startswith("../"):
        data["package"]["readme"] = "README.md"
    return data


def main():
    source = Path(sys.argv[1]).resolve()
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=source, text=True
    ).strip()
    if revision != PIN:
        raise SystemExit(f"Expected GPUI Kit {PIN}, found {revision}")
    workspace = tomllib.loads((source / "Cargo.toml").read_text())["workspace"]
    for name, directory in CRATES.items():
        original = source / "crates" / directory
        target = ROOT / "vendor" / name
        if not target.exists():
            shutil.copytree(original, target, ignore=shutil.ignore_patterns("examples", "target"))
        if not (target / "README.md").exists():
            shutil.copy2(source / "README.md", target / "README.md")
        if not (target / "LICENSE-APACHE").exists():
            shutil.copy2(source / "LICENSE-APACHE", target / "LICENSE-APACHE")
        data = tomllib.loads((original / "Cargo.toml").read_text())
        (target / "Cargo.toml").write_text(serialize(normalize(data, workspace)))
        print(f"{FORK_PACKAGES[name]} {FORK_VERSION}: pinned source, original Rust alias")
    shutil.copy2(
        source / "crates/component/src/lib.rs",
        ROOT / "docs/upstream-component-exports.rs",
    )


if __name__ == "__main__":
    main()
