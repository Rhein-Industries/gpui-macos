//! Auditable macOS presentation contract for the complete component catalog.
//!
//! A counterpart describes the design being followed, not a claim that a
//! GPUI element is an AppKit control instance. System-backed entries identify
//! their actual native integrations and retain deterministic canvas fallbacks.
use crate::NativeTextStyle as _;

/// How a component obtains its macOS presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderingKind {
    /// Host AppKit chrome, material, or artwork, composed with GPUI content.
    AppKitBacked,
    /// GPUI rendering and behavior following an existing macOS control.
    GpuiReplica,
    /// A macOS composition for a workflow without one equivalent AppKit control.
    MacInterpretation,
}

impl RenderingKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AppKitBacked => "AppKit backed",
            Self::GpuiReplica => "GPUI replica",
            Self::MacInterpretation => "macOS interpretation",
        }
    }
}

/// The operating-system integration actually used by an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeIntegration {
    WindowChrome,
    UnifiedToolbar,
    GlassMaterial,
    SfSymbols,
    SystemAppearance,
    Clipboard,
    InputMethod,
    SpeechRecognizer,
}

/// Optional upstream capabilities keep their original feature requirements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeatureRequirement {
    TreeSitterLanguages,
    Speech,
}

/// The counterpart, reusable API, and behavior retained by a gallery page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentInterpretation {
    pub id: &'static str,
    pub rendering: RenderingKind,
    pub counterpart: &'static str,
    pub design: &'static str,
    pub api: &'static str,
    pub capabilities: &'static [&'static str],
    pub native_integrations: &'static [NativeIntegration],
    pub optional_features: &'static [FeatureRequirement],
}

/// Each pinned public Kit module maps to actual catalog pages, including
/// supporting modules that do not have a dedicated upstream documentation page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleCoverage {
    pub module: &'static str,
    pub pages: &'static [&'static str],
    pub role: &'static str,
}

pub const UPSTREAM_REVISION: &str = "0bbd9870ce420af9feacbf948b21d3824d82dc0b";

pub const INTERPRETATIONS: &[ComponentInterpretation] = &[
    ComponentInterpretation {
        id: "overview",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "System Settings preference pane",
        design: "Quiet grouped rows, aligned labels and trailing save actions",
        api: "gpui_macos::settings_group / gpui_macos::settings_row",
        capabilities: &[
            "Editable workspace",
            "retained selections",
            "save and cancel",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "theme",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS semantic appearance",
        design: "System font, label hierarchy, accent and appearance tokens",
        api: "gpui_macos::Theme",
        capabilities: &[
            "Light and dark",
            "system following",
            "accessibility preferences",
        ],
        native_integrations: &[NativeIntegration::SystemAppearance],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "root",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS application content host",
        design: "Transparent Root with native chrome and opaque content panes",
        api: "gpui_macos::kit::Root",
        capabilities: &[
            "Overlay hosting",
            "focus",
            "selectable text",
            "notifications",
        ],
        native_integrations: &[NativeIntegration::WindowChrome],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "focus_trap",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Modal keyboard focus scope",
        design: "Tab traversal remains inside a presented control group",
        api: "gpui_macos::kit::FocusTrapElement",
        capabilities: &["Forward and reverse focus", "stable trigger restoration"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "button",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSButton",
        design: "Compact system bezel and activation-aware accent",
        api: "gpui_macos::button",
        capabilities: &[
            "Primary and secondary",
            "disabled",
            "pointer and keyboard activation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "button_group",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSegmentedControl",
        design: "Joined 24-point segments with selected bezel",
        api: "gpui_macos::button_group / gpui_macos::segmented_control",
        capabilities: &[
            "Single selection",
            "disabled segments",
            "keyboard navigation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "dropdown_button",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSPopUpButton command menu",
        design: "Compact command bezel, disclosure and menu alignment",
        api: "gpui_macos::menu / gpui_macos::button",
        capabilities: &["Separators", "disabled commands", "action callbacks"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "link",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS linked text and URL action",
        design: "System accent text with focus and pointer feedback",
        api: "gpui_macos::link",
        capabilities: &["URL opening", "keyboard activation", "disabled state"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "toggle",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSButton toggle mode",
        design: "Retained selected bezel, compact spacing and focus ring",
        api: "gpui_macos::toggle",
        capabilities: &[
            "Controlled selection",
            "keyboard activation",
            "disabled state",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "toggle_group",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSegmentedControl multiple-selection mode",
        design: "Joined command segments with independent selection",
        api: "gpui_macos::toggle_group",
        capabilities: &[
            "Multiple selection",
            "disabled commands",
            "keyboard traversal",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "clipboard",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Copy command with inline confirmation",
        design: "Compact SF copy symbol and temporary success feedback",
        api: "gpui_macos::kit::clipboard::Clipboard",
        capabilities: &[
            "System clipboard",
            "value ownership",
            "accessible activation",
        ],
        native_integrations: &[NativeIntegration::Clipboard],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "command",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Spotlight-style command search",
        design: "Searchable compact command list with sections and shortcuts",
        api: "gpui_macos::kit::command::Command",
        capabilities: &[
            "Filtering",
            "grouped commands",
            "disabled entries",
            "Enter confirmation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "input",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTextField",
        design: "24-point inset content well and native label density",
        api: "gpui_macos::input",
        capabilities: &[
            "Selection",
            "undo and redo",
            "clipboard",
            "IME",
            "validation",
        ],
        native_integrations: &[NativeIntegration::InputMethod, NativeIntegration::Clipboard],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "textarea",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTextView",
        design: "Multiline content well with ordinary document typography",
        api: "gpui_macos::textarea",
        capabilities: &[
            "Multiline editing",
            "selection",
            "clipboard",
            "IME",
            "scrolling",
        ],
        native_integrations: &[NativeIntegration::InputMethod, NativeIntegration::Clipboard],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "editor",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Xcode-style source editor",
        design: "Monospaced editing pane, line gutters and restrained chrome",
        api: "gpui_macos::kit::input::Editor",
        capabilities: &[
            "Find",
            "multiple cursors",
            "folding",
            "diagnostics",
            "optional syntax grammars",
        ],
        native_integrations: &[NativeIntegration::InputMethod, NativeIntegration::Clipboard],
        optional_features: &[FeatureRequirement::TreeSitterLanguages],
    },
    ComponentInterpretation {
        id: "number_input",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTextField with NSStepper",
        design: "Compact numeric editor and increment/decrement commands",
        api: "gpui_macos::number_input",
        capabilities: &[
            "Numeric editing",
            "stepping",
            "bounds",
            "controlled updates",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "input_group",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Composite macOS text-field accessory row",
        design: "Field-aligned prefix, suffix and accessory commands",
        api: "gpui_macos::kit::input::InputGroup",
        capabilities: &[
            "Shared editor",
            "affixes",
            "accessory alignment",
            "keyboard input",
        ],
        native_integrations: &[NativeIntegration::InputMethod, NativeIntegration::Clipboard],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "select",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSPopUpButton selection mode",
        design: "Compact popup bezel with selected label and checkmark",
        api: "gpui_macos::select",
        capabilities: &[
            "Controlled selection",
            "grouped values",
            "disabled options",
            "native glass popup",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "combobox",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSComboBox",
        design: "Inset search field and compact matching-value list",
        api: "gpui_macos::combobox",
        capabilities: &[
            "Filtering",
            "keyboard selection",
            "focus",
            "retained value",
            "native glass popup",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "calendar",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSDatePicker calendar style",
        design: "Compact calendar grid, system labels and accent date",
        api: "gpui_macos::calendar",
        capabilities: &[
            "Month navigation",
            "date selection",
            "focus",
            "disabled dates",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "date_picker",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSDatePicker",
        design: "Compact date field with calendar disclosure",
        api: "gpui_macos::date_picker",
        capabilities: &[
            "Retained date",
            "date formatting",
            "popup calendar",
            "keyboard input",
            "native glass calendar popup",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "time_field",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSDatePicker hour/minute style",
        design: "Compact segmented time editing with native label size",
        api: "gpui_macos::kit::time_field::TimeField",
        capabilities: &["Time value", "segment focus", "increment and decrement"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "color_picker",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSColorWell with NSColorPanel",
        design: "Small color well opening an editable color inspector",
        api: "gpui_macos::color_picker",
        capabilities: &[
            "Hex and channel editing",
            "commit and cancel",
            "retained color",
            "native glass inspector popup",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "otp_input",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS verification-code entry",
        design: "Compact aligned fields in a verification form",
        api: "gpui_macos::otp_input",
        capabilities: &[
            "Digit filtering",
            "paste distribution",
            "completion",
            "focus movement",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "slider",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSlider",
        design: "Thin track and horizontal pill thumb, with inactive colors",
        api: "gpui_macos::slider",
        capabilities: &[
            "Single value",
            "range values",
            "keyboard changes",
            "disabled state",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "checkbox",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSButton checkbox type",
        design: "16-point rounded indicator and SF check/minus marks",
        api: "gpui_macos::checkbox",
        capabilities: &[
            "Checked and mixed",
            "controlled updates",
            "disabled",
            "keyboard activation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "switch",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSwitch",
        design: "54-by-24 track, elongated thumb and activation-aware color",
        api: "gpui_macos::switch",
        capabilities: &[
            "Controlled value",
            "animated thumb",
            "reduced motion",
            "disabled",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "radio",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSButton radio type",
        design: "16-point circular indicator and compact option rows",
        api: "gpui_macos::radio",
        capabilities: &[
            "Exclusive selection",
            "arrow navigation",
            "focus",
            "disabled",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "form",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "System Settings editable preference group",
        design: "Aligned labels, secondary help and trailing save action",
        api: "gpui_macos::interpretation::form / gpui_macos::kit::form::Form",
        capabilities: &[
            "Fields",
            "validation ownership",
            "horizontal and multicolumn layout",
            "submission",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "questionnaire",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Setup Assistant guided preferences",
        design: "Quiet grouped radio rows, progress and Back/Continue actions",
        api: "gpui_macos::interpretation::questionnaire / gpui_macos::kit::questionnaire::*",
        capabilities: &[
            "Retained answers",
            "validation",
            "shortcuts",
            "previous/next",
            "submission",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "stepper",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Setup Assistant stage navigation",
        design: "Compact stages with current-step emphasis",
        api: "gpui_macos::kit::stepper::Stepper",
        capabilities: &["Selected stage", "pointer and keyboard stage activation"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "search_field",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSearchField",
        design: "Inset field with SF magnifier and native editing density",
        api: "gpui_macos::search_field",
        capabilities: &["Search text", "selection", "undo", "clipboard", "IME"],
        native_integrations: &[NativeIntegration::InputMethod, NativeIntegration::Clipboard],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "secure_field",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSecureTextField",
        design: "Inset field with masked display and secure copy behavior",
        api: "gpui_macos::secure_field",
        capabilities: &[
            "Masked editing",
            "secure clipboard policy",
            "IME",
            "retained value",
        ],
        native_integrations: &[NativeIntegration::InputMethod],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "menu",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSMenu",
        design: "Compact command rows, checks, separators and submenus",
        api: "gpui_macos::menu / gpui_macos::kit::native_menu::NativeMenu",
        capabilities: &[
            "GPUI command foreground",
            "full native-menu API",
            "disabled commands",
            "shortcuts",
            "separate native glass submenu panels",
            "owner-window command dispatch",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "tabs",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTabView with segmented tab selector",
        design: "Compact joined tab labels above retained content",
        api: "gpui_macos::tabs / gpui_macos::tab_list",
        capabilities: &["Controlled tab", "keyboard traversal", "disabled tabs"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "accordion",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS disclosure preference sections",
        design: "Disclosure symbol and plain expandable content rows",
        api: "gpui_macos::accordion",
        capabilities: &[
            "Open state",
            "exclusive or multiple expansion",
            "keyboard activation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "collapsible",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS disclosure group",
        design: "Quiet disclosure row with measured expansion",
        api: "gpui_macos::collapsible",
        capabilities: &["Retained open state", "accessible expansion", "animation"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "nav_stack",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS drill-down navigation",
        design: "Back/Forward commands and retained detail panes",
        api: "gpui_macos::nav_stack",
        capabilities: &[
            "Push and pop",
            "forward history",
            "retained page state",
            "motion",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "pagination",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS paged document navigation",
        design: "Compact previous/next controls and current-page label",
        api: "gpui_macos::pagination",
        capabilities: &[
            "Controlled page",
            "disabled boundaries",
            "keyboard activation",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "breadcrumb",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Finder-style path navigation",
        design: "Compact path segments and disclosure separators",
        api: "gpui_macos::kit::breadcrumb::Breadcrumb",
        capabilities: &[
            "Clickable ancestors",
            "disabled current location",
            "overflow composition",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "sidebar",
        rendering: RenderingKind::AppKitBacked,
        counterpart: "NSOutlineView-style source list on NSGlassEffectView",
        design: "Edge-aligned glass source list with semibold selected labels",
        api: "gpui_macos::sidebar / gpui_macos::sidebar_item",
        capabilities: &[
            "Selection",
            "keyboard traversal",
            "native glass",
            "deterministic fallback",
        ],
        native_integrations: &[
            NativeIntegration::GlassMaterial,
            NativeIntegration::SfSymbols,
        ],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "sheet",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSWindow sheet presentation",
        design: "Attached task pane with content, actions and focus containment",
        api: "gpui_macos::sheet",
        capabilities: &[
            "Open and close",
            "Escape",
            "focus trap",
            "trigger restoration",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "dialog",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSPanel task dialog",
        design: "Compact modal task layout with trailing actions",
        api: "gpui_macos::dialog",
        capabilities: &[
            "Controlled open state",
            "focus trap",
            "Escape",
            "trigger restoration",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "alert_dialog",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSAlert",
        design: "Clear warning hierarchy and safe default action",
        api: "gpui_macos::alert_dialog",
        capabilities: &["Destructive confirmation", "cancellation", "keyboard focus"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "popover",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSPopover",
        design: "Compact anchored content well with rounded edges",
        api: "gpui_macos::popover",
        capabilities: &[
            "Anchor placement",
            "outside dismiss",
            "focus",
            "collision handling",
            "native glass foreground panel",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "tooltip",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSView toolTip",
        design: "Small supplementary label with delayed appearance",
        api: "gpui_macos::tooltip",
        capabilities: &["Hover and focus", "delay", "placement", "accessible label"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "hover_card",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS contextual preview popover",
        design: "Compact preview content with secondary metadata",
        api: "gpui_macos::hover_card",
        capabilities: &[
            "Delayed hover",
            "focus retention",
            "dismiss and placement",
            "native glass preview panel",
            "opaque fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "toast",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS transient in-app status",
        design: "Compact status row with symbol, text and optional undo",
        api: "gpui_macos::toast / gpui_kit::base::ToastManager",
        capabilities: &[
            "Independent messages",
            "timeout",
            "pause on hover/focus",
            "actions",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "notification",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS in-app notification banner",
        design: "Compact semantic notification with close and optional action",
        api: "gpui_macos::kit::notification::Notification",
        capabilities: &["Success/warning/error", "stacking", "dismissal", "timeout"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "icon",
        rendering: RenderingKind::AppKitBacked,
        counterpart: "NSImage system symbols",
        design: "Host SF geometry, optical weight and inherited template color",
        api: "gpui_macos::icon",
        capabilities: &["SF lookup", "tint", "cached masks", "full Lucide fallback"],
        native_integrations: &[NativeIntegration::SfSymbols],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "avatar",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS account/contact image",
        design: "Small circular contact image with initials fallback",
        api: "gpui_macos::avatar",
        capabilities: &["Image content", "initials", "loading fallback", "sizing"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "text_view",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTextView document presentation",
        design: "Readable document typography and restrained link styling",
        api: "gpui_macos::markdown / gpui_macos::html",
        capabilities: &["HTML and Markdown", "selection", "links", "text layout"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "label",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS primary and secondary labels",
        design: "13-point primary text and semantic secondary help",
        api: "gpui_macos::kit::label::Label",
        capabilities: &[
            "Highlights",
            "masking",
            "secondary text",
            "content composition",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "panel",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS grouped content section",
        design: "Quiet section heading and natural-height content",
        api: "gpui_macos::panel",
        capabilities: &[
            "Heading",
            "content",
            "configurable layout",
            "grouped controls",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "group_box",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSBox",
        design: "Quiet filled or outlined group with an overlapping title",
        api: "gpui_macos::kit::group_box::GroupBox",
        capabilities: &[
            "Title",
            "footer",
            "filled/outlined variants",
            "arbitrary content",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "description_list",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Finder Get Info property rows",
        design: "Aligned secondary labels and primary values",
        api: "gpui_macos::interpretation::description_list / gpui_macos::kit::description_list::DescriptionList",
        capabilities: &[
            "Horizontal/vertical",
            "columns",
            "spans",
            "structured values",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "list",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTableView single-column list",
        design: "Flat selectable rows with SF file symbols",
        api: "gpui_macos::kit::list::ListItem",
        capabilities: &["Selection", "pointer callbacks", "custom row content"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "virtual_list",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTableView virtualized source rows",
        design: "Compact lazy rows measured at the current zoom",
        api: "gpui_macos::virtual_list",
        capabilities: &[
            "Virtualization",
            "scroll handle",
            "visible range",
            "retained selection",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "scrollbar",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSScroller",
        design: "Thin overlay scroller with native density",
        api: "gpui_macos::scrollbar",
        capabilities: &["Horizontal and vertical", "scroll position", "hover/drag"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "scrollable",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSScrollView",
        design: "Bounded document pane with ordinary row separators",
        api: "gpui_kit::StatefulInteractiveElement",
        capabilities: &[
            "Overflow",
            "wheel and trackpad",
            "scroll position",
            "clipping",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "table",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTableView",
        design: "Compact header labels and alternating document rows",
        api: "gpui_macos::table",
        capabilities: &["Headers", "cells", "selection", "custom rendering"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "data_table",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSTableView sortable data view",
        design: "Compact sortable headers, document metadata and row stripes",
        api: "gpui_macos::kit::table::DataTable",
        capabilities: &[
            "Virtual rows",
            "selection",
            "column sorting",
            "resizing",
            "delegate callbacks",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "tree",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSOutlineView",
        design: "Indented disclosure rows with compact selection",
        api: "gpui_macos::tree",
        capabilities: &[
            "Expand/collapse",
            "hierarchy",
            "selection",
            "keyboard traversal",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "resizable",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSSplitView",
        design: "Thin divider separating ordinary desktop panes",
        api: "gpui_macos::resizable",
        capabilities: &[
            "Drag resize",
            "minimum sizes",
            "orientation",
            "retained state",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "dock",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Xcode-style docked workbench",
        design: "Pane titles, compact tabs and resizable tool/content regions",
        api: "gpui_macos::dock_area",
        capabilities: &[
            "Dock layout",
            "tabs",
            "split panes",
            "serialization",
            "panel state",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "separator",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSBox separator type",
        design: "One-pixel semantic divider between related content",
        api: "gpui_macos::separator",
        capabilities: &[
            "Horizontal and vertical",
            "semantic color",
            "zoom-safe hairline",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "keycap",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS keyboard shortcut annotation",
        design: "Compact system key names and unobtrusive key bezel",
        api: "gpui_macos::keycap",
        capabilities: &[
            "Modifier symbols",
            "key combinations",
            "readable command hints",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "badge",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS compact status annotation",
        design: "Small semantic capsule with system label density",
        api: "gpui_macos::badge",
        capabilities: &["Status variants", "color", "text", "sizing"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "tag",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Finder-style document tag",
        design: "Compact semantic document label with restrained outline",
        api: "gpui_macos::kit::tag::Tag",
        capabilities: &["Semantic variants", "outline", "custom content"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "alert",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS inline validation/status message",
        design: "Compact symbol, explanation and semantic accent",
        api: "gpui_macos::alert",
        capabilities: &["Status variants", "inline content", "custom actions"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "empty_state",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS empty document/library pane",
        design: "Centered SF symbol, short explanation and optional command",
        api: "gpui_macos::empty_state",
        capabilities: &["Empty content", "primary action", "layout customization"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "progress",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSProgressIndicator bar style",
        design: "Thin activation-aware track and progress fill",
        api: "gpui_macos::progress",
        capabilities: &["Controlled progress", "bounds", "semantic status"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "spinner",
        rendering: RenderingKind::AppKitBacked,
        counterpart: "NSImage progress.indicator with GPUI animation",
        design: "Host radial artwork, system tint and linear activity motion",
        api: "gpui_macos::native_spinner / kit::spinner::Spinner",
        capabilities: &[
            "Size",
            "color",
            "easing",
            "custom artwork",
            "reduced motion",
        ],
        native_integrations: &[NativeIntegration::SfSymbols],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "skeleton",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS loading content placeholders",
        design: "Quiet neutral blocks matching the eventual document rows",
        api: "gpui_macos::kit::skeleton::Skeleton",
        capabilities: &["Shapes", "primary/secondary placeholders", "reduced motion"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "shimmer",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS subtle loading text",
        design: "Secondary text with restrained animated emphasis",
        api: "gpui_macos::kit::shimmer::ShimmerText",
        capabilities: &["Text content", "tint", "animation", "reduced motion"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "rating",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSLevelIndicator rating style",
        design: "Compact SF star row with a retained score",
        api: "gpui_macos::kit::rating::Rating",
        capabilities: &[
            "Editable rating",
            "read-only rating",
            "disabled",
            "size and color",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "carousel",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Quick Look document preview navigation",
        design: "Document thumbnails, filename metadata and compact previous/next controls",
        api: "gpui_macos::interpretation::carousel / gpui_macos::kit::carousel::*",
        capabilities: &[
            "Retained selection",
            "arrows",
            "pagination",
            "keyboard",
            "reduced motion",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "image",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSImageView",
        design: "Actual document illustrations in bounded image previews",
        api: "gpui_kit::img",
        capabilities: &[
            "Asset/local/remote sources",
            "contain/cover",
            "loading and fallback",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "diff",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Xcode-style source comparison",
        design: "Monospaced code panes and compact comparison-mode command",
        api: "gpui_macos::kit::diff::Diff",
        capabilities: &[
            "Unified/split modes",
            "file hunks",
            "synchronized scroll",
            "source styles",
        ],
        native_integrations: &[],
        optional_features: &[FeatureRequirement::TreeSitterLanguages],
    },
    ComponentInterpretation {
        id: "message",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Messages-style conversation row",
        design: "Compact contact label, aligned bubble and delivery metadata",
        api: "gpui_macos::interpretation::message / gpui_macos::kit::message::*",
        capabilities: &[
            "Avatar/header/content/footer slots",
            "alignment",
            "accessible identity",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "bubble",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Messages-style conversation surface",
        design: "Neutral incoming and accent outgoing bubbles with compact typography",
        api: "gpui_macos::interpretation::bubble_content / gpui_macos::kit::bubble::*",
        capabilities: &[
            "Variants",
            "alignment",
            "rich content",
            "reactions",
            "grouped bubbles",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "marker",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Conversation date and activity separator",
        design: "Centered secondary timestamp or compact activity label",
        api: "gpui_macos::kit::marker::Marker",
        capabilities: &["Text markers", "loading marker", "arbitrary content"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "attachment",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Mail-style attached document row",
        design: "SF file symbol, filename, size and trailing retry/remove commands",
        api: "gpui_macos::interpretation::attachment / gpui_macos::kit::attachment::*",
        capabilities: &[
            "Pending/success/error",
            "media",
            "retry",
            "remove",
            "action slots",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "message_scroller",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Messages-style retained transcript",
        design: "Bounded virtual conversation pane that follows new messages",
        api: "gpui_macos::kit::message_scroller::MessageScroller",
        capabilities: &[
            "Virtualization",
            "append/prepend",
            "jump to bottom",
            "retained scroll",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "speech",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS dictation command and level feedback",
        design: "Compact microphone command and waveform in a transcript pane",
        api: "gpui_macos::kit::speech::{SpeechButton, SpeechWaveform}",
        capabilities: &[
            "System recognition",
            "partial/final text",
            "errors",
            "supported/disabled state",
        ],
        native_integrations: &[NativeIntegration::SpeechRecognizer],
        optional_features: &[FeatureRequirement::Speech],
    },
    ComponentInterpretation {
        id: "title_bar",
        rendering: RenderingKind::GpuiReplica,
        counterpart: "NSWindow titlebar and traffic-light controls",
        design: "Clear current-view title with space for real system window controls",
        api: "gpui_macos::title_bar / gpui_macos::native_window_options / gpui_macos::window_options",
        capabilities: &[
            "Drag region",
            "native traffic lights",
            "custom title content",
        ],
        native_integrations: &[NativeIntegration::WindowChrome],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "toolbar",
        rendering: RenderingKind::AppKitBacked,
        counterpart: "NSToolbar",
        design: "Unified AppKit commands and search, with a composable GPUI fallback",
        api: "gpui_macos::toolbar / gpui_macos::toolbar_group / gpui_macos::toolbar_button / gpui_macos::native_toolbar_group / gpui_macos::native::install_native_toolbar",
        capabilities: &[
            "Native commands",
            "native search",
            "enable state",
            "title updates",
            "canvas groups",
        ],
        native_integrations: &[
            NativeIntegration::UnifiedToolbar,
            NativeIntegration::SfSymbols,
        ],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "status_bar",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Finder-style document status row",
        design: "Small secondary status labels at the bottom of the content",
        api: "gpui_macos::status_bar",
        capabilities: &["Leading/trailing content", "selection count", "saved state"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "settings",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "System Settings grouped preferences",
        design: "Natural-height gray groups and aligned labeled control rows",
        api: "gpui_macos::settings_group / gpui_macos::settings_row",
        capabilities: &[
            "Editable values",
            "help text",
            "live updates",
            "save actions",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "inspector",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "Finder Get Info/Xcode inspector pane",
        design: "Narrow property pane with secondary labels and editable values",
        api: "gpui_macos::inspector",
        capabilities: &["Property rows", "editable content", "debug inspection API"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "inset_list",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "System Settings inset navigation rows",
        design: "Quiet inset source rows with trailing disclosure",
        api: "gpui_macos::inset_list / gpui_macos::inset_list_row",
        capabilities: &[
            "Selectable rows",
            "custom controls",
            "disclosure",
            "callbacks",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "material",
        rendering: RenderingKind::AppKitBacked,
        counterpart: "NSGlassEffectView navigation material",
        design: "Opt-in native system material plus deterministic canvas fills",
        api: "gpui_macos::material / gpui_macos::native_material / gpui_macos::native::open_glass_panel",
        capabilities: &[
            "Native underlay",
            "separate system glass foreground panels",
            "semantic roles",
            "tint",
            "clipping",
            "accessibility fallback",
        ],
        native_integrations: &[NativeIntegration::GlassMaterial],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "plot",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS analytical chart canvas",
        design: "Ordinary content pane with readable axes and precise data labels",
        api: "gpui_macos::plot",
        capabilities: &["Scales", "axes", "points", "tooltip", "custom marks"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "line_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS time-series chart",
        design: "Thin series, restrained grid and compact legend",
        api: "gpui_macos::LineChart",
        capabilities: &["Series", "scales", "axes", "points", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "bar_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS comparison chart",
        design: "Clear category bars and readable baseline labels",
        api: "gpui_macos::BarChart",
        capabilities: &["Categories", "multiple series", "axes", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "area_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS cumulative/time-series chart",
        design: "Translucent series fill and restrained plot chrome",
        api: "gpui_macos::AreaChart",
        capabilities: &["Multiple series", "scales", "axes", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "pie_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS proportion chart",
        design: "Color-coded sectors, readable legend and proportion labels",
        api: "gpui_macos::PieChart",
        capabilities: &["Values", "sectors", "colors", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "radar_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS multi-attribute chart",
        design: "Restrained radial grid and labeled attribute axes",
        api: "gpui_macos::RadarChart",
        capabilities: &["Multiple series", "attributes", "scales", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "candlestick_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS financial chart",
        design: "Precise open/high/low/close marks with readable axes",
        api: "gpui_macos::CandlestickChart",
        capabilities: &[
            "OHLC values",
            "time scale",
            "rising/falling colors",
            "tooltip",
        ],
        native_integrations: &[],
        optional_features: &[],
    },
    ComponentInterpretation {
        id: "sankey_chart",
        rendering: RenderingKind::MacInterpretation,
        counterpart: "macOS flow-analysis chart",
        design: "Readable node labels and restrained weighted links",
        api: "gpui_macos::SankeyChart",
        capabilities: &["Nodes", "links", "flow widths", "colors", "tooltip"],
        native_integrations: &[],
        optional_features: &[],
    },
];
pub const MODULE_COVERAGE: &[ModuleCoverage] = &[
    ModuleCoverage {
        module: "global_state",
        pages: &["root"],
        role: "Window-level overlay and interaction coordination",
    },
    ModuleCoverage {
        module: "accordion",
        pages: &["accordion"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "alert",
        pages: &["alert"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "attachment",
        pages: &["attachment"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "avatar",
        pages: &["avatar"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "badge",
        pages: &["badge"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "breadcrumb",
        pages: &["breadcrumb"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "bubble",
        pages: &["bubble"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "button",
        pages: &["button"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "carousel",
        pages: &["carousel"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "chart",
        pages: &[
            "line_chart",
            "bar_chart",
            "area_chart",
            "pie_chart",
            "radar_chart",
            "candlestick_chart",
            "sankey_chart",
        ],
        role: "All seven public chart families",
    },
    ModuleCoverage {
        module: "checkbox",
        pages: &["checkbox"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "clipboard",
        pages: &["clipboard"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "collapsible",
        pages: &["collapsible"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "color_picker",
        pages: &["color_picker"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "combobox",
        pages: &["combobox"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "command",
        pages: &["command"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "description_list",
        pages: &["description_list"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "dialog",
        pages: &["dialog"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "diff",
        pages: &["diff"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "dock",
        pages: &["dock"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "empty",
        pages: &["empty_state"],
        role: "Catalog naming alias",
    },
    ModuleCoverage {
        module: "form",
        pages: &["form"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "group_box",
        pages: &["group_box"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "highlighter",
        pages: &["editor", "diff"],
        role: "Diagnostics, syntax registries and optional language grammars",
    },
    ModuleCoverage {
        module: "history",
        pages: &["input", "textarea", "editor"],
        role: "Undo/redo state used by the editor primitives",
    },
    ModuleCoverage {
        module: "hover_card",
        pages: &["hover_card"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "input",
        pages: &["input"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "kbd",
        pages: &["keycap"],
        role: "Catalog naming alias",
    },
    ModuleCoverage {
        module: "label",
        pages: &["label"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "link",
        pages: &["link"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "list",
        pages: &["list"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "marker",
        pages: &["marker"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "menu",
        pages: &["menu"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "message",
        pages: &["message"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "message_scroller",
        pages: &["message_scroller"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "native_menu",
        pages: &["menu"],
        role: "OS-native popup menus alongside the GPUI menu replica",
    },
    ModuleCoverage {
        module: "notification",
        pages: &["notification"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "pagination",
        pages: &["pagination"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "plot",
        pages: &["plot"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "popover",
        pages: &["popover"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "progress",
        pages: &["progress"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "questionnaire",
        pages: &["questionnaire"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "radio",
        pages: &["radio"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "rating",
        pages: &["rating"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "resizable",
        pages: &["resizable"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "scroll",
        pages: &["scrollable", "scrollbar"],
        role: "Scroll containers, overlay scrollers and handle helpers",
    },
    ModuleCoverage {
        module: "searchable_list",
        pages: &["command", "combobox", "list"],
        role: "Search delegates, grouped values, filtering and keyboard selection",
    },
    ModuleCoverage {
        module: "select",
        pages: &["select"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "separator",
        pages: &["separator"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "setting",
        pages: &["settings"],
        role: "Grouped preference composition",
    },
    ModuleCoverage {
        module: "sheet",
        pages: &["sheet"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "shimmer",
        pages: &["shimmer"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "sidebar",
        pages: &["sidebar"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "skeleton",
        pages: &["skeleton"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "slider",
        pages: &["slider"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "speech",
        pages: &["speech"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "spinner",
        pages: &["spinner"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "status_bar",
        pages: &["status_bar"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "stepper",
        pages: &["stepper"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "switch",
        pages: &["switch"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "tab",
        pages: &["tabs"],
        role: "Tab controls and retained panel content",
    },
    ModuleCoverage {
        module: "table",
        pages: &["table"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "tag",
        pages: &["tag"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "text",
        pages: &["text_view", "label"],
        role: "Document text, labels and selectable presentation",
    },
    ModuleCoverage {
        module: "theme",
        pages: &["theme"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "toolbar",
        pages: &["toolbar"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "tooltip",
        pages: &["tooltip"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
    ModuleCoverage {
        module: "tree",
        pages: &["tree"],
        role: "Full public Kit module and its macOS catalog presentation",
    },
];

/// Look up the explicit contract for a catalog identifier.
pub fn interpretation(id: &str) -> Option<&'static ComponentInterpretation> {
    INTERPRETATIONS.iter().find(|entry| entry.id == id)
}

use crate::{ActiveTheme as _, kit};
use gpui_kit::{
    App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StyleRefinement, Styled as _, div, rems,
};
use kit::Sizable as _;

/// A setup-assistant form retaining the complete Kit questionnaire state API.
pub fn questionnaire(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> kit::questionnaire::Questionnaire {
    kit::questionnaire::Questionnaire::new(state)
        .with_size(kit::Size::Small)
        .font_family(cx.macos().font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .gap(rems(0.875))
}

/// A grouped macOS preference form with the complete Kit Field/footer API.
pub fn form(window: &gpui_kit::Window, cx: &App) -> kit::form::Form {
    kit::form::Form::horizontal()
        .label_width(rems(8.5).to_pixels(window.rem_size()))
        .label_text_size(rems(crate::TextStyle::Body.token().size / 16.))
        .font_family(cx.macos().font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .bg(cx.macos().normal_fill())
        .rounded(rems(0.625))
        .p(rems(1.))
}

/// Finder-style aligned property rows rather than a bordered web data grid.
pub fn description_list() -> kit::description_list::DescriptionList {
    kit::description_list::DescriptionList::horizontal()
        .columns(1)
        .label_width(rems(8.5))
        .bordered(false)
}

/// A quiet macOS radio/checkbox row, with Kit validation, focus and shortcuts.
/// Custom indicator and shortcut builders remain available on the return value.
pub fn questionnaire_choice(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    item: impl Into<SharedString>,
    value: impl Into<SharedString>,
    cx: &App,
) -> kit::questionnaire::QuestionnaireChoice {
    let item = item.into();
    let value = value.into();
    let selector = format!("mac-questionnaire-{}-{item}-{value}", state.entity_id());
    let multiple = state
        .read(cx)
        .item_state(&item)
        .is_some_and(|item| item.is_multiple());
    let content = state
        .read(cx)
        .choice_definition(&item, &value)
        .map(|definition| {
            (
                definition.accessibility_label().clone(),
                definition.description().cloned(),
            )
        });
    // Kit fades a whole disabled row to 50%; the macOS row instead uses the
    // ControlLabel and Check disabled parts (spec §6.2.6, §6.2.10).
    let disabled = state
        .read(cx)
        .choice_state(&item, &value)
        .is_some_and(|choice| choice.is_disabled());
    let label_color = crate::state::paint(
        crate::state::PaintRole::ControlLabel,
        crate::state::Part::Label,
        crate::state::States::active().disabled(disabled),
        cx.macos(),
    );
    let description_color = if disabled {
        label_color
    } else {
        cx.macos().secondary
    };
    kit::questionnaire::QuestionnaireChoice::new(state, item, value)
        // Overrides Kit's disabled row opacity; enabled rows are already opaque.
        .opacity(1.)
        // Kit uses this scale for a 16-point indicator on a 24-point answer
        // line. The named label below keeps the native 13-point typography.
        .with_size(kit::Size::Size(gpui_kit::px(16.)))
        .min_h(rems(1.5))
        .px(rems(0.))
        .py(rems(0.))
        .gap(rems(0.5))
        .rounded(rems(0.25))
        .border_0()
        .bg(cx.macos().background.opacity(0.))
        .content_style(
            StyleRefinement::default()
                .native_text_style(crate::TextStyle::Body, cx.macos())
                .gap(rems(0.)),
        )
        .children(content.map(|(label, description)| {
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .native_text_style(crate::TextStyle::Body, cx.macos())
                        .line_height(rems(1.5))
                        .text_color(label_color)
                        .child(label),
                )
                .children(description.map(|description| {
                    div()
                        .native_text_style(crate::TextStyle::Subheadline, cx.macos())
                        .line_height(rems(1.))
                        .text_color(description_color)
                        .child(description)
                }))
        }))
        .render_indicator(move |choice, window, cx| {
            // CheckOn / CheckOff, spec §6.2.6.
            let (well, mark) = crate::controls::check_paint(
                cx.macos(),
                crate::state::States::in_window(window).disabled(choice.is_disabled()),
                choice.is_selected(),
            );
            div()
                .debug_selector({
                    let selector = selector.clone();
                    move || selector.clone()
                })
                .size(rems(1.))
                .flex_shrink_0()
                .rounded(rems(if multiple { 0.25 } else { 0.5 }))
                .bg(well)
                .flex()
                .items_center()
                .justify_center()
                .children(choice.is_selected().then(|| {
                    if multiple {
                        crate::icon(crate::IconName::Check)
                            .size(rems(0.875))
                            .text_color(mark)
                            .into_any_element()
                    } else {
                        div()
                            .size(rems(0.25))
                            .rounded_full()
                            .bg(mark)
                            .into_any_element()
                    }
                }))
                .into_any_element()
        })
}

/// Setup navigation uses the native command density and a trailing action row.
pub fn questionnaire_actions(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> kit::questionnaire::QuestionnaireActions {
    kit::questionnaire::QuestionnaireActions::new(state)
        .justify_end()
        .gap(rems(0.5))
        .pt(rems(0.75))
        .border_t_1()
        .border_color(cx.macos().divider())
}

/// The Back command, present when Kit can navigate to an earlier question.
pub fn questionnaire_previous(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> Option<crate::Button> {
    state
        .read(cx)
        .navigation_state()
        .is_previous_visible()
        .then(|| {
            questionnaire_action(
                state,
                "Previous",
                "Back",
                crate::ButtonVariant::Secondary,
                kit::questionnaire::QuestionnaireState::go_previous,
                cx,
            )
        })
}

/// The Continue command. Kit retains validation and invalid-answer focus.
pub fn questionnaire_next(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> Option<crate::Button> {
    state
        .read(cx)
        .navigation_state()
        .is_next_visible()
        .then(|| {
            questionnaire_action(
                state,
                "Next",
                "Continue",
                crate::ButtonVariant::Primary,
                kit::questionnaire::QuestionnaireState::go_next,
                cx,
            )
        })
}

/// The Done command, present on the final enabled question.
pub fn questionnaire_submit(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> Option<crate::Button> {
    state
        .read(cx)
        .navigation_state()
        .is_submit_visible()
        .then(|| {
            questionnaire_action(
                state,
                "Submit",
                "Done",
                crate::ButtonVariant::Primary,
                kit::questionnaire::QuestionnaireState::submit,
                cx,
            )
        })
}

/// Skip an optional question using Kit's original clear-and-advance behavior.
pub fn questionnaire_skip(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    cx: &App,
) -> Option<crate::Button> {
    state
        .read(cx)
        .navigation_state()
        .is_skip_visible()
        .then(|| {
            questionnaire_action(
                state,
                "Skip",
                "Skip",
                crate::ButtonVariant::Secondary,
                kit::questionnaire::QuestionnaireState::skip_current,
                cx,
            )
        })
}

fn questionnaire_action(
    state: &Entity<kit::questionnaire::QuestionnaireState>,
    action: &'static str,
    label: &'static str,
    variant: crate::ButtonVariant,
    handler: fn(
        &mut kit::questionnaire::QuestionnaireState,
        &mut gpui_kit::Window,
        &mut gpui_kit::Context<kit::questionnaire::QuestionnaireState>,
    ) -> bool,
    cx: &App,
) -> crate::Button {
    let selector = format!("questionnaire-{}-{action}", state.entity_id());
    let target = state.clone();
    crate::button(ElementId::Name(selector.clone().into()), label, variant, cx)
        .debug_selector(move || selector.clone())
        .h(rems(1.5))
        .on_click(move |_, window, cx| {
            target.update(cx, |state, cx| {
                handler(state, window, cx);
            });
        })
}

/// A compact Messages-style row. All named Kit slots remain configurable.
pub fn message(alignment: kit::message::MessageAlignment, cx: &App) -> kit::message::Message {
    kit::message::Message::new()
        .alignment(alignment)
        .font_family(cx.macos().font.clone())
        .gap(rems(0.25))
        .with_stack_style(StyleRefinement::default().gap(rems(0.25)))
}

/// The visible conversation surface; the surrounding Bubble keeps its full API.
pub fn bubble_content(
    alignment: kit::message::MessageAlignment,
    cx: &App,
) -> kit::bubble::BubbleContent {
    let t = cx.macos();
    let outgoing = alignment == kit::message::MessageAlignment::End;
    kit::bubble::BubbleContent::new()
        .max_w(rems(31.))
        .rounded(rems(1.))
        .border_0()
        .px(rems(0.75))
        .py(rems(0.5))
        .native_text_style(crate::TextStyle::Body, cx.macos())
        .bg(if outgoing { t.accent } else { t.normal_fill() })
        .text_color(if outgoing { t.on_accent } else { t.foreground })
}

/// A compact Mail-style file row with Kit's complete lifecycle and action API.
pub fn attachment(cx: &App) -> kit::attachment::Attachment {
    kit::attachment::Attachment::new()
        .with_size(kit::Size::Small)
        .axis(gpui_kit::Axis::Horizontal)
        .rounded(rems(0.5))
        .border_0()
        .bg(cx.macos().normal_fill())
        .native_text_style(crate::TextStyle::Body, cx.macos())
}

/// A bounded document-preview navigator, retaining Kit's full carousel API.
pub fn carousel(
    id: impl Into<ElementId>,
    state: &Entity<kit::carousel::CarouselState>,
    cx: &App,
) -> kit::carousel::Carousel {
    kit::carousel::Carousel::new(id, state)
        .font_family(cx.macos().font.clone())
        .native_text_style(crate::TextStyle::Body, cx.macos())
}

/// An inline previous-document command for the macOS carousel interpretation.
pub fn carousel_previous(state: &Entity<kit::carousel::CarouselState>, cx: &App) -> crate::Button {
    let target = state.clone();
    crate::toolbar_button(
        ("mac-carousel-previous", state.entity_id()),
        crate::IconName::ChevronLeft,
        "Previous document",
        cx,
    )
    .disabled(!state.read(cx).has_previous())
    .on_click(move |_, _, cx| {
        target.update(cx, |state, cx| {
            state.select_previous(cx);
        });
    })
}

/// An inline next-document command for the macOS carousel interpretation.
pub fn carousel_next(state: &Entity<kit::carousel::CarouselState>, cx: &App) -> crate::Button {
    let target = state.clone();
    crate::toolbar_button(
        ("mac-carousel-next", state.entity_id()),
        crate::IconName::ChevronRight,
        "Next document",
        cx,
    )
    .disabled(!state.read(cx).has_next())
    .on_click(move |_, _, cx| {
        target.update(cx, |state, cx| {
            state.select_next(cx);
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{COMPONENTS, component};
    use gpui_kit::test::TestQueryExt as _;
    use gpui_kit::{AppContext as _, Context, Render, TestAppContext, Window, px};
    use std::collections::HashSet;

    // Compilation proves the untrimmed public facade still exposes every
    // pinned component module, including support modules without doc pages.
    #[allow(unused_imports)]
    use crate::kit::{
        accordion, alert, attachment, avatar, badge, breadcrumb, bubble, button, carousel, chart,
        checkbox, clipboard, collapsible, color_picker, combobox, command, description_list,
        dialog, diff, dock, empty, form, global_state, group_box, highlighter, history, hover_card,
        input, kbd, label, link, list, marker, menu, message, message_scroller, native_menu,
        notification, pagination, plot, popover, progress, questionnaire, radio, rating, resizable,
        scroll, searchable_list, select, separator, setting, sheet, shimmer, sidebar, skeleton,
        slider, speech, spinner, status_bar, stepper, switch, tab, table, tag, text, theme,
        toolbar, tooltip, tree,
    };

    fn inventory() -> serde_json::Value {
        serde_json::from_str(include_str!("../docs/upstream-inventory.json")).unwrap()
    }

    fn strings(value: &serde_json::Value) -> Vec<&str> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap())
            .collect()
    }

    struct OptionalQuestion {
        state: Entity<kit::questionnaire::QuestionnaireState>,
    }

    impl Render for OptionalQuestion {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            questionnaire(&self.state, cx).child(
                questionnaire_actions(&self.state, cx)
                    .children(questionnaire_skip(&self.state, cx)),
            )
        }
    }

    #[gpui_kit::test]
    fn disabled_choices_use_check_disabled_parts_instead_of_row_opacity(cx: &mut TestAppContext) {
        use kit::questionnaire::{
            QuestionnaireChoiceDefinition, QuestionnaireItemDefinition, QuestionnaireState,
        };
        struct DisabledChoice {
            state: Entity<QuestionnaireState>,
        }
        impl Render for DisabledChoice {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                // The underline paints in the label ink.
                questionnaire_choice(&self.state, "plan", "team", cx).underline()
            }
        }
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| DisabledChoice {
            state: cx.new(|cx| {
                QuestionnaireState::new(
                    vec![
                        QuestionnaireItemDefinition::new("plan", "Plan").with_choices([
                            QuestionnaireChoiceDefinition::new("team", "Team")
                                .with_default_selected(true),
                            QuestionnaireChoiceDefinition::new("solo", "Solo"),
                        ]),
                    ],
                    cx,
                )
                .unwrap()
            }),
        });
        // Selected while enabled, then disabled: CheckOn disabled.
        cx.update(|_, cx| {
            view.read(cx).state.clone().update(cx, |state, cx| {
                state.set_choice_disabled("plan", "team", true, cx).unwrap()
            })
        });
        let id = cx.update(|_, cx| view.read(cx).state.entity_id());
        let selector: &'static str = format!("mac-questionnaire-{id}-plan-team").leak();
        for active in [true, false] {
            if active {
                cx.update(|window, _| window.activate_window());
                cx.run_until_parked();
            } else {
                cx.deactivate_window();
            }
            cx.update(|window, cx| {
                view.update(cx, |_, cx| cx.notify());
                window.draw(cx).clear(cx);
            });
            let indicator = cx.debug_bounds(selector).unwrap();
            cx.update(|window, cx| {
                let choice = view
                    .read(cx)
                    .state
                    .read(cx)
                    .choice_state("plan", "team")
                    .unwrap();
                assert!(choice.is_selected() && choice.is_disabled());
                let states = crate::state::States::in_window(window).disabled(true);
                let (well, mark) = crate::controls::check_paint(cx.macos(), states, true);
                assert_eq!(
                    well,
                    crate::state::paint(
                        crate::state::PaintRole::CheckOn,
                        crate::state::Part::Fill,
                        states,
                        cx.macos()
                    )
                );
                let scale = window.scale_factor();
                let quads = window.painted_quads();
                let fill_at = |bounds: gpui_kit::Bounds<gpui_kit::Pixels>| {
                    quads
                        .iter()
                        .find(|quad| quad.bounds == bounds.scale(scale))
                        .and_then(|quad| quad.background.as_solid())
                };
                // The CheckOn disabled well, not the row's 50% opacity over it.
                assert_eq!(fill_at(indicator), Some(well), "well, active {active}");
                let dot = quads
                    .iter()
                    .find(|quad| {
                        quad.bounds.center() == indicator.center().scale(scale)
                            && quad.bounds.size == gpui_kit::size(px(4.), px(4.)).scale(scale)
                    })
                    .and_then(|quad| quad.background.as_solid());
                assert_eq!(dot, Some(mark), "mark, active {active}");
                let label = window
                    .painted_underlines()
                    .into_iter()
                    .map(|line| line.color)
                    .collect::<Vec<_>>();
                assert!(!label.is_empty());
                assert!(
                    label
                        .iter()
                        .all(|color| *color == cx.macos().tertiary_label()),
                    "disabled label uses tertiary ink, active {active}"
                );
            });
        }
    }

    #[gpui_kit::test]
    fn native_skip_command_clears_the_answer_and_submits_the_optional_last_question(
        cx: &mut TestAppContext,
    ) {
        use kit::questionnaire::{
            QuestionnaireChoiceDefinition, QuestionnaireItemDefinition, QuestionnaireItemStatus,
            QuestionnaireState,
        };

        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|_, cx| OptionalQuestion {
            state: cx.new(|cx| {
                QuestionnaireState::new(
                    vec![
                        QuestionnaireItemDefinition::new("preference", "Optional preference")
                            .with_choices([QuestionnaireChoiceDefinition::new("yes", "Yes")
                                .with_default_selected(true)]),
                    ],
                    cx,
                )
                .unwrap()
            }),
        });
        let point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let state = view.read(cx).state.read(cx);
            assert!(!state.answer("preference").unwrap().is_empty());
            let skip = window.find_by_label("Skip");
            assert_eq!(skip.role(), Some(gpui_kit::Role::Button));
            assert_eq!(skip.bounds().size.height, px(24.));
            skip.bounds().center()
        });
        cx.simulate_click(point, Default::default());
        cx.update(|_, cx| {
            let state = view.read(cx).state.read(cx);
            assert!(state.answer("preference").unwrap().is_empty());
            assert_eq!(
                state.item_state("preference").unwrap().status(),
                QuestionnaireItemStatus::Skipped
            );
            assert!(state.is_complete());
        });
    }

    #[test]
    fn every_catalog_page_has_one_explicit_and_useful_contract() {
        let ids: HashSet<_> = INTERPRETATIONS.iter().map(|entry| entry.id).collect();
        assert_eq!(ids.len(), INTERPRETATIONS.len(), "contracts are unique");
        let catalog: HashSet<_> = COMPONENTS.iter().map(|entry| entry.id).collect();
        assert_eq!(
            ids, catalog,
            "new pages cannot bypass interpretation review"
        );
        for entry in INTERPRETATIONS {
            assert!(
                !entry.counterpart.is_empty(),
                "{}: name its native counterpart or composition",
                entry.id
            );
            assert!(
                !entry.design.is_empty() && !entry.api.is_empty(),
                "{}: publish the design and reusable API",
                entry.id
            );
            assert!(
                !entry.capabilities.is_empty()
                    && entry
                        .capabilities
                        .iter()
                        .all(|capability| !capability.is_empty())
            );
            if entry.rendering == RenderingKind::AppKitBacked {
                assert!(
                    !entry.native_integrations.is_empty(),
                    "{}: system-backed claims require an actual native integration",
                    entry.id
                );
            }
            assert_eq!(component(entry.id).unwrap().interpretation(), entry);
        }
    }

    #[test]
    fn all_pinned_documentation_topics_and_modules_are_represented() {
        let inventory = inventory();
        assert_eq!(inventory["revision"], UPSTREAM_REVISION);
        let documented: HashSet<_> = COMPONENTS
            .iter()
            .filter_map(|entry| entry.documentation)
            .collect();
        for topic in strings(&inventory["documentation_topics"]) {
            assert!(
                documented.contains(topic),
                "missing upstream topic: {topic}"
            );
        }
        let mapped: HashSet<_> = MODULE_COVERAGE.iter().map(|entry| entry.module).collect();
        assert_eq!(mapped.len(), MODULE_COVERAGE.len());
        let expected: HashSet<_> = strings(&inventory["component_modules"])
            .into_iter()
            .collect();
        assert_eq!(
            mapped, expected,
            "all public modules require a page or supporting role"
        );
        for entry in MODULE_COVERAGE {
            assert!(!entry.pages.is_empty() && !entry.role.is_empty());
            for page in entry.pages {
                assert!(
                    component(page).is_some(),
                    "{} maps to missing page {page}",
                    entry.module
                );
            }
        }
    }

    #[test]
    fn snapshot_matches_the_vendored_component_exports() {
        let source = include_str!("../docs/upstream-component-exports.rs");
        let vendored = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vendor/gpui-component/src/lib.rs");
        if vendored.exists() {
            assert_eq!(
                std::fs::read_to_string(vendored).unwrap(),
                source,
                "the packaged source fixture must match the actual checkout exports"
            );
        }
        let inventory = inventory();
        let modules: Vec<_> = source
            .lines()
            .filter_map(|line| {
                line.strip_prefix("pub mod ")
                    .and_then(|module| module.strip_suffix(';'))
            })
            .collect();
        assert_eq!(
            modules,
            strings(&inventory["component_modules"]),
            "audit exported modules when updating the pinned source"
        );
        let mut uses = Vec::new();
        let mut current = String::new();
        for line in source.lines() {
            if let Some(statement) = line.strip_prefix("pub use ") {
                current = statement.to_string();
            } else if !current.is_empty() {
                current.push(' ');
                current.push_str(line);
            } else {
                continue;
            }
            if let Some((statement, _)) = current.split_once(';') {
                uses.push(statement.split_whitespace().collect::<Vec<_>>().join(" "));
                current.clear();
            }
        }
        assert_eq!(
            uses,
            strings(&inventory["root_reexport_groups"]),
            "root reexports must remain complete as well as modules"
        );
    }

    #[test]
    fn interpretations_do_not_mislabel_gpui_controls_as_appkit_instances() {
        for id in [
            "button",
            "checkbox",
            "radio",
            "switch",
            "input",
            "secure_field",
            "slider",
            "select",
            "table",
            "tree",
        ] {
            assert_eq!(
                interpretation(id).unwrap().rendering,
                RenderingKind::GpuiReplica,
                "{id} is rendered by GPUI"
            );
        }
        for id in [
            "questionnaire",
            "message",
            "bubble",
            "carousel",
            "diff",
            "stepper",
            "dock",
            "sankey_chart",
        ] {
            assert_eq!(
                interpretation(id).unwrap().rendering,
                RenderingKind::MacInterpretation
            );
        }
        for id in [
            "select",
            "combobox",
            "menu",
            "date_picker",
            "color_picker",
            "popover",
            "hover_card",
        ] {
            let entry = interpretation(id).unwrap();
            assert!(
                entry
                    .native_integrations
                    .contains(&NativeIntegration::GlassMaterial),
                "{id} retains GPUI foreground controls while using native glass presentation"
            );
            assert_ne!(entry.rendering, RenderingKind::AppKitBacked);
        }
        assert_eq!(
            interpretation("toolbar").unwrap().native_integrations,
            &[
                NativeIntegration::UnifiedToolbar,
                NativeIntegration::SfSymbols
            ]
        );
        assert_eq!(
            interpretation("speech").unwrap().optional_features,
            &[FeatureRequirement::Speech]
        );
        assert_eq!(
            interpretation("editor").unwrap().optional_features,
            &[FeatureRequirement::TreeSitterLanguages]
        );
        assert!(interpretation("unknown-component").is_none());
    }
}
