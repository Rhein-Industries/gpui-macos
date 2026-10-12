//! Window presentation shared by the native and advanced component layers.
use crate::{ActiveTheme, NativeTextStyle as _, TextStyle};
use gpui_kit::base::{Root, RootPlugin};
use gpui_kit::{
    App, Context, Hsla, IntoElement, Refineable, Render, Stateful, Styled, Window, div, px,
};

pub(crate) struct WindowStyle {
    scale: f32,
}
impl RootPlugin for WindowStyle {
    fn prepare(&mut self, window: &mut Window, _: &mut Context<Self>) {
        // The Kit plugin also supplies a rem size. Our per-window scale takes
        // precedence so it survives redraws, overlays, and appearance changes.
        window.set_rem_size(px(16. * self.scale));
    }
    fn style(&self, surface: &mut Stateful<gpui_kit::Div>, _: &mut Window, cx: &mut App) {
        surface.style().refine(
            &gpui_kit::StyleRefinement::default()
                .bg(if cx.macos().reduce_transparency {
                    cx.macos().background
                } else {
                    Hsla::transparent_black()
                })
                .native_text_style(TextStyle::Body, cx.macos()),
        );
    }
}
impl Render for WindowStyle {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
pub(crate) fn init(cx: &mut App) {
    Root::register_plugin(cx, |window, _| WindowStyle {
        scale: window.rem_size().as_f32() / 16.,
    });
}
/// Set this window's interface zoom from 50% to 200%, preserving it on redraw.
/// Ordinary native window bounds and physical hairline borders remain in pixels.
pub fn set_zoom(percent: u16, window: &mut Window, cx: &mut App) {
    let scale = percent.clamp(50, 200) as f32 / 100.;
    if let Some(style) = window
        .root::<Root>()
        .flatten()
        .and_then(|root| root.read(cx).plugin::<WindowStyle>())
    {
        style.update(cx, |style, cx| {
            style.scale = scale;
            cx.notify();
        });
    }
    window.set_rem_size(px(16. * scale));
    window.refresh();
}
/// Current interface zoom as a percentage of the 16px rem baseline.
pub fn zoom_percent(window: &Window) -> u16 {
    (window.rem_size().as_f32() / 16. * 100.).round() as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        AppContext, InteractiveElement, ParentElement, TestAppContext, VisualTestContext,
    };

    struct Body;
    impl Render for Body {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(
                    div()
                        // Body is inherited from the Root plugin.
                        .debug_selector(|| "body-label".into())
                        .child("Label"),
                )
                .child(
                    div()
                        .debug_selector(|| "title3-label".into())
                        .native_text_style(TextStyle::Title3, cx.macos())
                        .child("Title"),
                )
                .child(crate::keycap("⌘Q", cx).debug_selector(|| "tight-keycap".into()))
                .child(
                    div()
                        .debug_selector(|| "dialog-title".into())
                        .child(crate::dialog_title("Dialog", cx)),
                )
                .child(crate::panel("Panel", cx).debug_selector(|| "titled-panel".into()))
        }
    }

    #[gpui_kit::test]
    fn root_text_uses_the_native_line_height_at_every_zoom(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let window = cx.add_window(|window, cx| {
            let body = cx.new(|_| Body);
            Root::new(body, window, cx)
        });
        let cx = &mut VisualTestContext::from_window(window.into(), cx);
        for (zoom, height) in [(100, 16.), (200, 32.)] {
            cx.update(|window, cx| {
                set_zoom(zoom, window, cx);
                window.draw(cx).clear(cx);
            });
            let scale = zoom as f32 / 100.;
            assert_eq!(
                cx.debug_bounds("dialog-title").unwrap().size.height,
                px(20. * scale)
            );
            assert_eq!(
                cx.debug_bounds("titled-panel").unwrap().size.height,
                px(56. * scale)
            );
            let keycap_advance = if cfg!(target_os = "macos") { 12. } else { 11. };
            assert_eq!(
                cx.debug_bounds("tight-keycap").unwrap().size.height,
                px((keycap_advance + 4.) * scale + 2.)
            );
            let bounds = cx.debug_bounds("body-label").unwrap();
            assert_eq!(bounds.size.height, px(height), "13 point label at {zoom}%");
            assert_eq!(
                cx.debug_bounds("title3-label").unwrap().size.height,
                px(20. * zoom as f32 / 100.)
            );
        }
    }
}
