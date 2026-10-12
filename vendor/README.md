# Pinned GPUI Kit sources

These five crates come from [longbridge/gpui-kit](https://github.com/longbridge/gpui-kit)
at `0bbd9870ce420af9feacbf948b21d3824d82dc0b` (0.7.1). Each crate includes its
upstream Apache-2.0 license; the assets crate also includes Lucide's license.
Examples and build outputs are excluded.

For registry publication, the packages are independently named
`gpui-macos-kit`, `gpui-macos-base`, `gpui-macos-component`,
`gpui-macos-component-macros`, and `gpui-macos-assets`, all at
`0.7.1-macos.1`. Their original Rust library names and dependency aliases are
preserved. Every internal dependency requires that exact fork version;
package metadata records the original 0.7.1 source and revision.
See [publishing](../docs/publishing.md) for local archive verification and
dependency-ordered release commands.

Their manifests resolve workspace values locally and use sibling paths for
the five internal dependencies. This preserves the pinned GPUI Fast feature
graph and makes a path dependency on gpui-macos work without a consumer patch
table. `python3 scripts/vendor-kit.py <pinned-checkout>` regenerates manifests
and copies missing crates, while preserving existing source modifications.
Normalization also omits Base's unused `reqwest_client` development dependency
(`gpui-pre-reqwest-client`); its only Rust use is in an upstream native example
that is excluded here. Regenerating the manifests preserves that omission.
The assets fork enables Rust Embed's `debug-embed` feature. Both debug and
release binaries embed their fallback SVGs instead of reading the source
checkout at runtime, so copied app bundles do not depend on Documents access
or a developer's original path. Manifest regeneration preserves this feature.

Local behavioral changes in GPUI Base include:

- Checkbox, Radio, Switch, Toggle, and Tab render-time `ControlState` children via
  `child_with_state`; `child_with_disabled` remains an adapter. Toggle and Tab
  also take render-time `default_styles` beneath caller styles. Focus visibility
  is reserved (currently false), with existing focus behavior preserved.
- Disabled buttons report AccessKit's disabled flag on their existing node,
  while preserving their role, label and input/focus suppression.
- Secure inputs accept a configurable mask character. Rendering, selection
  and IME offsets use its UTF-8 length, and copy/cut still protect secrets.
- Repeatable popup content factories allow each popup window to construct
  its own frame-local elements. Existing one-shot content builders retain
  their inline presentation.
- Popup presenters and explicit viewport bounds let native submenus extend
  outside their owner panel while preserving normal inline positioning.
- Popovers track their original owner window for focus restoration and native
  command dispatch. Interactive triggers support accessibility activation
  without repeating the ordinary pointer-down toggle.

The component proc-macro crate's `src/crate_path.rs` also recognizes the
independently named fork packages and the public gpui-macos facade, so
`IntoPlot` resolves correctly when consumers rename either dependency.

The Mac-specific font, geometry, materials and native popup window integration
live in gpui-macos. No global Cargo cache or registry sources are modified.
