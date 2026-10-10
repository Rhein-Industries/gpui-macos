//! A retained GPUI foreground above native system glass, beside an inline pane.
//! Build with `scripts/bundle-glass-compare.sh`; pass --light or --dark.
extern crate gpui_kit as gpui;

use gpui_kit::base::CheckboxState;
use gpui_kit::{
    App, Bounds, Context, Entity, FontWeight, Render, SharedString, Window, WindowBounds,
    WindowOptions, div, point, prelude::*, px, rgb, size,
};
use gpui_macos::{
    ActiveTheme, ButtonVariant, NativeAssets, Theme, button, checkbox, native, title_bar,
};

gpui_kit::actions!(glass_compare, [Quit]);

struct PaneContent {
    title: &'static str,
    enabled: CheckboxState,
    count: usize,
}

impl PaneContent {
    fn new(title: &'static str) -> Self {
        Self {
            title,
            enabled: CheckboxState::Checked,
            count: 0,
        }
    }
}

impl Render for PaneContent {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let foreground = cx.macos().foreground;
        let target = cx.entity();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(22.))
            .text_color(foreground)
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title),
            )
            .child(div().child("The foreground is ordinary, retained GPUI content."))
            .child(
                checkbox("pane-enabled", "Enable counter", self.enabled, cx).on_change(
                    move |checked, _, _, cx| {
                        target.update(cx, |pane, cx| {
                            pane.enabled = checked;
                            cx.notify();
                        });
                    },
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.))
                    .child(
                        button("pane-increment", "Add one", ButtonVariant::Primary, cx)
                            .disabled(self.enabled != CheckboxState::Checked)
                            .on_click(cx.listener(|pane, _, _, cx| {
                                pane.count += 1;
                                cx.notify();
                            })),
                    )
                    .child(div().child(format!("Count: {}", self.count))),
            )
    }
}

struct GlassCompare {
    inline: Entity<PaneContent>,
    panel: Option<native::GlassPanelHandle<PaneContent>>,
    moved: bool,
    status: SharedString,
}

impl GlassCompare {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Wait until the owning window has returned its root entity. The new
        // popup then builds its content in its own element arena.
        cx.defer_in(window, |this, window, cx| this.open_panel(window, cx));
        Self {
            inline: cx.new(|_| PaneContent::new("Inline GPUI pane")),
            panel: None,
            moved: false,
            status: "Preparing native glass…".into(),
        }
    }

    fn panel_bounds(&self) -> Bounds<gpui_kit::Pixels> {
        Bounds::new(
            if self.moved {
                point(px(220.), px(340.))
            } else {
                point(px(70.), px(190.))
            },
            size(px(350.), px(235.)),
        )
    }

    fn open_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.panel.as_ref().is_some_and(|panel| panel.is_open()) {
            return;
        }
        let mut options = native::GlassPanelOptions::new(self.panel_bounds());
        options.focus = false;
        options.dismiss_on_deactivate = false;
        options.corner_radius = px(20.);
        let target = cx.weak_entity();
        match native::open_glass_panel(
            options,
            window,
            cx,
            |_, cx| cx.new(|_| PaneContent::new("Native system glass")),
            move |_, cx| {
                let _ = target.update(cx, |this, cx| {
                    this.status = "Panel closed. Choose Open panel to restore it.".into();
                    cx.notify();
                });
            },
        ) {
            Ok(panel) => {
                self.status = if panel.is_native() {
                    "System glass is installed. Move the panel across the colored canvas."
                } else {
                    "Opaque fallback: system glass is unavailable or reduced transparency is enabled."
                }
                .into();
                self.panel = Some(panel);
            }
            Err(error) => self.status = format!("Could not open panel: {error}").into(),
        }
        cx.notify();
    }

    fn move_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(panel) = self.panel.as_ref().filter(|panel| panel.is_open()) else {
            self.open_panel(window, cx);
            return;
        };
        self.moved = !self.moved;
        match panel.reposition(self.panel_bounds(), window, cx) {
            Ok(()) => {
                self.status = "Panel moved; its counter and checkbox state are retained.".into()
            }
            Err(error) => {
                self.moved = !self.moved;
                self.status = format!("Could not move panel: {error}").into();
            }
        }
        cx.notify();
    }
}

impl Render for GlassCompare {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.macos().clone();
        let open = self.panel.as_ref().is_some_and(|panel| panel.is_open());
        div()
            .size_full()
            .relative()
            .bg(t.background)
            .text_color(t.foreground)
            .child(
                div()
                    .absolute()
                    .top(px(145.))
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .flex()
                    .children(
                        [0x6554c0, 0x297fa5, 0x12a9a0, 0xf0a246]
                            .map(|color| div().flex_1().h_full().bg(rgb(color))),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(460.))
                    .left(px(25.))
                    .right(px(25.))
                    .h(px(90.))
                    .rounded(px(28.))
                    .bg(rgb(0xe65d81)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(160.))
                    .left(px(70.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xffffff))
                    .child("Native backdrop samples the owner window"),
            )
            .child(
                div()
                    .absolute()
                    .top(px(160.))
                    .left(px(565.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xffffff))
                    .child("Inline GPUI color and border"),
            )
            .child(
                div()
                    .absolute()
                    .top(px(190.))
                    .left(px(565.))
                    .w(px(350.))
                    .h(px(235.))
                    .rounded(px(20.))
                    .bg(t.glass_fill())
                    .border_1()
                    .border_color(t.glass_border())
                    .shadow(t.popover_shadow())
                    .child(self.inline.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .child(title_bar("Glass Comparison", cx))
                    .child(
                        div()
                            .px(px(26.))
                            .py(px(12.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                button("open-panel", "Open panel", ButtonVariant::Secondary, cx)
                                    .disabled(open)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_panel(window, cx);
                                    })),
                            )
                            .child(
                                button("move-panel", "Move panel", ButtonVariant::Secondary, cx)
                                    .disabled(!open)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.move_panel(window, cx);
                                    })),
                            )
                            .child(
                                button("close-panel", "Close panel", ButtonVariant::Secondary, cx)
                                    .disabled(!open)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(panel) = this.panel.as_ref()
                                            && let Err(error) = panel.dismiss(cx)
                                        {
                                            this.status =
                                                format!("Could not close panel: {error}").into();
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .px(px(26.))
                            .pb(px(12.))
                            .text_color(t.secondary)
                            .child(self.status.clone()),
                    ),
            )
    }
}

fn main() {
    gpui_kit::platform::application()
        .with_assets(NativeAssets::new(gpui_kit::assets::AllAssets))
        .run(|cx: &mut App| {
            gpui_macos::init(cx);
            if let Some(mode) = std::env::args()
                .rev()
                .find(|arg| arg == "--light" || arg == "--dark")
            {
                if mode == "--dark" {
                    Theme::dark().apply(cx);
                } else {
                    Theme::light().apply(cx);
                }
            }
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([gpui_kit::KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus([gpui_kit::Menu::new("GPUI Glass Compare")
                .items([gpui_kit::MenuItem::action("Quit GPUI Glass Compare", Quit)])]);
            gpui_macos::open_window(
                WindowOptions {
                    focus: false,
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1000.), px(650.)),
                        cx,
                    ))),
                    window_min_size: Some(size(px(960.), px(620.))),
                    ..gpui_macos::window_options("GPUI Glass Compare", cx)
                },
                cx,
                |window, cx| cx.new(|cx| GlassCompare::new(window, cx)),
            )
            .expect("open glass comparison");
        });
}
