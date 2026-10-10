This source comes from `longbridge/gpui-kit` commit
`0bbd9870ce420af9feacbf948b21d3824d82dc0b`, crate `gpui-base` 0.7.1,
under its unchanged Apache-2.0 license. The workspace manifest is normalized
for the local dependency graph; development examples are excluded.

The modified Rust files carry prominent change notices as required by Apache
2.0 section 4(b). They are `src/button.rs`, `src/popup.rs`, `src/lib.rs`,
`src/positioner.rs`, `src/popover.rs`, `src/hover_card.rs`,
`src/input/base/element.rs`, and `src/input/base/state.rs`. Popup-related
changes add repeatable content factories, presenter callbacks, explicit
viewport positioning, owner-window focus/command restoration, and
keyboard/accessibility activation without duplicating pointer-down toggles.
All original Apache-2.0 rights, license text, and upstream attribution remain.

The local editor patch adds `InputState::masked_character`,
`InputState::mask_character`, and `InputState::set_mask_character`.
The upstream bullet remains the default. Control characters and whitespace
are rejected. Painting, caret/selection offsets, IME run boundaries, and
display-to-buffer hit testing use the chosen character's UTF-8 byte width.
Copy, Cut, accessibility values, original text, and undo behavior keep the
existing editor implementation.

The macOS input adapter uses a filled circle in Helvetica Neue at 12.8 points
for masked, nonempty editor content. At regular size its advance is 8 points
and its ink diameter approximately 5.5 points, matching an actual offscreen
AppKit secure cell. Placeholders, affixes, and revealed text keep their normal
font. Measurements can be reproduced with
`tools/native-reference/measure-secure-mask.swift`, using the installed macOS
SDK explicitly with `swiftc -swift-version 6 -sdk <SDK path>`.

Buttons now expose their disabled state to AccessKit. A private forwarding
element delegates the original element's identity, source, layout, prepaint,
paint, role and synthetic accessibility children directly, then adds the
disabled flag in `write_a11y_info`. It creates no extra layout or accessibility
node and preserves the existing styles, focus and activation handlers. The
button accessibility probe checks the flag together with label/role and the
absence of a Click action. Pointer/keyboard tests still check that disabled
buttons suppress their own and their parent's activation; these are headless
tests, not injected OS accessibility actions.
