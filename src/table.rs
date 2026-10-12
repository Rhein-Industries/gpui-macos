//! Primitive tables: application-owned columns, rows and content.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use gpui_kit::base::{Table, TableCell, TableHead, TableRow};
use gpui_kit::rems;
use gpui_kit::{App, ElementId, FontWeight, InteractiveElement, Styled, prelude::FluentBuilder};

pub fn table(id: impl Into<ElementId>, cx: &App) -> Table {
    let t = cx.macos();
    Table::new(id)
        .w_full()
        .overflow_hidden()
        .bg(t.background)
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
}

/// `index` is the one-based accessibility row index, including the header.
/// Alternating content rows use the standard subtle background fill.
pub fn table_row(id: impl Into<ElementId>, index: usize, cx: &App) -> TableRow {
    let t = cx.macos();
    TableRow::new(id, index)
        .flex()
        .min_h(rems(1.75))
        .when(index > 1 && index % 2 == 1, |row| row.bg(t.normal_fill()))
        .hover(|s| s.bg(t.hover_fill()))
}

/// `index` is the one-based accessibility column index.
pub fn table_head(id: impl Into<ElementId>, index: usize, cx: &App) -> TableHead {
    let t = cx.macos();
    TableHead::new(id, index)
        .flex_1()
        .min_w_0()
        .px(rems(0.5))
        .py(rems(0.375))
        .bg(t.normal_fill())
        .border_b_1()
        .border_color(t.divider())
        .text_color(t.secondary)
        .font_weight(FontWeight::MEDIUM)
}

pub fn table_cell(id: impl Into<ElementId>, index: usize, _cx: &App) -> TableCell {
    TableCell::new(id, index)
        .flex_1()
        .min_w_0()
        .px(rems(0.5))
        .py(rems(0.375))
}
