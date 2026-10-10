//! Rounded content surfaces and non-interactive information components.
use crate::ActiveTheme;
use gpui_kit::base::{Progress, ProgressIndicator, ProgressTrack};
use gpui_kit::rems;
use gpui_kit::{
    Action, App, Div, ElementId, FontWeight, IntoElement, Modifiers, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, relative,
};

pub fn panel(title: impl Into<SharedString>, cx: &App) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(rems(0.875))
        .p(rems(1.125))
        .rounded(rems(0.75))
        .bg(t.normal_fill())
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .text_color(t.foreground)
        .child(
            div()
                .text_size(rems(0.875))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
}

pub fn separator(cx: &App) -> Div {
    div()
        .w_full()
        .h(rems(0.0625))
        .flex_shrink_0()
        .bg(cx.macos().divider())
}

/// A vertical rule for toolbars. Override height to fit a different row size.
pub fn vertical_separator(cx: &App) -> Div {
    div()
        .w(rems(0.0625))
        .h(rems(1.25))
        .flex_shrink_0()
        .bg(cx.macos().divider())
}

pub fn keycap(key: impl Into<SharedString>, cx: &App) -> Div {
    let t = cx.macos();
    // macOS draws ⌘⌥⇧ from the system UI face (Menlo lacks them and falls back
    // to undersized glyphs), a touch larger so the symbols stay legible.
    let (font, size) = if cfg!(target_os = "macos") {
        (t.font.clone(), rems(0.75))
    } else {
        (t.mono_font.clone(), rems(0.6875))
    };
    div()
        .px(rems(0.25))
        .py(rems(0.125))
        .border_1()
        .rounded(rems(0.25))
        .border_color(t.divider())
        .bg(t.inset)
        .font_family(font)
        .text_size(size)
        // Pin to the cap's own font size so an inherited line height can't inflate it.
        .line_height(relative(1.))
        .font_weight(FontWeight::NORMAL)
        .text_color(t.secondary)
        .child(key.into())
}

/// A row of keycaps, one per keystroke, e.g. `["⌘Q"]` or `["ctrl+k", "ctrl+s"]`.
pub fn keycaps(keys: impl IntoIterator<Item = impl Into<SharedString>>, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap(rems(0.25))
        .children(keys.into_iter().map(|key| keycap(key, cx)))
}

/// The keycaps for whichever binding the keymap resolves for `action`, or
/// `None` when nothing is bound. The lookup reads the last rendered frame, so
/// context-scoped bindings show once their context has been painted.
pub fn keycap_for_action(action: &dyn Action, window: &Window, cx: &App) -> Option<Div> {
    let binding = window.highest_precedence_binding_for_action(action)?;
    let keys: Vec<_> = binding
        .keystrokes()
        .iter()
        .map(|stroke| keystroke_label(stroke.modifiers(), stroke.key()))
        .collect();
    (!keys.is_empty()).then(|| keycaps(keys, cx))
}

/// One keystroke as a single keycap label: `⌘Q` on macOS, `ctrl+q` elsewhere.
pub fn keystroke_label(modifiers: &Modifiers, key: &str) -> SharedString {
    let separator = if cfg!(target_os = "macos") { "" } else { "+" };
    let labels = keystroke_labels(modifiers, key);
    let labels: Vec<&str> = labels.iter().map(AsRef::as_ref).collect();
    labels.join(separator).into()
}

/// Split a keystroke into the labels this platform prints on its keys: on
/// macOS symbols in its order (⌃⌥⇧⌘) with capitalized keys like its menus,
/// elsewhere lowercase words in macOS's style (ctrl alt shift super).
pub fn keystroke_labels(modifiers: &Modifiers, key: &str) -> Vec<SharedString> {
    let mac = cfg!(target_os = "macos");
    let pick = |symbol: &'static str, word: &'static str| if mac { symbol } else { word };
    let platform = pick(
        "⌘",
        if cfg!(target_os = "windows") {
            "win"
        } else {
            "super"
        },
    );
    let mut labels: Vec<SharedString> = [
        (modifiers.control, pick("⌃", "ctrl")),
        (modifiers.alt, pick("⌥", "alt")),
        (modifiers.shift, pick("⇧", "shift")),
        (modifiers.platform, platform),
        (modifiers.function, "fn"),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .map(|(_, label)| label.into())
    .collect();
    let label = match key {
        "" => return labels,
        "ctrl" | "control" => pick("⌃", "ctrl"),
        "alt" => pick("⌥", "alt"),
        "shift" => pick("⇧", "shift"),
        "cmd" | "super" | "win" | "platform" => platform,
        "enter" => pick("⏎", "enter"),
        "escape" => pick("⎋", "esc"),
        "backspace" => pick("⌫", "backspace"),
        "delete" => pick("⌦", "delete"),
        "tab" => pick("⇥", "tab"),
        "left" => pick("←", "left"),
        "right" => pick("→", "right"),
        "up" => pick("↑", "up"),
        "down" => pick("↓", "down"),
        "pageup" => pick("⇞", "page up"),
        "pagedown" => pick("⇟", "page down"),
        key if mac => {
            let mut chars = key.chars();
            let first = chars.next().map(char::to_uppercase);
            labels.push(
                first
                    .into_iter()
                    .flatten()
                    .chain(chars)
                    .collect::<String>()
                    .into(),
            );
            return labels;
        }
        key => {
            labels.push(key.to_lowercase().into());
            return labels;
        }
    };
    labels.push(label.into());
    labels
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Neutral,
    Success,
    Warning,
    Error,
}
impl Status {
    /// The theme colour that carries this status.
    pub fn color(self, cx: &App) -> gpui_kit::Hsla {
        let t = cx.macos();
        match self {
            Status::Neutral => t.secondary,
            Status::Success => t.success,
            Status::Warning => t.warning,
            Status::Error => t.danger,
        }
    }
}

/// Inline feedback in a softly tinted surface with semantic theme colors.
///
/// Unlike [`crate::alert_dialog`] this stays in the layout, so it suits
/// persistent conditions the reader can act on without leaving the view.
pub fn alert(message: impl Into<SharedString>, status: Status, cx: &App) -> Div {
    let t = cx.macos();
    let color = status.color(cx);
    div()
        .flex()
        .items_center()
        .gap(rems(0.625))
        .p(rems(0.625))
        .border_1()
        .rounded(rems(0.625))
        .border_color(color.opacity(0.2))
        .bg(color.opacity(0.08))
        .text_color(color)
        .font_family(t.font.clone())
        .child(
            crate::icon(match status {
                Status::Success => crate::IconName::Check,
                Status::Neutral => crate::IconName::Minus,
                Status::Warning | Status::Error => crate::IconName::TriangleAlert,
            })
            .size(rems(0.875))
            .flex_shrink_0()
            .text_color(color),
        )
        .child(div().flex_1().min_w_0().child(message.into()))
}

pub fn badge(label: impl Into<SharedString>, status: Status, cx: &App) -> Div {
    let t = cx.macos();
    let color = match status {
        Status::Neutral => t.secondary,
        Status::Success => t.success,
        Status::Warning => t.warning,
        Status::Error => t.danger,
    };
    div()
        .px(rems(0.375))
        .py(rems(0.125))
        .rounded_full()
        .bg(color.opacity(0.12))
        .text_color(color)
        .font_family(t.font.clone())
        .text_size(rems(0.6875))
        .child(label.into())
}

pub fn empty_state(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    cx: &App,
) -> Div {
    let t = cx.macos();
    div()
        .flex()
        .flex_col()
        .items_center()
        .text_center()
        .gap(rems(0.5))
        .p(rems(1.125))
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .text_color(t.foreground)
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title.into()))
        .child(div().text_color(t.secondary).child(description.into()))
}

/// Determinate progress, clamped to 0..=100; non-finite values become zero.
pub fn progress(id: impl Into<ElementId>, value: f32, _: &App) -> Progress {
    let value = normalized_progress(value);
    Progress::new(id)
        .value(value)
        .w_full()
        .child(NativeProgressPaint(value))
}

#[derive(IntoElement)]
struct NativeProgressPaint(f32);

impl RenderOnce for NativeProgressPaint {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.macos();
        let fill = if window.is_window_active() {
            t.accent
        } else {
            // Both captured AppKit appearances resolve the inactive fill to
            // RGB 170; the track and surrounding surfaces keep their own roles.
            gpui_kit::rgb(0xaaaaaa).into()
        };
        ProgressTrack::new()
            .relative()
            .w_full()
            .h(rems(0.5))
            .rounded_full()
            .overflow_hidden()
            .bg(gpui_kit::rgb(if t.is_dark() { 0x404144 } else { 0xd8d8d8 }))
            .child(
                div()
                    .absolute()
                    .top(rems(0.03125))
                    .bottom(rems(0.03125))
                    .left(rems(0.03125))
                    .right(rems(0.03125))
                    .rounded_full()
                    .bg(gpui_kit::rgb(if t.is_dark() { 0x38393c } else { 0xe8e8e8 })),
            )
            .child(
                ProgressIndicator::new()
                    .h_full()
                    .rounded_full()
                    .w(relative(self.0 / 100.))
                    .bg(fill),
            )
    }
}

fn normalized_progress(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., 100.)
    } else {
        0.
    }
}

/// Notification surface; compose actions and use gpui-base's ToastManager for
/// application-owned stacking and timeout policy.
pub fn toast(id: impl Into<gpui_kit::ElementId>, cx: &App) -> gpui_kit::base::Toast {
    let t = cx.macos();
    gpui_kit::base::Toast::new(id)
        .flex()
        .flex_col()
        .gap(rems(0.625))
        .p(rems(0.875))
        .w_full()
        .border_1()
        .rounded(rems(1.))
        .border_color(t.glass_border())
        .bg(t.glass_fill())
        .shadow(t.popover_shadow())
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
}

/// A circular identity marker. Supply initials or an icon in the fallback slot;
/// callers can replace it with `avatar_image` through the base image builder.
pub fn avatar(initials: impl Into<SharedString>, cx: &App) -> gpui_kit::base::Avatar {
    let t = cx.macos();
    gpui_kit::base::Avatar::new()
        .size(rems(2.))
        .flex_shrink_0()
        .overflow_hidden()
        .rounded_full()
        .border_1()
        .border_color(t.control_border())
        .bg(t.hover_fill())
        .font_family(t.font.clone())
        .text_size(rems(0.75))
        .text_color(t.foreground)
        .fallback(
            gpui_kit::base::AvatarFallback::new()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(initials.into()),
        )
}

/// An image slot sized to its avatar. Image loading and failure policy remain
/// with the application, matching gpui-base's explicit slot API.
pub fn avatar_image(source: impl Into<gpui_kit::ImageSource>) -> gpui_kit::base::AvatarImage {
    gpui_kit::base::AvatarImage::new(source).size_full()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui_kit::test]
    fn progress_paints_native_active_and_inactive_colors_without_changing_value(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        use gpui_kit::{Context, Render, px, size};
        struct Harness;
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().w(px(200.)).child(progress("transfer", 50., cx))
            }
        }
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| Harness);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            let active_bounds = cx.update(|window, cx| {
                assert!(window.is_window_active());
                window.draw(cx).clear(cx);
                let quads = window.painted_quads();
                let border: gpui_kit::Hsla =
                    gpui_kit::rgb(if theme.is_dark() { 0x404144 } else { 0xd8d8d8 }).into();
                let interior: gpui_kit::Hsla =
                    gpui_kit::rgb(if theme.is_dark() { 0x38393c } else { 0xe8e8e8 }).into();
                let track = quads
                    .iter()
                    .find(|quad| quad.background.as_solid() == Some(border))
                    .expect("native progress keeps its subtle outer border");
                assert_eq!(
                    track.bounds.size,
                    size(px(200.), px(8.)).scale(window.scale_factor())
                );
                let inner = quads
                    .iter()
                    .find(|quad| quad.background.as_solid() == Some(interior))
                    .expect("native progress paints its lighter empty interior");
                assert_eq!(
                    inner.bounds.size,
                    size(px(199.), px(7.)).scale(window.scale_factor())
                );
                let indicator = quads
                    .iter()
                    .find(|quad| quad.background.as_solid() == Some(theme.accent))
                    .expect("active progress paints the native accent");
                assert_eq!(
                    indicator.bounds.size,
                    size(px(100.), px(8.)).scale(window.scale_factor())
                );
                indicator.bounds
            });
            cx.deactivate_window();
            cx.update(|window, cx| {
                assert!(!window.is_window_active());
                window.draw(cx).clear(cx);
                let indicator = window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.bounds == active_bounds)
                    .expect("activation does not change the progress value or track geometry");
                let fill = indicator.background.as_solid().unwrap();
                assert_eq!(fill.s, 0., "inactive progress is neutral");
                assert_eq!(fill.a, 1.);
                assert!(
                    fill.l > 0.5 && fill.l < 0.8,
                    "the native inactive fill remains visible"
                );
                assert_ne!(fill, theme.accent);
            });
            cx.update(|window, _| window.activate_window());
            cx.run_until_parked();
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                assert!(
                    window
                        .painted_quads()
                        .into_iter()
                        .any(|quad| quad.bounds == active_bounds
                            && quad.background.as_solid() == Some(theme.accent)),
                    "reactivation restores the native accent"
                );
            });
        }
    }
    #[test]
    fn keystroke_labels_follow_platform_case_and_order() {
        let modifiers = Modifiers {
            platform: true,
            shift: true,
            ..Default::default()
        };
        let label = keystroke_label(&modifiers, "Q");
        if cfg!(target_os = "macos") {
            assert_eq!(label.as_ref(), "⇧⌘Q");
            assert_eq!(keystroke_labels(&Modifiers::none(), "f12"), ["F12"]);
        } else if cfg!(target_os = "windows") {
            assert_eq!(label.as_ref(), "shift+win+q");
        } else {
            assert_eq!(label.as_ref(), "shift+super+q");
        }
        assert_eq!(
            keystroke_labels(&Modifiers::none(), "pagedown"),
            [if cfg!(target_os = "macos") {
                "⇟"
            } else {
                "page down"
            }]
        );
    }
    #[test]
    fn progress_handles_invalid_and_out_of_range_values() {
        for (input, expected) in [
            (-1., 0.),
            (42., 42.),
            (150., 100.),
            (f32::NAN, 0.),
            (f32::INFINITY, 0.),
        ] {
            assert_eq!(normalized_progress(input), expected);
        }
    }
}
