extern crate gpui_kit as gpui;

use gpui_kit::{
    AssetSource, Context, IntoElement, Render, TestAppContext, VisualTestContext, Window,
    WindowBackgroundAppearance, div, prelude::*, px, rems,
};
use gpui_macos::{
    ActiveTheme, ButtonVariant, IconName, Theme, button, open_window, set_zoom, title_bar,
    window_options, zoom_percent,
};

struct NativeWindow;

impl Render for NativeWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(title_bar("Native window", cx).debug_selector(|| "native-titlebar".into()))
            .child(
                button("apply", "Apply", ButtonVariant::Primary, cx)
                    .debug_selector(|| "native-apply".into())
                    .self_start()
                    .w(rems(7.5))
                    .h(rems(1.75)),
            )
    }
}

fn native_window(cx: &mut TestAppContext) -> VisualTestContext {
    cx.update(|cx| {
        gpui_macos::init(cx);
        Theme::light().apply(cx);
    });
    let (window, _) = cx.update(|cx| {
        open_window(window_options("Native window", cx), cx, |_, cx| {
            cx.new(|_| NativeWindow)
        })
        .unwrap()
    });
    let mut visual = VisualTestContext::from_window(window, cx);
    visual.run_until_parked();
    redraw(&mut visual);
    visual
}

fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

#[gpui_kit::test]
fn root_zoom_survives_redraws_and_appearance_policy_changes(cx: &mut TestAppContext) {
    let mut cx = native_window(cx);
    cx.update(|window, _| {
        assert!(window.root::<gpui_kit::base::Root>().flatten().is_some());
        assert_eq!(window.rem_size(), px(16.));
        assert_eq!(zoom_percent(window), 100);
    });
    let baseline = cx.debug_bounds("native-apply").unwrap();
    assert_eq!(baseline.size.width, px(120.));
    assert_eq!(baseline.size.height, px(28.));
    assert_eq!(
        cx.debug_bounds("native-titlebar").unwrap().size.height,
        px(52.)
    );

    cx.update(|window, cx| set_zoom(125, window, cx));
    for appearance in [Theme::dark(), Theme::light()] {
        cx.update(|_, cx| appearance.apply(cx));
        for _ in 0..3 {
            redraw(&mut cx);
            cx.update(|window, _| {
                assert_eq!(window.rem_size(), px(20.));
                assert_eq!(zoom_percent(window), 125);
            });
            let scaled = cx.debug_bounds("native-apply").unwrap();
            assert_eq!(scaled.size.width, baseline.size.width * 1.25);
            assert_eq!(scaled.size.height, baseline.size.height * 1.25);
            assert_eq!(
                cx.debug_bounds("native-titlebar").unwrap().size.height,
                px(65.)
            );
        }
    }
    cx.update(|_, cx| Theme::follow_system(cx));
    redraw(&mut cx);
    cx.update(|window, cx| {
        assert!(Theme::is_following_system(cx));
        assert_eq!(window.rem_size(), px(20.));
        assert_eq!(zoom_percent(window), 125);
    });
}

#[gpui_kit::test]
fn root_zoom_clamps_to_supported_bounds_and_keeps_them_after_redraw(cx: &mut TestAppContext) {
    let mut cx = native_window(cx);
    for (requested, expected, rem_size) in [
        (0, 50, 8.),
        (49, 50, 8.),
        (201, 200, 32.),
        (65535, 200, 32.),
    ] {
        cx.update(|window, cx| set_zoom(requested, window, cx));
        redraw(&mut cx);
        redraw(&mut cx);
        cx.update(|window, _| {
            assert_eq!(zoom_percent(window), expected);
            assert_eq!(window.rem_size(), px(rem_size));
        });
        assert_eq!(
            cx.debug_bounds("native-apply").unwrap().size.width,
            px(7.5 * rem_size)
        );
    }
}

#[gpui_kit::test]
fn native_window_options_respect_reduce_transparency(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        for (reduce, expected) in [
            (false, WindowBackgroundAppearance::Blurred),
            (true, WindowBackgroundAppearance::Opaque),
        ] {
            Theme::light().with_reduce_transparency(reduce).apply(cx);
            let options = window_options("Preferences", cx);
            assert_eq!(options.window_background, expected);
            let titlebar = options.titlebar.unwrap();
            assert_eq!(titlebar.title.as_deref(), Some("Preferences"));
            assert!(titlebar.appears_transparent);
            assert_eq!(
                options.window_min_size.unwrap(),
                gpui_kit::size(px(640.), px(480.))
            );
            if reduce {
                assert_eq!(cx.macos().glass_fill().a, 1.);
            } else {
                assert!(cx.macos().glass_fill().a < 1.);
            }
        }
    });
}

#[gpui_kit::test]
fn system_policy_preserves_custom_accent_and_material_tint(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_macos::init(cx);
        assert!(Theme::is_following_system(cx));
        let accent = gpui_kit::Hsla::from(gpui_kit::rgb(0xaf52de));
        Theme::dark()
            .with_accent(accent)
            .with_glass_tint(0.23)
            .apply(cx);
        assert!(!Theme::is_following_system(cx));
        assert!(cx.macos().is_dark());

        for _ in 0..2 {
            Theme::follow_system(cx);
            assert!(Theme::is_following_system(cx));
            assert_eq!(cx.macos().accent, accent);
            assert_eq!(cx.macos().chart[0], accent);
            assert_eq!(cx.macos().glass_tint, 0.23);
            assert_eq!(
                gpui_kit::base::Theme::global(cx).tokens.colors.primary,
                accent
            );
        }
        cx.macos().clone().apply(cx);
        assert!(!Theme::is_following_system(cx));
        assert_eq!(cx.macos().accent, accent);
        assert_eq!(cx.macos().glass_tint, 0.23);
    });
}

#[test]
fn complete_asset_bundle_resolves_the_lock_glyph() {
    let path = IconName::Lock.path();
    let bytes = gpui_kit::assets::AllAssets.load(&path).unwrap().unwrap();
    assert!(!bytes.is_empty());
    assert!(String::from_utf8_lossy(&bytes).contains("<svg"));
}
