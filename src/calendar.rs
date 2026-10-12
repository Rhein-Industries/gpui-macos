//! Calendar presentation over gpui-base's date and view state.
use crate::NativeTextStyle as _;
use crate::{ActiveTheme, IconName, icon};
use gpui_kit::base::{Calendar, CalendarItemKind, CalendarState};
use gpui_kit::rems;
use gpui_kit::{App, ElementId, Entity, Role, prelude::*};

/// A calendar with selectable days, month/year navigation and range styling.
/// Observe the supplied state or subscribe to base CalendarEvent::Selected.
pub fn calendar(id: impl Into<ElementId>, state: &Entity<CalendarState>, cx: &App) -> Calendar {
    let t = cx.macos().clone();
    Calendar::new(id, state)
        .w(rems(15.5))
        .p(rems(0.75))
        .gap(rems(0.5))
        .border_1()
        .rounded(rems(0.75))
        .border_color(t.control_border())
        .shadow(t.popover_shadow())
        .bg(t.surface)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .text_color(t.foreground)
        .label(|kind, value| match kind {
            CalendarItemKind::Weekday => {
                ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"][value as usize].into()
            }
            CalendarItemKind::Month | CalendarItemKind::MonthToggle => [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ][(value - 1) as usize]
                .into(),
            CalendarItemKind::Previous => "Previous".into(),
            CalendarItemKind::Next => "Next".into(),
            _ => value.to_string().into(),
        })
        .item(move |item, state, _, _| {
            let kind = state.kind();
            let interactive = kind != CalendarItemKind::Weekday;
            let mut item = item
                .flex()
                .items_center()
                .justify_center()
                .h(rems(
                    if matches!(
                        kind,
                        CalendarItemKind::Previous
                            | CalendarItemKind::Next
                            | CalendarItemKind::MonthToggle
                            | CalendarItemKind::YearToggle
                    ) {
                        1.5
                    } else {
                        1.75
                    },
                ))
                .px(rems(0.25))
                .border_1()
                .rounded(rems(0.375))
                .border_color(if state.is_today() {
                    t.accent
                } else {
                    t.foreground.opacity(0.)
                })
                .text_color(if state.is_active() {
                    t.on_accent
                } else if state.is_muted() || kind == CalendarItemKind::Weekday {
                    t.secondary
                } else {
                    t.foreground
                })
                .bg(if state.is_active() {
                    t.accent
                } else if state.is_in_range() {
                    t.selection
                } else {
                    t.foreground.opacity(0.)
                })
                .when(interactive, |item| {
                    item.role(Role::Button)
                        .when(!state.is_disabled(), |item| item.focusable())
                })
                // ControlLabel, spec §6.2.10: disabled text uses tertiary ink.
                .when(state.is_disabled() && interactive, |item| {
                    item.text_color(crate::state::paint(
                        crate::state::PaintRole::ControlLabel,
                        crate::state::Part::Label,
                        crate::state::States::active().disabled(true),
                        &t,
                    ))
                })
                .when(!state.is_disabled() && interactive, |item| {
                    item.cursor_pointer()
                        .hover(|s| {
                            s.bg(if state.is_active() {
                                t.accent
                            } else {
                                t.hover_fill()
                            })
                        })
                        .focus_visible(|s| s.border_color(t.accent).shadow(t.focus_shadow()))
                })
                .when(
                    matches!(kind, CalendarItemKind::Day | CalendarItemKind::Weekday),
                    |item| item.w(rems(2.)).flex_shrink_0(),
                )
                .when(
                    matches!(kind, CalendarItemKind::Previous | CalendarItemKind::Next),
                    |item| item.w(rems(1.5)).flex_shrink_0(),
                );
            if matches!(kind, CalendarItemKind::Previous | CalendarItemKind::Next) {
                let previous = kind == CalendarItemKind::Previous;
                item = item
                    .clear_children()
                    .aria_label(if previous {
                        "Previous calendar page"
                    } else {
                        "Next calendar page"
                    })
                    .debug_selector(move || {
                        if previous {
                            "calendar-previous"
                        } else {
                            "calendar-next"
                        }
                        .into()
                    })
                    .child(
                        icon(if previous {
                            IconName::ChevronLeft
                        } else {
                            IconName::ChevronRight
                        })
                        .size(rems(0.875)),
                    );
            }
            item.into_any_element()
        })
}
