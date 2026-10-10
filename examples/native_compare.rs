//! Matched content for the real AppKit app in tools/native-reference/main.swift.
//! Run with --light or --dark and optionally --page workspace.
extern crate gpui_kit as gpui;

use gpui_kit::base::{CheckboxState, input::InputState, slider::SliderState};
use gpui_kit::{
    AnyElement, App, Bounds, Context, Div, Entity, FontWeight, Render, SharedString, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_macos::kit::Sizable;
use gpui_macos::*;

gpui_kit::actions!(native_compare, [Quit]);

struct NativeCompare {
    native_toolbar: Option<native::NativeToolbarHandle>,
    page: usize,
    sidebar_visible: bool,
    status: SharedString,
    remembered: CheckboxState,
    mixed: CheckboxState,
    radio: usize,
    wifi: bool,
    airdrop: bool,
    automatic_updates: bool,
    notifications: CheckboxState,
    segment: usize,
    name: Entity<InputState>,
    search: Entity<InputState>,
    toolbar_search: Entity<InputState>,
    password: Entity<InputState>,
    number: Entity<InputState>,
    quality: Entity<ChoiceState>,
    appearance: Entity<ChoiceState>,
    slider: Entity<SliderState>,
    tick_slider: Entity<SliderState>,
}

impl NativeCompare {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Name")
                .default_value("Taylor")
        });
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let toolbar_search =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search components"));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Password")
                .default_value("macOS")
                .masked(true)
        });
        let number = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("42")
                .min(0.)
                .max(100.)
                .step(1.)
        });
        number.update(cx, |state, cx| {
            state.set_text_align(gpui_kit::TextAlign::Right, cx)
        });
        let choices = || {
            ["Standard", "High", "Best"]
                .into_iter()
                .map(|label| ChoiceItem::new(label, label))
                .collect()
        };
        let quality = cx.new(|cx| {
            ChoiceState::new(choices(), window, cx)
                .label("Quality")
                .default_selected(0)
        });
        let appearance = cx.new(|cx| {
            ChoiceState::new(choices(), window, cx)
                .label("Appearance")
                .default_selected(0)
        });
        let slider = cx.new(|_| SliderState::new().default_value(65.));
        let tick_slider = cx.new(|_| SliderState::new().step(25.).default_value(50.));
        for state in [&name, &search, &toolbar_search, &password, &number] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        for state in [&quality, &appearance] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        for state in [&slider, &tick_slider] {
            cx.observe(state, |_, _, cx| cx.notify()).detach();
        }
        let args: Vec<_> = std::env::args().collect();
        let page = args
            .windows(2)
            .any(|pair| pair[0] == "--page" && pair[1] == "workspace") as usize;
        let target = cx.entity().downgrade();
        let native_toolbar = native::install_native_toolbar(
            "gpui-native-compare",
            "GPUI Native Compare",
            vec![
                native::NativeToolbarItem::button("sidebar", "Toggle sidebar", "sidebar.left"),
                native::NativeToolbarItem::FlexibleSpace,
                native::NativeToolbarItem::search(
                    "search",
                    "Search components",
                    "Search components",
                    px(200.),
                ),
                native::NativeToolbarItem::button("add", "Add", "plus"),
            ],
            window,
            cx,
            move |command, _, cx| {
                let _ = target.update(cx, |this, cx| {
                    match command.as_ref() {
                        "sidebar" => this.sidebar_visible = !this.sidebar_visible,
                        "add" => this.status = "Last action: Add".into(),
                        "search" => {
                            let value = this
                                .native_toolbar
                                .as_ref()
                                .and_then(|bar| bar.search_text("search"))
                                .unwrap_or_default();
                            this.status = format!("Search: {value}").into();
                        }
                        _ => {}
                    }
                    cx.notify();
                });
            },
        )
        .ok();
        Self {
            native_toolbar,
            page,
            sidebar_visible: true,
            status: "GPUI Kit adapters on the current macOS. Controls are drawn by GPUI.".into(),
            remembered: CheckboxState::Checked,
            mixed: CheckboxState::Indeterminate,
            radio: 0,
            wifi: true,
            airdrop: false,
            automatic_updates: true,
            notifications: CheckboxState::Checked,
            segment: 0,
            name,
            search,
            toolbar_search,
            password,
            number,
            quality,
            appearance,
            slider,
            tick_slider,
        }
    }

    fn controls(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let t = cx.macos().clone();
        let buttons = row(
            "Buttons",
            vec![
                button("continue", "Continue", ButtonVariant::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.status = "Last action: Continue".into();
                        cx.notify();
                    }))
                    .into_any_element(),
                button("cancel", "Cancel", ButtonVariant::Secondary, cx)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.status = "Last action: Cancel".into();
                        cx.notify();
                    }))
                    .into_any_element(),
                button("disabled", "Disabled", ButtonVariant::Secondary, cx)
                    .disabled(true)
                    .into_any_element(),
            ],
            cx,
        );
        let checkboxes = row(
            "Checkboxes",
            vec![
                checkbox("remember", "Remember me", self.remembered, cx)
                    .on_change({
                        let target = cx.entity();
                        move |value, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.remembered = value;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
                checkbox("mixed", "Mixed selection", self.mixed, cx)
                    .on_change({
                        let target = cx.entity();
                        move |value, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.mixed = value;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
            ],
            cx,
        );
        let radios = row(
            "Radio buttons",
            vec![
                radio("automatic", "Automatic", self.radio == 0, cx)
                    .on_change({
                        let target = cx.entity();
                        move |_, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.radio = 0;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
                radio("manual", "Manual", self.radio == 1, cx)
                    .on_change({
                        let target = cx.entity();
                        move |_, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.radio = 1;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
            ],
            cx,
        );
        let switches = row(
            "Switches",
            vec![
                switch("wifi", "", self.wifi, cx)
                    .accessibility_label("Wi-Fi")
                    .on_change({
                        let target = cx.entity();
                        move |value, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.wifi = value;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
                div().child("Wi-Fi").into_any_element(),
                switch("airdrop", "", self.airdrop, cx)
                    .accessibility_label("AirDrop")
                    .on_change({
                        let target = cx.entity();
                        move |value, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.airdrop = value;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
                div().child("AirDrop").into_any_element(),
            ],
            cx,
        );
        let fields = row(
            "Text fields",
            vec![
                input("name", &self.name, window, cx)
                    .w(px(170.))
                    .into_any_element(),
                search_field("search", &self.search, window, cx)
                    .w(px(170.))
                    .into_any_element(),
                secure_field("password", &self.password, window, cx)
                    .w(px(170.))
                    .into_any_element(),
            ],
            cx,
        );
        let target = cx.entity();
        let segments = segmented_control(
            "segments",
            ["General", "Network", "Advanced"]
                .into_iter()
                .map(|label| ChoiceItem::new(label, label))
                .collect(),
            Some(self.segment),
            move |index, _, cx| {
                target.update(cx, |this, cx| {
                    this.segment = index;
                    cx.notify();
                })
            },
            window,
            cx,
        )
        .w(px(221.));
        // The reference NSBox distributes its six extra layout points to this
        // stack row; the segmented control itself keeps its native 24pt height.
        let segment_row = row("Segmented", vec![segments.into_any_element()], cx).min_h(px(30.));
        let choices = row(
            "Pickers",
            vec![
                select("quality", &self.quality, window, cx)
                    .w(px(104.))
                    .into_any_element(),
                div()
                    .text_color(t.secondary)
                    .child("Number")
                    .into_any_element(),
                input("number", &self.number, window, cx)
                    .w(px(70.))
                    .into_any_element(),
                number_stepper(&self.number, cx).into_any_element(),
            ],
            cx,
        );
        let sliders =
            row(
                "Sliders",
                vec![
                    slider(&self.slider, false, window, cx)
                        .w(px(220.))
                        .into_any_element(),
                    div()
                        .relative()
                        .w(px(220.))
                        .child(slider(&self.tick_slider, false, window, cx).w_full())
                        .child(
                            div()
                                .absolute()
                                .bottom(px(0.))
                                .left(px(8.))
                                .right(px(8.))
                                .flex()
                                .justify_between()
                                .children((0..5).map(|_| {
                                    div().w(px(2.)).h(px(2.)).rounded_full().bg(t.secondary)
                                })),
                        )
                        .into_any_element(),
                ],
                cx,
            );
        let progresses = row(
            "Progress",
            vec![
                div()
                    .flex()
                    .items_center()
                    .h(px(20.))
                    .w(px(220.))
                    .child(progress("progress", 62., cx))
                    .into_any_element(),
                native_spinner(cx)
                    .with_size(kit::Size::Size(px(16.)))
                    .color(t.secondary)
                    .into_any_element(),
                div()
                    .text_color(t.secondary)
                    .child("Loading…")
                    .into_any_element(),
            ],
            cx,
        );
        let preference_choice = row(
            "Appearance",
            vec![
                select("appearance", &self.appearance, window, cx)
                    .w(px(104.))
                    .into_any_element(),
            ],
            cx,
        );
        let updates = row(
            "Updates",
            vec![
                div().child("Download automatically").into_any_element(),
                switch("updates", "", self.automatic_updates, cx)
                    .accessibility_label("Download automatically")
                    .on_change({
                        let target = cx.entity();
                        move |value, _, _, cx| {
                            target.update(cx, |this, cx| {
                                this.automatic_updates = value;
                                cx.notify();
                            })
                        }
                    })
                    .into_any_element(),
            ],
            cx,
        );
        let notifications = row(
            "Notifications",
            vec![
                checkbox(
                    "notifications",
                    "Allow notifications",
                    self.notifications,
                    cx,
                )
                .on_change({
                    let target = cx.entity();
                    move |value, _, _, cx| {
                        target.update(cx, |this, cx| {
                            this.notifications = value;
                            cx.notify();
                        })
                    }
                })
                .into_any_element(),
            ],
            cx,
        )
        .min_h(px(22.));
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .w_full()
            .child(
                div()
                    .text_size(px(24.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Native controls"),
            )
            .child(div().text_color(t.secondary).child(self.status.clone()))
            .child(reference_group(
                "Standard system controls",
                vec![
                    buttons,
                    checkboxes,
                    radios,
                    switches,
                    fields,
                    segment_row,
                    choices,
                    sliders,
                    progresses,
                ],
                cx,
            ))
            .child(reference_group(
                "Preferences",
                vec![preference_choice, updates, notifications],
                cx,
            ))
    }

    fn workspace(&self, cx: &App) -> Div {
        let mut files = table("files", cx).child(
            table_row("head", 1, cx)
                .child(
                    table_head("name", 1, cx)
                        .w(px(300.))
                        .flex_none()
                        .child("Name"),
                )
                .child(
                    table_head("kind", 2, cx)
                        .w(px(180.))
                        .flex_none()
                        .child("Kind"),
                )
                .child(table_head("modified", 3, cx).child("Modified")),
        );
        for (index, (name, kind, modified)) in [
            ("Design system", "Folder", "Today"),
            ("Components.swift", "Swift source", "Today"),
            ("Interface.sketch", "Design document", "Yesterday"),
            ("Research", "Folder", "Yesterday"),
            ("Release notes.md", "Markdown", "Monday"),
        ]
        .into_iter()
        .enumerate()
        {
            files = files.child(
                table_row(index, index + 2, cx)
                    .child(
                        table_cell(("name", index), 1, cx)
                            .w(px(300.))
                            .flex_none()
                            .child(name),
                    )
                    .child(
                        table_cell(("kind", index), 2, cx)
                            .w(px(180.))
                            .flex_none()
                            .child(kind),
                    )
                    .child(table_cell(("modified", index), 3, cx).child(modified)),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .text_size(px(24.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Workspace"),
            )
            .child(
                div()
                    .text_color(cx.macos().secondary)
                    .child("Native table, sidebar and unified toolbar."),
            )
            .child(
                div()
                    .h(px(340.))
                    .w_full()
                    .border_1()
                    .border_color(cx.macos().divider())
                    .child(files),
            )
    }
}

impl Render for NativeCompare {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.macos().clone();
        let toolbar = title_bar("GPUI Native Compare", cx)
            .gap(px(12.))
            .child(div().flex_1())
            .child(
                toolbar_group(cx).child(
                    toolbar_button("toggle-sidebar", IconName::PanelLeft, "Toggle sidebar", cx)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sidebar_visible = !this.sidebar_visible;
                            cx.notify();
                        })),
                ),
            )
            .child(search_field("toolbar-search", &self.toolbar_search, window, cx).w(px(200.)))
            .child(
                toolbar_button("add", IconName::Plus, "Add", cx).on_click(cx.listener(
                    |this, _, _, cx| {
                        this.status = "Last action: Add".into();
                        cx.notify();
                    },
                )),
            )
            .pr(px(14.));
        let mut navigation = sidebar("navigation", cx)
            .w(px(190.))
            .pt(px(0.))
            .pb(px(10.))
            .pl(px(18.))
            .pr(px(2.))
            .border_0();
        for (index, (label, icon)) in [
            ("Controls", IconName::SlidersHorizontal),
            ("Workspace", IconName::Folder),
        ]
        .into_iter()
        .enumerate()
        {
            navigation = navigation.child(
                sidebar_item(index, label, icon, self.page == index, cx)
                    .h(px(30.))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = index;
                        cx.notify();
                    })),
            );
        }
        let page = if self.page == 0 {
            self.controls(window, cx)
        } else {
            self.workspace(cx)
        };
        focus_scope("native-compare")
            .size_full()
            .flex()
            .flex_col()
            .font_family(t.font)
            .text_size(px(13.))
            .line_height(gpui_kit::relative(1.2))
            .text_color(t.foreground)
            .child(if self.native_toolbar.is_some() {
                div()
                    .h(native::native_toolbar_inset(window))
                    .flex_shrink_0()
                    .into_any_element()
            } else {
                toolbar.into_any_element()
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .when(self.sidebar_visible, |content| {
                        content
                            .child(navigation)
                            .child(div().w(px(8.)).bg(t.background).flex_shrink_0())
                    })
                    .child(
                        div()
                            .id("detail")
                            .bg(t.background)
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .overflow_y_scroll()
                            .px(px(28.))
                            .py(px(24.))
                            .child(page),
                    ),
            )
    }
}

fn row(title: &'static str, controls: Vec<AnyElement>, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .w(px(116.))
                .flex_shrink_0()
                .text_color(cx.macos().secondary)
                .child(title),
        )
        .children(controls)
}

fn reference_group(title: &'static str, rows: Vec<Div>, cx: &App) -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .w_full()
        // NSBox's title overlaps the top edge of its content fill. Keeping
        // the title above the fill avoids adding a full text line to the gap.
        .pt(px(12.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(14.))
                .w_full()
                .px(px(16.))
                .py(px(12.))
                .rounded(px(12.))
                .bg(cx.macos().normal_fill())
                .children(rows),
        )
        .child(div().absolute().top(px(0.)).left(px(11.)).child(title))
}

// Public activation primitives paired with the separately sized number editor.
fn number_stepper(state: &Entity<InputState>, cx: &App) -> Div {
    let mut stepper = div()
        .flex()
        .flex_col()
        .w(px(20.))
        .h(px(26.))
        .overflow_hidden()
        .rounded(px(4.))
        .border_1()
        .border_color(cx.macos().control_border())
        .bg(cx.macos().control_fill());
    for (id, glyph, label, delta) in [
        ("increment", IconName::ChevronUp, "Increase number", 1.),
        ("decrement", IconName::ChevronDown, "Decrease number", -1.),
    ] {
        let target = state.clone();
        stepper = stepper.child(
            button(id, "", ButtonVariant::Secondary, cx)
                .accessibility_label(label)
                .min_h(px(0.))
                .h(px(12.))
                .w_full()
                .p(px(0.))
                .border_0()
                .rounded(px(0.))
                .shadow_none()
                .child(icon(glyph).size(px(12.)))
                .on_click(move |_, window, cx| {
                    target.update(cx, |state, cx| {
                        let value = state
                            .value()
                            .parse::<f64>()
                            .ok()
                            .filter(|value| value.is_finite())
                            .unwrap_or(42.);
                        state.set_value(format!("{}", (value + delta).clamp(0., 100.)), window, cx);
                    })
                }),
        );
    }
    stepper
}

fn main() {
    gpui_kit::platform::application()
        .with_assets(NativeAssets::new(gpui_kit::assets::AllAssets))
        .run(|cx| {
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
            cx.set_menus([gpui_kit::Menu::new("GPUI Native Compare")
                .items([gpui_kit::MenuItem::action("Quit GPUI Native Compare", Quit)])]);
            gpui_macos::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1060.), px(840.)),
                        cx,
                    ))),
                    window_min_size: Some(size(px(840.), px(680.))),
                    ..gpui_macos::native_window_options("GPUI Native Compare", cx)
                },
                cx,
                |window, cx| cx.new(|cx| NativeCompare::new(window, cx)),
            )
            .expect("open native comparison");
            cx.activate(true);
        });
}
