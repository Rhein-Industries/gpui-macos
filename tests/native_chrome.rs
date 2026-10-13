extern crate gpui_kit as gpui;

use gpui_kit::base::test_support;
use gpui_kit::{
    Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Render, TestAppContext,
    VisualTestContext, Window, prelude::*, px, rems,
};
use gpui_macos::{
    ActiveTheme, IconName, Material, Theme, focus_scope, material, sidebar, sidebar_item,
};

struct SidebarHarness {
    selected: usize,
    activations: usize,
    focus: [FocusHandle; 3],
}

fn press(cx: &mut VisualTestContext, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}

impl Render for SidebarHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut navigation = sidebar("navigation", cx);
        for (index, (id, label)) in [
            ("favorites", "Favorites"),
            ("documents", "Documents"),
            ("archive", "Archive"),
        ]
        .into_iter()
        .enumerate()
        {
            let entity = cx.entity();
            navigation = navigation.child(
                sidebar_item(id, label, IconName::Star, self.selected == index, cx)
                    .debug_selector(move || format!("sidebar-{id}"))
                    .track_focus(&self.focus[index])
                    .disabled(index == 2)
                    .on_click(move |_, _, cx| {
                        entity.update(cx, |state, cx| {
                            state.selected = index;
                            state.activations += 1;
                            cx.notify();
                        });
                    }),
            );
        }
        focus_scope("root").size(rems(20.)).child(navigation)
    }
}

#[gpui_kit::test]
fn sidebar_pointer_and_keyboard_update_selection_and_skip_disabled_rows(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        Theme::light().apply(cx);
    });
    let (view, cx) = cx.add_window_view(|_, cx| SidebarHarness {
        selected: 0,
        activations: 0,
        focus: std::array::from_fn(|_| cx.focus_handle()),
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, _| {
        assert_eq!(
            test_support::find(window, &[], &"favorites".into())
                .unwrap()
                .selected(),
            Some(true),
        );
        assert_eq!(
            test_support::find(window, &[], &"archive".into())
                .unwrap()
                .focused(),
            None,
        );
    });

    let documents = cx.debug_bounds("sidebar-documents").unwrap().center();
    cx.simulate_click(documents, Default::default());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).selected, 1);
        assert_eq!(view.read(cx).activations, 1);
        assert!(view.read(cx).focus[1].is_focused(window));
        assert_eq!(
            test_support::find(window, &[], &"documents".into())
                .unwrap()
                .selected(),
            Some(true),
        );
        assert_eq!(
            test_support::find(window, &[], &"favorites".into())
                .unwrap()
                .selected(),
            Some(false),
        );
    });

    press(cx, "enter");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|_, cx| assert_eq!(view.read(cx).activations, 2));
    cx.simulate_keystrokes("tab");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| {
        assert!(view.read(cx).focus[0].is_focused(window));
        assert!(!view.read(cx).focus[2].is_focused(window));
    });
    press(cx, "space");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).selected, 0);
        assert_eq!(view.read(cx).activations, 3);
    });

    let archive = cx.debug_bounds("sidebar-archive").unwrap().center();
    cx.simulate_click(archive, Default::default());
    cx.update(|window, cx| {
        assert!(!view.read(cx).focus[2].is_focused(window));
        assert_eq!(view.read(cx).activations, 3);
    });
    // Even an application explicitly focusing a disabled command cannot activate it.
    cx.update(|window, cx| view.read(cx).focus[2].clone().focus(window, cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    press(cx, "enter");
    press(cx, "space");
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).selected, 0);
        assert_eq!(view.read(cx).activations, 3);
    });
}

#[gpui_kit::test]
fn native_materials_honor_accessibility_preferences_and_sidebar_rows_stay_unfilled(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        for theme in [Theme::light(), Theme::dark()] {
            theme
                .with_reduce_transparency(true)
                .with_increased_contrast(true)
                .apply(cx);
            for kind in [
                Material::Sidebar,
                Material::Toolbar,
                Material::Popover,
                Material::Hud,
            ] {
                let mut surface = material(kind, cx).border_1();
                let fill = surface
                    .style()
                    .background
                    .as_ref()
                    .unwrap()
                    .color()
                    .unwrap()
                    .as_solid()
                    .unwrap();
                assert_eq!(
                    fill.a, 1.,
                    "Reduce transparency must make every material opaque"
                );
                let border = surface.style().border_color.unwrap();
                assert_eq!(border, cx.macos().control_border());
            }
            let mut unselected = sidebar_item("unselected", "Documents", IconName::Star, false, cx);
            let fill = unselected
                .style()
                .background
                .as_ref()
                .unwrap()
                .color()
                .unwrap()
                .as_solid()
                .unwrap();
            assert_eq!(
                fill.a, 0.,
                "Unselected commands must show the sidebar material beneath them"
            );
            let mut selected = sidebar_item("selected", "Favorites", IconName::Star, true, cx);
            let fill = selected
                .style()
                .background
                .as_ref()
                .unwrap()
                .color()
                .unwrap()
                .as_solid()
                .unwrap();
            assert_eq!(fill, cx.macos().accent);
            assert_eq!(selected.style().text.color, Some(cx.macos().on_accent));
        }
    });
}

#[gpui_kit::test]
fn sidebar_geometry_matches_native_metrics_and_selection_tracks_window_activity(
    cx: &mut TestAppContext,
) {
    cx.update(gpui_macos::init);
    let (view, cx) = cx.add_window_view(|_, cx| SidebarHarness {
        selected: 0,
        activations: 0,
        focus: std::array::from_fn(|_| cx.focus_handle()),
    });
    for theme in [Theme::light(), Theme::dark()] {
        cx.update(|_, cx| theme.apply(cx));
        for active in [true, false] {
            if active {
                cx.update(|window, _| window.activate_window());
                cx.run_until_parked();
            } else {
                cx.deactivate_window();
            }
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let selected = cx.debug_bounds("sidebar-favorites").unwrap();
            let next = cx.debug_bounds("sidebar-documents").unwrap();
            cx.update(|window, cx| {
                assert_eq!(window.is_window_active(), active);
                // Native rows select from x 10 to 210 of the 220 pt sidebar.
                assert_eq!(selected.origin.x, px(10.));
                assert_eq!(selected.size.width, px(200.));
                assert_eq!(
                    selected.size.height,
                    gpui_macos::native::sidebar_metrics().row_height
                );
                assert_eq!(next.origin.y, selected.bottom());
                let expected = if active {
                    cx.macos().accent
                } else {
                    gpui_macos::native::sidebar_inactive_selection(cx)
                };
                assert!(
                    window.painted_quads().iter().any(|quad| {
                        quad.bounds == selected.scale(window.scale_factor())
                            && quad.background.as_solid() == Some(expected)
                    }),
                    "Selected fill must follow the AppKit active/inactive semantic color"
                );
                assert!(
                    !window.painted_quads().iter().any(|quad| {
                        quad.bounds == next.scale(window.scale_factor())
                            && quad.background.as_solid().is_some_and(|fill| fill.a > 0.)
                    }),
                    "An unselected row leaves the sidebar material visible"
                );
                assert_eq!(view.read(cx).selected, 0);
                assert_eq!(
                    test_support::find(window, &[], &"favorites".into())
                        .unwrap()
                        .selected(),
                    Some(true)
                );
            });
        }
    }
}
