# Publishing gpui macos to crates io

The release consists of `gpui-macos` and five independently named forks of GPUI Kit. Local development uses sibling paths; published manifests use exact registry versions. Applications continue importing `gpui_macos::gpui_kit` and `gpui_macos::kit`, with no consumer patch table.

The initial `gpui-macos` 0.1.0 release and all five fork packages are published on crates.io. The 0.1.1 patch release updates only the root library; unchanged fork packages retain their existing versions. The packaging verification workflow does not upload packages.

Use **Python 3.12 or newer** and the repository's tested **Rust 1.98.1** toolchain for the packaging helpers. The Python version bundled with macOS may be too old: the scripts use `tomllib` and safe filtered archive extraction. The verifier preserves the active repository toolchain when compiling external consumers.

## Package names and provenance

| Registry package | Version | Original package | Rust library name |
| --- | --- | --- | --- |
| `gpui-macos` | `0.1.1` | This library | `gpui_macos` |
| `gpui-macos-kit` | `0.7.1-macos.1` | `gpui-kit` | `gpui_kit` |
| `gpui-macos-base` | `0.7.1-macos.1` | `gpui-base` | `gpui_base` |
| `gpui-macos-component` | `0.7.1-macos.1` | `gpui-component` | `gpui_component` |
| `gpui-macos-component-macros` | `0.7.1-macos.1` | `gpui-component-macros` | `gpui_component_macros` |
| `gpui-macos-assets` | `0.7.1-macos.1` | `gpui-kit-assets` | `gpui_kit_assets` |

The forks retain upstream GPUI Kit **0.7.1**, pinned at `0bbd9870ce420af9feacbf948b21d3824d82dc0b`, and its Apache-2.0 provenance. Package metadata records the original name, version, revision and source. The fork prerelease suffix identifies this distribution; every internal dependency requires `=0.7.1-macos.1`. Exact requirements keep the component facade, base behavior, macros and assets on the same fork release. Build metadata is avoided because [Cargo ignores it in version requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-metadata).

The root package declares `MIT AND Apache-2.0`: its implementation is MIT, and the packaged upstream export fixture retains Apache-2.0. Fork source remains Apache-2.0. The assets package declares `Apache-2.0 AND ISC AND MIT`, accounting for Lucide and Feather-derived artwork in its included `LICENSE-LUCIDE`. Keep the complete source licenses and notices in each archive.

Dependency keys and explicit library names preserve existing Rust paths. The proc-macro resolver also recognizes the real fork package names and the `gpui-macos` facade, including renamed consumer dependencies. Source modifications and licenses are recorded in [vendor provenance](../vendor/README.md).

The assets fork enables `rust-embed/debug-embed`. Fallback SVGs are compiled into both debug and release binaries; app bundles do not read their developer's source checkout at runtime. This preserves bundled artwork when the checkout is moved, removed, or inaccessible under macOS folder privacy controls.

Registry metadata links all six packages to [Rhein-Industries/gpui-macos](https://github.com/Rhein-Industries/gpui-macos), while fork metadata retains its pinned upstream source. Hosted API documentation builds only for Linux on docs.rs; native macOS documentation can be generated locally with `cargo doc --no-deps` on a Mac with Xcode command-line tools. This avoids cross-compiling Apple build scripts on docs.rs's Linux hosts.

All six packages belong to one workspace and share the checked-in Cargo.lock. The root remains the default workspace member, so ordinary checks and tests retain their application scope. Dependency-ordered package and publish commands use that same tested lockfile. The packaged export fixture under `docs/` allows archive tests to retain the upstream inventory audit even though Cargo excludes nested vendor packages from the root archive; checkout tests additionally compare that fixture with the actual vendored source.

Regenerate the fork manifests from a clean upstream checkout with:

```sh
python3 -B scripts/vendor-kit.py /path/to/the/pinned/gpui-kit-checkout
cargo check --all-targets --locked
```

The generator checks the checkout's commit against the pin; it does not attest to uncommitted working-tree changes. Keep the upstream input clean so generated manifests and the export fixture retain their recorded provenance.

## Verify packages without publishing

```sh
python3 -B scripts/check-packages.py
python3 -B scripts/check-packages.py --optional
```

The helper copies sources to a temporary staging directory, packages all six crates in dependency order with the checked-in lockfile, and adds their actual archives and normalized manifests to an isolated local registry. Existing crates.io cache entries and archives supply third-party dependencies. It then compiles two external consumers using registry dependencies: a renamed `gpui-macos` dependency and a renamed fork Kit dependency. Both derive `IntoPlot` and declare GPUI actions, exercising proc-macro path resolution. The root consumer proves that every fork resolves from a registry, rather than a checkout or patch table. The unpacked root archive also runs its inventory and interpretation contract tests. `--optional` additionally checks speech and the Tree-sitter language bundle.

The helper operates offline, does not read registry credentials or modify cached crate sources and archives, and performs no upload. It seeds the temporary registry from Cargo's crates.io **sparse-index cache**; a Git-protocol index alone is insufficient. A cold machine must first fetch the locked dependency graph with `cargo fetch --locked` using the sparse protocol. Missing optional dependencies require a normal optional-feature build before an offline check. Archives remain in `target/package/`; `registry-verification.json` records their checksums, dependency metadata, consumer checks and upload status. The temporary registry is removed after verification.

This local registry check validates packaging and compilation. crates.io availability, account ownership, registry policy and final upload acceptance are verified during the authorized release.

## Release order

Review the source changes, license notices and final archives, commit the release, then print the current commands:

```sh
python3 -B scripts/release-plan.py
```

For new fork versions, publish the macro and assets packages first, then Base, Component, Kit, and finally `gpui-macos`. Skip package versions that are already published and unchanged. Publish one at a time and wait until crates.io indexes each exact version before publishing its dependents. The printed commands use `cargo publish --locked --registry crates-io`; the script itself does not execute them. Authentication and upload are separate release actions.

After the fork versions are on crates.io, a normal `cargo publish --dry-run --locked` for `gpui-macos` can verify against the public registry. Before then, a public-registry dry run cannot resolve the unpublished fork names; use the local registry check above. Cargo's [multiple locations guidance](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations) explains how path-plus-version dependencies become registry dependencies when packaged.
