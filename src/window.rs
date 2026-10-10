//! Window presentation shared by the native and advanced component layers.
use crate::ActiveTheme;
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
        surface
            .style()
            .refine(
                &gpui_kit::StyleRefinement::default().bg(if cx.macos().reduce_transparency {
                    cx.macos().background
                } else {
                    Hsla::transparent_black()
                }),
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
