//! The radial macOS activity indicator on GPUI Kit's animation primitive.
use crate::{ActiveTheme, kit};
use gpui_kit::App;
use std::sync::LazyLock;

static RADIAL_ARTWORK: LazyLock<String> = LazyLock::new(|| {
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round">"#,
    );
    for ray in 0..12 {
        svg.push_str(&format!(
            r#"<path d="M12 3v4" transform="rotate({} 12 12)" opacity="{}"/>"#,
            ray * 30,
            0.15 + ray as f32 / 11. * 0.85,
        ));
    }
    svg.push_str("</g></svg>");
    svg
});

/// A system SF activity indicator on macOS, with a twelve-ray fallback.
/// Keeps Kit's size, color, custom-icon and easing APIs, and its static Reduce
/// Motion fallback.
pub fn native_spinner(cx: &App) -> kit::spinner::Spinner {
    #[cfg(target_os = "macos")]
    let artwork = crate::native_symbols::svg_for_path(&crate::IconName::Loader.path());
    #[cfg(target_os = "macos")]
    let artwork = artwork
        .as_deref()
        .unwrap_or_else(|| RADIAL_ARTWORK.as_bytes());
    #[cfg(not(target_os = "macos"))]
    let artwork = RADIAL_ARTWORK.as_bytes();
    kit::spinner::Spinner::new()
        .icon(kit::Icon::default().data(artwork))
        .color(cx.macos().secondary)
        .ease(|progress| progress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, IntoElement, Render, TestAppContext, Window, px, size};

    struct Indicator;
    impl Render for Indicator {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            native_spinner(cx)
        }
    }

    #[gpui_kit::test]
    fn radial_indicator_respects_reduce_motion(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        let window = cx.open_window(size(px(80.), px(80.)), |_, _| Indicator);
        cx.run_until_parked();
        assert_eq!(
            window
                .update(cx, |_, window, cx| window.simulate_next_frame(cx))
                .unwrap(),
            0
        );
    }
}
