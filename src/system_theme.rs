//! Appearance policy and per-window system observation.
use crate::{ActiveTheme, Theme};
use gpui_kit::{App, Global, Window};
use std::collections::HashSet;
#[derive(Default)]
struct AppearancePolicy {
    following: bool,
    windows: HashSet<gpui_kit::WindowId>,
}
impl Global for AppearancePolicy {}

impl Theme {
    /// Restore system appearance, preserving accent and material preferences.
    /// Attach [`crate::observe_window`] once to each newly opened window.
    pub fn follow_system(cx: &mut App) {
        cx.set_window_appearance(None);
        cx.default_global::<AppearancePolicy>().following = true;
        sync(cx);
    }
    pub fn is_following_system(cx: &App) -> bool {
        cx.try_global::<AppearancePolicy>()
            .is_some_and(|policy| policy.following)
    }
}
pub(crate) fn stop_following(cx: &mut App) {
    cx.default_global::<AppearancePolicy>().following = false;
}
fn sync(cx: &mut App) {
    let mut next = Theme::system(cx);
    if let Some(current) = cx.try_global::<Theme>() {
        next.glass_tint = current.glass_tint;
        // An accent explicitly set by an app should survive system appearance changes.
        let default = if current.is_dark() {
            Theme::dark()
        } else {
            Theme::light()
        };
        if current.accent != default.accent {
            next = next.with_accent(current.accent);
        }
    }
    if cx.try_global::<Theme>() != Some(&next) {
        next.apply_palette(cx);
    }
}
/// Follow macOS light/dark changes for this window. The observer lives with it.
/// Repeated calls on the same window do not install duplicate subscriptions.
pub fn observe_window(window: &mut Window, cx: &mut App) {
    if cx
        .default_global::<AppearancePolicy>()
        .windows
        .insert(window.window_handle().window_id())
    {
        window
            .observe_window_appearance(|_, cx| {
                if Theme::is_following_system(cx) {
                    sync(cx);
                }
                gpui_kit::base::apply_system_reduce_motion(cx);
            })
            .detach();
    }
}
/// Re-read Display accessibility preferences after an application settings change.
pub fn refresh_accessibility(cx: &mut App) {
    crate::native::refresh_sidebar_metrics();
    let mut theme = cx.macos().clone();
    theme.read_accessibility();
    theme.apply_palette(cx);
    gpui_kit::base::apply_system_reduce_motion(cx);
}
