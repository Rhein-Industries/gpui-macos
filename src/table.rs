//! Primitive tables: application-owned columns, rows and content.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use crate::tokens::{ROW_HEIGHT, ROW_PADDING_Y, TABLE_HEADER_HEIGHT, TABLE_SEPARATOR, pt};
use gpui_kit::base::{Table, TableCell, TableHead, TableRow};
use gpui_kit::rems;
use gpui_kit::{App, ElementId, FontWeight, Styled, prelude::FluentBuilder};

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
/// Alternating content rows use `alternating_row` (spec §2A.2). Rows have no
/// hover (§6.5, C17); paint a selected row with [`crate::row_selection`].
/// Body rows are 24 pt (spec §5.2, C16).
pub fn table_row(id: impl Into<ElementId>, index: usize, cx: &App) -> TableRow {
    let t = cx.macos();
    TableRow::new(id, index)
        .flex()
        .min_h(pt(ROW_HEIGHT))
        .when(index > 1 && index % 2 == 1, |row| {
            row.bg(t.alternating_row())
        })
}

/// `index` is the one-based accessibility column index.
/// A header cell is 27 pt above a separator (spec §5.2, C16). The separator
/// is a one-point hairline at 100% zoom (two device pixels on Retina) that
/// scales with zoom, approximating the measured 1.5 pt.
pub fn table_head(id: impl Into<ElementId>, index: usize, cx: &App) -> TableHead {
    let t = cx.macos();
    let mut head = TableHead::new(id, index)
        .flex_1()
        .min_w_0()
        .px(rems(0.5))
        // One exact height on the device grid: a fixed 28 pt border box
        // (27 pt header plus the separator) that content cannot push past.
        // Block layout keeps caller labels width-constrained, so `truncate`
        // still ellipsizes; a 27 pt line centers the Body label vertically
        // without separately rounded padding.
        .h(pt(TABLE_HEADER_HEIGHT + TABLE_SEPARATOR))
        .py(rems(0.))
        .overflow_hidden()
        .line_height(pt(TABLE_HEADER_HEIGHT))
        .bg(t.normal_fill())
        .border_color(t.divider())
        .text_color(t.secondary)
        .font_weight(FontWeight::MEDIUM);
    head.style().border_widths.bottom = Some(pt(TABLE_SEPARATOR).into());
    head
}

/// A Body cell: 4 pt above and below the 16 pt line makes the 24 pt row.
pub fn table_cell(id: impl Into<ElementId>, index: usize, _cx: &App) -> TableCell {
    TableCell::new(id, index)
        .flex_1()
        .min_w_0()
        .px(rems(0.5))
        .py(pt(ROW_PADDING_Y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        Context, InteractiveElement, IntoElement, ParentElement, Render, TestAppContext, Window,
        div, px,
    };

    const LABEL: &str = "A very long column title that cannot fit in its header cell";

    struct Harness;
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(rems(10.)).child(
                table("table", cx).child(
                    table_row("heading", 1, cx).child(
                        table_head("title", 1, cx)
                            .truncate()
                            .underline()
                            .debug_selector(|| "long-head".into())
                            .child(LABEL),
                    ),
                ),
            )
        }
    }

    /// A long label in a constrained header ellipsizes inside the cell rather
    /// than keeping its full width, and the header is one exact height, on
    /// 1x and 2x displays at 100% to 200% zoom.
    #[gpui_kit::test]
    fn long_header_labels_truncate_inside_the_cell(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        for (scale, zoom) in [1f32, 2.]
            .into_iter()
            .flat_map(|s| [1f32, 1.25, 1.5, 1.75, 2.].map(|z| (s, z)))
        {
            cx.simulate_scale_factor_change(scale);
            cx.update(|window, cx| {
                window.set_rem_size(px(16. * zoom));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let head = cx.debug_bounds("long-head").unwrap();
            // One exact 28 pt length on the device grid (header plus
            // separator), whatever the label's own rounding.
            let device = ((28. * zoom * scale) - 0.5).ceil() / scale;
            assert_eq!(head.size.height.as_f32(), device, "{scale}x {zoom}");
            assert_truncated(cx, head, zoom);
        }
    }

    fn assert_truncated(
        cx: &mut gpui_kit::VisualTestContext,
        head: gpui_kit::Bounds<gpui_kit::Pixels>,
        zoom: f32,
    ) {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let full = window
                .text_system()
                .layout_line(
                    LABEL,
                    px(13. * zoom),
                    &[gpui_kit::TextRun {
                        len: LABEL.len(),
                        font: gpui_kit::font(cx.macos().font.clone()),
                        color: gpui_kit::Hsla::black(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    }],
                    None,
                )
                .width;
            // The underline spans the laid-out line, so its width is the
            // truncated label (with its ellipsis), not the full text.
            let underlines = window.painted_underlines();
            assert_eq!(underlines.len(), 1);
            let line = underlines[0].bounds;
            assert!(line.size.width.0 > 0.);
            assert!(
                line.size.width.0 / scale <= (head.size.width - px(16. * zoom)).as_f32() + 0.5,
                "label fits the cell: {line:?} in {head:?}"
            );
            assert!(line.size.width.0 / scale < full.as_f32() / 2., "{full:?}");
        });
    }
}
