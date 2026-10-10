#!/usr/bin/env python3
"""Package all six crates and verify them against an isolated offline registry.

Reads cached crates.io indexes/archives only; never reads credentials, uploads,
changes the global Cargo cache, or patches a consumer's dependency graph.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
MANIFESTS = [
    ROOT / "vendor/gpui-component-macros/Cargo.toml",
    ROOT / "vendor/gpui-kit-assets/Cargo.toml",
    ROOT / "vendor/gpui-base/Cargo.toml",
    ROOT / "vendor/gpui-component/Cargo.toml",
    ROOT / "vendor/gpui-kit/Cargo.toml",
    ROOT / "Cargo.toml",
]


def read_manifest(path):
    return tomllib.loads(path.read_text())


def index_path(name):
    if len(name) == 1:
        return Path("1") / name
    if len(name) == 2:
        return Path("2") / name
    if len(name) == 3:
        return Path("3") / name[0] / name
    return Path(name[:2]) / name[2:4] / name


def seed_registry(registry):
    cargo_cache = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    archives = {}
    for cache in (cargo_cache / "registry/cache").glob("index.crates.io-*"):
        for archive in cache.glob("*.crate"):
            archives.setdefault(archive.name, archive)
    entries = {}
    for cache in (cargo_cache / "registry/index").glob("index.crates.io-*/.cache"):
        for path in cache.rglob("*"):
            if not path.is_file():
                continue
            for chunk in path.read_bytes().split(b"\0"):
                if not chunk.startswith(b'{"name":'):
                    continue
                entry = json.loads(chunk)
                archive = f"{entry['name']}-{entry['vers']}.crate"
                entries.setdefault(entry["name"], {})[entry["vers"]] = entry
    for name, versions in entries.items():
        destination = registry / "index" / index_path(name)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text("".join(json.dumps(v) + "\n" for v in versions.values()))
        for version in versions:
            filename = f"{name}-{version}.crate"
            if filename in archives:
                (registry / filename).symlink_to(archives[filename])
    if not entries:
        raise SystemExit("No cached crates.io packages. Run cargo fetch --locked first.")
    print(f"Seeded temporary registry with {len(entries)} cached crate names.", flush=True)


def dependencies(manifest):
    result = []
    scopes = [(None, manifest)] + list(manifest.get("target", {}).items())
    for target, tables in scopes:
        for section, kind in (
            ("dependencies", "normal"),
            ("build-dependencies", "build"),
            ("dev-dependencies", "dev"),
        ):
            for name, spec in tables.get(section, {}).items():
                if isinstance(spec, str):
                    spec = {"version": spec}
                if "version" not in spec:
                    raise SystemExit(f"Dependency {name} has no registry version.")
                if "path" in spec or "git" in spec or "registry" in spec:
                    raise SystemExit(f"Packaged dependency {name} is not crates.io compatible.")
                result.append({
                    "name": name,
                    "req": spec["version"],
                    "features": spec.get("features", []),
                    "optional": spec.get("optional", False),
                    "default_features": spec.get("default-features", True),
                    "target": target,
                    "kind": kind,
                    "registry": None,
                    "package": spec.get("package"),
                })
    return result


def register_archive(archive, registry):
    with tarfile.open(archive, "r:gz") as package:
        manifests = [m for m in package.getmembers() if m.name.count("/") == 1 and m.name.endswith("/Cargo.toml")]
        if len(manifests) != 1:
            raise SystemExit(f"Expected one normalized manifest in {archive}.")
        manifest = tomllib.loads(package.extractfile(manifests[0]).read().decode())
    info = manifest["package"]
    entry = {
        "name": info["name"],
        "vers": info["version"],
        "deps": dependencies(manifest),
        "cksum": hashlib.sha256(archive.read_bytes()).hexdigest(),
        "features": {},
        "features2": manifest.get("features", {}),
        "v": 2,
        "yanked": False,
        "links": info.get("links"),
    }
    if "rust-version" in info:
        entry["rust_version"] = info["rust-version"]
    path = registry / "index" / index_path(info["name"])
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(entry) + "\n")
    shutil.copy2(archive, registry / archive.name)
    return entry


def run(command, cwd, environment, config):
    log = Path(environment["CARGO_TARGET_DIR"]) / "package" / "verification.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("a") as output:
        output.write("\n" + " ".join(command) + "\n")
        output.flush()
        result = subprocess.run(["cargo", "--config", str(config), *command], cwd=cwd, env=environment, stdout=output, stderr=subprocess.STDOUT)
    if result.returncode:
        print("\n".join(log.read_text().splitlines()[-50:]), flush=True)
        raise SystemExit(f"Cargo verification failed; full output: {log}")


def consumer_source(api, component):
    return f"""use {api} as gpui;
use {api}::{{App, Bounds, IntoElement, Pixels, Window}};
use {component}::plot::{{IntoPlot, Plot}};
gpui::actions!(registry_checks, [RegistryAction]);
#[derive(gpui::Action, Clone, PartialEq)]
pub struct ManualAction;
#[derive(IntoPlot)]
pub struct DerivedPlot;
impl Plot for DerivedPlot {{
    fn paint(&mut self, _: Bounds<Pixels>, _: &mut Window, _: &mut App) {{}}
}}
pub fn derived_element() -> impl IntoElement {{ DerivedPlot }}
pub fn checked_actions() {{
    fn accepts_action<T: gpui::Action>() {{}}
    accepts_action::<RegistryAction>();
    accepts_action::<ManualAction>();
}}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--optional", action="store_true", help="Also compile root speech and tree-sitter language features.")
    args = parser.parse_args()
    target = ROOT / "target"
    target.mkdir(exist_ok=True)
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(target)
    environment["CARGO_NET_OFFLINE"] = "true"
    environment["CARGO_TERM_COLOR"] = "never"
    # External consumers live outside the checkout, so preserve the tested
    # toolchain instead of inheriting a potentially older machine default.
    environment["RUSTUP_TOOLCHAIN"] = subprocess.check_output(
        ["rustup", "show", "active-toolchain"], cwd=ROOT, text=True,
    ).split()[0]
    platform = next(line.split(": ", 1)[1] for line in subprocess.check_output(["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
    results = []
    # Keep extracted archives outside the checkout's parent workspace. Cargo's
    # normalized package manifest intentionally omits the workspace declaration.
    with tempfile.TemporaryDirectory(prefix="gpui-macos-packages-") as temporary:
        temporary = Path(temporary)
        environment["CARGO_HOME"] = str(temporary / "cargo-home")
        Path(environment["CARGO_HOME"]).mkdir()
        registry = temporary / "registry"
        registry.mkdir()
        seed_registry(registry)
        config = temporary / "cargo-config.toml"
        config.write_text(
            '[source.crates-io]\nreplace-with = "package-check"\n'
            '[source.package-check]\nlocal-registry = ' + json.dumps(str(registry)) + '\n'
        )
        source = temporary / "source"
        shutil.copytree(ROOT, source, ignore=shutil.ignore_patterns(".git", "target", "artifacts", ".DS_Store", "__pycache__"))
        for original in MANIFESTS:
            staged = source / original.relative_to(ROOT)
            info = read_manifest(staged)["package"]
            print(f"Packaging {info['name']} {info['version']}...", flush=True)
            run(["package", "--manifest-path", str(staged), "--allow-dirty", "--locked", "--offline", "--no-verify"], source, environment, config)
            archive = target / "package" / f"{info['name']}-{info['version']}.crate"
            results.append(register_archive(archive, registry))
        root_info = read_manifest(ROOT / "Cargo.toml")["package"]
        kit_info = read_manifest(ROOT / "vendor/gpui-kit/Cargo.toml")["package"]
        consumers = [
            ("root_alias", "mac", root_info["name"], "=" + root_info["version"], [], "mac::gpui_kit", "mac::kit"),
            ("kit_alias", "toolkit", kit_info["name"], "=" + kit_info["version"], ["gpui-fast"], "toolkit", "toolkit::component"),
        ]
        for name, alias, package, version, features, api, component in consumers:
            consumer = temporary / name
            (consumer / "src").mkdir(parents=True)
            spec = {"package": package, "version": version}
            if features:
                spec["features"] = features
            consumer.joinpath("Cargo.toml").write_text(
                '[package]\nname = "' + name + '"\nversion = "0.0.0"\nedition = "2024"\n'
                '[dependencies]\n' + alias + ' = { ' + ', '.join(k + ' = ' + json.dumps(v) for k, v in spec.items()) + ' }\n'
                '[workspace]\n'
            )
            consumer.joinpath("src/lib.rs").write_text(consumer_source(api, component))
            shutil.copy2(source / "Cargo.lock", consumer / "Cargo.lock")
            print(f"Verifying registry consumer {name} (including IntoPlot)...", flush=True)
            run(["check", "--all-targets", "--offline"], consumer, environment, config)
            if args.optional and name == "root_alias":
                run(["check", "--offline", "--features", "mac/speech,mac/tree-sitter-languages"], consumer, environment, config)
            metadata = subprocess.check_output(
                ["cargo", "--config", str(config), "metadata", "--offline", "--format-version", "1", "--filter-platform", platform],
                cwd=consumer, env=environment, text=True,
            )
            packages = json.loads(metadata)["packages"]
            forks = [p for p in packages if p["name"].startswith("gpui-macos")]
            assert forks and all(p["source"] and p["source"].startswith("registry+") for p in forks), "Every released dependency must resolve from the registry."
        unpacked = temporary / "unpacked"
        unpacked.mkdir()
        root_archive = target / "package" / f"{root_info['name']}-{root_info['version']}.crate"
        with tarfile.open(root_archive, "r:gz") as package:
            package.extractall(unpacked, filter="data")
        archive_source = unpacked / f"{root_info['name']}-{root_info['version']}"
        print("Verifying the packaged inventory fixture and contracts...", flush=True)
        run(["test", "--lib", "--offline", "interpretation::tests"], archive_source, environment, config)
    report = target / "package" / "registry-verification.json"
    report.write_text(json.dumps({"packages": results, "consumer_checks": ["root_alias", "kit_alias"], "packaged_inventory_tests": True, "optional_features": args.optional, "uploaded": False}, indent=2) + "\n")
    print(f"Verified six crates and both aliased registry consumers. Report: {report}", flush=True)


if __name__ == "__main__":
    main()
