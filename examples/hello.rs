use gpui_kit::{
    AppContext, Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, rems,
};
use gpui_macos::{
    ActiveTheme, ButtonVariant, IconName, button, focus_scope, icon, panel, status_bar, title_bar,
};

struct Hello {
    clicks: usize,
}

impl Render for Hello {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.macos().clone();
        focus_scope("hello")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme.font.clone())
            .child(title_bar("Hello, Mac", cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(rems(2.))
                    .child(
                        panel("Welcome to GPUI macOS", cx)
                            .w(rems(26.))
                            .max_w(gpui_kit::relative(1.))
                            .gap(rems(1.))
                            .child(
                                icon(IconName::Layers)
                                    .size(rems(3.))
                                    .text_color(theme.accent),
                            )
                            .child(
                                div()
                                    .text_size(rems(1.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Make yourself at home."),
                            )
                            .child(div().text_color(theme.secondary).child(
                                "Native controls. System appearance. A familiar Mac workspace.",
                            ))
                            .child(
                                button(
                                    "hello",
                                    format!("Clicked {} times", self.clicks),
                                    ButtonVariant::Primary,
                                    cx,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    },
                                )),
                            ),
                    ),
            )
            .child(status_bar(cx).child("Ready").child(theme.name))
    }
}

fn main() {
    gpui_macos::application()
        .with_assets(gpui_macos::NativeAssets::new(gpui_kit::assets::AllAssets))
        .run(|cx| {
            gpui_macos::init(cx);
            gpui_macos::open_window(gpui_macos::window_options("Hello, Mac", cx), cx, |_, cx| {
                cx.new(|_| Hello { clicks: 0 })
            })
            .expect("open native window");
            cx.activate(true);
        });
}
