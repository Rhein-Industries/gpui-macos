# Changelog

## Unreleased — M1 native foundation

- Add spec-cited native text roles and `NativeTextStyle::native_text_style`,
  with absolute, zoom-scaled line advances; roots use Body and Title3 advances 20 pt.
- Project the native Base type/radius scales and repair later Kit overwrites with
  one retained, idempotent observer, preserving Kit scrollbar and resize settings.
- Consolidate disabled values into native tokens, including tertiary labels,
  switch tracks, check marks, and Bezel disabled paint for prominent buttons that are both inactive and disabled.
- Add Base `ControlState` / `child_with_state` to Checkbox, Radio and Switch,
  retaining `child_with_disabled` as an adapter; focus visibility remains reserved.
- Add a production token-literal ratchet and native text/projection/state tests.
- Visible typography changes: title bar 13 → 15 pt semibold; markdown headings
  20/16/14/12 → 22/17/15/13 pt (the 13 pt heading is bold); textarea and chat
  bubbles use 16 pt advance instead of 1.4× their font size.
- Light disabled control labels use 66/255 ink alpha instead of 63/255.
- Keycaps retain their tight font-size advance (12 pt on macOS); dialog titles
  change from 16 to 15 pt semibold and panel titles from 14 to 15 pt semibold,
  both with a 20 pt advance.
