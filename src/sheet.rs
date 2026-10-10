//! Edge-attached modal details, with dismissal and focus trapping owned by base.
use crate::{ActiveTheme, dialog_backdrop};
use gpui_kit::rems;
use gpui_kit::{App, Div, FocusHandle, div, prelude::*, relative};

/// Focus `focus` when opening; restore the trigger in `request_close`.
pub fn sheet(focus: &FocusHandle, cx: &mut App) -> gpui_kit::base::Sheet {
    gpui_kit::base::Sheet::new(cx)
        .focus_handle(focus.clone())
        .overlay(dialog_backdrop())
}

/// Floating right-edge sheet. Override width through the normal styling API.
pub fn sheet_surface(cx: &App) -> Div {
    let t = cx.macos();
    div()
        .absolute()
        .right(rems(0.75))
        .top(rems(0.75))
        .bottom(rems(0.75))
        .w(rems(22.5))
        .max_w(relative(1.))
        .flex()
        .flex_col()
        .gap(rems(0.875))
        .p(rems(1.125))
        .border_1()
        .rounded(rems(1.125))
        .border_color(t.glass_border())
        .bg(t.surface)
        .shadow(t.popover_shadow())
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .text_color(t.foreground)
        .occlude()
}
