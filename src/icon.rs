//! Native SF Symbols on macOS, with the complete Lucide catalog as fallback.
//!
//! [`IconName`] and the SVG paths behind it come from `gpui-kit-assets`, so an
//! application keeps the whole catalog and the same public names. On macOS,
//! matching SF Symbols are supplied and rendered by the installed AppKit.
//! Missing symbols and other platforms read SVGs from the application's `AssetSource`:
//! register `gpui_kit::assets::Assets` for the default bundle, `AllAssets` for
//! the full catalog, or compose either with `icon_assets!` for a few extras.
use gpui_kit::base::StyledExt;
use gpui_kit::rems;
use gpui_kit::{App, IntoElement, RenderOnce, StyleRefinement, Styled, Window, svg};

pub use gpui_kit::assets::IconName;

#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    style: StyleRefinement,
    /// Draw the SF Symbol at its natural size for this point size, with the
    /// large symbol scale when set, centred in the icon box.
    natural: Option<(gpui_kit::AbsoluteLength, bool)>,
}

impl Icon {
    /// Draw the native symbol as AppKit lays it out beside text: configured
    /// at `points` (zoom-scaled) and the large or medium symbol scale, at its
    /// natural size, centred in the icon box. Without this, a symbol is
    /// fitted to the box. The Lucide fallback is unaffected.
    pub(crate) fn natural_symbol(
        mut self,
        points: impl Into<gpui_kit::AbsoluteLength>,
        large: bool,
    ) -> Self {
        self.natural = Some((points.into(), large));
        self
    }
}
impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        #[cfg(target_os = "macos")]
        {
            let text = window.text_style();
            let color = self.style.text.color.unwrap_or(text.color);
            let weight = self.style.text.font_weight.unwrap_or(text.font_weight);
            let points = self
                .style
                .size
                .height
                .or(self.style.size.width)
                .and_then(|length| match length {
                    gpui_kit::Length::Definite(gpui_kit::DefiniteLength::Absolute(length)) => {
                        Some(length.to_pixels(window.rem_size()).as_f32())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| rems(1.).to_pixels(window.rem_size()).as_f32());
            let natural =
                self.natural
                    .map(|(points, large)| crate::native_symbols::NaturalSymbol {
                        points: points.to_pixels(window.rem_size()).as_f32(),
                        large,
                    });
            if let Some(image) = crate::native_symbols::image_in_box(
                self.name,
                points,
                window.scale_factor(),
                weight,
                color,
                natural,
            ) {
                return gpui_kit::img(image)
                    .refine_style(&self.style)
                    .into_any_element();
            }
        }
        // Svg only paints when its own text color is set. Resolve inheritance here,
        // then apply caller refinements so explicit size and color still win.
        svg()
            .path(self.name.path())
            .text_color(window.text_style().color)
            .refine_style(&self.style)
            .into_any_element()
    }
}

/// A 16px SF Symbol when supplied by macOS, otherwise a bundled SVG icon.
pub fn icon(name: IconName) -> Icon {
    Icon {
        name,
        style: StyleRefinement::default(),
        natural: None,
    }
    .size(rems(1.))
    .flex_shrink_0()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::AssetSource;

    #[test]
    fn the_icons_this_crate_draws_are_in_the_default_bundle() {
        // Components reach for these without the application asking, so they
        // must resolve against the bundle every gpui-kit application gets.
        for name in [
            IconName::Check,
            IconName::Minus,
            IconName::Plus,
            IconName::ChevronDown,
            IconName::ChevronRight,
            IconName::ChevronLeft,
            IconName::Calendar,
            IconName::Star,
            IconName::ExternalLink,
            IconName::Close,
            IconName::Search,
            IconName::Menu,
            IconName::Settings,
            IconName::TriangleAlert,
        ] {
            let path = name.path();
            assert!(
                gpui_kit::assets::Assets.load(&path).unwrap().is_some(),
                "{path}"
            );
        }
    }
}
