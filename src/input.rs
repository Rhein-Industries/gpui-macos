//! Native text editing, selection, clipboard and IME supplied by gpui-base.
use crate::ActiveTheme;
use crate::NativeTextStyle as _;
use crate::tokens::{CONTROL_RADIUS, FIELD_HEIGHT, pt};
use gpui_kit::base::{
    Input as BaseInput, InputBase, Textarea,
    input::{InputState, TextareaState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::rems;
use gpui_kit::{
    AnyElement, App, ElementId, Entity, Focusable, InteractiveElement, Interactivity, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div,
};

/// A native single-line editor with optional, non-editable prefix and suffix slots.
/// The caller retains the `InputState`; affixes never become part of its value.
#[derive(IntoElement)]
pub struct Input {
    base: InputBase,
    editor: BaseInput,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    native_mask_text: bool,
}

impl Input {
    pub fn prefix(mut self, content: impl IntoElement) -> Self {
        self.prefix = Some(content.into_any_element());
        self
    }

    pub fn suffix(mut self, content: impl IntoElement) -> Self {
        self.suffix = Some(content.into_any_element());
        self
    }
}

impl Styled for Input {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Input {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Input {}

impl RenderOnce for Input {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let affix = |content| {
            div()
                .flex()
                .items_center()
                .flex_shrink_0()
                .text_color(cx.macos().secondary)
                .child(content)
        };
        self.base
            .children(self.prefix.map(affix))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    // AppKit's secure cell draws an 8-point advance and a
                    // 5.5-point dot at regular size. This public font's filled
                    // circle matches that raster at 12.8 points. Apply it only
                    // to masked text; placeholders and affixes retain SF.
                    .when(self.native_mask_text, |editor| {
                        editor.font_family("Helvetica Neue").text_size(rems(0.8))
                    })
                    .child(self.editor),
            )
            .children(self.suffix.map(affix))
    }
}

fn frame(id: impl Into<ElementId>, focused: bool, cx: &App) -> InputBase {
    let t = cx.macos();
    InputBase::new(id)
        .focused(focused)
        .w_full()
        .min_w_0()
        .px(rems(0.375))
        .border_1()
        // Text fields: radius 6, spec §4.1 (C13).
        .rounded(pt(CONTROL_RADIUS))
        .border_color(t.control_border())
        .bg(t.surface)
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .styles(|s| s.focused(|s| s.border_color(t.accent).shadow(t.focus_shadow())))
}

pub fn input(
    id: impl Into<ElementId>,
    state: &Entity<InputState>,
    window: &Window,
    cx: &mut App,
) -> Input {
    let style = cx.macos().input_style();
    state.update(cx, |state, cx| {
        state.set_editor_style(style);
        if state.presentation().is_masked() {
            state.set_mask_character('●', cx);
        }
    });
    let masked = state.read(cx).presentation().is_masked();
    let native_mask_text = masked && !state.read(cx).value().is_empty();
    let focused = window.is_window_active() && state.read(cx).focus_handle(cx).is_focused(window);
    let target = state.clone();
    let base = frame(id, focused, cx)
        .when(masked, |frame| frame.role(gpui_kit::Role::PasswordInput))
        // 24 tall, spec §4.3.
        .min_h(pt(FIELD_HEIGHT))
        .py(rems(0.1875))
        .flex()
        .items_center()
        .gap(rems(0.25))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            target.update(cx, |state, cx| state.focus(window, cx));
        });
    Input {
        base,
        editor: BaseInput::new(state),
        prefix: None,
        suffix: None,
        native_mask_text,
    }
}

pub fn textarea(
    id: impl Into<ElementId>,
    state: &Entity<TextareaState>,
    window: &Window,
    cx: &mut App,
) -> InputBase {
    let style = cx.macos().input_style();
    state.update(cx, |state, _| state.set_editor_style(style));
    let focused = window.is_window_active() && state.read(cx).focus_handle(cx).is_focused(window);
    let target = state.clone();
    frame(id, focused, cx)
        .min_h(rems(6.))
        .py(rems(0.3125))
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            target.update(cx, |state, cx| state.focus(window, cx));
        })
        .child(Textarea::new(state))
}

/// Numeric editing with native arrow-key stepping and stacked arrow buttons.
pub fn number_input(state: &Entity<InputState>, cx: &mut App) -> gpui_kit::base::NumberInput {
    use gpui_kit::ParentElement as _;
    let t = cx.macos().clone();
    state.update(cx, |state, cx| {
        state.set_editor_style(t.input_style());
        state.set_text_align(gpui_kit::TextAlign::Right, cx);
    });
    let minus = t.clone();
    gpui_kit::base::NumberInput::new(state)
        .w(rems(7.5))
        .h(pt(FIELD_HEIGHT))
        .controls_right()
        .overflow_hidden()
        .flex()
        .items_center()
        .border_1()
        .rounded(pt(CONTROL_RADIUS))
        .border_color(t.control_border())
        .bg(t.surface)
        .text_color(t.foreground)
        .font_family(t.font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .input(
            gpui_kit::div()
                .flex_1()
                .min_w_0()
                .px(rems(0.375))
                .child(BaseInput::new(state)),
        )
        .decrement_button(move |button| {
            button
                .w(rems(1.25))
                .text_color(minus.foreground)
                .bg(minus.control_fill())
                .border_l_1()
                .border_color(minus.divider())
                .hover(|s| s.bg(minus.hover_fill()))
                .child(crate::icon(crate::IconName::ChevronDown).size(rems(0.75)))
        })
        .increment_button(move |button| {
            button
                .w(rems(1.25))
                .text_color(t.foreground)
                .bg(t.control_fill())
                .border_l_1()
                .border_b_1()
                .border_color(t.divider())
                .hover(|s| s.bg(t.hover_fill()))
                .child(crate::icon(crate::IconName::ChevronUp).size(rems(0.75)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{Context, Modifiers, Render, TestAppContext, prelude::*};

    struct Harness(Entity<InputState>);

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            input("price", &self.0, window, cx)
                .debug_selector(|| "price-frame".into())
                .w(rems(15.))
                .prefix(div().debug_selector(|| "price-prefix".into()).child("$"))
                .suffix(div().debug_selector(|| "price-suffix".into()).child("USD"))
        }
    }

    #[gpui_kit::test]
    fn input_and_affixes_scale_with_the_window_rem(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            Harness(cx.new(|cx| InputState::new(window, cx).default_value("181.170")))
        });
        let mut previous_suffix_width = gpui_kit::Pixels::ZERO;
        for rem_size in [16., 20., 24.] {
            cx.update(|window, cx| {
                window.set_rem_size(gpui_kit::px(rem_size));
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let frame = cx.debug_bounds("price-frame").unwrap();
            let prefix = cx.debug_bounds("price-prefix").unwrap();
            let suffix = cx.debug_bounds("price-suffix").unwrap();
            assert_eq!(frame.size.width, gpui_kit::px(15. * rem_size));
            assert!(suffix.size.width > previous_suffix_width);
            assert!(prefix.right() < suffix.left());
            assert_eq!(
                prefix.left() - frame.left(),
                gpui_kit::px(rem_size * 0.375 + 1.),
                "native field padding scales independently of its one-point border"
            );
            assert!(suffix.right() < frame.right());
            previous_suffix_width = suffix.size.width;
        }
    }

    #[gpui_kit::test]
    fn affixes_preserve_editing_and_focus(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) =
            cx.add_window_view(|window, cx| Harness(cx.new(|cx| InputState::new(window, cx))));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let prefix = cx.debug_bounds("price-prefix").unwrap();
        let suffix = cx.debug_bounds("price-suffix").unwrap();
        assert!(prefix.right() < suffix.left());
        cx.simulate_click(suffix.center(), Modifiers::default());
        cx.update(|window, cx| {
            assert!(view.read(cx).0.read(cx).focus_handle(cx).is_focused(window));
        });
        cx.simulate_input("181.170");
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).0.read(cx).value().as_ref(), "181.170");
        });
    }

    #[gpui_kit::test]
    fn native_secure_mask_preserves_privacy_unicode_selection_and_ime(cx: &mut TestAppContext) {
        use gpui_kit::{ClipboardItem, EntityInputHandler, Role, test::TestWindowExt};
        struct SecureHarness(Entity<InputState>);
        impl Render for SecureHarness {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                input("native-secret", &self.0, window, cx).w(rems(10.625))
            }
        }
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            SecureHarness(cx.new(|cx| {
                InputState::new(window, cx)
                    .masked(true)
                    .default_value("A猫🦀Z")
            }))
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(
                window.find("native-secret").role(),
                Some(Role::PasswordInput)
            );
            assert_eq!(window.find("native-secret").value(), None);
            let state = view.read(cx).0.clone();
            state.update(cx, |state, cx| {
                assert_eq!(state.mask_character(), '●');
                state.focus(window, cx);
                state.select_all(window, cx);
                cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into()));
            });
        });
        cx.simulate_keystrokes("cmd-c cmd-x");
        cx.update(|window, cx| {
            assert_eq!(
                cx.read_from_clipboard().and_then(|item| item.text()),
                Some("sentinel".into())
            );
            let state = view.read(cx).0.clone();
            state.update(cx, |state, cx| {
                assert_eq!(state.value(), "A猫🦀Z");
                state.set_selected_range(1..8, cx);
                state.replace_and_mark_text_in_range(None, "日本", Some(1..1), window, cx);
                assert_eq!(state.value(), "A日本Z");
                assert_eq!(state.selected_range(), 4..4);
            });
            window.draw(cx).clear(cx);
            assert_eq!(window.find("native-secret").value(), None);
            state.update(cx, |state, cx| {
                state.replace_and_mark_text_in_range(None, "日本語", Some(3..3), window, cx);
                assert_eq!(state.selected_range(), 10..10);
            });
            window.draw(cx).clear(cx);
            state.update(cx, |state, cx| {
                state.replace_text_in_range(None, "日🦀", window, cx);
                assert_eq!(state.value(), "A日🦀Z");
                assert_eq!(state.selected_range(), 8..8);
            });
            window.draw(cx).clear(cx);
            assert_eq!(
                window.find("native-secret").role(),
                Some(Role::PasswordInput)
            );
            assert_eq!(window.find("native-secret").value(), None);
            state.update(cx, |state, cx| state.set_masked(false, window, cx));
            window.draw(cx).clear(cx);
            assert_eq!(window.find("native-secret").role(), Some(Role::TextInput));
            assert_eq!(state.read(cx).value(), "A日🦀Z");
        });
    }
}
