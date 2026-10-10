//! Dock presentation. Layout, reconciliation and drag operations stay in base.
use crate::{ActiveTheme, ButtonVariant, button};
use gpui_kit::base::dock::*;
use gpui_kit::rems;
use gpui_kit::{AnyElement, App, Context, Div, SharedString, Stateful, Window, div, prelude::*};
use std::{rc::Rc, time::Duration};

pub fn dock_area(
    id: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut Context<DockArea>,
) -> DockArea {
    DockArea::new(id, None, window, cx).with_renderer(Rc::new(MacosDock))
}

struct MacosDock;
impl DockAreaRenderer for MacosDock {
    fn frame(&self, _: &mut Window, cx: &mut App) -> Stateful<Div> {
        let t = cx.macos();
        div()
            .id("macos-dock")
            .bg(t.background)
            .font_family(t.font.clone())
            .text_size(rems(0.8125))
            .text_color(t.foreground)
    }
    fn tab_group_renderer(&self) -> Rc<dyn TabGroupRenderer> {
        Rc::new(MacosDock)
    }
}

impl TabGroupRenderer for MacosDock {
    fn render_tab_bar(&self, group: &TabGroupContext, _: &mut Window, cx: &mut App) -> AnyElement {
        let t = cx.macos();
        let mut bar = div()
            .id("dock-tab-bar")
            .flex()
            .items_center()
            .gap(rems(0.375))
            .p(rems(0.375))
            .border_b_1()
            .border_color(t.divider())
            .bg(t.normal_fill());
        for (index, panel) in group.panels().iter().enumerate() {
            let target = group.clone();
            let drop_target = group.clone();
            let label = panel.panel_name(cx);
            let tab = button(index, label, ButtonVariant::Outline, cx)
                .debug_selector(move || format!("dock-tab-{label}"))
                .rounded(rems(0.5))
                .border_0()
                .shadow_none()
                .fill_duration(Duration::ZERO)
                .bg(t.foreground.opacity(0.))
                .selected_fill(t.surface)
                .hover(|s| {
                    s.bg(if index == group.active_ix() {
                        t.surface
                    } else {
                        t.hover_fill()
                    })
                })
                .active(|s| {
                    s.bg(if index == group.active_ix() {
                        t.surface
                    } else {
                        t.pressed_fill()
                    })
                })
                .focus_visible(|s| {
                    s.bg(if index == group.active_ix() {
                        t.surface
                    } else {
                        t.hover_fill()
                    })
                    .shadow(t.focus_shadow())
                })
                .min_w(rems(6.))
                .selected(index == group.active_ix())
                .on_click(move |_, window, cx| target.select_tab(index, window, cx))
                .when(group.is_draggable(), |tab| {
                    tab.when_some(group.drag_panel(index, cx), |tab, drag| {
                        tab.on_drag(drag, move |_, _, _, cx| cx.new(|_| DockDragLabel(label)))
                    })
                })
                .when(group.is_droppable(), |tab| {
                    tab.on_drop(move |drag: &DragPanel, window, cx| {
                        drop_target.drop_panel(drag.clone(), Some(index), true, window, cx);
                    })
                });
            bar = bar.child(tab);
        }
        if let Some(panel) = group.active_panel() {
            let target = group.clone();
            let id = panel.panel_id(cx);
            if group.is_closable() && panel.closable(cx) {
                bar = bar.child(
                    button("dock-close", "Close", ButtonVariant::Secondary, cx)
                        .ml_auto()
                        .on_click(move |_, window, cx| target.close(id, window, cx)),
                );
            }
        }
        bar.into_any_element()
    }
    fn render_drop_indicator(
        &self,
        indicator: DropIndicator,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let hint = div()
            .absolute()
            .inset_0()
            .bg(cx.macos().accent.opacity(0.15))
            .border_1()
            .border_color(cx.macos().accent);
        Some(
            match indicator.placement() {
                Some(gpui_kit::base::Placement::Left) => hint.w(gpui_kit::relative(0.5)),
                Some(gpui_kit::base::Placement::Right) => hint.left(gpui_kit::relative(0.5)),
                Some(gpui_kit::base::Placement::Top) => hint.h(gpui_kit::relative(0.5)),
                Some(gpui_kit::base::Placement::Bottom) => hint.top(gpui_kit::relative(0.5)),
                _ => hint,
            }
            .into_any_element(),
        )
    }
}

struct DockDragLabel(&'static str);
impl gpui_kit::Render for DockDragLabel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(rems(0.625))
            .py(rems(0.375))
            .border_1()
            .rounded(rems(0.5))
            .shadow(cx.macos().popover_shadow())
            .border_color(cx.macos().glass_border())
            .bg(cx.macos().glass_fill())
            .text_color(cx.macos().foreground)
            .child(self.0)
    }
}
