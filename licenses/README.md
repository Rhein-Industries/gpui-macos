# License texts and dependency notices

`gpui-omarchy-MIT.txt` is the unchanged license from
[`huacnlee/gpui-omarchy` at e63ff32b73299d4f8abd8401af5d04924b2a9d18](https://github.com/huacnlee/gpui-omarchy/blob/e63ff32b73299d4f8abd8401af5d04924b2a9d18/LICENSE).
It covers the Omarchy-derived adapters, gallery, charts, and plot tooltip.
`gpui-kit-Apache-2.0.txt` preserves the unchanged Longbridge license, including
its copyright notice, for the conservative chart/plot attribution described
in the repository NOTICE. This does not assert a copying direction between
the two pinned snapshots or remove the MIT attribution.

The five GPUI Kit forks retain their upstream Apache license files. The assets
fork also retains the complete `LICENSE-LUCIDE`: Lucide's ISC notice and
Feather's MIT notice must both be preserved. Modified source files identify
their changes; normalized manifests identify their local normalization.
The component-macro crate additionally identifies its renamed-package lookup
change in `src/crate_path.rs`.

Run `python3 scripts/generate-notices.py --offline` to generate application
notices in `target/licenses`. It follows the locked Cargo dependency graph
for `aarch64-apple-darwin`, default features and all optional features, and
excludes development-only and unreachable packages. Runtime and build/proc
macro paths are recorded separately. The output includes actual upstream
license/NOTICE files, package source links, SHA-256 hashes, and source archives
for MPL-2.0 dependencies. `option-ext` is runtime-reachable; `cbindgen` is a
build tool and is not described as linked into the application.

Some published crate archives omit their license text. `dependencies/` caches
those texts from each crate's declared official repository. `SOURCES.json`
records immutable commits, source URLs, hashes, and whether the source commit
was present in the exact crate's VCS metadata. Older crates without VCS data
use a matching release tag when available; otherwise the recorded upstream
commit is explicitly distinguished from the release source. Exact published
crate archive links remain in the generated notice index.

Nine older crates declare MIT (sometimes as an alternative to Apache) but have
no license text in their published archive or retrievable declared repository.
Their reviewed records contain the full standard MIT grant and disclaimer,
the exact published manifest's license declaration, declared authors, and any
copyright notices found in their Rust source. MIT is selected for the
dual-licensed records. The source of the standard terms is
[`spdx/license-list-data` at 31ba1a50e5397e00a304dbadc76531740e89ee48](https://github.com/spdx/license-list-data/blob/31ba1a50e5397e00a304dbadc76531740e89ee48/text/MIT.txt),
also preserved in `MIT-standard-terms.txt`. Template holder/year placeholders
are omitted; authors are identified as declared authors, not invented rights
holders. These records explicitly describe this sourcing instead of presenting
the standard terms as an upstream license file. Newly missing texts still fail
generation and require review.

After a dependency update, run the generator with `--fetch-missing` to retrieve
missing texts, review the resulting source records, and regenerate offline.
Generation fails if any reachable package lacks a license text or a cached
text's hash changes. SPDX metadata alone never counts as a collected license.
Bundle scripts copy the completed notice directory into
`Contents/Resources/Licenses` before signing the application.
