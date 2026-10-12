This source comes from `longbridge/gpui-kit` commit
`0bbd9870ce420af9feacbf948b21d3824d82dc0b`, crate `gpui-component` 0.7.1,
under its unchanged Apache-2.0 license. The workspace manifest is normalized
for the local dependency graph; development examples are excluded.

The modified Rust files carry prominent change notices as required by Apache
2.0 section 4(b). They are `src/list/list_item.rs`, `src/table/state.rs` and
`src/diff/mod.rs`. All original Apache-2.0 rights, license text, and upstream
attribution remain.

List items, table rows and diff gutters treat a fully transparent hover token
(`list_hover`, `table_hover`) as "no hover": they skip the hover background
refinement, so the resting background (a selection or stripe fill) persists
under the pointer. Any other hover color behaves as before. gpui-macos sets
these tokens transparent because native rows have no hover.
