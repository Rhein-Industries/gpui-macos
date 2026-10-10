#!/usr/bin/env python3
"""Regression checks for notice graph scope and the generated license/source payload."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("notices", Path(__file__).with_name("generate-notices.py"))
notices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notices)


class NoticeTests(unittest.TestCase):
    def test_archived_license_pointer_and_spdx_only_file_do_not_count_as_license_text(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package = {"manifest_path": str(root / "Cargo.toml"), "license": "MIT OR Apache-2.0"}
            (root / "LICENSE").write_text("../LICENSE")
            (root / "LICENSE-SPDX").write_text(package["license"])
            self.assertEqual(notices.local_licenses(package), [])
            (root / "LICENSE-MIT").write_bytes((notices.ROOT / "licenses/MIT-standard-terms.txt").read_bytes())
            self.assertEqual([name for name, _ in notices.local_licenses(package)], ["LICENSE-MIT"])

    def test_cached_license_hash_mismatch_fails_generation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            package_dir = root / "package"
            package_dir.mkdir()
            package = {"id": "example", "name": "example", "version": "1.0.0",
                       "manifest_path": str(package_dir / "Cargo.toml"), "targets": [{"kind": ["lib"]}]}
            data = {"packages": [package], "resolve": {"root": "example", "nodes": [
                {"id": "example", "deps": [], "features": []}]}}
            cache = root / "cache"
            (cache / "example-1.0.0").mkdir(parents=True)
            (cache / "example-1.0.0/LICENSE").write_text("altered cached source")
            index = {"example-1.0.0": {"repository": "https://example.org/source", "revision": "abc",
                                     "path": "", "files": [{"path": "LICENSE", "sha256": "invalid"}]}}
            with patch.object(notices, "CACHE", cache):
                with self.assertRaisesRegex(ValueError, "Cached license hash mismatch"):
                    notices.generate(data, data, root / "output", index)

    def test_dependency_roles_exclude_dev_and_unreachable_and_preserve_build_context(self):
        def edge(ident, kind=None):
            return {"pkg": ident, "dep_kinds": [{"kind": kind, "target": None}]}
        data = {"packages": [{"id": ident, "targets": [{"kind": ["proc-macro" if ident == "macro" else "lib"]}]}
                             for ident in ["app", "runtime", "builder", "macro", "shared", "dev", "unreachable"]],
                "resolve": {"root": "app", "nodes": [
                    {"id": "app", "deps": [edge("runtime"), edge("builder", "build"), edge("macro"), edge("dev", "dev")]},
                    {"id": "runtime", "deps": [edge("shared")]},
                    {"id": "builder", "deps": [edge("shared")]},
                    {"id": "macro", "deps": [edge("shared")]},
                    *[{"id": ident, "deps": []} for ident in ["shared", "dev", "unreachable"]],
                ]}}
        roles = notices.dependency_roles(data)
        self.assertEqual(roles["runtime"], {"runtime"})
        self.assertEqual(roles["builder"], {"build"})
        self.assertEqual(roles["macro"], {"build"})
        self.assertEqual(roles["shared"], {"runtime", "build"})
        self.assertNotIn("dev", roles)
        self.assertNotIn("unreachable", roles)

    def test_generated_payload_has_real_texts_verified_hashes_and_mpl_source(self):
        output = notices.ROOT / "target/licenses"
        self.assertTrue((output / "SOURCE-LINKS.json").is_file(), "Run generate-notices.py --offline first")
        manifest = json.loads((output / "SOURCE-LINKS.json").read_text())
        self.assertEqual(manifest["target"], "aarch64-apple-darwin")
        packages = {(p["name"], p["version"]): p for p in manifest["packages"]}
        self.assertGreater(len(packages), 100)
        for package in packages.values():
            self.assertTrue(package["files"], package["name"])
            for record in package["files"]:
                text = (output / record["path"]).read_bytes()
                self.assertGreater(len(text), 30, record["path"])
                self.assertEqual(hashlib.sha256(text).hexdigest(), record["sha256"])
        option = packages[("option-ext", "0.2.0")]
        cbindgen = packages[("cbindgen", "0.28.0")]
        self.assertIn("runtime", option["roles"])
        self.assertEqual(cbindgen["roles"], ["build"])
        graphql = packages[("tree-sitter-graphql", "0.1.0")]
        self.assertEqual(graphql["license_file"], "LICENSE")
        self.assertTrue(any("Permission is hereby granted" in (output / f["path"]).read_text(errors="replace")
                            for f in graphql["files"]))
        for package in [option, cbindgen]:
            with tarfile.open(output / package["included_source"], "r:gz") as archive:
                self.assertTrue(any(member.name.endswith("src/lib.rs") for member in archive.getmembers()))
                self.assertTrue(any("LICENSE" in member.name for member in archive.getmembers()))

    def test_project_only_payload_includes_project_license_and_original_attributions(self):
        with tempfile.TemporaryDirectory() as directory:
            import subprocess
            subprocess.run(["python3", str(Path(__file__).with_name("generate-notices.py")),
                            "--project-only", "--output", directory], check=True)
            self.assertIn("Jason Lee", (Path(directory) / "LICENSE").read_text())
            self.assertIn("src/chart", (Path(directory) / "NOTICE").read_text())
            self.assertTrue((Path(directory) / "licenses/gpui-omarchy-MIT.txt").is_file())


if __name__ == "__main__":
    unittest.main()
