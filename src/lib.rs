//! macOS's presentation language on gpui-base's interaction primitives.
//!
//! Call [`init`] once, then use the constructors in [`controls`] and [`surface`]
//! inside `Render`. They retain base builder APIs; Button adds disabled-state style gating.
#![doc = include_str!("../README.md")]

// GPUI macros expand through the canonical `gpui` crate path.
extern crate gpui_kit as gpui;

pub mod button;
pub mod button_group;
pub mod calendar;
pub mod catalog;
pub mod chart;
pub mod chrome;
pub mod color_picker;
pub mod controls;
pub mod date_picker;
pub mod dialog;
pub mod dock;
pub mod fields;
pub mod focus;
pub mod hover_card;
pub mod icon;
pub mod input;
pub mod interpretation;
pub mod link;
pub mod list;
pub mod loading;
pub mod menu;
mod motion;
pub mod native;
pub mod native_assets;
#[cfg(target_os = "macos")]
mod native_symbols;
pub mod navigation;
pub mod otp_input;
pub mod plot;
pub mod popover;
pub mod resizable;
pub mod select;
pub mod sheet;
pub mod slider;
mod state;
pub mod surface;
mod switch_motion;
mod system_theme;
pub mod table;
pub mod text;
pub mod theme;
pub mod tokens;
pub mod tooltip;
pub mod tree;
pub mod window;

pub use button::Button;
pub use button_group::{button_group, tab_list};
pub use calendar::calendar;
pub use chart::{
    AreaChart, BarChart, CandlestickChart, LineChart, PieChart, RadarChart, SankeyChart,
};
pub use chrome::*;
pub use color_picker::color_picker;
pub use controls::*;
pub use date_picker::{DatePickerState, date_picker};
pub use dialog::*;
pub use dock::dock_area;
pub use fields::{search_field, secure_field};
pub use focus::focus_scope;
pub use hover_card::hover_card;
pub use icon::{Icon, IconName, icon};
pub use input::{Input, input, number_input, textarea};
pub use link::Link;
pub use list::{inset_list, inset_list_row, scrollbar, virtual_list};
pub use loading::native_spinner;
pub use menu::{MenuItem, menu};
pub use native_assets::NativeAssets;
pub use navigation::*;
pub use otp_input::{OtpInput, otp_input};
pub use popover::{popover, popover_surface};
pub use resizable::{resizable, resizable_panel};
pub use select::{ChoiceItem, ChoiceState, combobox, select};
pub use sheet::{sheet, sheet_surface};
pub use slider::slider;
pub use surface::*;

#[doc(hidden)]
pub use gpui_kit;

/// Create an application using the current desktop platform.
pub use gpui_kit::application;
pub use system_theme::{observe_window, refresh_accessibility};
pub use table::*;
pub use text::{html, markdown, text_view_style};
pub use theme::{ActiveTheme, Theme};
pub use tokens::{NativeTextMetrics, NativeTextStyle, TextStyle};
pub use tooltip::{tooltip, with_tooltip};
pub use tree::tree;
pub use window::{set_zoom, zoom_percent};

/// Complete GPUI Kit components, styled by the same macOS palette and geometry.
/// Advanced stateful controls require a `gpui_kit::base::Root` window root.
pub use gpui_kit::component as kit;

// Advanced components keep their full upstream state and builder APIs.
pub use button_group::button_group as segmented_control;
pub use kit::time_field;
pub use kit::{
    attachment, breadcrumb, bubble, carousel, clipboard, command, description_list, diff, form,
    group_box, label, marker, message, message_scroller, notification, questionnaire, rating,
    setting, shimmer, skeleton, speech, spinner, tag,
};

/// Native window settings with real system traffic lights and a blurred backdrop.
/// Place a [`title_bar`] at the top of the content when using this configuration.
pub fn window_options(
    title: impl Into<gpui_kit::SharedString>,
    cx: &gpui_kit::App,
) -> gpui_kit::WindowOptions {
    use gpui_kit::*;
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(16.), px(16.))),
        }),
        window_background: if cx.macos().reduce_transparency {
            WindowBackgroundAppearance::Opaque
        } else {
            WindowBackgroundAppearance::Blurred
        },
        window_min_size: Some(size(px(640.), px(480.))),
        ..Default::default()
    }
}

/// Window settings for an AppKit toolbar and native glass underlays.
/// Use with [`native::install_native_toolbar`]. AppKit places the traffic lights;
/// transparent renderer regions reveal system glass without a legacy blur layer.
pub fn native_window_options(
    title: impl Into<gpui_kit::SharedString>,
    cx: &gpui_kit::App,
) -> gpui_kit::WindowOptions {
    let mut options = window_options(title, cx);
    if let Some(titlebar) = &mut options.titlebar {
        titlebar.traffic_light_position = None;
    }
    options.window_background = if cx.macos().reduce_transparency {
        gpui_kit::WindowBackgroundAppearance::Opaque
    } else {
        gpui_kit::WindowBackgroundAppearance::Transparent
    };
    options
}

/// Open a Root-hosted window and install appearance observation automatically.
/// Root provides the overlay, notification, focus and text-selection services.
pub fn open_window<V: gpui_kit::Render + 'static>(
    options: gpui_kit::WindowOptions,
    cx: &mut gpui_kit::App,
    build: impl FnOnce(&mut gpui_kit::Window, &mut gpui_kit::App) -> gpui_kit::Entity<V>,
) -> anyhow::Result<(gpui_kit::AnyWindowHandle, gpui_kit::Entity<V>)> {
    gpui_kit::open_window(options, cx, |window, cx| {
        observe_window(window, cx);
        build(window, cx)
    })
}

/// Initialize all behavior, native adapters, and system appearance once at startup.
pub fn init(cx: &mut gpui_kit::App) {
    gpui_kit::init(cx);
    theme::init(cx);
    window::init(cx);
    focus::init(cx);
    button_group::init(cx);
    popover::init(cx);
    date_picker::init(cx);
    native::install_glass_popup_presenter(cx);
    #[cfg(not(target_family = "wasm"))]
    Theme::follow_system(cx);
    #[cfg(target_family = "wasm")]
    Theme::dark().apply(cx);
}
