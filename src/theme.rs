//! macOS semantic colors, typography, geometry, and material preferences.
use gpui_kit::base::motion::{Easing, Transition};
use gpui_kit::base::{ColorTokens, PlotMotion, PlotTheme, RadiusTokens, Spring, ThemeAppearance};
use gpui_kit::{App, BoxShadow, Global, Hsla, SharedString, WindowAppearance, px, rgb};
use std::time::Duration;

/// Shared presentation for native adapters and the complete Kit facade.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: SharedString,
    pub appearance: ThemeAppearance,
    pub background: Hsla,
    pub surface: Hsla,
    pub inset: Hsla,
    pub foreground: Hsla,
    pub secondary: Hsla,
    pub bright: Hsla,
    pub accent: Hsla,
    pub on_accent: Hsla,
    pub selection: Hsla,
    pub border: Hsla,
    pub danger: Hsla,
    pub warning: Hsla,
    pub success: Hsla,
    pub chart: [Hsla; 5],
    pub font: SharedString,
    pub mono_font: SharedString,
    /// Material tint from clear (0) to fully tinted (1). Default: 0.65.
    pub glass_tint: f32,
    /// Opaque surfaces for Reduce transparency.
    pub reduce_transparency: bool,
    /// Stronger borders and separators for Increase contrast.
    pub increased_contrast: bool,
}
impl Global for Theme {}
impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    pub fn light() -> Self {
        Self {
            name: "macOS Light".into(),
            appearance: ThemeAppearance::Light,
            background: rgb(0xffffff).into(),
            surface: rgb(0xffffff).into(),
            inset: rgb(0xf7f7f7).into(),
            foreground: Hsla::black().opacity(216. / 255.),
            secondary: Hsla::black().opacity(127. / 255.),
            bright: rgb(0).into(),
            accent: rgb(0x007aff).into(),
            on_accent: rgb(0xffffff).into(),
            selection: rgb(0xb3d7ff).into(),
            border: Hsla::from(rgb(0)).opacity(0.12),
            danger: rgb(0xff3b30).into(),
            warning: rgb(0xa46100).into(),
            success: rgb(0x34c759).into(),
            chart: [
                rgb(0x007aff).into(),
                rgb(0xaf52de).into(),
                rgb(0x30b0c7).into(),
                rgb(0xff9500).into(),
                rgb(0x34c759).into(),
            ],
            font: ".SystemUIFont".into(),
            mono_font: default_mono_font(),
            glass_tint: 0.65,
            reduce_transparency: false,
            increased_contrast: false,
        }
    }
    pub fn dark() -> Self {
        Self {
            name: "macOS Dark".into(),
            appearance: ThemeAppearance::Dark,
            background: rgb(0x1e1e1e).into(),
            surface: rgb(0x1e1e1e).into(),
            inset: rgb(0x262626).into(),
            foreground: Hsla::white().opacity(216. / 255.),
            secondary: Hsla::white().opacity(140. / 255.),
            bright: rgb(0xffffff).into(),
            accent: rgb(0x007aff).into(),
            on_accent: rgb(0xffffff).into(),
            selection: rgb(0x3f638b).into(),
            border: Hsla::from(rgb(0xffffff)).opacity(0.13),
            danger: rgb(0xff453a).into(),
            warning: rgb(0xffd60a).into(),
            success: rgb(0x30d158).into(),
            chart: [
                rgb(0x007aff).into(),
                rgb(0xbf5af2).into(),
                rgb(0x64d2ff).into(),
                rgb(0xff9f0a).into(),
                rgb(0x30d158).into(),
            ],
            font: ".SystemUIFont".into(),
            mono_font: default_mono_font(),
            glass_tint: 0.65,
            reduce_transparency: false,
            increased_contrast: false,
        }
    }
    pub fn from_appearance(appearance: WindowAppearance) -> Self {
        let mut theme = match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        };
        theme.read_accessibility();
        theme
    }
    /// Snapshot without changing whether the application follows the system.
    pub fn system(cx: &App) -> Self {
        Self::from_appearance(cx.window_appearance())
    }
    pub fn is_dark(&self) -> bool {
        self.appearance == ThemeAppearance::Dark
    }
    pub fn with_accent(mut self, accent: Hsla) -> Self {
        self.accent = accent;
        self.selection = accent.opacity(if self.is_dark() { 0.32 } else { 0.3 });
        self.chart[0] = accent;
        self
    }
    pub fn with_glass_tint(mut self, tint: f32) -> Self {
        self.glass_tint = if tint.is_finite() {
            tint.clamp(0., 1.)
        } else {
            0.65
        };
        self
    }
    pub fn with_reduce_transparency(mut self, reduce: bool) -> Self {
        self.reduce_transparency = reduce;
        self
    }
    pub fn with_increased_contrast(mut self, increase: bool) -> Self {
        self.increased_contrast = increase;
        self
    }
    pub fn normal_fill(&self) -> Hsla {
        self.foreground.opacity(0.04)
    }
    pub fn hover_fill(&self) -> Hsla {
        self.foreground.opacity(0.07)
    }
    pub fn selected_fill(&self) -> Hsla {
        self.accent
            .opacity(if self.is_dark() { 0.25 } else { 0.12 })
    }
    pub fn pressed_fill(&self) -> Hsla {
        self.foreground.opacity(0.12)
    }
    pub fn control_fill(&self) -> Hsla {
        // A neutral bezel is distinct from the text editor's white/dark well.
        // AppKit resolves these roles independently, even in the same group.
        self.background.blend(self.foreground.opacity(0.11))
    }
    pub fn control_border(&self) -> Hsla {
        self.foreground.opacity(if self.increased_contrast {
            0.55
        } else if self.is_dark() {
            0.1
        } else {
            0.08
        })
    }
    pub fn focus_border(&self) -> Hsla {
        self.accent
    }
    pub fn divider(&self) -> Hsla {
        self.foreground.opacity(if self.increased_contrast {
            0.4
        } else {
            25. / 255.
        })
    }
    /// Translucent fill over GPUI's native blurred window backdrop.
    /// An approximation of the material; it does not implement Apple's refraction shader.
    pub fn glass_fill(&self) -> Hsla {
        if self.reduce_transparency {
            self.surface
        } else {
            self.surface
                .opacity(0.42 + 0.5 * self.glass_tint.clamp(0., 1.))
        }
    }
    pub fn glass_border(&self) -> Hsla {
        if self.increased_contrast {
            self.control_border()
        } else if self.is_dark() {
            Hsla::from(rgb(0xffffff)).opacity(0.18)
        } else {
            Hsla::from(rgb(0)).opacity(0.1)
        }
    }
    pub fn control_shadow(&self) -> Vec<BoxShadow> {
        vec![
            BoxShadow::new(
                px(0.),
                px(1.),
                Hsla::from(rgb(0)).opacity(if self.is_dark() { 0.2 } else { 0.08 }),
            )
            .blur_radius(px(2.)),
        ]
    }
    pub fn focus_shadow(&self) -> Vec<BoxShadow> {
        vec![BoxShadow::new(px(0.), px(0.), self.accent.opacity(0.35)).spread_radius(px(3.))]
    }
    pub fn popover_shadow(&self) -> Vec<BoxShadow> {
        vec![
            BoxShadow::new(
                px(0.),
                px(8.),
                Hsla::from(rgb(0)).opacity(if self.is_dark() { 0.4 } else { 0.16 }),
            )
            .blur_radius(px(28.)),
            BoxShadow::new(px(0.), px(1.), Hsla::from(rgb(0)).opacity(0.09)).blur_radius(px(4.)),
        ]
    }
    pub fn input_style(&self) -> gpui_kit::base::input::InputEditorStyle {
        gpui_kit::base::input::InputEditorStyle {
            foreground: self.foreground,
            muted_foreground: self.secondary,
            background: self.surface,
            border: self.control_border(),
            selection: self.selection,
            caret: self.accent,
            ..Default::default()
        }
    }
    pub fn tokens(&self) -> ColorTokens {
        ColorTokens {
            background: self.background,
            foreground: self.foreground,
            surface: self.surface,
            surface_foreground: self.foreground,
            primary: self.accent,
            primary_foreground: self.on_accent,
            secondary: self.surface,
            secondary_foreground: self.foreground,
            muted: self.inset,
            muted_foreground: self.secondary,
            accent: self.selected_fill(),
            accent_foreground: self.foreground,
            destructive: self.danger,
            destructive_foreground: self.on_accent,
            border: self.control_border(),
            input: self.control_border(),
            ring: self.accent,
            selection: self.selection,
        }
    }
    /// Apply explicit appearance and stop following the system.
    pub fn apply(self, cx: &mut App) {
        crate::system_theme::stop_following(cx);
        cx.set_window_appearance(Some(if self.is_dark() {
            WindowAppearance::Dark
        } else {
            WindowAppearance::Light
        }));
        self.apply_palette(cx);
    }
    pub(crate) fn read_accessibility(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
            self.reduce_transparency = workspace.accessibilityDisplayShouldReduceTransparency();
            self.increased_contrast = workspace.accessibilityDisplayShouldIncreaseContrast();
        }
    }
    pub(crate) fn apply_palette(self, cx: &mut App) {
        self.sync_kit(cx);
        let base = gpui_kit::base::Theme::global_mut(cx);
        base.appearance = self.appearance;
        base.tokens.colors = self.tokens();
        base.tokens.radius = RadiusTokens {
            none: px(0.),
            sm: px(4.),
            md: px(6.),
            lg: px(10.),
            xl: px(16.),
            full: px(9999.),
        };
        base.tokens.shadow.sm = self.control_shadow();
        base.tokens.shadow.md = self.popover_shadow();
        base.tokens.shadow.lg = self.popover_shadow();
        let type_scale = &mut base.tokens.typography;
        type_scale.sans = self.font.clone();
        type_scale.mono = self.mono_font.clone();
        for (token, size) in [
            (&mut type_scale.xs, 10.),
            (&mut type_scale.sm, 11.),
            (&mut type_scale.md, 13.),
            (&mut type_scale.lg, 15.),
            (&mut type_scale.xl, 20.),
            (&mut type_scale.mono_md, 12.),
        ] {
            token.size = px(size);
            token.line_height = px(size * 1.4);
        }
        let duration = Duration::from_millis(180);
        base.plot = PlotTheme::new().with_motion(
            PlotMotion::default()
                .with_pointer(Spring::new(duration).with_epsilon(0.1))
                .with_enter(
                    Transition::new(duration)
                        .easing(Easing::cubic_bezier(0.16, 1., 0.3, 1.).expect("static curve")),
                )
                .with_exit(Transition::new(Duration::from_millis(120))),
        );
        cx.set_global(self);
        cx.refresh_windows();
    }
    fn sync_kit(&self, cx: &mut App) {
        use gpui_kit::component::theme::{Theme as KitTheme, ThemeMode};
        let mode = if self.is_dark() {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        if KitTheme::global(cx).mode != mode {
            KitTheme::change(mode, None, cx);
        }
        KitTheme::update(cx, |kit| {
            kit.font_family = self.font.clone();
            kit.mono_font_family = self.mono_font.clone();
            // Kit uses this value as Window::rem_size, rather than a label size.
            kit.font_size = px(16.);
            kit.mono_font_size = px(12.);
            kit.radius = px(6.);
            kit.radius_lg = px(16.);
            kit.shadow = true;
            kit.focus_ring = true;
            let c = &mut kit.colors;
            c.background = self.background;
            c.foreground = self.foreground;
            c.border = self.control_border();
            c.input = self.control_border();
            c.accent = self.selected_fill();
            c.accent_foreground = self.foreground;
            c.primary = self.accent;
            c.primary_foreground = self.on_accent;
            c.primary_hover = self.accent.opacity(0.85);
            c.primary_active = self.accent.opacity(0.7);
            c.secondary = self.control_fill();
            c.secondary_foreground = self.foreground;
            c.secondary_hover = self.hover_fill();
            c.secondary_active = self.pressed_fill();
            c.button = self.control_fill();
            c.button_foreground = self.foreground;
            c.button_hover = self.hover_fill();
            c.button_active = self.pressed_fill();
            c.button_primary = c.primary;
            c.button_primary_foreground = c.primary_foreground;
            c.button_primary_hover = c.primary_hover;
            c.button_primary_active = c.primary_active;
            c.button_secondary = c.secondary;
            c.button_secondary_foreground = c.secondary_foreground;
            c.button_secondary_hover = c.secondary_hover;
            c.button_secondary_active = c.secondary_active;
            c.danger = self.danger;
            c.danger_foreground = self.on_accent;
            c.danger_hover = self.danger.opacity(0.85);
            c.danger_active = self.danger.opacity(0.7);
            c.button_danger = c.danger;
            c.button_danger_foreground = c.danger_foreground;
            c.button_danger_hover = c.danger_hover;
            c.button_danger_active = c.danger_active;
            c.success = self.success;
            c.success_foreground = Hsla::from(rgb(0x003b11));
            c.success_hover = self.success.opacity(0.85);
            c.success_active = self.success.opacity(0.7);
            c.button_success = c.success;
            c.button_success_foreground = c.success_foreground;
            c.button_success_hover = c.success_hover;
            c.button_success_active = c.success_active;
            c.warning = self.warning;
            c.warning_foreground = if self.is_dark() {
                self.background
            } else {
                self.on_accent
            };
            c.warning_hover = self.warning.opacity(0.85);
            c.warning_active = self.warning.opacity(0.7);
            c.button_warning = c.warning;
            c.button_warning_foreground = c.warning_foreground;
            c.button_warning_hover = c.warning_hover;
            c.button_warning_active = c.warning_active;
            c.info = self.accent;
            c.info_foreground = self.on_accent;
            c.info_hover = c.primary_hover;
            c.info_active = c.primary_active;
            c.button_info = c.info;
            c.button_info_foreground = c.info_foreground;
            c.button_info_hover = c.info_hover;
            c.button_info_active = c.info_active;
            c.muted = self.inset;
            c.muted_foreground = self.secondary;
            c.popover = self.glass_fill();
            c.popover_foreground = self.foreground;
            c.group_box = self.inset;
            c.group_box_foreground = self.foreground;
            c.accordion = self.surface;
            c.description_list_label = self.background;
            c.description_list_label_foreground = self.secondary;
            c.caret = self.accent;
            c.selection = self.selection;
            c.ring = self.accent.opacity(0.4);
            c.link = self.accent;
            c.link_hover = self.accent;
            c.link_active = self.accent.opacity(0.75);
            c.drag_border = self.accent;
            c.drop_target = self.selected_fill();
            c.sidebar = self.glass_fill();
            c.sidebar_foreground = self.foreground;
            c.sidebar_accent = self.selected_fill();
            c.sidebar_accent_foreground = self.foreground;
            c.sidebar_primary = self.accent;
            c.sidebar_primary_foreground = self.on_accent;
            c.sidebar_border = self.divider();
            c.list = self.surface;
            c.list_active = self.selected_fill();
            c.list_active_border = self.accent;
            c.list_hover = self.hover_fill();
            c.list_even = self.normal_fill();
            c.list_head = self.background;
            c.table = self.surface;
            c.table_active = self.selected_fill();
            c.table_active_border = self.accent;
            c.table_even = self.normal_fill();
            c.table_head = self.background;
            c.table_head_foreground = self.secondary;
            c.table_foot = self.background;
            c.table_foot_foreground = self.secondary;
            c.table_hover = self.hover_fill();
            c.table_row_border = self.divider();
            c.tab = self.background;
            c.tab_foreground = self.secondary;
            c.tab_active = self.surface;
            c.tab_active_foreground = self.foreground;
            c.tab_bar = self.background;
            c.tab_bar_segmented = self.inset;
            c.title_bar = self.glass_fill();
            c.title_bar_border = self.divider();
            c.status_bar = self.background;
            c.status_bar_border = self.divider();
            c.skeleton = self.inset;
            c.progress_bar = self.accent;
            c.slider_bar = self.accent;
            c.slider_thumb = Hsla::from(rgb(0xffffff));
            c.switch = self.control_fill();
            c.switch_thumb = Hsla::from(rgb(0xffffff));
            c.scrollbar = self.foreground.opacity(0.);
            c.scrollbar_thumb = self.foreground.opacity(0.28);
            c.scrollbar_thumb_hover = self.foreground.opacity(0.48);
            c.overlay = Hsla::from(rgb(0)).opacity(0.24);
            c.window_border = self.border;
            c.chart_1 = self.chart[0];
            c.chart_2 = self.chart[1];
            c.chart_3 = self.chart[2];
            c.chart_4 = self.chart[3];
            c.chart_5 = self.chart[4];
            c.chart_bullish = self.success;
            c.chart_bearish = self.danger;
            c.chart_grid = self.divider();
            c.red = self.danger;
            c.red_light = self.danger.opacity(0.14);
            c.green = self.success;
            c.green_light = self.success.opacity(0.14);
            c.blue = self.accent;
            c.blue_light = self.selected_fill();
            c.yellow = self.warning;
            c.yellow_light = self.warning.opacity(0.14);
            c.magenta = self.chart[1];
            c.magenta_light = self.chart[1].opacity(0.14);
            c.cyan = self.chart[2];
            c.cyan_light = self.chart[2].opacity(0.14);
        });
    }
}
pub(crate) fn default_mono_font() -> SharedString {
    if cfg!(target_os = "macos") {
        "Menlo".into()
    } else if cfg!(target_os = "windows") {
        "Consolas".into()
    } else {
        "monospace".into()
    }
}
pub trait ActiveTheme {
    fn macos(&self) -> &Theme;
}
impl ActiveTheme for App {
    fn macos(&self) -> &Theme {
        self.global::<Theme>()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::TestAppContext;
    #[test]
    fn material_tint_is_bounded_and_accessibility_is_opaque() {
        let clear = Theme::light().with_glass_tint(-1.);
        let tinted = Theme::light().with_glass_tint(2.);
        assert!(clear.glass_fill().a < tinted.glass_fill().a);
        assert_eq!(tinted.with_reduce_transparency(true).glass_fill().a, 1.);
        assert_eq!(Theme::dark().with_glass_tint(f32::NAN).glass_tint, 0.65);
    }
    #[gpui_kit::test]
    fn all_three_theme_layers_change_together(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for theme in [Theme::dark(), Theme::light()] {
            cx.update(|cx| {
                theme.clone().apply(cx);
                let base = gpui_kit::base::Theme::global(cx);
                let kit = gpui_kit::component::theme::Theme::global(cx);
                assert_eq!(base.tokens.colors.primary, theme.accent);
                assert_eq!(kit.primary, theme.accent);
                assert_eq!(kit.tokens.primary.color, theme.accent);
                assert_eq!(base.tokens.radius.md, px(6.));
                assert_eq!(cx.macos(), &theme);
            });
        }
    }
}
