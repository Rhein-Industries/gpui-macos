//! Gentle control feedback; GPUI Base honors the user's Reduce motion setting.
use gpui_kit::base::motion::Transition;
use std::time::Duration;
pub(crate) const CONTROL_COLOR: Duration = Duration::from_millis(120);
pub(crate) const CURSOR_COLOR: Duration = Duration::from_millis(80);
pub(crate) const SWITCH_TRAVEL: Duration = Duration::from_millis(180);
pub(crate) const POPUP_FADE: Duration = Duration::from_millis(160);
pub(crate) const SLIDER_TRAVEL: Duration = Duration::from_millis(140);
pub(crate) const SLIDER_KNOB_SCALE: Duration = Duration::from_millis(110);
pub(crate) fn color(duration: Duration) -> Transition {
    Transition::new(duration).ease(|t| t)
}
pub(crate) fn out_cubic(duration: Duration) -> Transition {
    Transition::new(duration).ease(|t| 1. - (1. - t).powi(3))
}
