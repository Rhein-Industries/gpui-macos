//! Role × state paint from `docs/native-spec.md` §6.2.
//!
//! Every arm is a measured value (or an explicit fallback for a cell marked
//! **O**). States never compose by multiplication (§6): disabled, pressed and
//! inactive-window combinations each have their own arm.
use crate::Theme;
use crate::tokens::{
    disabled_bezel, disabled_control_fill, disabled_control_mark, disabled_switch_off, ink,
};
use gpui_kit::{Hsla, Window, rgb, transparent_black};

/// Which tinted family a prominent bezel uses (§6.2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tint {
    /// `prominent_tint`, the theme accent.
    Accent,
    /// `prominent_red`, the theme danger hue (`.tint(.red)`, C5).
    Red,
}

/// The paint family of a control part. `Custom` opts out of the resolver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaintRole {
    /// The caller paints every state itself.
    Custom,
    /// Bordered buttons, pop-ups and toggle buttons that are off (§6.2.1).
    Bezel,
    /// `.borderedProminent`, accent or red (§6.2.3).
    Prominent(Tint),
    /// `.borderless` text and icon buttons (§6.2.10 BorderlessLabel, C4).
    Borderless,
    /// A toggle button that is on (§6.2.4).
    ToggleOn,
    /// Segmented pickers and segmented tab strips (§6.2.5, C6).
    Segment { selected: bool },
    /// Checkbox and radio wells that are off (§6.2.6).
    CheckOff,
    /// Checkbox and radio wells that are on or mixed, and their mark (§6.2.6).
    CheckOn,
    /// The switch track (§6.2.7).
    SwitchTrack { on: bool },
    /// The state-aware switch knob (§6.2.7, C19).
    SwitchKnob { on: bool },
    /// Slider track, fill and knob (§6.2.8, C18).
    Slider,
    /// Determinate linear progress (§6.2.9).
    Progress,
    /// Links and `.link` buttons (§6.2.10 LinkLabel).
    Link,
    /// The label beside a checkbox, radio or switch, and text-only rows (§6.2.10).
    ControlLabel,
}

/// The painted part of a role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    /// The bezel, well, track fill or selected segment.
    Fill,
    /// Label ink, or the check mark / radio dot of `CheckOn`.
    Label,
    /// The unfilled track of a slider or segmented control.
    Track,
    /// The slider knob.
    Knob,
}

/// Render-time control state (§6 columns AN, AD, IN, ID, P).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct States {
    pub disabled: bool,
    pub pressed: bool,
    pub window_active: bool,
}

impl States {
    /// Active, enabled and at rest (column AN).
    pub(crate) const fn active() -> Self {
        Self {
            disabled: false,
            pressed: false,
            window_active: true,
        }
    }

    /// Enabled and at rest in the given window, which may be inactive.
    pub(crate) fn in_window(window: &Window) -> Self {
        Self {
            window_active: window.is_window_active(),
            ..Self::active()
        }
    }

    pub(crate) const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) const fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    #[cfg(test)]
    pub(crate) const fn window_active(mut self, active: bool) -> Self {
        self.window_active = active;
        self
    }
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Light / dark pair.
fn ld(theme: &Theme, light: Hsla, dark: Hsla) -> Hsla {
    if theme.is_dark() { dark } else { light }
}

/// Pressed accent (§6.6, C1, C32, plan D7): measured targets for the default
/// accent. Custom tints mix 10% toward black in light and stay unchanged in
/// dark, because the dark formula is not fitted.
fn pressed_tint(theme: &Theme, tint: Hsla, measured: Option<(u32, u32)>) -> Hsla {
    match (
        measured.filter(|_| theme.has_default_accent()),
        theme.is_dark(),
    ) {
        (Some((light, _)), false) => color(light),
        (Some((_, dark)), true) => color(dark),
        (None, false) => tint.blend(Hsla::black().opacity(0.1)),
        (None, true) => tint,
    }
}

/// The paint of `part` of `role` in `states`, spec §6.2.
pub(crate) fn paint(role: PaintRole, part: Part, states: States, theme: &Theme) -> Hsla {
    let States {
        disabled,
        pressed,
        window_active: active,
    } = states;
    let dark = theme.is_dark();
    let ink = ink(theme);
    let label = theme.foreground;
    let white = theme.on_accent;
    let indicator = theme.indicator_accent();
    match (role, part) {
        (PaintRole::Custom, _) => transparent_black(),

        // §6.2.1 Bezel: inactive equals active (C14).
        (PaintRole::Bezel, Part::Fill) => match (disabled, pressed) {
            (true, _) => disabled_bezel(theme).0,
            (false, true) => ink.opacity(0.165),
            (false, false) => theme.bezel_fill(),
        },
        (PaintRole::Bezel, Part::Label) => match disabled {
            true => disabled_bezel(theme).1,
            false => label,
        },

        // §6.2.3 Prominent: an inactive window shows Bezel; inactive and
        // disabled shows Bezel disabled.
        (PaintRole::Prominent(_), Part::Fill | Part::Label) if !active => {
            paint(PaintRole::Bezel, part, states, theme)
        }
        (PaintRole::Prominent(tint), Part::Fill) => {
            // Red pressed is not measured (O); it uses the custom-tint rule.
            let (tint, measured) = match tint {
                Tint::Accent => (theme.accent, Some((0x036ee6, 0x2d87ff))),
                Tint::Red => (theme.danger, None),
            };
            match (disabled, pressed) {
                (true, _) => tint.opacity(crate::tokens::DISABLED_TINT_OPACITY),
                (false, true) => pressed_tint(theme, tint, measured),
                (false, false) => tint,
            }
        }
        (PaintRole::Prominent(_), Part::Label) => match disabled {
            true => white.opacity(crate::tokens::DISABLED_TINT_LABEL_OPACITY),
            false => white,
        },

        // §6.2.10 BorderlessLabel (C4): no bezel; inactive equals active.
        (PaintRole::Borderless, Part::Fill) => transparent_black(),
        (PaintRole::Borderless, Part::Label) => match (disabled, pressed) {
            (true, _) => ink.opacity(0.25),
            (false, true) => ink.opacity(0.85),
            (false, false) => theme.secondary,
        },

        // §6.2.4 ToggleOn.
        (PaintRole::ToggleOn, Part::Fill) => match (active, disabled, pressed) {
            (true, false, true) => pressed_tint(theme, indicator, Some((0x036ce1, 0x338aff))),
            (true, false, false) => indicator,
            (true, true, _) => indicator.opacity(0.70),
            (false, false, _) => ink.opacity(0.125),
            (false, true, _) => ink.opacity(0.067),
        },
        (PaintRole::ToggleOn, Part::Label) => match (active, disabled) {
            (true, false) => white,
            (true, true) => white.opacity(0.50),
            (false, false) => label,
            (false, true) => label.opacity(0.31),
        },

        // §6.2.5 Segmented (C6). The track never changes.
        (PaintRole::Segment { .. }, Part::Track) => theme.bezel_fill(),
        // Pressing an unselected segment turns it accent (§6.2.5 P).
        (PaintRole::Segment { selected: false }, Part::Fill) => match (active, disabled, pressed) {
            (true, false, true) => indicator,
            _ => transparent_black(),
        },
        (PaintRole::Segment { selected: true }, Part::Fill) => match (active, disabled) {
            (true, false) => indicator,
            (true, true) => indicator.opacity(0.71),
            (false, false) => ink.opacity(0.125),
            // Seen #DCDCDC / #3E3E3E over the track, which is ink @0.067.
            (false, true) => ink.opacity(0.067),
        },
        (PaintRole::Segment { selected }, Part::Label) => match (selected, active, disabled) {
            (true, true, false) => white,
            // The pressed label over the accent is not measured (O): white.
            (false, true, false) if pressed => white,
            // Active disabled selected label is not measured (O): ToggleOn AD.
            (true, true, true) => white.opacity(0.50),
            (_, _, false) => label,
            // Inactive disabled aggregate ×0.35; active disabled unselected is O.
            (_, _, true) => label.opacity(0.35),
        },

        // §6.2.6 CheckOff: inactive equals active.
        (PaintRole::CheckOff, Part::Fill) => match (disabled, pressed) {
            (true, _) => disabled_control_fill(theme, false),
            (false, true) => ink.opacity(0.188),
            (false, false) => ink.opacity(0.098),
        },
        // §6.2.6 CheckOn: inactive and disabled is ink @0.075, not IN × AD.
        (PaintRole::CheckOn, Part::Fill) => match (active, disabled, pressed) {
            (true, false, true) => pressed_tint(theme, indicator, Some((0x036ce1, 0x338aff))),
            (true, false, false) => indicator,
            (true, true, _) => disabled_control_fill(theme, true),
            (false, false, _) => ink.opacity(0.141),
            (false, true, _) => disabled_control_fill(theme, true),
        },
        (PaintRole::CheckOn, Part::Label) => match (active, disabled) {
            (true, false) => white,
            (true, true) => disabled_control_mark(theme),
            (false, false) => label,
            (false, true) => label.opacity(0.30),
        },

        // §6.2.7 Switch track: inactive off equals active off.
        (PaintRole::SwitchTrack { on: false }, Part::Fill) => match (disabled, pressed) {
            (true, _) => disabled_switch_off(theme),
            (false, true) => ink.opacity(0.188),
            (false, false) => ld(theme, ink.opacity(0.098), color(0xe4e4e4).opacity(0.11)),
        },
        // Inactive and disabled is L ink @0.075, D #DCDCDC @0.086.
        (PaintRole::SwitchTrack { on: true }, Part::Fill) => match (active, disabled, pressed) {
            (true, false, true) => pressed_tint(theme, indicator, Some((0x036ce0, 0x338aff))),
            (true, false, false) => indicator,
            (true, true, _) => indicator.opacity(crate::tokens::DISABLED_SWITCH_OPACITY),
            (false, false, _) => ld(theme, ink.opacity(0.141), color(0xebebeb).opacity(0.153)),
            (false, true, _) => ld(theme, ink.opacity(0.075), color(0xdcdcdc).opacity(0.086)),
        },
        // §6.2.7 knob (C19). Light knobs are opaque white in every state.
        (PaintRole::SwitchKnob { .. }, Part::Fill) if !dark => color(0xffffff),
        (PaintRole::SwitchKnob { on: false }, Part::Fill) => {
            Hsla::white().opacity(if disabled { 0.859 } else { 0.867 })
        }
        // The dark tinted knob is accent-derived; custom accents use the off knob (D7).
        (PaintRole::SwitchKnob { on: true }, Part::Fill) if !theme.has_default_accent() => {
            paint(PaintRole::SwitchKnob { on: false }, part, states, theme)
        }
        (PaintRole::SwitchKnob { on: true }, Part::Fill) => match (active, disabled) {
            (true, false) => color(0xdaecff),
            (true, true) => color(0xe4f1ff).opacity(0.954),
            (false, false) => Hsla::white().opacity(0.871),
            (false, true) => Hsla::white().opacity(0.863),
        },

        // §6.2.8 Slider (C18): only the fill changes with state. The seen fill
        // values are composites over the track; the layer alpha is
        // 1 − (1 − seen) / (1 − 0.098).
        (PaintRole::Slider, Part::Track) => ink.opacity(0.098),
        (PaintRole::Slider, Part::Fill) => match (active, disabled) {
            (true, false) => indicator,
            // Seen #DADADA / #3F3F3F, about ink 0.15 over the window.
            (true, true) => ink.opacity(0.053),
            // Seen #C5C5C5 / #515151, about ink 0.22 over the window.
            (false, false) => ink.opacity(0.143),
            // Seen #D5D5D5 / #434343, about ink 0.165 over the window.
            (false, true) => ink.opacity(0.074),
        },
        (PaintRole::Slider, Part::Knob) => ld(theme, color(0xffffff), color(0xdddddd)),

        // §6.2.9 determinate linear progress.
        (PaintRole::Progress, Part::Fill) => match active {
            true => theme.accent,
            false => color(0xaaaaaa),
        },

        // §6.2.10 LinkLabel: inactive equals active.
        (PaintRole::Link, Part::Label) => {
            let link = theme.link();
            match (disabled, pressed) {
                (true, _) => link.opacity(0.50),
                (false, true) => pressed_tint(theme, link, Some((0x004cb4, 0x6fcaff))),
                (false, false) => link,
            }
        }

        // §6.2.10 ControlLabel.
        (PaintRole::ControlLabel, Part::Label) => match disabled {
            true => theme.tertiary_label(),
            false => label,
        },

        // Parts a role does not paint.
        (
            PaintRole::Bezel
            | PaintRole::Prominent(_)
            | PaintRole::Borderless
            | PaintRole::ToggleOn
            | PaintRole::CheckOff
            | PaintRole::CheckOn
            | PaintRole::SwitchTrack { .. }
            | PaintRole::SwitchKnob { .. }
            | PaintRole::Progress
            | PaintRole::Link
            | PaintRole::ControlLabel,
            Part::Track | Part::Knob,
        )
        | (PaintRole::Segment { .. }, Part::Knob)
        | (PaintRole::CheckOff, Part::Label)
        | (PaintRole::SwitchTrack { .. } | PaintRole::SwitchKnob { .. }, Part::Label)
        | (PaintRole::Slider, Part::Label)
        | (PaintRole::Progress, Part::Label)
        | (PaintRole::Link | PaintRole::ControlLabel, Part::Fill) => transparent_black(),
    }
}

/// Source-over compositing of `top` onto a possibly translucent `bottom`.
/// `Hsla::blend` keeps the bottom alpha, which hides a same-ink overlay.
pub(crate) fn over(bottom: Hsla, top: Hsla) -> Hsla {
    let alpha = top.a + bottom.a * (1. - top.a);
    if alpha <= 0. {
        return transparent_black();
    }
    let (b, t) = (gpui_kit::Rgba::from(bottom), gpui_kit::Rgba::from(top));
    let mix = |b: f32, t: f32| (t * top.a + b * bottom.a * (1. - top.a)) / alpha;
    gpui_kit::Rgba {
        r: mix(b.r, t.r),
        g: mix(b.g, t.g),
        b: mix(b.b, t.b),
        a: alpha,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLES_WITH_STATE: [PaintRole; 4] = [
        PaintRole::Bezel,
        PaintRole::Prominent(Tint::Accent),
        PaintRole::ToggleOn,
        PaintRole::CheckOn,
    ];

    fn rest() -> States {
        States::active()
    }
    fn disabled() -> States {
        rest().disabled(true)
    }
    fn inactive() -> States {
        rest().window_active(false)
    }
    fn inactive_disabled() -> States {
        inactive().disabled(true)
    }
    fn pressed() -> States {
        rest().pressed(true)
    }

    fn assert_close(actual: Hsla, expected: Hsla, context: &str) {
        let (a, e) = (gpui_kit::Rgba::from(actual), gpui_kit::Rgba::from(expected));
        let channel = |a: f32, e: f32| (a - e).abs() * 255.;
        assert!(
            (actual.a - expected.a).abs() <= 0.01,
            "{context}: alpha {} vs {}",
            actual.a,
            expected.a
        );
        // Spec §10: colours within 2/255 when the ink is visible at all.
        if expected.a > 0. {
            for (a, e) in [(a.r, e.r), (a.g, e.g), (a.b, e.b)] {
                assert!(channel(a, e) <= 2., "{context}: {actual:?} vs {expected:?}");
            }
        }
    }

    fn ink_at(theme: &Theme, alpha: f32) -> Hsla {
        ink(theme).opacity(alpha)
    }

    #[test]
    fn bezel_matches_6_2_1() {
        for t in [Theme::light(), Theme::dark()] {
            let fill = |s| paint(PaintRole::Bezel, Part::Fill, s, &t);
            let label = |s| paint(PaintRole::Bezel, Part::Label, s, &t);
            assert_close(fill(rest()), ink_at(&t, 0.078), "AN");
            assert_close(fill(disabled()), ink_at(&t, 0.039), "AD");
            assert_close(fill(pressed()), ink_at(&t, 0.165), "P");
            assert_eq!(fill(inactive()), fill(rest()), "IN = AN (C14)");
            assert_eq!(fill(inactive_disabled()), fill(disabled()), "ID = AD");
            assert_eq!(label(rest()), t.foreground);
            let disabled_label = if t.is_dark() { 0.290 } else { 0.257 };
            assert_close(label(disabled()), ink_at(&t, disabled_label), "AD label");
            assert_eq!(label(inactive_disabled()), label(disabled()));
        }
    }

    #[test]
    fn prominent_matches_6_2_3_and_d7() {
        for t in [Theme::light(), Theme::dark()] {
            let dark = t.is_dark();
            for tint in [Tint::Accent, Tint::Red] {
                let role = PaintRole::Prominent(tint);
                let base = if tint == Tint::Accent {
                    t.accent
                } else {
                    t.danger
                };
                let fill = |s| paint(role, Part::Fill, s, &t);
                let label = |s| paint(role, Part::Label, s, &t);
                assert_eq!(fill(rest()), base);
                assert_eq!(label(rest()), t.on_accent);
                assert_close(fill(disabled()), base.opacity(0.69), "AD tint ×0.69");
                assert_close(label(disabled()), t.on_accent.opacity(0.50), "AD label");
                for part in [Part::Fill, Part::Label] {
                    assert_eq!(
                        paint(role, part, inactive(), &t),
                        paint(PaintRole::Bezel, part, rest(), &t),
                        "IN = Bezel AN"
                    );
                    assert_eq!(
                        paint(role, part, inactive_disabled(), &t),
                        paint(PaintRole::Bezel, part, disabled(), &t),
                        "ID = Bezel AD, not IN × AD"
                    );
                }
            }
            let accent_pressed = paint(
                PaintRole::Prominent(Tint::Accent),
                Part::Fill,
                pressed(),
                &t,
            );
            assert_close(
                accent_pressed,
                color(if dark { 0x2d87ff } else { 0x036ee6 }),
                "default accent pressed (C1)",
            );
            let custom = t.clone().with_accent(color(0x34a853));
            let custom_pressed = paint(
                PaintRole::Prominent(Tint::Accent),
                Part::Fill,
                pressed(),
                &custom,
            );
            if dark {
                assert_eq!(
                    custom_pressed, custom.accent,
                    "dark custom pressed unchanged (C32)"
                );
            } else {
                assert_close(
                    custom_pressed,
                    custom.accent.blend(Hsla::black().opacity(0.1)),
                    "light custom pressed mixes 10% black",
                );
            }
        }
    }

    #[test]
    fn borderless_matches_6_2_10() {
        for t in [Theme::light(), Theme::dark()] {
            let label = |s| paint(PaintRole::Borderless, Part::Label, s, &t);
            let secondary = if t.is_dark() { 0.549 } else { 0.498 };
            assert_close(label(rest()), ink_at(&t, secondary), "secondary_label");
            assert_close(label(disabled()), ink_at(&t, 0.25), "AD @0.25");
            assert_close(label(inactive_disabled()), ink_at(&t, 0.25), "ID @0.25");
            assert_close(label(pressed()), ink_at(&t, 0.85), "P @0.85");
            assert_eq!(label(inactive()), label(rest()));
            assert_eq!(paint(PaintRole::Borderless, Part::Fill, rest(), &t).a, 0.);
        }
    }

    #[test]
    fn toggle_on_matches_6_2_4_without_composing() {
        for t in [Theme::light(), Theme::dark()] {
            let fill = |s| paint(PaintRole::ToggleOn, Part::Fill, s, &t);
            let label = |s| paint(PaintRole::ToggleOn, Part::Label, s, &t);
            assert_eq!(fill(rest()), t.indicator_accent());
            assert_close(
                fill(disabled()),
                t.indicator_accent().opacity(0.70),
                "AD ×0.70",
            );
            assert_close(fill(inactive()), ink_at(&t, 0.125), "IN");
            assert_close(fill(inactive_disabled()), ink_at(&t, 0.067), "ID @0.067");
            assert_close(
                fill(pressed()),
                color(if t.is_dark() { 0x338aff } else { 0x036ce1 }),
                "P",
            );
            assert_eq!(label(rest()), t.on_accent);
            assert_close(label(disabled()), t.on_accent.opacity(0.5), "AD label");
            assert_eq!(label(inactive()), t.foreground);
            assert_close(
                label(inactive_disabled()),
                t.foreground.opacity(0.31),
                "ID label",
            );
        }
    }

    #[test]
    fn segment_matches_6_2_5() {
        for t in [Theme::light(), Theme::dark()] {
            let selected = PaintRole::Segment { selected: true };
            let other = PaintRole::Segment { selected: false };
            for s in [rest(), disabled(), inactive(), inactive_disabled()] {
                assert_close(
                    paint(selected, Part::Track, s, &t),
                    ink_at(&t, 0.078),
                    "track unchanged (C6)",
                );
                assert_eq!(paint(other, Part::Fill, s, &t).a, 0.);
            }
            assert_eq!(
                paint(selected, Part::Fill, rest(), &t),
                t.indicator_accent()
            );
            assert_eq!(paint(selected, Part::Label, rest(), &t), t.on_accent);
            assert_eq!(paint(other, Part::Label, rest(), &t), t.foreground);
            assert_eq!(
                paint(other, Part::Fill, pressed(), &t),
                t.indicator_accent(),
                "pressing an unselected segment turns it accent"
            );
            assert_eq!(paint(other, Part::Label, pressed(), &t), t.on_accent);
            assert_eq!(paint(other, Part::Fill, disabled().pressed(true), &t).a, 0.);
            assert_close(
                paint(selected, Part::Fill, inactive(), &t),
                ink_at(&t, 0.125),
                "inactive selected",
            );
            assert_eq!(paint(selected, Part::Label, inactive(), &t), t.foreground);
            assert_close(
                paint(selected, Part::Fill, disabled(), &t),
                t.indicator_accent().opacity(0.71),
                "AD ×0.71",
            );
        }
    }

    #[test]
    fn checks_match_6_2_6_including_inactive_disabled() {
        for t in [Theme::light(), Theme::dark()] {
            let off = |s| paint(PaintRole::CheckOff, Part::Fill, s, &t);
            let on = |s| paint(PaintRole::CheckOn, Part::Fill, s, &t);
            let mark = |s| paint(PaintRole::CheckOn, Part::Label, s, &t);
            assert_close(off(rest()), ink_at(&t, 0.098), "off AN");
            assert_close(off(disabled()), ink_at(&t, 0.051), "off AD");
            assert_close(off(pressed()), ink_at(&t, 0.188), "off P");
            assert_eq!(off(inactive()), off(rest()));
            assert_eq!(off(inactive_disabled()), off(disabled()));
            assert_eq!(on(rest()), t.indicator_accent());
            assert_close(on(disabled()), ink_at(&t, 0.075), "on AD");
            assert_close(on(inactive()), ink_at(&t, 0.141), "on IN");
            assert_close(on(inactive_disabled()), ink_at(&t, 0.075), "on ID");
            assert_eq!(mark(rest()), t.on_accent);
            let disabled_mark = if t.is_dark() { 0.30 } else { 0.31 };
            assert_close(mark(disabled()), ink_at(&t, disabled_mark), "mark AD");
            assert_eq!(mark(inactive()), t.foreground);
            assert_close(
                mark(inactive_disabled()),
                t.foreground.opacity(0.30),
                "mark ID",
            );
        }
    }

    #[test]
    fn switch_matches_6_2_7_including_inactive_disabled() {
        let light = Theme::light();
        let dark = Theme::dark();
        let on = PaintRole::SwitchTrack { on: true };
        let off = PaintRole::SwitchTrack { on: false };
        assert_close(
            paint(on, Part::Fill, inactive_disabled(), &light),
            ink_at(&light, 0.075),
            "L ID on-track @0.075",
        );
        assert_close(
            paint(on, Part::Fill, inactive_disabled(), &dark),
            color(0xdcdcdc).opacity(0.086),
            "D ID on-track #DCDCDC @0.086",
        );
        assert_close(
            paint(on, Part::Fill, inactive(), &light),
            ink_at(&light, 0.141),
            "L IN",
        );
        assert_close(
            paint(on, Part::Fill, inactive(), &dark),
            color(0xebebeb).opacity(0.153),
            "D IN",
        );
        for t in [&light, &dark] {
            assert_eq!(paint(on, Part::Fill, rest(), t), t.indicator_accent());
            assert_close(
                paint(on, Part::Fill, disabled(), t),
                t.indicator_accent().opacity(0.71),
                "AD ×0.71",
            );
            assert_eq!(
                paint(off, Part::Fill, inactive(), t),
                paint(off, Part::Fill, rest(), t)
            );
            assert_close(
                paint(off, Part::Fill, pressed(), t),
                ink_at(t, 0.188),
                "off P",
            );
        }
        assert_close(
            paint(off, Part::Fill, rest(), &light),
            ink_at(&light, 0.098),
            "L off",
        );
        assert_close(
            paint(off, Part::Fill, rest(), &dark),
            color(0xe4e4e4).opacity(0.11),
            "D off",
        );
        assert_close(
            paint(off, Part::Fill, disabled(), &dark),
            color(0xcfcfcf).opacity(0.063),
            "D off AD",
        );
        let knob = |on, s, t: &Theme| paint(PaintRole::SwitchKnob { on }, Part::Fill, s, t);
        for s in [rest(), disabled(), inactive(), inactive_disabled()] {
            for on in [false, true] {
                assert_eq!(knob(on, s, &light), color(0xffffff), "light knob is white");
            }
        }
        assert_close(
            knob(false, rest(), &dark),
            Hsla::white().opacity(0.867),
            "D off",
        );
        assert_close(
            knob(false, disabled(), &dark),
            Hsla::white().opacity(0.859),
            "D off AD",
        );
        assert_close(knob(true, rest(), &dark), color(0xdaecff), "D on");
        assert_close(
            knob(true, disabled(), &dark),
            color(0xe4f1ff).opacity(0.954),
            "D on AD",
        );
        assert_close(
            knob(true, inactive(), &dark),
            Hsla::white().opacity(0.871),
            "D on IN",
        );
        assert_close(
            knob(true, inactive_disabled(), &dark),
            Hsla::white().opacity(0.863),
            "D on ID",
        );
        let custom = dark.clone().with_accent(color(0x34a853));
        assert_eq!(
            knob(true, rest(), &custom),
            knob(false, rest(), &custom),
            "tinted knob only for the default accent"
        );
    }

    #[test]
    fn slider_neutralizes_only_its_fill_6_2_8() {
        for t in [Theme::light(), Theme::dark()] {
            for s in [disabled(), inactive(), inactive_disabled()] {
                for part in [Part::Track, Part::Knob] {
                    assert_eq!(
                        paint(PaintRole::Slider, part, s, &t),
                        paint(PaintRole::Slider, part, rest(), &t),
                        "C18: track and knob unchanged"
                    );
                }
            }
            let window = if t.is_dark() {
                color(0x1e1e1e)
            } else {
                color(0xffffff)
            };
            let track = paint(PaintRole::Slider, Part::Track, rest(), &t);
            let seen = |s| {
                let fill = paint(PaintRole::Slider, Part::Fill, s, &t);
                window.blend(track).blend(fill)
            };
            let (ad, r#in, id) = if t.is_dark() {
                (0x3f3f3f, 0x515151, 0x434343)
            } else {
                (0xdadada, 0xc5c5c5, 0xd5d5d5)
            };
            assert_close(seen(disabled()), color(ad), "AD seen");
            assert_close(seen(inactive()), color(r#in), "IN seen");
            assert_close(seen(inactive_disabled()), color(id), "ID seen");
            assert_eq!(
                paint(PaintRole::Slider, Part::Fill, rest(), &t),
                t.indicator_accent()
            );
        }
    }

    #[test]
    fn progress_link_and_control_label_match_6_2_9_and_6_2_10() {
        for t in [Theme::light(), Theme::dark()] {
            let dark = t.is_dark();
            assert_eq!(paint(PaintRole::Progress, Part::Fill, rest(), &t), t.accent);
            assert_eq!(
                paint(PaintRole::Progress, Part::Fill, inactive(), &t),
                color(0xaaaaaa)
            );
            let link = |s| paint(PaintRole::Link, Part::Label, s, &t);
            assert_close(
                link(rest()),
                color(if dark { 0x419cff } else { 0x0068da }),
                "link",
            );
            assert_close(link(disabled()), link(rest()).opacity(0.5), "link AD");
            assert_close(
                link(pressed()),
                color(if dark { 0x6fcaff } else { 0x004cb4 }),
                "link P",
            );
            assert_eq!(link(inactive()), link(rest()));
            let control = |s| paint(PaintRole::ControlLabel, Part::Label, s, &t);
            assert_eq!(control(rest()), t.foreground);
            assert_eq!(control(disabled()), t.tertiary_label());
            assert_eq!(control(inactive_disabled()), t.tertiary_label());
        }
    }

    #[test]
    fn disabled_wins_over_pressed_and_custom_paints_nothing() {
        for t in [Theme::light(), Theme::dark()] {
            for role in ROLES_WITH_STATE {
                for part in [Part::Fill, Part::Label] {
                    assert_eq!(
                        paint(role, part, disabled().pressed(true), &t),
                        paint(role, part, disabled(), &t),
                        "{role:?} {part:?}: disabled > pressed (§6.7)"
                    );
                }
            }
            assert_eq!(paint(PaintRole::Custom, Part::Fill, rest(), &t).a, 0.);
        }
    }

    #[test]
    fn over_composites_translucent_layers() {
        let base = Hsla::black().opacity(0.078);
        let result = over(base, Hsla::black().opacity(0.07));
        assert!((result.a - (1. - 0.922 * 0.93)).abs() < 1e-5);
        assert_eq!(over(base, Hsla::white()), Hsla::white());
        assert_eq!(over(transparent_black(), transparent_black()).a, 0.);
    }
}
