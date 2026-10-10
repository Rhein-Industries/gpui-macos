#!/usr/bin/env python3
"""Collect actual licenses and source links from the locked macOS dependency graph.

No SPDX text substitution: a dependency without a local or reviewed cached license
causes failure. --fetch-missing obtains omitted texts from the declared upstream
repository and records their immutable revision and hash for subsequent offline use.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
from urllib.error import HTTPError
from urllib.parse import quote, urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / "licenses/dependencies"
LICENSE_NAME = re.compile(r"^(licen[cs]e|copying|copyright|notice)([._-].*)?$", re.I)
KIT_REV = "0bbd9870ce420af9feacbf948b21d3824d82dc0b"
KIT_PATHS = {
    "gpui-kit": "crates/kit", "gpui-base": "crates/base",
    "gpui-component": "crates/component", "gpui-component-macros": "crates/component-macros",
    "gpui-kit-assets": "crates/assets",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def license_pointer(data, expression=None):
    short = data.decode("utf-8", errors="replace").strip()
    return len(data) < 100 and (short.startswith(("../", "./")) or short == expression
                                or short in {"MIT", "Apache-2.0", "MIT OR Apache-2.0"})


def metadata(manifest, target, all_features, offline):
    command = ["cargo", "metadata", "--locked", "--format-version", "1",
               "--manifest-path", str(manifest), "--filter-platform", target]
    if all_features:
        command.append("--all-features")
    if offline:
        command.append("--offline")
    return json.loads(subprocess.check_output(command))


def dependency_roles(data):
    packages = {p["id"]: p for p in data["packages"]}
    nodes = {n["id"]: n for n in data["resolve"]["nodes"]}
    roles = defaultdict(set)
    pending = [(data["resolve"]["root"], "runtime")]
    while pending:
        ident, role = pending.pop()
        if role in roles[ident]:
            continue
        roles[ident].add(role)
        for dep in nodes[ident]["deps"]:
            macro = any("proc-macro" in t["kind"] for t in packages[dep["pkg"]]["targets"])
            for kind in dep["dep_kinds"]:
                # Metadata is already target-filtered. A normal dependency of a
                # build script/proc macro remains build-only unless another path
                # reaches it at runtime. Development edges never become runtime.
                if kind["kind"] == "dev":
                    continue
                elif role == "build" or kind["kind"] == "build" or macro:
                    next_role = "build"
                else:
                    next_role = "runtime"
                pending.append((dep["pkg"], next_role))
    return roles


def local_licenses(package):
    directory = Path(package["manifest_path"]).parent
    # Do not scan build outputs or the whole workspace for the library package.
    if directory == ROOT:
        paths = {ROOT / "LICENSE", ROOT / "NOTICE", *ROOT.joinpath("licenses").glob("*.txt")}
        return [(str(p.relative_to(directory)), p.read_bytes()) for p in sorted(paths)]
    paths = {p for p in directory.rglob("*") if p.is_file() and LICENSE_NAME.match(p.name)
             and ".git" not in p.relative_to(directory).parts
             and "target" not in p.relative_to(directory).parts}
    if package.get("license_file"):
        declared = directory / package["license_file"]
        if declared.is_file():
            paths.add(declared)
    result = []
    for path in sorted(paths):
        data = path.read_bytes()
        # Cargo can archive a workspace symlink as the literal '../LICENSE',
        # and some crates put only an SPDX expression in a LICENSE file.
        # Neither supplies legal terms. Such entries require the reviewed cache.
        if license_pointer(data, package.get("license")):
            continue
        result.append((str(path.relative_to(directory)), data))
    return result


def upstream(package):
    directory = Path(package["manifest_path"]).parent
    if directory.parent == ROOT / "vendor":
        original = directory.name
        return {"repository": "https://github.com/longbridge/gpui-kit", "revision": KIT_REV,
                "path": KIT_PATHS[original], "revision_matches_package": True}
    vcs_path = directory / ".cargo_vcs_info.json"
    vcs = json.loads(vcs_path.read_text()) if vcs_path.is_file() else {}
    repository = package.get("repository") or package.get("homepage") or ""
    match = re.match(r"https?://github\.com/([^/]+/[^/#]+)", repository)
    if match:
        repository = "https://github.com/" + match.group(1).removesuffix(".git")
    return {"repository": repository.removesuffix(".git").rstrip("/"),
            "revision": vcs.get("git", {}).get("sha1"),
            "path": vcs.get("path_in_vcs", ""),
            "revision_matches_package": bool(vcs.get("git", {}).get("sha1"))}


def get(url):
    request = Request(url, headers={"User-Agent": "gpui-macos-license-collector/1.0"})
    with urlopen(request, timeout=40) as response:
        return response.read()


def repository_licenses(info):
    repository, revision = info["repository"], info["revision"]
    if not revision:
        # Old crate archives sometimes omit VCS data. Prefer the matching tag;
        # if absent, pin the upstream HEAD and mark that distinction explicitly.
        lines = subprocess.check_output(["git", "ls-remote", repository,
                                        f"refs/tags/{info['version']}",
                                        f"refs/tags/v{info['version']}", "HEAD"], text=True).splitlines()
        tags = [line for line in lines if "refs/tags/" in line]
        revision = (tags or lines)[0].split()[0]
        info["revision"] = revision
        info["revision_matches_package"] = bool(tags)
    parsed = urlparse(repository)
    if parsed.netloc == "github.com":
        slug = parsed.path.strip("/")
        # Codeload avoids API rate limits and retrieves license/NOTICE files
        # together, including package-level and inherited workspace notices.
        archive = get(f"https://codeload.github.com/{slug}/tar.gz/{revision}")
        files = {}
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as tar:
            for member in tar.getmembers():
                path = PurePosixPath(*PurePosixPath(member.name).parts[1:])
                if member.isfile() and LICENSE_NAME.match(path.name):
                    content = tar.extractfile(member).read()
                    if not license_pointer(content):
                        files[str(path)] = content
        return files
    if "gitlab" in parsed.netloc:
        result = {}
        for name in ["LICENSE", "LICENSE.txt", "LICENSE-MIT", "COPYING", "NOTICE"]:
            url = f"{repository}/-/raw/{revision}/{name}"
            try:
                content = get(url)
                if b"<html" not in content.lower():
                    result[name] = content
            except HTTPError as error:
                if error.code not in (403, 404):
                    raise
        return result
    raise ValueError(f"No supported official source repository: {repository!r}")


def fetch_missing(packages, index):
    groups = defaultdict(list)
    for package in packages:
        key = f"{package['name']}-{package['version']}"
        if local_licenses(package) or key in index:
            continue
        info = upstream(package)
        info["version"] = package["version"]
        groups[(info["repository"], info["revision"], package["version"] if not info["revision"] else "")].append((package, info))

    def fetch_group(group):
        files = repository_licenses(group[0][1])
        entries = []
        for package, info in group:
            info.update({k: group[0][1][k] for k in ("revision", "revision_matches_package")})
            package_path = PurePosixPath(info["path"] or ".")
            ancestors = {".", str(package_path), *(str(p) for p in package_path.parents)}
            selected = {path: content for path, content in files.items()
                        if str(PurePosixPath(path).parent) in ancestors}
            if not selected:
                raise ValueError(f"No upstream license files for {package['name']} at {info}")
            key = f"{package['name']}-{package['version']}"
            record = {**info, "files": []}
            for path, content in sorted(selected.items()):
                destination = CACHE / key / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
                url = f"{info['repository']}/blob/{info['revision']}/{path}"
                record["files"].append({"path": path, "sha256": digest(content), "url": url})
            entries.append((key, record))
        return entries

    with ThreadPoolExecutor(max_workers=6) as executor:
        futures = [executor.submit(fetch_group, group) for group in groups.values()]
        errors = []
        for group, future in zip(groups.values(), futures):
            try:
                for key, record in future.result():
                    index[key] = record
            except Exception as error:
                errors.append(f"{', '.join(p['name'] for p, _ in group)}: {error}")
        CACHE.mkdir(parents=True, exist_ok=True)
        (CACHE / "SOURCES.json").write_text(json.dumps(index, indent=2, sort_keys=True) + "\n")
        if errors:
            raise ValueError("License retrieval failed:\n" + "\n".join(errors))


def write_source_archive(package, destination):
    directory = Path(package["manifest_path"]).parent
    with tarfile.open(destination, "w:gz") as tar:
        for path in sorted(directory.rglob("*")):
            if path.is_file() and not any(p in {".git", "target", ".cargo-ok"}
                                         for p in path.relative_to(directory).parts):
                tar.add(path, arcname=f"{package['name']}-{package['version']}/{path.relative_to(directory)}")


def generate(default, complete, output, index):
    default_roles, all_roles = dependency_roles(default), dependency_roles(complete)
    packages = sorted((p for p in complete["packages"] if p["id"] in all_roles),
                      key=lambda p: (p["name"], p["version"]))
    missing = [f"{p['name']}-{p['version']}" for p in packages
               if not local_licenses(p) and f"{p['name']}-{p['version']}" not in index]
    if missing:
        raise ValueError("Missing license texts (run --fetch-missing and review sources): " + ", ".join(missing))
    if output.resolve() in {ROOT, ROOT / "licenses", CACHE}:
        raise ValueError("Generated output must not replace project or cached license sources")
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)
    manifest = {"target": "", "scope": "macOS default and all optional features", "packages": []}
    text = ["# Dependency licenses and source availability", "",
            "Generated from locked Cargo metadata for macOS, covering default and all optional features.",
            "Runtime/build roles follow resolved dependency edges; proc macros and their dependencies are build tools.",
            "Development-only dependency edges and unreachable metadata packages are excluded.",
            "A role describes dependency reachability, not a claim that every feature is present in this executable.",
            "License expressions are upstream metadata; the actual accompanying texts govern. No missing text is replaced by an SPDX label.",
            "", "MPL-2.0 dependencies: unmodified source archives accompany these notices in `sources/`.",
            "These source archives remain available under MPL-2.0, with their original notices and license unchanged.",
            "option-ext is reachable at runtime. cbindgen is build-only; it is not declared a linked application dependency.", ""]
    default_nodes = {n["id"]: n for n in default["resolve"]["nodes"]}
    all_nodes = {n["id"]: n for n in complete["resolve"]["nodes"]}
    for package in packages:
        key = f"{package['name']}-{package['version']}"
        info = upstream(package)
        cached = index.get(key)
        contents = local_licenses(package)
        if not contents:
            info = cached
            contents = []
            for record in cached["files"]:
                data = (CACHE / key / record["path"]).read_bytes()
                if digest(data) != record["sha256"]:
                    raise ValueError(f"Cached license hash mismatch: {key}/{record['path']}")
                contents.append((record["path"], data))
        archive_url = (f"https://crates.io/api/v1/crates/{package['name']}/{package['version']}/download"
                       if package.get("source", "") and package["source"].startswith("registry+") else None)
        upstream_url = (f"{info['repository']}/tree/{info['revision']}/{info['path'] or ''}".rstrip("/")
                        if info["repository"] and info["revision"] else package.get("repository"))
        source_url = archive_url or upstream_url
        record = {"name": package["name"], "version": package["version"],
                  "license_expression": package.get("license"), "license_file": package.get("license_file"),
                  "authors": package.get("authors", []),
                  "roles": sorted(all_roles[package["id"]]),
                  "default_roles": sorted(default_roles.get(package["id"], [])),
                  "default_features": default_nodes.get(package["id"], {}).get("features", []),
                  "all_optional_features": all_nodes[package["id"]]["features"],
                  "feature_scope": "default" if package["id"] in default_roles else "optional",
                  "source_url": source_url, "crate_archive_url": archive_url,
                  "upstream_source_url": upstream_url,
                  "revision_matches_package": info.get("revision_matches_package", False), "files": []}
        text.extend([f"## {package['name']} {package['version']}", "",
                     f"Roles: {', '.join(record['roles'])}; feature scope: {record['feature_scope']}.",
                     f"License metadata: {package.get('license') or 'license-file: ' + str(package.get('license_file'))}.",
                     f"Source: {source_url or archive_url or 'local project source'}."])
        if archive_url:
            text.append(f"Exact published crate source: {archive_url}.")
        elif Path(package["manifest_path"]).parent.parent == ROOT / "vendor":
            record["source_scope"] = "locally modified fork; upstream URL identifies the original pinned source"
            text.append("This package is a locally modified fork. The upstream URL identifies its original pinned source; local changes are described in the project NOTICE and vendor provenance documentation.")
        if cached and not cached["revision_matches_package"]:
            text.append("Its license text is retained from the recorded immutable upstream revision; that revision is not asserted to be the exact published crate commit. The published crate archive remains the exact source reference.")
        if cached and cached.get("license_sourcing_note"):
            text.append(cached["license_sourcing_note"])
            record["license_sourcing_note"] = cached["license_sourcing_note"]
        if cached and cached.get("selected_license"):
            record["selected_license"] = cached["selected_license"]
        for name, content in contents:
            destination = output / "licenses" / key / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
            relative = str(destination.relative_to(output))
            source_record = next((entry for entry in cached["files"] if entry["path"] == name), None) if cached else None
            if source_record:
                source_license_url = source_record["url"]
            elif archive_url:
                source_license_url = f"https://docs.rs/crate/{package['name']}/{package['version']}/source/{quote(name)}"
            elif Path(package["manifest_path"]).parent.parent == ROOT / "vendor":
                upstream_path = "LICENSE-APACHE" if name == "LICENSE-APACHE" else f"{info['path']}/{name}"
                source_license_url = f"{info['repository']}/blob/{info['revision']}/{upstream_path}"
            else:
                source_license_url = package.get("repository")
            record["files"].append({"path": relative, "sha256": digest(content), "source_url": source_license_url})
            text.append(f"License/notice text: [{name}]({quote(relative)}).")
        if package.get("license") == "MPL-2.0":
            sources = output / "sources"
            sources.mkdir(exist_ok=True)
            destination = sources / f"{key}.tar.gz"
            write_source_archive(package, destination)
            record["included_source"] = str(destination.relative_to(output))
            text.append(f"Unmodified MPL source included: [{destination.name}](sources/{destination.name}).")
        text.append("")
        manifest["packages"].append(record)
    (output / "THIRD-PARTY-NOTICES.md").write_text("\n".join(text))
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", type=Path, default=ROOT / "Cargo.toml")
    parser.add_argument("--target", default="aarch64-apple-darwin")
    parser.add_argument("--output", type=Path, default=ROOT / "target/licenses")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--fetch-missing", action="store_true")
    parser.add_argument("--project-only", action="store_true", help="Native Swift reference: include project license and NOTICE only")
    args = parser.parse_args()
    if args.project_only:
        args.output.mkdir(parents=True, exist_ok=True)
        for name in ["LICENSE", "NOTICE"]:
            shutil.copy2(ROOT / name, args.output / name)
        for path in ROOT.joinpath("licenses").glob("*.txt"):
            (args.output / "licenses").mkdir(exist_ok=True)
            shutil.copy2(path, args.output / "licenses" / path.name)
        return
    default = metadata(args.manifest_path, args.target, False, args.offline)
    complete = metadata(args.manifest_path, args.target, True, args.offline)
    index_path = CACHE / "SOURCES.json"
    index = json.loads(index_path.read_text()) if index_path.is_file() else {}
    if args.fetch_missing:
        reachable = dependency_roles(complete)
        fetch_missing([p for p in complete["packages"] if p["id"] in reachable], index)
    manifest = generate(default, complete, args.output, index)
    manifest["target"] = args.target
    (args.output / "SOURCE-LINKS.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"Wrote actual license texts for {len(manifest['packages'])} packages to {args.output}")


if __name__ == "__main__":
    main()
