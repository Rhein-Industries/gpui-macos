//! Button presentation that suppresses transient styles while disabled.
use crate::ActiveTheme as _;
use crate::state::{PaintRole, Part, States, focus_ring_for, paint};
use gpui_kit::base::motion::transition;
use gpui_kit::base::{ButtonStyles, RoleOverride};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, Fill, FocusHandle, Hsla, InteractiveElement,
    Interactivity, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window,
};
use std::{rc::Rc, time::Duration};

type HoverListener = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
/// A child resolved at render, in the order it was added.
enum ButtonChild {
    Ready(AnyElement),
    /// Built with whether the row shows its emphasized selection.
    Highlight(Box<dyn FnOnce(bool) -> AnyElement>),
    /// Built with whether the pointer is over the button or it shows
    /// keyboard focus.
    Reveal(Box<dyn FnOnce(bool) -> AnyElement>),
}

/// A macOS button backed by gpui-base's activation and focus behavior.
/// Transient styles are applied only after the final disabled state is known.
///
/// The fill eases between states over 120ms;
/// borders and text change at once.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    base: gpui_kit::base::Button,
    disabled: bool,
    selected: bool,
    selected_fill: Option<Hsla>,
    selection_emphasis: bool,
    children: Vec<ButtonChild>,
    paint_role: PaintRole,
    role_default: Option<(Option<Hsla>, Option<Hsla>)>,
    fill_duration: Duration,
    snapped_frame: Option<SnappedFrame>,
    focus: Option<FocusHandle>,
    on_hover: Option<HoverListener>,
    hover: Option<Box<StyleRefinement>>,
    active: Option<Box<StyleRefinement>>,
    focus_visible: Option<Box<StyleRefinement>>,
}

#[derive(Clone, Copy)]
struct SnappedFrame {
    bezel: (f32, f32),
    frame: (f32, f32),
    /// The constructor's (size, leading margin, trailing margin) per axis.
    /// An axis is snapped only while the caller has not changed any of them.
    defaults: [AxisLengths; 2],
}

type AxisLengths = [Option<gpui_kit::Length>; 3];

fn axis_lengths(style: &StyleRefinement) -> [AxisLengths; 2] {
    [
        [style.size.width, style.margin.left, style.margin.right],
        [style.size.height, style.margin.top, style.margin.bottom],
    ]
}

impl SnappedFrame {
    /// Device-pixel bezel lengths and margins (leading, trailing) per axis.
    /// Lengths round half toward zero, as GPUI's layout does; the leading
    /// margin takes the larger half of an odd remainder.
    fn device_pixels(self, points_to_device: f32) -> [(f32, f32, f32); 2] {
        let round = |points: f32| ((points * points_to_device).abs() - 0.5).ceil().max(0.);
        let axis = |bezel: f32, frame: f32| {
            let bezel = round(bezel);
            let remainder = (round(frame) - bezel).max(0.);
            let trailing = (remainder / 2.).floor();
            (bezel, remainder - trailing, trailing)
        };
        [
            axis(self.bezel.0, self.frame.0),
            axis(self.bezel.1, self.frame.1),
        ]
    }
}

/// The pointer state the fill follows, kept per button across frames.
#[derive(Default)]
struct PointerState {
    hovered: bool,
    pressed: bool,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            base: gpui_kit::base::Button::new(id.clone()),
            id,
            disabled: false,
            selected: false,
            selected_fill: None,
            selection_emphasis: true,
            children: Vec::new(),
            paint_role: PaintRole::Custom,
            role_default: None,
            fill_duration: crate::motion::CONTROL_COLOR,
            snapped_frame: None,
            focus: None,
            on_hover: None,
            hover: None,
            active: None,
            focus_visible: None,
        }
    }

    /// The fill while selected. Unlike a `styles` selected refinement, it
    /// takes part in the fill transition.
    pub(crate) fn selected_fill(mut self, fill: Hsla) -> Self {
        self.selected_fill = Some(fill);
        self
    }

    /// A bezel (width, height) centered in a larger layout frame, both in
    /// points. Applied at render in whole device pixels: the frame rounds
    /// once and the margins take the bezel's rounding remainder, so the
    /// frame stays exact while the hit area remains the bezel. Call it after
    /// setting the nominal size and margins; a caller that later changes an
    /// axis's size or margins keeps its own values on that axis.
    pub(crate) fn snapped_frame(mut self, bezel: (f32, f32), frame: (f32, f32)) -> Self {
        let defaults = axis_lengths(self.base.style());
        self.snapped_frame = Some(SnappedFrame {
            bezel,
            frame,
            defaults,
        });
        self
    }

    /// How long the fill takes to reach a new state.
    pub(crate) fn fill_duration(mut self, duration: Duration) -> Self {
        self.fill_duration = duration;
        self
    }

    /// The paint family resolved at render (spec §6.2). This is not the
    /// accessibility role; see [`Button::role`].
    pub(crate) fn paint_role(mut self, role: PaintRole) -> Self {
        self.paint_role = role;
        self.role_default = None;
        self
    }

    /// Records the current rest fill and label as the role's default. While
    /// the fill is unchanged at render, the resolver paints every fill state;
    /// the label is resolved too unless the caller set its own (plan D6).
    pub(crate) fn record_role_default(mut self) -> Self {
        let style = self.base.style();
        self.role_default = Some((solid(style), style.text.color));
        self
    }

    /// Whether a selected row shows the emphasized (focused) selection. A
    /// sidebar row is emphasized while its list has keyboard focus: pass
    /// `focus.contains_focused(window, cx)` for the sidebar's focus handle.
    /// Unemphasized rows show a neutral selection with the label color, and
    /// an inactive window is never emphasized (spec §6.4, C20). Default
    /// `true`. Only row-style buttons such as [`crate::sidebar_item`] use it.
    pub fn selection_emphasis(mut self, emphasized: bool) -> Self {
        self.selection_emphasis = emphasized;
        self
    }

    /// Adds a child built at render with whether the row shows its
    /// emphasized selection. Children keep the order they were added in.
    pub(crate) fn child_with_highlight<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.children
            .push(ButtonChild::Highlight(Box::new(move |highlighted| {
                build(highlighted).into_any_element()
            })));
        self
    }

    /// Adds a child built at render with whether it should be revealed: the
    /// pointer is over the enabled button in an active window, or the button
    /// shows keyboard focus (spec §6.3). Children keep the order they were
    /// added in.
    pub(crate) fn child_with_reveal<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.children
            .push(ButtonChild::Reveal(Box::new(move |revealed| {
                build(revealed).into_any_element()
            })));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.base = self.base.disabled(disabled);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.base = self.base.selected(selected);
        self
    }

    pub fn styles(mut self, build: impl FnOnce(ButtonStyles) -> ButtonStyles) -> Self {
        self.base = self.base.styles(build);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.base = self.base.accessibility_label(label);
        self
    }

    pub fn role(mut self, role: impl Into<RoleOverride>) -> Self {
        self.base = self.base.role(role);
        self
    }

    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self.base = self.base.track_focus(focus);
        self
    }

    pub fn tab_index(mut self, index: isize) -> Self {
        self.base = self.base.tab_index(index);
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.base = self.base.tab_stop(tab_stop);
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.base = self.base.focusable(focusable);
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.base = self.base.on_click(handler);
        self
    }
}

impl gpui_kit::base::Selectable for Button {
    fn selected(self, selected: bool) -> Self {
        Button::selected(self, selected)
    }

    fn is_selected(&self) -> bool {
        self.base.is_selected()
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(ButtonChild::Ready));
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }

    fn hover(mut self, build: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.hover = Some(Box::new(build(StyleRefinement::default())));
        self
    }
    fn focus_visible(mut self, build: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.focus_visible = Some(Box::new(build(StyleRefinement::default())));
        self
    }
}

impl StatefulInteractiveElement for Button {
    /// Chained with the listener that drives the fill transition.
    fn on_hover(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(listener));
        self
    }

    fn active(mut self, build: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        self.active = Some(Box::new(build(StyleRefinement::default())));
        self
    }
}

/// The solid color of a refinement's background, if it sets one.
fn solid(style: &StyleRefinement) -> Option<Hsla> {
    style
        .background
        .as_ref()
        .and_then(Fill::color)
        .and_then(|background| background.as_solid())
}

impl RenderOnce for Button {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(frame) = self.snapped_frame {
            let scale = window.scale_factor();
            let [x, y] = frame.device_pixels(window.rem_size().as_f32() / 16. * scale);
            let length = |device: f32| gpui_kit::px(device / scale);
            let [current_x, current_y] = axis_lengths(self.base.style());
            if current_x == frame.defaults[0] {
                self.base = self.base.w(length(x.0)).ml(length(x.1)).mr(length(x.2));
            }
            if current_y == frame.defaults[1] {
                self.base = self.base.h(length(y.0)).mt(length(y.1)).mb(length(y.2));
            }
        }
        let states = States::in_window(window).disabled(self.disabled);
        let window_active = states.window_active;
        // Rows resolve their selection from the final selected and emphasis
        // values (spec §6.4).
        let role = match self.paint_role {
            PaintRole::Row { kind, .. } => PaintRole::Row {
                kind,
                selected: self.selected,
                emphasized: self.selection_emphasis,
            },
            role => role,
        };
        let highlighted = window_active
            && matches!(
                role,
                PaintRole::Row {
                    selected: true,
                    emphasized: true,
                    ..
                }
            );
        // Plan D6: the role owns the fill while the rest background is the
        // role default; the label is resolved separately, so a caller
        // `text_color` survives while the role still paints the bezel.
        let default = self.role_default.filter(|_| role != PaintRole::Custom);
        let owns_fill = default.is_some_and(|(fill, _)| fill == solid(self.base.style()));
        let owns_label =
            owns_fill && default.is_some_and(|(_, label)| label == self.base.style().text.color);
        if owns_fill {
            let theme = cx.macos();
            let pressed = states.pressed(true);
            self.base = self
                .base
                .bg(paint(role, Part::Fill, states, theme))
                .styles(|s| s.disabled(|s| s.opacity(1.)));
            let mut active = self.active.take().map(|style| *style).unwrap_or_default();
            if active.background.is_none() {
                active.background = Some(paint(role, Part::Fill, pressed, theme).into());
            }
            if owns_label {
                self.base = self
                    .base
                    .text_color(paint(role, Part::Label, states, theme));
                if active.text.color.is_none() {
                    active.text.color = Some(paint(role, Part::Label, pressed, theme));
                }
            }
            self.active = Some(Box::new(active));
            // Hover follows the role (spec §6.5, C17): only toolbar items and
            // menu rows respond to the pointer. A caller hover still wins.
            if let Some(fill) = crate::state::hover(role, theme) {
                let mut hover = self.hover.take().map(|style| *style).unwrap_or_default();
                if hover.background.is_none() {
                    hover.background = Some(fill.into());
                }
                self.hover = Some(Box::new(hover));
            }
            // A row's fill is its selection; the generic selected tint does not apply.
            if matches!(role, PaintRole::Row { .. }) {
                self.selected_fill = None;
            }
        }
        let focus = self.focus.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "macos-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let pointer = window.use_keyed_state((self.id.clone(), "macos-pointer"), cx, |_, _| {
            PointerState::default()
        });
        if !window_active {
            pointer.update(cx, |state, _| {
                state.hovered = false;
                state.pressed = false;
            });
        }

        // The fill for the current state, in interaction precedence: pressed,
        // keyboard focus, hover, selected, rest. A gradient fill is left to the
        // instant state styles.
        let rest = match self.base.style().background.as_ref() {
            Some(_) => solid(self.base.style()),
            None => Some(gpui_kit::transparent_black()),
        };
        let fill = rest.map(|rest| {
            let state = pointer.read(cx);
            let focus_visible = crate::state::focus_visible(&focus, window);
            let transient = |style: &Option<Box<StyleRefinement>>| style.as_deref().and_then(solid);
            let target = if self.disabled || !window_active {
                None
            } else if state.pressed {
                transient(&self.active)
            } else if focus_visible {
                transient(&self.focus_visible)
            } else if state.hovered {
                transient(&self.hover)
            } else {
                None
            }
            .or((self.selected && window_active && !self.disabled)
                .then_some(self.selected_fill)
                .flatten())
            .unwrap_or(rest);
            transition(
                (self.id.clone(), "macos-fill"),
                target,
                // Background windows pause animation clocks. Apply theme and
                // control changes immediately rather than retaining stale fills.
                crate::motion::color(crate::motion::animated_duration(window, self.fill_duration)),
                window,
                cx,
            )
        });
        #[cfg(test)]
        if let Some(fill) = fill {
            tests::record_fill(&self.id, fill);
        }

        let ring = (!self.disabled
            && role.has_focus_ring()
            && crate::state::focus_visible(&focus, window))
        .then(|| focus_ring_for(self.base.style(), window, cx.macos()));
        let revealed = !self.disabled
            && (pointer.read(cx).hovered || crate::state::focus_visible(&focus, window));
        let mut base = self.base;
        if self.focus.is_none() {
            base = base.track_focus(&focus);
        }
        base = base.children(self.children.into_iter().map(|child| match child {
            ButtonChild::Ready(element) => element,
            ButtonChild::Highlight(build) => build(highlighted),
            ButtonChild::Reveal(build) => build(revealed),
        }));
        // Spec §6.3: the outset ring around the whole control, keyboard focus
        // in an active window only. The fill and border stay unchanged.
        if let Some(ring) = ring {
            if base.style().position.is_none() {
                base = base.relative();
            }
            base = base.child(ring);
        }
        if let Some(fill) = fill {
            // Every state paints the sampled fill, so GPUI's instant state
            // styles keep their borders but cannot jump the background.
            base = base.bg(fill);
            for style in [&mut self.hover, &mut self.active, &mut self.focus_visible]
                .into_iter()
                .flatten()
            {
                if style.background.is_some() {
                    style.background = Some(fill.into());
                }
            }
        }
        if !self.disabled {
            if let Some(style) = self.hover.filter(|_| window_active) {
                base = base.hover(|_| *style);
            }
            if let Some(style) = self.active.filter(|_| window_active) {
                base = base.active(|_| *style);
            }
            if let Some(style) = self.focus_visible.filter(|_| window_active) {
                base = base.focus_visible(|_| *style);
            }
            let on_hover = self.on_hover;
            let hover_state = pointer.clone();
            let down_state = pointer.clone();
            let up_state = pointer.clone();
            let out_state = pointer;
            base = base
                .on_hover(move |hovered, window, cx| {
                    hover_state.update(cx, |state, cx| {
                        state.hovered = *hovered;
                        cx.notify();
                    });
                    if let Some(on_hover) = on_hover.as_ref() {
                        on_hover(hovered, window, cx);
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    down_state.update(cx, |state, cx| {
                        state.pressed = true;
                        cx.notify();
                    });
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up_state.update(cx, |state, cx| {
                        state.pressed = false;
                        cx.notify();
                    });
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    out_state.update(cx, |state, cx| {
                        state.pressed = false;
                        cx.notify();
                    });
                });
        } else if let Some(on_hover) = self.on_hover {
            base = base.on_hover(move |hovered, window, cx| on_hover(hovered, window, cx));
        }
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ActiveTheme;
    use gpui_kit::px;
    use gpui_kit::rems;
    use gpui_kit::{Context, Modifiers, MouseButton, Render, TestAppContext, div, point};
    use std::{cell::RefCell, collections::HashMap};

    thread_local! {
        static FILLS: RefCell<HashMap<String, Hsla>> = RefCell::default();
    }

    /// The fill a button sampled on its last render.
    pub(super) fn record_fill(id: &ElementId, fill: Hsla) {
        FILLS.with(|fills| fills.borrow_mut().insert(id.to_string(), fill));
    }

    fn fill(id: &str) -> Hsla {
        FILLS.with(|fills| fills.borrow()[id])
    }

    /// Toolbar items keep a hover disc (spec §6.5, C17); it eases in and out.
    #[gpui_kit::test]
    fn toolbar_item_fill_eases_to_the_hover_state_and_back(cx: &mut TestAppContext) {
        cx.update(crate::init);
        struct Harness;
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().size(rems(25.)).child(
                    crate::toolbar_button("fill", crate::IconName::Plus, "Add", cx)
                        .fill_duration(crate::motion::CONTROL_COLOR),
                )
            }
        }
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::default());
        let redraw = |cx: &mut gpui_kit::VisualTestContext| {
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| window.draw(cx).clear(cx));
        };
        redraw(cx);
        let (rest, hover) = cx.update(|_, cx| {
            let t = cx.macos();
            (
                gpui_kit::transparent_black(),
                crate::tokens::ink(t).opacity(0.05),
            )
        });
        assert_eq!(fill("fill").a, rest.a);

        cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::default());
        redraw(cx);
        cx.executor()
            .advance_clock(crate::motion::CONTROL_COLOR / 2);
        redraw(cx);
        let halfway = fill("fill");
        assert!(halfway.a > 0., "the hover transition has started");
        assert_ne!(halfway, hover, "the hover transition is still running");

        cx.executor().advance_clock(crate::motion::CONTROL_COLOR);
        redraw(cx);
        assert_eq!(fill("fill"), hover, "ink @0.05 disc (light)");

        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::default());
        redraw(cx);
        cx.executor()
            .advance_clock(crate::motion::CONTROL_COLOR * 2);
        redraw(cx);
        assert_eq!(fill("fill").a, 0.);
    }

    /// Bezels have no hover (spec §6.5, C17): the fill is unchanged under the
    /// pointer, for every bordered variant, in light and dark.
    #[gpui_kit::test]
    fn hovering_a_bezel_leaves_its_fill_unchanged(cx: &mut TestAppContext) {
        use crate::ButtonVariant::*;
        cx.update(crate::init);
        struct Harness;
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().size(rems(25.)).flex().flex_col().children(
                    [
                        ("hover-secondary", Secondary),
                        ("hover-outline", Outline),
                        ("hover-primary", Primary),
                        ("hover-danger", Danger),
                        ("hover-borderless", Borderless),
                    ]
                    .map(|(id, variant)| {
                        crate::button(id, "Apply", variant, cx)
                            .w(rems(6.25))
                            .h(rems(2.5))
                            .debug_selector(move || id.into())
                    }),
                )
            }
        }
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            let redraw = |cx: &mut gpui_kit::VisualTestContext| {
                view.update(cx, |_, cx| cx.notify());
                cx.update(|window, cx| window.draw(cx).clear(cx));
            };
            // Let the theme change finish its fill transition first.
            redraw(cx);
            cx.executor()
                .advance_clock(crate::motion::CONTROL_COLOR * 2);
            for id in [
                "hover-secondary",
                "hover-outline",
                "hover-primary",
                "hover-danger",
                "hover-borderless",
            ] {
                cx.simulate_mouse_move(point(px(390.), px(390.)), None, Modifiers::default());
                redraw(cx);
                let rest = fill(id);
                let center = cx.debug_bounds(id).unwrap().center();
                cx.simulate_mouse_move(center, None, Modifiers::default());
                redraw(cx);
                cx.executor()
                    .advance_clock(crate::motion::CONTROL_COLOR * 2);
                redraw(cx);
                assert_eq!(fill(id), rest, "{id} hovered");
            }
            assert_eq!(fill("hover-secondary"), theme.bezel_fill());
            assert_eq!(fill("hover-primary"), theme.accent);
        }
    }

    #[gpui_kit::test]
    fn pressed_fill_follows_the_paint_role(cx: &mut TestAppContext) {
        use crate::state::{PaintRole, Part, States, Tint, paint};
        cx.update(|cx| {
            crate::init(cx);
            cx.set_reduce_motion(true);
        });
        struct Harness;
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().size(rems(25.)).flex().flex_col().children(
                    [
                        ("press-bezel", crate::ButtonVariant::Secondary),
                        ("press-prominent", crate::ButtonVariant::Primary),
                    ]
                    .map(|(id, variant)| {
                        crate::button(id, "Apply", variant, cx)
                            .w(rems(6.25))
                            .h(rems(2.5))
                            .debug_selector(move || id.into())
                    }),
                )
            }
        }
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            cx.update(|window, cx| {
                theme.clone().apply(cx);
                window.activate_window();
            });
            cx.run_until_parked();
            for (id, role) in [
                ("press-bezel", PaintRole::Bezel),
                ("press-prominent", PaintRole::Prominent(Tint::Accent)),
            ] {
                let redraw = |cx: &mut gpui_kit::VisualTestContext| {
                    view.update(cx, |_, cx| cx.notify());
                    cx.update(|window, cx| window.draw(cx).clear(cx));
                };
                redraw(cx);
                let center = cx.debug_bounds(id).unwrap().center();
                cx.simulate_mouse_move(center, None, Modifiers::default());
                cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
                redraw(cx);
                let pressed = paint(role, Part::Fill, States::active().pressed(true), &theme);
                assert_eq!(fill(id), pressed, "{id} pressed (spec 6.6)");
                cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
                cx.simulate_mouse_move(point(px(390.), px(390.)), None, Modifiers::default());
                redraw(cx);
                assert_eq!(fill(id), paint(role, Part::Fill, States::active(), &theme));
            }
            let expected = if theme.is_dark() { 0x2d87ff } else { 0x036ee6 };
            assert_eq!(
                paint(
                    PaintRole::Prominent(Tint::Accent),
                    Part::Fill,
                    States::active().pressed(true),
                    &theme
                ),
                gpui_kit::rgb(expected).into()
            );
        }
    }

    #[gpui_kit::test]
    fn background_button_fill_updates_immediately_when_the_theme_changes(cx: &mut TestAppContext) {
        cx.update(crate::init);
        struct Harness;
        impl Render for Harness {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                div().children([false, true].map(|disabled| {
                    crate::button(
                        if disabled {
                            "disabled-fill"
                        } else {
                            "background-fill"
                        },
                        "Choose",
                        crate::ButtonVariant::Secondary,
                        cx,
                    )
                    .bg(cx.macos().control_fill())
                    .disabled(disabled)
                }))
            }
        }
        let (view, cx) = cx.add_window_view(|_, _| Harness);
        cx.deactivate_window();
        for theme in [
            crate::Theme::dark(),
            crate::Theme::light(),
            crate::Theme::dark(),
        ] {
            cx.update(|window, cx| {
                assert!(!window.is_window_active());
                theme.apply(cx);
            });
            view.update(cx, |_, cx| cx.notify());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                let expected = cx.macos().control_fill();
                for id in ["background-fill", "disabled-fill"] {
                    assert_eq!(
                        fill(id),
                        expected,
                        "Inactive fills must follow the new theme without animation frames"
                    );
                }
            });
        }
    }

    #[gpui_kit::test]
    fn disabled_button_suppresses_hover_and_pressed_geometry(cx: &mut TestAppContext) {
        cx.update(crate::init);
        struct States {
            disabled: bool,
            disabled_first: bool,
        }
        impl Render for States {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let button = crate::button("button", "Apply", crate::ButtonVariant::Primary, cx)
                    .w(rems(6.25))
                    .h(rems(2.5));
                let button = if self.disabled_first {
                    button
                        .disabled(self.disabled)
                        .hover(|s| s.w(rems(10.)))
                        .active(|s| s.w(rems(12.5)))
                } else {
                    button
                        .hover(|s| s.w(rems(10.)))
                        .active(|s| s.w(rems(12.5)))
                        .disabled(self.disabled)
                };
                div()
                    .size(rems(25.))
                    .child(button.debug_selector(|| "state-button".into()))
            }
        }
        for disabled_first in [false, true] {
            let (view, cx) = cx.add_window_view(move |_, _| States {
                disabled: true,
                disabled_first,
            });
            cx.update(|window, _| window.activate_window());
            cx.run_until_parked();
            cx.update(|window, cx| window.draw(cx).clear(cx));
            let position = point(px(10.), px(10.));
            cx.simulate_mouse_move(position, None, Modifiers::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.debug_bounds("state-button").unwrap().size.width,
                px(100.)
            );
            cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.debug_bounds("state-button").unwrap().size.width,
                px(100.)
            );
            cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
            view.update(cx, |this, cx| {
                this.disabled = false;
                cx.notify();
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::default());
            cx.simulate_mouse_move(position, None, Modifiers::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.debug_bounds("state-button").unwrap().size.width,
                px(160.)
            );
            cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
            assert_eq!(
                cx.debug_bounds("state-button").unwrap().size.width,
                px(200.)
            );
            cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
        }
    }

    #[gpui_kit::test]
    fn native_button_exposes_disabled_metadata_without_activation_or_focus_actions(
        cx: &mut TestAppContext,
    ) {
        use gpui_kit::{Element as _, Role, accesskit, canvas};

        type Captured = Rc<RefCell<Option<(accesskit::Node, accesskit::Node)>>>;
        struct MetadataProbe(Captured);

        impl Render for MetadataProbe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let captured = self.0.clone();
                canvas(
                    move |_, window, cx| {
                        let mut metadata = |id: &'static str, disabled| {
                            let native =
                                crate::button(id, "Apply", crate::ButtonVariant::Primary, cx)
                                    .accessibility_label("Apply changes")
                                    .disabled(disabled)
                                    .on_click(|_, _, _| {});
                            // Resolve the actual Base element used by the
                            // public macOS constructor and setters. The macOS
                            // RenderOnce output is an opaque Base component;
                            // its ViewElement delays this final node writer.
                            let element = native.base.render(window, cx).into_element();
                            assert_eq!(element.id(), Some(id.into()));
                            assert_eq!(element.a11y_role(), Some(Role::Button));
                            let mut node = accesskit::Node::new(Role::Button);
                            element.write_a11y_info(&mut node);
                            node
                        };
                        *captured.borrow_mut() = Some((
                            metadata("metadata-enabled", false),
                            metadata("metadata-disabled", true),
                        ));
                    },
                    |_, _, _, _| {},
                )
            }
        }

        cx.update(crate::init);
        let captured = Rc::new(RefCell::new(None));
        let (_, cx) = cx.add_window_view({
            let captured = captured.clone();
            move |_, _| MetadataProbe(captured)
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let (enabled, disabled) = captured.borrow_mut().take().unwrap();
        for node in [&enabled, &disabled] {
            assert_eq!(node.role(), Role::Button);
            assert_eq!(node.label(), Some("Apply changes"));
        }
        assert!(!enabled.is_disabled());
        assert!(enabled.supports_action(accesskit::Action::Click));
        assert!(enabled.supports_action(accesskit::Action::Focus));
        assert!(disabled.is_disabled());
        assert!(!disabled.supports_action(accesskit::Action::Click));
        assert!(!disabled.supports_action(accesskit::Action::Focus));
    }
}
