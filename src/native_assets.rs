//! Runtime SF Symbols for both macOS adapters and upstream GPUI Kit icons.
use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;

/// An asset source that substitutes host-supplied SF Symbols on macOS.
///
/// Wrap the application's existing asset source with `NativeAssets::new(source)`.
/// Icon paths keep their upstream names, so GPUI Kit's own nested icons also use
/// the native artwork. Unavailable symbols, custom assets, and other platforms
/// delegate to the supplied source. System artwork is generated only in memory.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeAssets<A> {
    fallback: A,
}

impl<A> NativeAssets<A> {
    pub fn new(fallback: A) -> Self {
        Self { fallback }
    }
}

impl<A: AssetSource> AssetSource for NativeAssets<A> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        #[cfg(target_os = "macos")]
        if let Some(svg) = crate::native_symbols::svg_for_path(path) {
            return Ok(Some(Cow::Owned(svg.as_ref().to_vec())));
        }
        self.fallback.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        self.fallback.list(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CustomAssets;
    impl AssetSource for CustomAssets {
        fn load(&self, _: &str) -> Result<Option<Cow<'static, [u8]>>> {
            Ok(Some(Cow::Borrowed(b"custom asset bytes")))
        }
        fn list(&self, _: &str) -> Result<Vec<SharedString>> {
            Ok(vec!["custom.svg".into()])
        }
    }

    #[test]
    fn custom_assets_and_listing_pass_through() {
        let assets = NativeAssets::new(CustomAssets);
        assert_eq!(
            assets.load("custom.svg").unwrap().unwrap().as_ref(),
            b"custom asset bytes"
        );
        assert_eq!(
            assets.list("").unwrap(),
            vec![SharedString::from("custom.svg")]
        );
    }

    #[test]
    fn fallback_artwork_is_embedded_in_debug_application_bundles() {
        let bytes = gpui_kit::assets::AllAssets
            .load("icons/star.svg")
            .expect("bundled fallback icon loads")
            .expect("bundled fallback icon exists");
        assert!(
            matches!(bytes, Cow::Borrowed(_)),
            "App bundles must load embedded bytes without opening checkout files"
        );
        assert!(String::from_utf8_lossy(&bytes).contains("<svg"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn gpui_svg_renderer_accepts_native_symbols_with_embedded_alpha() {
        use gpui_kit::{DevicePixels, SvgRenderer, SvgSize, size};
        use std::sync::Arc;
        let assets = NativeAssets::new(gpui_kit::assets::AllAssets);
        let bytes = assets.load("icons/check.svg").unwrap().unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("data:image/png;base64,"));
        let renderer = SvgRenderer::new(Arc::new(assets));
        let parsed = renderer
            .parse_svg(&bytes)
            .expect("embedded SF Symbol PNG parses");
        let image = renderer
            .render_parsed(
                &parsed,
                SvgSize::ExactSize(size(DevicePixels(128), DevicePixels(128))),
            )
            .expect("embedded SF Symbol PNG rasterizes");
        let pixels = image.as_bytes(0).unwrap();
        assert_eq!(pixels.len(), 128 * 128 * 4);
        assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0));
        assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 0));
        let native = crate::native_symbols::image(
            crate::IconName::Check,
            16.,
            8.,
            gpui_kit::FontWeight::NORMAL,
            gpui_kit::rgb(0xffffff).into(),
        )
        .unwrap();
        let native_alpha = native
            .as_bytes(0)
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[3]);
        let rendered_alpha = pixels.as_chunks::<4>().0.iter().map(|pixel| pixel[3]);
        assert!(
            native_alpha.eq(rendered_alpha),
            "the Kit SVG pipeline preserves native symbol coverage exactly"
        );
    }
}
