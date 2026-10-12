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
impl Theme {
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
