//! Native values copied by hand from `docs/native-spec.md`.
use crate::Theme;
use gpui_kit::{FontWeight, Hsla, Styled, rems, rgb};

/// macOS text roles; sizes and advances follow spec §3.1–3.2.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyle {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Body,
    Callout,
    Subheadline,
    Footnote,
    Caption,
    Caption2,
    ToolbarTitle,
    MenuItem,
    Tooltip,
}

/// A point-based text token from spec §3.1 (absolute advance, §3.2).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeTextMetrics {
    pub size: f32,
    pub line_height: f32,
    pub weight: FontWeight,
}
impl TextStyle {
    /// Values copied from spec §3.1; Title3 uses the 20 pt advance (C11).
    pub const fn token(self) -> NativeTextMetrics {
        let (size, line_height, weight) = match self {
            Self::LargeTitle => (26., 32., FontWeight::NORMAL),
            Self::Title => (22., 26., FontWeight::NORMAL),
            Self::Title2 => (17., 22., FontWeight::NORMAL),
            Self::Title3 => (15., 20., FontWeight::NORMAL),
            Self::Headline => (13., 16., FontWeight::BOLD),
            Self::Body | Self::MenuItem => (13., 16., FontWeight::NORMAL),
            Self::Callout => (12., 15., FontWeight::NORMAL),
            Self::Subheadline | Self::Tooltip => (11., 14., FontWeight::NORMAL),
            Self::Footnote | Self::Caption => (10., 13., FontWeight::NORMAL),
            Self::Caption2 => (10., 13., FontWeight::MEDIUM),
            Self::ToolbarTitle => (15., 20., FontWeight::SEMIBOLD),
        };
        NativeTextMetrics {
            size,
            line_height,
            weight,
        }
    }
}

/// Apply a native text role in zoom-scaled points (spec §3.1–3.2).
pub trait NativeTextStyle: Styled {
    fn native_text_style(self, style: TextStyle, theme: &Theme) -> Self {
        let token = style.token();
        self.font_family(theme.font.clone())
            .text_size(rems(token.size / 16.))
            .font_weight(token.weight)
            .line_height(rems(token.line_height / 16.))
    }
}
impl<T: Styled> NativeTextStyle for T {}

/// Base radius scale: spec §4.1 (small bezel, regular bezel, GroupBox, window); plan D3.
pub(crate) const BASE_RADII: [f32; 4] = [5., 6., 12., 16.];

/// A point token at the current zoom: points are rems of a 16 pt root.
pub(crate) fn pt(points: f32) -> gpui_kit::Rems {
    rems(points / 16.)
}

// Control metrics, regular size only (plan D1). Values are points.

/// Bezel, pop-up and segmented height, spec §4.2.
pub(crate) const BEZEL_HEIGHT: f32 = 24.;
/// Bezel, pop-up and segmented horizontal padding, spec §4.2.
pub(crate) const BEZEL_PADDING_X: f32 = 12.;
/// Bezel, pop-up, segmented and text-field corner radius, spec §4.1 (C13).
pub(crate) const CONTROL_RADIUS: f32 = 6.;
/// Icon-only bordered button: visible bezel (width, height), spec §4.2 (C7).
pub(crate) const ICON_BUTTON_BEZEL: (f32, f32) = (37.5, 20.5);
/// Icon-only margins (horizontal, vertical) that give the 39 × 21 layout
/// frame, spec §4.2 (C7). The hit area is the visible bezel.
pub(crate) const ICON_BUTTON_MARGIN: (f32, f32) = (0.75, 0.25);
/// Icon-only glyph box (the `plus` symbol), spec §4.2 (C7).
pub(crate) const ICON_BUTTON_GLYPH: f32 = 10.5;
/// Checkbox box, spec §4.4.
pub(crate) const CHECKBOX_SIZE: f32 = 16.;
/// Checkbox corner radius, spec §4.1 (C9).
pub(crate) const CHECKBOX_RADIUS: f32 = 5.5;
/// Labeled checkbox row; the box sits at y 0.5, spec §4.4 (C2).
pub(crate) const CHECKBOX_ROW_HEIGHT: f32 = 17.;
/// Checkbox and radio label gap, spec §4.4 (C21).
pub(crate) const INDICATOR_LABEL_GAP: f32 = 5.;
/// Radio circle and selected dot, spec §4.4.
pub(crate) const RADIO_SIZE: f32 = 16.;
pub(crate) const RADIO_DOT: f32 = 5.;
/// Text, secure and search field height, spec §4.3 (the search field is a capsule).
pub(crate) const FIELD_HEIGHT: f32 = 24.;
/// Switch track (width, height), knob (width, height) and knob inset, spec §4.5.
pub(crate) const SWITCH_TRACK: (f32, f32) = (54., 24.);
pub(crate) const SWITCH_KNOB: (f32, f32) = (32., 20.);
pub(crate) const SWITCH_INSET: f32 = 2.;
/// Labeled switch gap, spec §4.5.
pub(crate) const SWITCH_LABEL_GAP: f32 = 8.;
/// Slider frame height, track thickness and resting knob (width, height), spec §4.5.
pub(crate) const SLIDER_HEIGHT: f32 = 16.;
pub(crate) const SLIDER_TRACK: f32 = 6.;
pub(crate) const SLIDER_KNOB: (f32, f32) = (20., 16.);
/// Linear progress bar thickness, spec §4.5.
pub(crate) const PROGRESS_LINEAR: f32 = 8.;

// Rows (C16).

/// List, table and tree row height, spec §5.2.
pub(crate) const ROW_HEIGHT: f32 = 24.;
/// Table header height above its separator, spec §5.2.
pub(crate) const TABLE_HEADER_HEIGHT: f32 = 27.;
/// Table header separator: a one-point hairline approximating the measured
/// 1.5 pt, spec §5.2.
pub(crate) const TABLE_SEPARATOR: f32 = 1.;
/// Body text line advance used to derive row padding, spec §3.1.
const BODY_LINE: f32 = 16.;
/// Vertical cell padding that gives a Body row its native height.
pub(crate) const ROW_PADDING_Y: f32 = (ROW_HEIGHT - BODY_LINE) / 2.;
/// Grouped card-list row (`inset_list_row`): a deliberate specialization,
/// not the measured 24 pt list row (plan D12).
pub(crate) const INSET_LIST_ROW_HEIGHT: f32 = 40.;
/// Sidebar ideal width in points, used by [`crate::sidebar`] (C3, policy;
/// SwiftUI's 144 default is content-driven). Scale it by the window's rem
/// size over 16 to get pixels at the current zoom.
pub const SIDEBAR_IDEAL_WIDTH: f32 = 220.;
/// Prominent disabled tint multiplier, spec §6.2.3.
pub(crate) const DISABLED_TINT_OPACITY: f32 = 0.69;
/// Prominent disabled label alpha, spec §6.2.3.
pub(crate) const DISABLED_TINT_LABEL_OPACITY: f32 = 0.50;
/// Switch disabled on-track multiplier, spec §6.2.7.
pub(crate) const DISABLED_SWITCH_OPACITY: f32 = 0.71;
/// Tertiary label alpha (light / dark), spec §2A.1 and §8.2.
const TERTIARY_LABEL: [f32; 2] = [66. / 255., 63. / 255.];
/// Increased-contrast tertiary label alpha (light / dark), spec §2A.1.
const TERTIARY_LABEL_IC: [f32; 2] = [0.557, 0.498];
/// Disabled check wells (off / on), spec §6.2.6.
const DISABLED_CHECK_WELL: [f32; 2] = [0.051, 0.075];
/// Disabled check mark (light / dark), spec §6.2.6.
const DISABLED_CHECK_MARK: [f32; 2] = [0.31, 0.30];
/// Disabled bezel fill, spec §6.2.1.
const DISABLED_BEZEL_FILL: f32 = 0.039;
/// Disabled bezel label (light / dark), spec §6.2.1.
const DISABLED_BEZEL_LABEL: [f32; 2] = [0.257, 0.290];
/// Dark disabled off-switch track RGB, spec §6.2.7.
const DISABLED_SWITCH_OFF_DARK: u32 = 0xcfcfcf;
/// Dark disabled off-switch track alpha, spec §6.2.7.
const DISABLED_SWITCH_OFF_DARK_ALPHA: f32 = 0.063;
/// Default `controlAccentColor` / `prominent_tint`, spec §2A.3 and §2B.
const DEFAULT_ACCENT: u32 = 0x007aff;
/// Bezel fill ink alpha, spec §2B `bezel_fill` and §6.2.1 (C14).
const BEZEL_FILL: f32 = 0.078;
/// `indicator_accent` (light / dark) for the default accent, spec §2B.
const INDICATOR_ACCENT: [u32; 2] = [0x0078f9, 0x067eff];
/// Rendered `link` (light / dark), spec §2A.1 and §2B `link_rendered`.
const LINK: [u32; 2] = [0x0068da, 0x419cff];
/// Switch knob shadow ink (light / dark), tuned toward the measured darkening
/// of about 6/255 below the knob, spec §6.2.7 (C19; estimate).
const SWITCH_KNOB_SHADOW: [f32; 2] = [0.05, 0.2];
/// `focus_ring_control` (light / dark) for the default accent, spec §2B. Fields
/// use the same colour (plan D4, a deliberate approximation of
/// `focus_ring_field`).
const FOCUS_RING: [u32; 2] = [0x0c6bfb, 0x25afff];
/// `focus_ring_control` alpha, spec §2B.
const FOCUS_RING_ALPHA: f32 = 0.49;
/// Focus ring width and outset in points, spec §6.3 (C15). The outer corner
/// is the control radius plus the outset.
pub(crate) const FOCUS_RING_WIDTH: f32 = 3.;
pub(crate) const FOCUS_RING_OUTSET: f32 = 3.;
/// `selected_content` (light / dark), spec §2A.3 and §6.4.
const SELECTED_CONTENT: [u32; 2] = [0x0064e1, 0x0059d1];
/// `unemphasized_selected_content` (light / dark), spec §2A.3 and §6.4.
const UNEMPHASIZED_SELECTED_CONTENT: [u32; 2] = [0xdcdcdc, 0x464646];
/// `alternating_row`: light #F4F5F5, dark white @0.047, spec §2A.2.
const ALTERNATING_ROW_LIGHT: u32 = 0xf4f5f5;
const ALTERNATING_ROW_DARK_ALPHA: f32 = 0.047;
/// `tertiary_system_fill` ink alpha, spec §2A.2.
const TERTIARY_FILL: f32 = 0.047;
/// Sidebar row selection, spec §6.4 (C20): unfocused key window ink alpha
/// over the material (light / dark).
pub(crate) const SIDEBAR_UNEMPHASIZED: [f32; 2] = [0.085, 0.088];
/// Sidebar row selection in an inactive window: opaque (light / dark), on
/// the opaque #F0F0F0 / #282828 material, spec §6.4 and §7.2 (C20).
pub(crate) const SIDEBAR_INACTIVE: [u32; 2] = [0xe5e5e5, 0x363636];
/// Reduce Transparency sidebar selection (unfocused key, inactive) per
/// appearance (light, dark), spec §6.4 (Sim).
pub(crate) const SIDEBAR_RT: [[u32; 2]; 2] = [[0xdfdfdf, 0xe1e1e1], [0x545454, 0x232323]];
/// Sidebar material in an active window (light / dark): `visualEffect.sidebar`
/// tint and opacity, spec §7.1.
pub(crate) const SIDEBAR_MATERIAL: [(u32, f32); 2] = [(0xe7e7e7, 0.898), (0x2c2c2c, 0.816)];
/// Sidebar material in an inactive window: opaque (light / dark), spec §7.2 (C20, C29).
pub(crate) const SIDEBAR_MATERIAL_INACTIVE: [u32; 2] = [0xf0f0f0, 0x282828];
/// Reduce Transparency sidebar (active, inactive) per appearance, spec §7.6 (Sim).
pub(crate) const SIDEBAR_MATERIAL_RT: [[u32; 2]; 2] = [[0xf6f6f6, 0xececec], [0x383838, 0x141414]];
/// Toolbar item hover disc ink alpha (light / dark), spec §6.5 (C17, C28).
pub(crate) const TOOLBAR_HOVER: [f32; 2] = [0.05, 0.04];

// Menus, popovers and tooltips (spec §7.3–7.5). Values are points.

/// Menu, submenu and select/combobox popup row height, spec §7.3 (C25).
pub(crate) const MENU_ROW_HEIGHT: f32 = 24.;
/// Menu panel vertical padding above the first and below the last row,
/// spec §7.3 (C25).
pub(crate) const MENU_PADDING_Y: f32 = 5.;
/// Highlight inset from the panel's (leading, trailing) edges, spec §7.3 (C25).
pub(crate) const MENU_HIGHLIGHT_INSET: (f32, f32) = (5., 5.5);
/// Menu highlight corner radius, spec §7.3 and §4.1 (C25; estimate).
pub(crate) const MENU_HIGHLIGHT_RADIUS: f32 = 6.5;
/// Menu panel corner radius (light; dark measures 12.4), spec §7.3 and §4.1 (C25).
pub(crate) const MENU_RADIUS: f32 = 13.;
/// The width of GPUI's `border_1`, which inline popup fallbacks draw inside
/// their measured outer edge.
pub(crate) const BORDER_1: gpui_kit::Pixels = gpui_kit::px(1.);
/// Menu row label inset inside the highlight. Not measured; kept from the
/// pre-M5 menu so labels do not move.
pub(crate) const MENU_ROW_PADDING_X: f32 = 8.;
/// A separator row: its hairline centred in 11 pt, spec §7.3 (about 11 to 12;
/// a 9-row pull-down measures 200 = 2 × 5 + 7 × 24 + 2 × 11).
pub(crate) const MENU_SEPARATOR_HEIGHT: f32 = 11.;
/// Popover corner radius, spec §7.4 and §4.1 (C26; estimate).
pub(crate) const POPOVER_RADIUS: f32 = 16.;
/// Popover content padding, SwiftUI `.padding()`, spec §9 (C26).
pub(crate) const POPOVER_PADDING: f32 = 16.;
/// Tooltip body height, spec §7.5 (C27).
pub(crate) const TOOLTIP_HEIGHT: f32 = 20.;
/// Tooltip corner radius (5.13 light, 5.5 dark), spec §7.5 and §4.1 (C27).
pub(crate) const TOOLTIP_RADIUS: f32 = 5.;
/// Tooltip label inset. Not measured; kept from the pre-M5 tooltip.
pub(crate) const TOOLTIP_PADDING_X: f32 = 10.;
/// The tooltip's own measured material (light, dark): tint and opacity,
/// spec §7.5 (C27). Dark differs from the `toolTip` specimen in §7.1.
const TOOLTIP_MATERIAL: [(u32, f32); 2] = [(0xe7e7e7, 0.898), (0x3a3a3a, 0.828)];

/// Disabled dimming kept only for `PaintRole::Custom` buttons and adapters
/// that paint their own look until M4/M5 give them roles (plan D6).
pub(crate) const CUSTOM_DISABLED_OPACITY: f32 = 0.45;

pub(crate) fn ink(theme: &Theme) -> Hsla {
    if theme.is_dark() {
        Hsla::white()
    } else {
        Hsla::black()
    }
}
pub(crate) fn disabled_control_fill(theme: &Theme, selected: bool) -> Hsla {
    ink(theme).opacity(DISABLED_CHECK_WELL[usize::from(selected)])
}
pub(crate) fn disabled_control_mark(theme: &Theme) -> Hsla {
    ink(theme).opacity(DISABLED_CHECK_MARK[usize::from(theme.is_dark())])
}
pub(crate) fn disabled_switch_off(theme: &Theme) -> Hsla {
    if theme.is_dark() {
        Hsla::from(rgb(DISABLED_SWITCH_OFF_DARK)).opacity(DISABLED_SWITCH_OFF_DARK_ALPHA)
    } else {
        disabled_control_fill(theme, false)
    }
}
pub(crate) fn disabled_bezel(theme: &Theme) -> (Hsla, Hsla) {
    (
        ink(theme).opacity(DISABLED_BEZEL_FILL),
        ink(theme).opacity(DISABLED_BEZEL_LABEL[usize::from(theme.is_dark())]),
    )
}
pub(crate) fn switch_knob_shadow(theme: &Theme) -> Vec<gpui_kit::BoxShadow> {
    vec![
        gpui_kit::BoxShadow::new(
            gpui_kit::px(0.),
            gpui_kit::px(1.),
            Hsla::black().opacity(SWITCH_KNOB_SHADOW[usize::from(theme.is_dark())]),
        )
        .blur_radius(gpui_kit::px(2.)),
    ]
}
impl Theme {
    /// Whether the accent is the system default, so accent-derived measured
    /// values apply (plan D7).
    pub(crate) fn has_default_accent(&self) -> bool {
        self.accent == Hsla::from(rgb(DEFAULT_ACCENT))
    }
    /// Translucent bezel fill, spec §2B `bezel_fill` (C14).
    pub fn bezel_fill(&self) -> Hsla {
        ink(self).opacity(BEZEL_FILL)
    }
    /// Accent painted by checkbox, radio, toggle-on, segment, switch and
    /// slider fills, spec §2B. A custom accent is used as is (plan D7).
    pub fn indicator_accent(&self) -> Hsla {
        if self.has_default_accent() {
            rgb(INDICATOR_ACCENT[usize::from(self.is_dark())]).into()
        } else {
            self.accent
        }
    }
    /// Rendered link color, spec §2A.1 `link`. A custom accent is used as is.
    pub fn link(&self) -> Hsla {
        if self.has_default_accent() {
            rgb(LINK[usize::from(self.is_dark())]).into()
        } else {
            self.accent
        }
    }
    /// Keyboard focus ring, spec §2B `focus_ring_control` (about #0C6BFB /
    /// #25AFFF @0.49), used for controls and fields alike (plan D4). A custom
    /// accent is used at the same alpha (plan D7).
    pub fn focus_ring(&self) -> Hsla {
        let tint = if self.has_default_accent() {
            rgb(FOCUS_RING[usize::from(self.is_dark())]).into()
        } else {
            self.accent
        };
        tint.opacity(FOCUS_RING_ALPHA)
    }
    /// Emphasized list and table selection, spec §2A.3 `selected_content`
    /// (#0064E1 / #0059D1). A custom accent is used as is (plan D7).
    pub fn selected_content(&self) -> Hsla {
        if self.has_default_accent() {
            rgb(SELECTED_CONTENT[usize::from(self.is_dark())]).into()
        } else {
            self.accent
        }
    }
    /// Unemphasized list and table selection, spec §2A.3
    /// `unemphasized_selected_content` (#DCDCDC / #464646).
    pub fn unemphasized_selected_content(&self) -> Hsla {
        rgb(UNEMPHASIZED_SELECTED_CONTENT[usize::from(self.is_dark())]).into()
    }
    /// The second alternating row background, spec §2A.2 `alternating_row`
    /// (#F4F5F5 / white @0.047).
    pub fn alternating_row(&self) -> Hsla {
        if self.is_dark() {
            ink(self).opacity(ALTERNATING_ROW_DARK_ALPHA)
        } else {
            rgb(ALTERNATING_ROW_LIGHT).into()
        }
    }
    /// The sidebar material as a deterministic paint, spec §7.1, §7.2 and
    /// §7.6 (C20, C29): #E7E7E7 @0.898 / #2C2C2C @0.816 in an active window,
    /// opaque #F0F0F0 / #282828 in an inactive one, and #F6F6F6 / #383838
    /// (active), #ECECEC / #141414 (inactive) under Reduce Transparency.
    /// [`crate::sidebar`] paints it when real glass is unavailable; use it for
    /// a sidebar placed over opaque content, where the glass cannot show.
    pub fn sidebar_material(&self, window_active: bool) -> Hsla {
        let dark = usize::from(self.is_dark());
        let opaque = |hex: u32| -> Hsla { rgb(hex).into() };
        match (self.reduce_transparency, window_active) {
            (true, active) => opaque(SIDEBAR_MATERIAL_RT[dark][usize::from(!active)]),
            (false, false) => opaque(SIDEBAR_MATERIAL_INACTIVE[dark]),
            (false, true) => {
                let (tint, alpha) = SIDEBAR_MATERIAL[dark];
                opaque(tint).opacity(alpha)
            }
        }
    }
    /// The tooltip fill, spec §7.5 (C27): #E7E7E7 @0.898 / #3A3A3A @0.828,
    /// opaque at the same tint under Reduce Transparency (policy: §7.6 has
    /// no tooltip value).
    pub(crate) fn tooltip_material(&self) -> Hsla {
        let (tint, alpha) = TOOLTIP_MATERIAL[usize::from(self.is_dark())];
        Hsla::from(rgb(tint)).opacity(if self.reduce_transparency { 1. } else { alpha })
    }
    /// `tertiary_system_fill`, ink @0.047, spec §2A.2.
    pub fn tertiary_fill(&self) -> Hsla {
        ink(self).opacity(TERTIARY_FILL)
    }
    /// Semantic tertiary label, spec §2A.1; used for disabled control labels (§8.2).
    pub fn tertiary_label(&self) -> Hsla {
        let values = if self.increased_contrast {
            TERTIARY_LABEL_IC
        } else {
            TERTIARY_LABEL
        };
        ink(self).opacity(values[usize::from(self.is_dark())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_styles_match_spec_3_1() {
        use TextStyle::*;
        for (style, size, height, weight) in [
            (LargeTitle, 26., 32., FontWeight::NORMAL),
            (Title, 22., 26., FontWeight::NORMAL),
            (Title2, 17., 22., FontWeight::NORMAL),
            (Title3, 15., 20., FontWeight::NORMAL),
            (Headline, 13., 16., FontWeight::BOLD),
            (Body, 13., 16., FontWeight::NORMAL),
            (Callout, 12., 15., FontWeight::NORMAL),
            (Subheadline, 11., 14., FontWeight::NORMAL),
            (Footnote, 10., 13., FontWeight::NORMAL),
            (Caption, 10., 13., FontWeight::NORMAL),
            (Caption2, 10., 13., FontWeight::MEDIUM),
            (ToolbarTitle, 15., 20., FontWeight::SEMIBOLD),
            (MenuItem, 13., 16., FontWeight::NORMAL),
            (Tooltip, 11., 14., FontWeight::NORMAL),
        ] {
            assert_eq!(
                style.token(),
                NativeTextMetrics {
                    size,
                    line_height: height,
                    weight
                }
            );
        }
    }
    #[test]
    fn control_and_row_metrics_match_spec_4_and_5() {
        // §4.2 (C7): bezel plus margins is the 39 × 21 layout frame.
        let (w, h) = ICON_BUTTON_BEZEL;
        let (mx, my) = ICON_BUTTON_MARGIN;
        assert_eq!((w + 2. * mx, h + 2. * my), (39., 21.));
        assert_eq!((w, h, ICON_BUTTON_GLYPH), (37.5, 20.5, 10.5));
        assert_eq!(
            (BEZEL_HEIGHT, BEZEL_PADDING_X, CONTROL_RADIUS),
            (24., 12., 6.)
        );
        // §4.4 (C2, C9).
        assert_eq!(
            (CHECKBOX_SIZE, CHECKBOX_RADIUS, CHECKBOX_ROW_HEIGHT),
            (16., 5.5, 17.)
        );
        assert_eq!((INDICATOR_LABEL_GAP, RADIO_SIZE, RADIO_DOT), (5., 16., 5.));
        assert_eq!(FIELD_HEIGHT, 24.);
        // §4.5: the knob travels inside the inset track.
        assert_eq!(SWITCH_TRACK.1 - 2. * SWITCH_INSET, SWITCH_KNOB.1);
        assert_eq!((SWITCH_TRACK, SWITCH_KNOB), ((54., 24.), (32., 20.)));
        assert_eq!(
            (SLIDER_TRACK, SLIDER_KNOB, PROGRESS_LINEAR),
            (6., (20., 16.), 8.)
        );
        // §5.2 (C16) and plan D12.
        assert_eq!(ROW_PADDING_Y * 2. + 16., 24.);
        assert_eq!(TABLE_HEADER_HEIGHT, 27.);
        assert_eq!(TABLE_SEPARATOR, 1.);
        assert_eq!((INSET_LIST_ROW_HEIGHT, SIDEBAR_IDEAL_WIDTH), (40., 220.));
    }
    #[test]
    fn menu_popover_and_tooltip_tokens_match_spec_7_3_to_7_5() {
        // §7.3 (C25).
        assert_eq!(
            (MENU_ROW_HEIGHT, MENU_PADDING_Y, MENU_RADIUS),
            (24., 5., 13.)
        );
        assert_eq!(
            (MENU_HIGHLIGHT_INSET, MENU_HIGHLIGHT_RADIUS),
            ((5., 5.5), 6.5)
        );
        // A 9-row pull-down with two separators measures 200 pt.
        assert_eq!(
            2. * MENU_PADDING_Y + 7. * MENU_ROW_HEIGHT + 2. * MENU_SEPARATOR_HEIGHT,
            200.
        );
        // §7.4 (C26).
        assert_eq!((POPOVER_RADIUS, POPOVER_PADDING), (16., 16.));
        // §7.5 (C27): the 11 pt Tooltip line centred in 20 pt.
        assert_eq!((TOOLTIP_HEIGHT, TOOLTIP_RADIUS), (20., 5.));
        assert_eq!(TextStyle::Tooltip.token().size, 11.);
        let (light, dark) = (Theme::light(), Theme::dark());
        assert_eq!(
            light.tooltip_material(),
            Hsla::from(rgb(0xe7e7e7)).opacity(0.898)
        );
        assert_eq!(
            dark.tooltip_material(),
            Hsla::from(rgb(0x3a3a3a)).opacity(0.828)
        );
        for theme in [light, dark] {
            let opaque = theme.with_reduce_transparency(true).tooltip_material();
            assert_eq!(opaque.a, 1., "opaque under Reduce Transparency");
        }
    }
    #[test]
    fn focus_and_selection_tokens_match_spec_2a_2b() {
        let close = |actual: Hsla, hex: u32, alpha: f32| {
            let expected = gpui_kit::Rgba::from(Hsla::from(rgb(hex)));
            let actual_rgb = gpui_kit::Rgba::from(actual);
            for (a, e) in [
                (actual_rgb.r, expected.r),
                (actual_rgb.g, expected.g),
                (actual_rgb.b, expected.b),
            ] {
                assert!((a - e).abs() * 255. <= 2., "{actual:?} vs #{hex:06X}");
            }
            assert!((actual.a - alpha).abs() <= 0.01, "{actual:?} alpha");
        };
        let (light, dark) = (Theme::light(), Theme::dark());
        close(light.focus_ring(), 0x0c6bfb, 0.49);
        close(dark.focus_ring(), 0x25afff, 0.49);
        close(light.selected_content(), 0x0064e1, 1.);
        close(dark.selected_content(), 0x0059d1, 1.);
        close(light.unemphasized_selected_content(), 0xdcdcdc, 1.);
        close(dark.unemphasized_selected_content(), 0x464646, 1.);
        close(light.alternating_row(), 0xf4f5f5, 1.);
        close(dark.alternating_row(), 0xffffff, 0.047);
        assert_eq!(light.tertiary_fill(), Hsla::black().opacity(0.047));
        assert_eq!((FOCUS_RING_WIDTH, FOCUS_RING_OUTSET), (3., 3.));
        let custom = light.clone().with_accent(rgb(0x34a853).into());
        assert_eq!(custom.focus_ring(), custom.accent.opacity(0.49));
        assert_eq!(light.tokens().ring, light.focus_ring(), "Base ring");
        assert_eq!(light.focus_shadow()[0].color, light.focus_ring());
    }
    #[test]
    fn disabled_values_match_spec() {
        for theme in [Theme::light(), Theme::dark()] {
            let dark = theme.is_dark();
            assert_eq!(
                theme.tertiary_label().a,
                if dark { 63. / 255. } else { 66. / 255. }
            );
            assert_eq!(disabled_control_fill(&theme, false).a, 0.051);
            assert_eq!(disabled_control_fill(&theme, true).a, 0.075);
            assert_eq!(
                disabled_control_mark(&theme).a,
                if dark { 0.30 } else { 0.31 }
            );
            assert_eq!(disabled_bezel(&theme).0.a, 0.039);
            assert_eq!(disabled_bezel(&theme).1.a, if dark { 0.290 } else { 0.257 });
            assert_eq!(
                disabled_switch_off(&theme),
                if dark {
                    Hsla::from(rgb(0xcfcfcf)).opacity(0.063)
                } else {
                    Hsla::black().opacity(0.051)
                }
            );
            assert_eq!(theme.disabled_tint(theme.accent).a, 0.69);
            let tint = theme.accent.opacity(0.4);
            let disabled = theme.disabled_tint(tint);
            assert!((disabled.a - 0.4 * 0.69).abs() < f32::EPSILON);
            assert_eq!(
                (disabled.h, disabled.s, disabled.l),
                (tint.h, tint.s, tint.l)
            );
            assert_eq!(DISABLED_TINT_LABEL_OPACITY, 0.50);
            assert_eq!(DISABLED_SWITCH_OPACITY, 0.71);
        }
    }
}
