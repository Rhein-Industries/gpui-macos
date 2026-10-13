//! Role × state paint from `docs/native-spec.md` §6.2, the focus ring
//! (§6.3), row selection (§6.4) and the hover rule (§6.5).
//!
//! Every arm is a measured value (or an explicit fallback for a cell marked
//! **O**). States never compose by multiplication (§6): disabled, pressed and
//! inactive-window combinations each have their own arm.
use crate::Theme;
use crate::tokens::{
    FOCUS_RING_OUTSET, FOCUS_RING_WIDTH, SIDEBAR_INACTIVE, SIDEBAR_RT, SIDEBAR_UNEMPHASIZED,
    TOOLBAR_HOVER, disabled_bezel, disabled_control_fill, disabled_control_mark,
    disabled_switch_off, ink, pt,
};
use gpui_kit::{
    Div, FocusHandle, Hsla, Pixels, StyleRefinement, Styled, Window, div, rgb, transparent_black,
};

/// Which tinted family a prominent bezel uses (§6.2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tint {
    /// `prominent_tint`, the theme accent.
    Accent,
    /// `prominent_red`, the theme danger hue (`.tint(.red)`, C5).
    Red,
}

/// Which row family a row belongs to (spec §6.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RowKind {
    /// Source-list sidebar rows (C20).
    Sidebar,
    /// Lists, tables and trees.
    List,
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
    /// A sidebar, list, table or tree row (§6.4). `emphasized` is true while
    /// the row's list has keyboard focus (or the caller says so); an
    /// inactive window is never emphasized. Rows have no hover (C17).
    Row {
        kind: RowKind,
        selected: bool,
        emphasized: bool,
    },
    /// A toolbar item: a hover disc and no bezel (§6.5, C17, C28).
    ToolbarItem,
    /// A menu, submenu, select or combobox popup row (§7.3, C25, D10). The
    /// highlight follows the pointer and the keyboard cursor.
    MenuItem { highlighted: bool },
}

impl PaintRole {
    /// Whether keyboard focus draws the outset ring around this role's
    /// control (§6.3). Rows and menu items show focus through selection and
    /// highlight instead; `Custom` adapters draw their own.
    pub(crate) const fn has_focus_ring(self) -> bool {
        matches!(
            self,
            Self::Bezel
                | Self::Prominent(_)
                | Self::Borderless
                | Self::ToggleOn
                | Self::Segment { .. }
                | Self::ToolbarItem
        )
    }
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

    /// Enabled and at rest in an inactive window (columns IN, ID).
    pub(crate) const fn inactive() -> Self {
        Self {
            window_active: false,
            ..Self::active()
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

        // §6.4 rows (C20). Pressing a row does not change its paint.
        (
            PaintRole::Row {
                selected: false, ..
            },
            Part::Fill,
        ) => transparent_black(),
        (
            PaintRole::Row {
                kind: RowKind::List,
                emphasized,
                ..
            },
            Part::Fill,
        ) => match emphasized && active {
            true => theme.selected_content(),
            false => theme.unemphasized_selected_content(),
        },
        (
            PaintRole::Row {
                kind: RowKind::Sidebar,
                emphasized,
                ..
            },
            Part::Fill,
        ) => {
            let dark = usize::from(dark);
            match (active && emphasized, active, theme.reduce_transparency) {
                (true, _, _) => theme.accent,
                (false, true, true) => color(SIDEBAR_RT[dark][0]),
                (false, false, true) => color(SIDEBAR_RT[dark][1]),
                (false, true, false) => ink.opacity(SIDEBAR_UNEMPHASIZED[dark]),
                (false, false, false) => color(SIDEBAR_INACTIVE[dark]),
            }
        }
        // Sidebar row titles are full-strength ink: the SwiftUI sidebar draws
        // its labels vibrant over the material, measured pure black (0) in
        // light and white (255) in dark in the 2x captures, where `labelColor`
        // (216 / 255) rendered 36 / 219. Other rows keep `labelColor`.
        (
            PaintRole::Row {
                kind,
                selected,
                emphasized,
            },
            Part::Label,
        ) => match (disabled, selected && emphasized && active) {
            (true, _) => theme.tertiary_label(),
            (false, true) => white,
            (false, false) if kind == RowKind::Sidebar => ink,
            (false, false) => label,
        },

        // §6.5 toolbar items: no bezel at rest; the hover disc is `hover()`.
        // Pressed is not measured (O): twice the hover ink.
        (PaintRole::ToolbarItem, Part::Fill) => match (disabled, pressed) {
            (false, true) => ink.opacity(TOOLBAR_HOVER[usize::from(dark)] * 2.),
            _ => transparent_black(),
        },
        (PaintRole::ToolbarItem, Part::Label) => match disabled {
            true => theme.tertiary_label(),
            false => label,
        },

        // §7.3 (C25, D10): the highlight is `controlAccentColor` (#007AFF
        // exactly for the default accent) with a white label. Menus are key
        // while open; an inline fallback in an inactive window keeps a
        // neutral highlight. Destructive items are not red. Disabled items
        // are not measured (O): no highlight and the tertiary label (§8.2).
        (PaintRole::MenuItem { highlighted: false }, Part::Fill) => transparent_black(),
        (PaintRole::MenuItem { highlighted: true }, Part::Fill) => match (disabled, active) {
            (true, _) => transparent_black(),
            (false, true) => theme.accent,
            (false, false) => {
                theme
                    .control_fill()
                    .blend(label.opacity(if dark { 0.1 } else { 0.08 }))
            }
        },
        (PaintRole::MenuItem { highlighted }, Part::Label) => {
            match (disabled, highlighted && active) {
                (true, _) => theme.tertiary_label(),
                (false, true) => white,
                (false, false) => label,
            }
        }

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
            | PaintRole::ControlLabel
            | PaintRole::Row { .. }
            | PaintRole::ToolbarItem
            | PaintRole::MenuItem { .. },
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

/// The hover paint of a role (§6.5, C17), or `None` for the roles that do
/// not respond to the pointer: bezels, indicators, fields, tabs, segments,
/// rows, trees and tables. Toolbar items get an ink disc; menu rows get
/// their highlight.
pub(crate) fn hover(role: PaintRole, theme: &Theme) -> Option<Hsla> {
    match role {
        PaintRole::ToolbarItem => {
            Some(ink(theme).opacity(TOOLBAR_HOVER[usize::from(theme.is_dark())]))
        }
        PaintRole::MenuItem { .. } => Some(paint(
            PaintRole::MenuItem { highlighted: true },
            Part::Fill,
            States::active(),
            theme,
        )),
        PaintRole::Custom
        | PaintRole::Bezel
        | PaintRole::Prominent(_)
        | PaintRole::Borderless
        | PaintRole::ToggleOn
        | PaintRole::Segment { .. }
        | PaintRole::CheckOff
        | PaintRole::CheckOn
        | PaintRole::SwitchTrack { .. }
        | PaintRole::SwitchKnob { .. }
        | PaintRole::Slider
        | PaintRole::Progress
        | PaintRole::Link
        | PaintRole::ControlLabel
        | PaintRole::Row { .. } => None,
    }
}

/// Whether `focus` shows the keyboard focus ring (§6.3, C15): focused,
/// reached by the keyboard, in an active window.
pub(crate) fn focus_visible(focus: &FocusHandle, window: &Window) -> bool {
    window.is_window_active() && focus.is_focused(window) && window.last_input_was_keyboard()
}

/// Whether a text field shows its focus ring (§6.3): focused in an active
/// window. A field takes keyboard focus from a click as well.
pub(crate) fn field_focused(focus: &FocusHandle, window: &Window) -> bool {
    window.is_window_active() && focus.is_focused(window)
}

/// The keyboard focus ring (§6.3, plan D4): an absolutely positioned 3 pt
/// band outset 3 pt from its parent's border box, whose corner is `radius`
/// (the control's) plus the outset. `border` is the parent's border width,
/// because absolute children are placed inside it. The parent must be a
/// positioned element.
pub(crate) fn focus_ring(radius: Pixels, border: Pixels, window: &Window, theme: &Theme) -> Div {
    let outset = pt(FOCUS_RING_OUTSET).to_pixels(window.rem_size());
    let inset = -(outset + border);
    div()
        .absolute()
        .top(inset)
        .left(inset)
        .right(inset)
        .bottom(inset)
        .rounded(radius + outset)
        .border(pt(FOCUS_RING_WIDTH))
        .border_color(theme.focus_ring())
}

/// [`focus_ring`] around an element with `style`'s top-left corner radius
/// and top border width.
pub(crate) fn focus_ring_for(style: &StyleRefinement, window: &Window, theme: &Theme) -> Div {
    let length = |length: Option<gpui_kit::AbsoluteLength>| {
        length
            .map(|length| length.to_pixels(window.rem_size()))
            .unwrap_or_default()
    };
    focus_ring(
        length(style.corner_radii.top_left),
        length(style.border_widths.top),
        window,
        theme,
    )
}

/// [`focus_ring`] for a borderless control with a radius in points.
pub(crate) fn focus_ring_pt(radius: f32, window: &Window, theme: &Theme) -> Div {
    focus_ring(
        pt(radius).to_pixels(window.rem_size()),
        Pixels::ZERO,
        window,
        theme,
    )
}

/// The ring, or an empty out-of-flow placeholder, so a parent's gap and
/// layout never change with focus.
pub(crate) fn focus_ring_if(
    visible: bool,
    style: &StyleRefinement,
    window: &Window,
    theme: &Theme,
) -> Div {
    match visible {
        true => focus_ring_for(style, window, theme),
        false => div().absolute(),
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

/// Test support: painted focus rings, one entry per ring geometry.
#[cfg(test)]
pub(crate) fn painted_rings(window: &Window, theme: &Theme) -> Vec<gpui_kit::Quad> {
    // GPUI paints one bordered quad as several pieces with the same bounds
    // and different content masks; the same piece twice is a double ring.
    let quads: Vec<_> = window
        .painted_quads()
        .into_iter()
        .filter(|quad| quad.border_color == theme.focus_ring())
        .collect();
    for (index, quad) in quads.iter().enumerate() {
        assert!(
            !quads[..index]
                .iter()
                .any(|other| other.bounds == quad.bounds
                    && other.content_mask.bounds == quad.content_mask.bounds),
            "a ring is painted twice: {:?}",
            quad.bounds
        );
    }
    let mut rings: Vec<gpui_kit::Quad> = Vec::new();
    for quad in quads {
        if !rings.iter().any(|ring| ring.bounds == quad.bounds) {
            rings.push(quad);
        }
    }
    rings
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

    const NO_HOVER_ROLES: [PaintRole; 16] = [
        PaintRole::Custom,
        PaintRole::Bezel,
        PaintRole::Prominent(Tint::Accent),
        PaintRole::Prominent(Tint::Red),
        PaintRole::Borderless,
        PaintRole::ToggleOn,
        PaintRole::Segment { selected: false },
        PaintRole::Segment { selected: true },
        PaintRole::CheckOff,
        PaintRole::CheckOn,
        PaintRole::SwitchTrack { on: true },
        PaintRole::SwitchKnob { on: false },
        PaintRole::Slider,
        PaintRole::Link,
        PaintRole::Row {
            kind: RowKind::Sidebar,
            selected: true,
            emphasized: true,
        },
        PaintRole::Row {
            kind: RowKind::List,
            selected: false,
            emphasized: false,
        },
    ];

    #[test]
    fn hover_matches_6_5_and_c17() {
        for t in [Theme::light(), Theme::dark()] {
            for role in NO_HOVER_ROLES
                .into_iter()
                .chain([PaintRole::Progress, PaintRole::ControlLabel])
            {
                assert_eq!(hover(role, &t), None, "{role:?} has no hover");
            }
            let disc = if t.is_dark() { 0.04 } else { 0.05 };
            assert_close(
                hover(PaintRole::ToolbarItem, &t).unwrap(),
                ink_at(&t, disc),
                "toolbar disc",
            );
            assert_eq!(
                hover(PaintRole::MenuItem { highlighted: false }, &t),
                Some(t.accent),
                "menu highlight"
            );
        }
    }

    #[test]
    fn focus_ring_roles_match_6_3() {
        for role in [
            PaintRole::Bezel,
            PaintRole::Prominent(Tint::Accent),
            PaintRole::Borderless,
            PaintRole::ToggleOn,
            PaintRole::Segment { selected: true },
            PaintRole::ToolbarItem,
        ] {
            assert!(role.has_focus_ring(), "{role:?}");
        }
        for role in [
            PaintRole::Custom,
            PaintRole::MenuItem { highlighted: true },
            PaintRole::Row {
                kind: RowKind::Sidebar,
                selected: true,
                emphasized: true,
            },
        ] {
            assert!(!role.has_focus_ring(), "{role:?}");
        }
    }

    #[test]
    fn sidebar_rows_match_6_4_and_c20() {
        let row = |selected, emphasized| PaintRole::Row {
            kind: RowKind::Sidebar,
            selected,
            emphasized,
        };
        for t in [Theme::light(), Theme::dark()] {
            let dark = t.is_dark();
            let fill = |role, s| paint(role, Part::Fill, s, &t);
            let label = |role, s| paint(role, Part::Label, s, &t);
            // Focused key window: opaque accent with a white label.
            assert_eq!(fill(row(true, true), rest()), t.accent);
            assert_eq!(label(row(true, true), rest()), t.on_accent);
            // Unfocused key window: black @0.085 / white @0.088 over the material.
            assert_close(
                fill(row(true, false), rest()),
                ink_at(&t, if dark { 0.088 } else { 0.085 }),
                "unemphasized key",
            );
            // Sidebar titles are full-strength ink (measured pure black / white).
            assert_eq!(label(row(true, false), rest()), ink_at(&t, 1.));
            // Inactive window: opaque #E5E5E5 / #363636, emphasized or not.
            for emphasized in [true, false] {
                assert_close(
                    fill(row(true, emphasized), inactive()),
                    color(if dark { 0x363636 } else { 0xe5e5e5 }),
                    "inactive",
                );
                assert_eq!(fill(row(true, emphasized), inactive()).a, 1.);
                assert_eq!(label(row(true, emphasized), inactive()), ink_at(&t, 1.));
            }
            // Unselected rows stay clear; pressing changes nothing.
            assert_eq!(fill(row(false, true), rest()).a, 0.);
            assert_eq!(label(row(false, true), rest()), ink_at(&t, 1.));
            assert_eq!(
                fill(row(true, true), pressed()),
                fill(row(true, true), rest())
            );
            assert_eq!(label(row(true, true), disabled()), t.tertiary_label());
            // Reduce Transparency (Sim): #DFDFDF / #545454 and #E1E1E1 / #232323.
            let rt = t.clone().with_reduce_transparency(true);
            let (key, inactive_rt) = if dark {
                (0x545454, 0x232323)
            } else {
                (0xdfdfdf, 0xe1e1e1)
            };
            assert_eq!(paint(row(true, false), Part::Fill, rest(), &rt), color(key));
            assert_eq!(
                paint(row(true, true), Part::Fill, inactive(), &rt),
                color(inactive_rt)
            );
            assert_eq!(paint(row(true, true), Part::Fill, rest(), &rt), rt.accent);
        }
    }

    #[test]
    fn list_rows_match_6_4() {
        let row = |emphasized| PaintRole::Row {
            kind: RowKind::List,
            selected: true,
            emphasized,
        };
        for t in [Theme::light(), Theme::dark()] {
            let dark = t.is_dark();
            let emphasized = color(if dark { 0x0059d1 } else { 0x0064e1 });
            let unemphasized = color(if dark { 0x464646 } else { 0xdcdcdc });
            assert_eq!(paint(row(true), Part::Fill, rest(), &t), emphasized);
            assert_eq!(paint(row(true), Part::Label, rest(), &t), t.on_accent);
            for (role, s) in [
                (row(false), rest()),
                (row(true), inactive()),
                (row(false), inactive()),
            ] {
                assert_eq!(paint(role, Part::Fill, s, &t), unemphasized);
                assert_eq!(paint(role, Part::Label, s, &t), t.foreground);
            }
            let custom = t.clone().with_accent(color(0x34a853));
            assert_eq!(
                paint(row(true), Part::Fill, rest(), &custom),
                custom.accent,
                "custom accents select with the accent (D7)"
            );
        }
    }

    #[test]
    fn toolbar_and_menu_items_route_through_their_roles() {
        for t in [Theme::light(), Theme::dark()] {
            let item = PaintRole::ToolbarItem;
            assert_eq!(paint(item, Part::Fill, rest(), &t).a, 0.);
            assert_eq!(paint(item, Part::Label, rest(), &t), t.foreground);
            assert_eq!(paint(item, Part::Label, disabled(), &t), t.tertiary_label());
            assert_eq!(paint(item, Part::Fill, disabled().pressed(true), &t).a, 0.);
            let on = PaintRole::MenuItem { highlighted: true };
            let off = PaintRole::MenuItem { highlighted: false };
            assert_eq!(paint(on, Part::Fill, rest(), &t), t.accent);
            assert_eq!(paint(on, Part::Label, rest(), &t), t.on_accent);
            assert_eq!(paint(on, Part::Label, inactive(), &t), t.foreground);
            assert_eq!(paint(on, Part::Fill, inactive(), &t).a, 1.);
            assert_eq!(paint(off, Part::Fill, rest(), &t).a, 0.);
            assert_eq!(paint(off, Part::Label, rest(), &t), t.foreground);
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
