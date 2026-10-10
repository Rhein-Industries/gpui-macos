//! Composable section navigation with gpui-base state and accessibility.
use crate::{ActiveTheme, ButtonVariant, button};
use gpui_kit::base::{
    Accordion, AccordionPanel, AccordionTrigger, Pagination, PaginationItem, PaginationState,
};
use gpui_kit::rems;
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div,
};

pub fn accordion(id: impl Into<ElementId>, cx: &App) -> Accordion {
    Accordion::new(id)
        .w_full()
        .rounded(rems(0.625))
        .overflow_hidden()
        .bg(cx.macos().normal_fill())
}

pub fn accordion_trigger(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    open: bool,
    cx: &App,
) -> AccordionTrigger {
    let t = cx.macos();
    AccordionTrigger::new(id)
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .h(rems(2.25))
        .px(rems(0.75))
        .border_2()
        .rounded(rems(0.5))
        .border_color(t.foreground.opacity(0.))
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .font_weight(FontWeight::MEDIUM)
        .hover(|s| s.bg(t.hover_fill()))
        .focus_visible(|s| s.border_color(t.accent))
        .child(label.into())
        .child(crate::icon(if open {
            crate::IconName::ChevronDown
        } else {
            crate::IconName::ChevronRight
        }))
}

pub fn accordion_panel(cx: &App) -> AccordionPanel {
    let t = cx.macos();
    AccordionPanel::new()
        .px(rems(0.625))
        .py(rems(0.5))
        .text_size(rems(0.8125))
        .font_family(t.font.clone())
        .text_color(t.secondary)
}

pub fn pagination(id: impl Into<ElementId>, state: PaginationState, cx: &App) -> Pagination {
    let items = state.items();
    Pagination::new(id, state.clone())
        .flex()
        .flex_wrap()
        .items_center()
        .gap(rems(0.25))
        .children(items.into_iter().map(|item| {
            match item {
                PaginationItem::Page(page) => {
                    let state = state.clone();
                    button(
                        ("page", page),
                        page.to_string(),
                        ButtonVariant::Secondary,
                        cx,
                    )
                    .selected(page == state.current_page())
                    .w(rems(2.))
                    .px(rems(0.))
                    .on_click(move |_, window, cx| state.request_page(page, window, cx))
                    .into_any_element()
                }
                PaginationItem::Ellipsis(_) => div()
                    .w(rems(1.5))
                    .text_color(cx.macos().secondary)
                    .child("…")
                    .into_any_element(),
            }
        }))
}

/// A controlled expandable region. Ordinary children remain visible; `content`
/// is rendered only while open. The caller owns the trigger and its state.
pub fn collapsible(open: bool, cx: &App) -> gpui_kit::base::Collapsible {
    gpui_kit::base::Collapsible::new()
        .open(open)
        .flex()
        .flex_col()
        .gap(rems(0.875))
        .w_full()
        .text_color(cx.macos().foreground)
        .text_size(rems(0.8125))
}

/// Stateful page navigation with the base push/pop/forward lifecycle.
pub fn nav_stack(
    state: &gpui_kit::Entity<gpui_kit::base::NavStackState>,
    cx: &App,
) -> gpui_kit::base::NavStack {
    let t = cx.macos();
    gpui_kit::base::NavStack::new(state)
        .w_full()
        .h(rems(17.5))
        .border_1()
        .rounded(rems(0.75))
        .border_color(t.divider())
        .bg(t.surface)
        .font_family(t.font.clone())
        .text_size(rems(0.8125))
        .text_color(t.foreground)
}
