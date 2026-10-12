//! The extended GPUI Kit controls share the same macOS theme and native window host.
use super::*;
use gpui_macos::interpretation as mac;
use gpui_macos::kit::{
    self, Disableable as _, FocusTrapElement as _, Sizable as _, WindowExt as _,
};
use kit::{
    attachment::{AttachmentContent, AttachmentDescription, AttachmentStatus, AttachmentTitle},
    breadcrumb::{Breadcrumb, BreadcrumbItem},
    bubble::Bubble,
    carousel::{
        CarouselContent, CarouselItem, CarouselPagination, CarouselPaginationItem, CarouselState,
    },
    clipboard::Clipboard,
    command::{Command, CommandGroup, CommandItem, CommandState},
    diff::{Diff, DiffFile, DiffMode, DiffState},
    form::Field,
    group_box::{GroupBox, GroupBoxVariants as _},
    input::{Editor, EditorState, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText},
    label::Label,
    marker::{Marker, MarkerContent},
    message::{Message, MessageAlignment, MessageContent, MessageFooter, MessageHeader},
    message_scroller::{MessageScroller, MessageScrollerState},
    notification::Notification,
    questionnaire::*,
    rating::Rating,
    shimmer::ShimmerText,
    skeleton::Skeleton,
    speech::{SpeechButton, SpeechState, SpeechWaveform},
    stepper::{Stepper, StepperItem},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    tag::Tag,
    time_field::{TimeField, TimeFieldState},
};

pub(super) struct Extended {
    editor: Entity<EditorState>,
    search: Entity<gpui_kit::base::input::InputState>,
    password: Entity<gpui_kit::base::input::InputState>,
    url: Entity<gpui_kit::base::input::InputState>,
    diff: Entity<DiffState>,
    command: Entity<CommandState>,
    carousel: Entity<CarouselState>,
    time: Entity<TimeFieldState>,
    questionnaire: Entity<QuestionnaireState>,
    speech: Entity<SpeechState>,
    messages: Entity<MessageScrollerState>,
    table: Entity<TableState<FilesTable>>,
    focus: FocusHandle,
    rating: usize,
    step: usize,
    selected_row: usize,
    toolbar_location: usize,
    transcript: String,
    question_result: String,
    command_result: String,
    attachment_visible: bool,
    attachment_failed: bool,
    message_count: usize,
    /// `--open` on the material page: present the glass panel once (set by the gallery).
    pub(super) launch_popup: bool,
}

impl Extended {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Gallery>) -> Self {
        let search = cx.new(|cx| {
            gpui_kit::base::input::InputState::new(window, cx).placeholder("Search documents")
        });
        let password = cx.new(|cx| {
            gpui_kit::base::input::InputState::new(window, cx)
                .placeholder("Password")
                .default_value("studio-workspace")
        });
        let url = cx.new(|cx| {
            gpui_kit::base::input::InputState::new(window, cx)
                .placeholder("workspace")
                .default_value("studio")
        });
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("rust")
                .line_number(true)
                .folding(true)
                .default_value("use gpui_macos::*;\n\n// Native controls, one macOS theme.\nfn build(cx: &App) -> impl IntoElement {\n    panel(\"Workspace\", cx)\n        .child(button(\"save\", \"Save\", ButtonVariant::Primary, cx))\n}\n")
        });
        let files = DiffFile::parse("diff --git a/preferences.rs b/preferences.rs\n--- a/preferences.rs\n+++ b/preferences.rs\n@@ -1,4 +1,5 @@\n fn preferences() {\n-    set_theme(Theme::dark());\n+    follow_system_appearance();\n+    set_accent(SystemAccent::Blue);\n     save();\n }\n").expect("valid gallery diff");
        let diff = cx.new(|cx| DiffState::new(files, cx));
        let command = cx.new(|cx| CommandState::new(window, cx));
        let carousel = cx.new(|_| CarouselState::new(3));
        let time = cx.new(|cx| TimeFieldState::new(window, cx));
        time.update(cx, |state, cx| {
            state.set_time(
                chrono::NaiveTime::from_hms_opt(9, 41, 0).unwrap(),
                window,
                cx,
            )
        });
        let questionnaire = cx.new(|cx| {
            QuestionnaireState::new(
                vec![
                    QuestionnaireItemDefinition::new(
                        "appearance",
                        "Choose your preferred appearance",
                    )
                    .with_required(true)
                    .with_choices([
                        QuestionnaireChoiceDefinition::new("system", "Follow System")
                            .with_default_selected(true),
                        QuestionnaireChoiceDefinition::new("light", "Light"),
                        QuestionnaireChoiceDefinition::new("dark", "Dark"),
                    ]),
                    QuestionnaireItemDefinition::new(
                        "density",
                        "How much space should controls use?",
                    )
                    .with_choices([
                        QuestionnaireChoiceDefinition::new("regular", "Regular"),
                        QuestionnaireChoiceDefinition::new("compact", "Compact"),
                    ]),
                ],
                cx,
            )
            .expect("valid gallery questions")
        });
        cx.subscribe(&questionnaire, |this, _, event: &QuestionnaireEvent, cx| {
            if let QuestionnaireEvent::Submit(submission) = event {
                this.extended.question_result =
                    format!("Saved {} appearance preferences", submission.items().len());
                cx.notify();
            }
        })
        .detach();
        let speech = cx.new(SpeechState::new);
        cx.subscribe(&speech, |this, _, event: &kit::speech::SpeechEvent, cx| {
            this.extended.transcript = match event {
                kit::speech::SpeechEvent::Partial(text) | kit::speech::SpeechEvent::Final(text) => {
                    text.to_string()
                }
                kit::speech::SpeechEvent::Error(error) => format!("{error}"),
                _ => String::new(),
            };
            cx.notify();
        })
        .detach();
        let messages = cx.new(|cx| MessageScrollerState::new(20, cx));
        let table = cx.new(|cx| TableState::new(FilesTable::new(), window, cx));
        cx.observe(&carousel, |_, _, cx| cx.notify()).detach();
        cx.observe(&time, |_, _, cx| cx.notify()).detach();
        cx.observe(&questionnaire, |_, _, cx| cx.notify()).detach();
        Self {
            editor,
            search,
            password,
            url,
            diff,
            command,
            carousel,
            time,
            questionnaire,
            speech,
            messages,
            table,
            focus: cx.focus_handle(),
            rating: 4,
            step: 1,
            selected_row: 0,
            toolbar_location: 1,
            transcript: String::new(),
            question_result: String::new(),
            command_result: "Choose a command".into(),
            attachment_visible: true,
            attachment_failed: true,
            message_count: 20,
            launch_popup: false,
        }
    }

    pub(super) fn render(
        &mut self,
        page: &str,
        mut content: gpui_kit::Div,
        workspace_state: &Entity<gpui_kit::base::input::InputState>,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Gallery>,
    ) -> gpui_kit::Div {
        let t = cx.macos().clone();
        match page {
            "dropdown_button" => {
                let owner = cx.entity();
                content = content.child(menu("export-dropdown", button("export-trigger", "Export", ButtonVariant::Primary, cx).child(icon(IconName::ChevronDown)), vec![MenuItem::new("Export as PDF"), MenuItem::new("Export as Markdown"), MenuItem::new("Export selection").disabled(true)], move |index, _, cx| { owner.update(cx, |this, cx| { this.extended.command_result = if index == 0 { "PDF exported" } else { "Markdown exported" }.into(); cx.notify(); }); }))
                    .child(note(self.command_result.clone(), cx));
            }
            "search_field" => content = content.child(search_field("native-search", &self.search, window, cx).w(rems(28.))).child(note("Type a query. Standard selection, clipboard, and IME editing work in the native field.", cx)),
            "secure_field" => content = content.child(secure_field("native-password", &self.password, window, cx).w(rems(28.))).child(note("The field masks its value while retaining native editing.", cx)),
            "scrollable" => content = content.child(div().id("scrollable-preview").h(rems(22.)).w(rems(32.)).overflow_y_scroll().rounded(rems(0.75)).border_1().border_color(t.border).bg(t.surface).p(rems(1.)).children((0..60).map(|index| div().py(rems(0.625)).border_b_1().border_color(t.divider()).child(format!("Activity {:02} — Document saved to iCloud", index + 1))))),
            "title_bar" => content = content.child(title_bar("Workspace — Documents", cx).rounded_t(rems(0.75)).border_1().border_color(t.border)).child(note("The leading space belongs to macOS's real traffic-light buttons in a native window.", cx)),
            "toolbar" => {
                let locations = ["Recents", "Documents", "Shared"];
                content = content.child(toolbar("toolbar-preview", cx)
                    .child(toolbar_group(cx)
                        .child(toolbar_button("tool-prev", IconName::ChevronLeft, "Back", cx)
                            .disabled(self.toolbar_location == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.extended.toolbar_location = this.extended.toolbar_location.saturating_sub(1);
                                cx.notify();
                            })))
                        .child(toolbar_button("tool-next", IconName::ChevronRight, "Forward", cx)
                            .disabled(self.toolbar_location == locations.len() - 1)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.extended.toolbar_location = (this.extended.toolbar_location + 1).min(2);
                                cx.notify();
                            }))))
                    .child(div().flex_1().font_weight(FontWeight::SEMIBOLD).child(locations[self.toolbar_location]))
                    .child(toolbar_group(cx).child(toolbar_button("tool-new", IconName::Plus, "New document", cx)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.extended.command_result = "New document created".into();
                            cx.notify();
                        })))))
                    .child(note(self.command_result.clone(), cx));
            }
            "sidebar" => {
                let mut shell = sidebar("sidebar-preview", cx).h(rems(19.)).rounded(rems(0.75));
                shell = shell.child(sidebar_section("Favorites", cx));
                for (index, (name, symbol)) in [("Recents", IconName::Clock), ("Documents", IconName::File), ("Shared", IconName::Users)].into_iter().enumerate() {
                    shell = shell.child(sidebar_item(("native-sidebar-row", index), name, symbol, self.selected_row == index, cx).on_click(cx.listener(move |this, _, _, cx| { this.extended.selected_row = index; cx.notify(); })));
                }
                content = content.child(shell);
            }
            "status_bar" => content = content.child(status_bar(cx).rounded(rems(0.75)).child("3 documents selected").child("All changes saved")),
            "settings" => content = content.child(settings_group("Workspace", cx)
                .child(settings_row("Sync with iCloud", "Keep documents available on your devices", cx).child(switch("settings-sync", "Sync with iCloud", enabled, cx).on_change(change(cx.listener(|this, value, _, cx| { this.enabled = *value; cx.notify(); })))))
                .child(settings_row("Workspace name", "Shown in the sidebar", cx).child(input("settings-name", workspace_state, window, cx).w(rems(16.))))
                .child(settings_row("Notifications", "Receive updates when shared files change", cx).child(button("settings-alert", "Send preview", ButtonVariant::Secondary, cx).on_click(|_, window, cx| { window.push_notification("Preview notification", cx); })))),
            "inspector" => content = content.child(inspector("Document information", cx).h(rems(18.)).rounded(rems(0.75))
                .child(mac::description_list().label_width(rems(5.)).item("Name", "Design review", 1).item("Type", "Markdown document", 1).item("Modified", "Today at 9:41 AM", 1))
                .child(input("inspector-title", workspace_state, window, cx))),
            "inset_list" => {
                let mut rows = inset_list(cx).w(rems(32.));
                for (index, name) in ["General", "Appearance", "Privacy & Security"].into_iter().enumerate() {
                    rows = rows.child(inset_list_row(("inset-row", index), cx).child(div().flex().items_center().justify_between().w_full().child(name).child(icon(IconName::ChevronRight))).on_click(cx.listener(move |this, _, _, cx| { this.extended.selected_row = index; cx.notify(); })));
                }
                content = content.child(rows).child(note(format!("Selected row {}", self.selected_row + 1), cx));
            }
            "material" => content = content.child(div().flex().flex_wrap().gap(rems(1.)).children([(Material::Sidebar, "Sidebar"), (Material::Toolbar, "Toolbar"), (Material::Popover, "Popover"), (Material::Hud, "HUD")].map(|(kind, label)| material(kind, cx).w(rems(13.)).h(rems(8.)).rounded(rems(1.)).border_1().shadow_md().flex().flex_col().items_center().justify_center().gap(rems(0.5)).child(icon(IconName::Layers).size(rems(1.75)).text_color(t.accent)).child(label))))
                .child(popover("material-glass-preview", button("material-glass-trigger", "Show Glass Panel", ButtonVariant::Secondary, cx), |_, _, cx| {
                    let t = cx.macos();
                    div().flex().flex_col().gap(rems(0.625)).debug_selector(|| "material-glass-content".into())
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("System glass"))
                        .child("The foreground stays interactive while AppKit draws the material behind it.")
                        .child(div().text_color(t.secondary).child("The native route is available on compatible macOS hosts. Appearance and transparency preferences determine the fallback."))
                }).when(std::mem::take(&mut self.launch_popup), |popover| popover.open(true)))
                .child(note("These samples show semantic canvas fills. Native sidebar and toolbar materials use AppKit glass; menus, popovers and picker panels use separate system glass windows above the GPUI scene. Reduced transparency uses opaque fills.", cx)),
            "editor" => content = content.child(Editor::new(&self.editor).w_full().h(rems(23.)).border_1().border_color(t.border).rounded(rems(0.625)))
                .child(note("Edit this source. ⌘F opens Find; Option-click adds another cursor. Enable tree-sitter-languages for syntax grammars.", cx)),
            "diff" => content = content.child(
                button("diff-mode", if self.diff.read(cx).mode() == DiffMode::Split { "Use unified view" } else { "Use side-by-side view" }, ButtonVariant::Secondary, cx)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.extended.diff.update(cx, |state, cx| state.set_mode(if state.mode() == DiffMode::Split { DiffMode::Unified } else { DiffMode::Split }, cx)); cx.notify();
                    })))
                .child(Diff::new(&self.diff).w_full().h(rems(22.)).border_1().border_color(t.border).rounded(rems(0.625))),
            "command" => {
                let owner = cx.entity();
                content = content.child(Command::new(&self.command).w(rems(30.)).max_h(rems(18.))
                    .placeholder("Search components and commands…")
                    .group(CommandGroup::new().label("Components").items([
                        CommandItem::new().label("Buttons").icon(kit::IconName::CircleCheck),
                        CommandItem::new().label("Calendar").icon(kit::IconName::Calendar),
                        CommandItem::new().label("Settings").icon(kit::IconName::Settings),
                    ]))
                    .separator()
                    .group(CommandGroup::new().label("Workspace").items([
                        CommandItem::new().label("Create a document").icon(kit::IconName::Plus),
                        CommandItem::new().label("Move to Trash").disabled(true),
                    ]))
                    .on_confirm(move |index, _, cx| owner.update(cx, |this, cx| {
                        let pages = ["button", "calendar", "settings"];
                        if index.section == 0 && index.row < pages.len() { this.page = pages[index.row]; }
                        this.extended.command_result = format!("Selected command {}:{}", index.section, index.row); cx.notify();
                    })))
                    .child(note(self.command_result.clone(), cx));
            }
            "clipboard" => {
                content = content.child(div().flex().items_center().gap(rems(0.75)).child("cargo add gpui-macos")
                    .child(Clipboard::new("copy-install").value("cargo add gpui-macos").accessibility_label("Copy install command")))
                    .child(note("Click the copy symbol to place the command on the system clipboard.", cx));
            }
            "label" => content = content.child(Label::new("Workspace preferences").secondary("Saved locally"))
                .child(Label::new("Find documents by name").highlights("documents"))
                .child(Label::new("Password: correct-horse-battery").masked(true)),
            "tag" => content = content.child(div().flex().gap(rems(0.5))
                .child(Tag::primary().child("Selected"))
                .child(Tag::secondary().child("Draft"))
                .child(Tag::success().child("Synced"))
                .child(Tag::warning().child("Review"))
                .child(Tag::danger().outline().child("Offline"))),
            "spinner" => content = content.child(div().flex().items_center().gap(rems(0.75)).child(native_spinner(cx).color(t.accent).with_size(kit::Size::Large)).child("Syncing with iCloud…"))
                .child(div().flex().items_center().gap(rems(0.75)).child(native_spinner(cx).with_size(kit::Size::Small)).child("Loading preview")),
            "skeleton" => content = content.child(div().flex().gap(rems(0.875))
                .child(Skeleton::new().size(rems(3.)).rounded_full())
                .child(div().flex().flex_col().gap(rems(0.5)).child(Skeleton::new().w(rems(12.)).h(rems(0.875))).child(Skeleton::new().secondary().w(rems(19.)).h(rems(0.75)))))
                .child(Skeleton::new().w(rems(30.)).h(rems(10.)).rounded(rems(0.75))),
            "shimmer" => content = content.child(ShimmerText::new("Organizing your workspace…").text_size(rems(1.)) )
                .child(ShimmerText::new("Preparing a preview").text_color(t.secondary)),
            "rating" => content = content.child(Rating::new("rating-edit").value(self.rating).color(t.warning).on_click(cx.listener(|this, value, _, cx| { this.extended.rating = *value; cx.notify(); })))
                .child(note(format!("Your rating: {} of 5", self.rating), cx))
                .child(Rating::new("rating-readonly").value(3).disabled(true).color(t.warning)),
            "group_box" => content = content.child(GroupBox::new().title("Appearance").fill().w_full()
                .child(div().flex().items_center().justify_between().child("Use system appearance").child(switch("group-appearance", "Use system appearance", enabled, cx).on_change(change(cx.listener(|this, value, _, cx| { this.enabled = *value; cx.notify(); }))))))
                .child(GroupBox::new().title("Keyboard").outline().w_full().child("Press Tab to move keyboard focus between controls.").footer(note("Your settings apply to this workspace.", cx))),
            "description_list" => content = content.child(mac::description_list()
                .item("Name", "Studio", 1).item("Owner", "You", 1).item("Modified", "Today, 9:41 AM", 1).item("Storage", "128 MB", 1).item("Location", "iCloud Drive / Documents", 1)),
            "breadcrumb" => content = content.child(Breadcrumb::new().child(BreadcrumbItem::new("Components").on_click(cx.listener(|this, _, _, cx| { this.page = "overview"; cx.notify(); })))
                .child(BreadcrumbItem::new("Navigation").on_click(cx.listener(|this, _, _, cx| { this.page = "sidebar"; cx.notify(); })))
                .child(BreadcrumbItem::new("Breadcrumb").disabled(true))),
            "carousel" => {
                let mut slides = CarouselContent::new(&self.carousel);
                let mut dots = CarouselPagination::new();
                for (index, (label, detail)) in DOCUMENT_PREVIEWS.into_iter().enumerate() {
                    slides = slides.child(CarouselItem::new(("slide", index), index, &self.carousel)
                        .child(div().w_full().flex().flex_col().gap(rems(0.625))
                            .child(div().id(("document-preview-pane", index)).debug_selector(move || format!("document-preview-pane-{index}"))
                                .h(rems(20.)).w_full().bg(t.normal_fill()).rounded(rems(0.5)).p(rems(1.))
                                .overflow_hidden().flex().items_center().justify_center()
                                .child(gpui_kit::img(document_preview(index)).id(("document-preview-image", index)).debug_selector(move || format!("document-preview-image-{index}"))
                                    .w_full().h(rems(18.)).flex_shrink_0().object_fit(gpui_kit::ObjectFit::Contain)))
                            .child(div().flex().items_center().gap(rems(0.5)).child(icon(IconName::FileText).text_color(t.accent)).child(label))
                            .child(note(detail, cx))));
                    dots = dots.child(CarouselPaginationItem::new(("slide-dot", index), index, &self.carousel)
                        .size(rems(0.375)).min_w(rems(0.375)).min_h(rems(0.375)).p(rems(0.))
                        .rounded_full().border_0().shadow_none()
                        .bg(t.foreground.opacity(if self.carousel.read(cx).selected_index() == Some(index) { 0.5 } else { 0.18 })));
                }
                content = content.child(mac::carousel("feature-carousel", &self.carousel, cx).accessibility_label("Document previews").w(rems(36.)).max_w_full().child(slides).child(div().flex().items_center().justify_between().mt(rems(0.75)).child(mac::carousel_previous(&self.carousel, cx).debug_selector(|| "document-preview-previous".into())).child(dots).child(mac::carousel_next(&self.carousel, cx).debug_selector(|| "document-preview-next".into()))))
                    .child(note("Browse document previews with the arrow controls, pagination, or keyboard. Each document keeps its position.", cx));
            }
            "image" => content = content.child(div().flex().flex_wrap().gap(rems(1.)).children(DOCUMENT_PREVIEWS.into_iter().enumerate().map(|(index, (title, detail))| {
                div().w(rems(19.)).flex().flex_col().gap(rems(0.5))
                    .child(div().id(("document-thumbnail-pane", index)).debug_selector(move || format!("document-thumbnail-pane-{index}"))
                        .h(rems(13.)).w_full().p(rems(0.5)).bg(t.normal_fill()).rounded(rems(0.5))
                        .overflow_hidden().flex().items_center().justify_center()
                        .child(gpui_kit::img(document_preview(index)).id(("document-image", index)).debug_selector(move || format!("document-thumbnail-image-{index}"))
                            .w_full().h(rems(11.)).flex_shrink_0().object_fit(gpui_kit::ObjectFit::Contain)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(title))
                    .child(note(detail, cx))
            })))
                .child(note("These original document illustrations load through the real image API. Asset identifiers, local files, and remote sources use the same component.", cx)),
            "time_field" => content = content.child(TimeField::new(&self.time)).child(note(format!("Meeting starts at {}", self.time.read(cx).time().format("%H:%M")), cx)),
            "input_group" => content = content.child(InputGroup::new("url-input-group")
                .input(InputGroupInput::new(&self.url))
                .addon(InputGroupAddon::new("url-prefix").child(InputGroupText::new().child("https://")))
                .addon(InputGroupAddon::new("url-suffix").align(kit::input::InputGroupAddonAlignment::InlineEnd).child(InputGroupText::new().child(".app"))).w(rems(30.))),
            "form" => {
                content = content.child(mac::form(window, cx).w_full()
                    .child(Field::new().label("Workspace name").required(true).description("Choose a name that is easy to find.").child(input("form-name", workspace_state, window, cx)))
                    .child(Field::new().label("Sync").description("Keep files available on other devices.").child(switch("form-sync", "Sync", enabled, cx).on_change(change(cx.listener(|this, value, _, cx| { this.enabled = *value; cx.notify(); })))))
                    .footer(button("form-save", "Save changes", ButtonVariant::Primary, cx).on_click(cx.listener(|this, _, window, cx| this.save_workspace(false, window, cx)))));
            }
            "questionnaire" => {
                let state = &self.questionnaire;
                let mut questions = mac::questionnaire(state, cx)
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Set up your workspace"))
                    .child(note("Choose how this workspace appears. You can change these preferences later.", cx))
                    .child(QuestionnaireProgress::new(state).text_color(t.secondary).text_size(rems(0.6875)));
                for (name, choices) in [("appearance", &["system", "light", "dark"][..]), ("density", &["regular", "compact"][..])] {
                    questions = questions.child(QuestionnaireItem::new(state, name)
                        .bg(t.normal_fill()).rounded(rems(0.625)).p(rems(1.))
                        .child(QuestionnaireTitle::new(state, name).text_size(rems(0.8125)))
                        .child(QuestionnaireChoices::new(state, name).gap(rems(0.125)).children(choices.iter().map(|value| mac::questionnaire_choice(state, name, *value, cx))))
                        .child(QuestionnaireError::new(state, name)));
                }
                content = content.child(questions.child(mac::questionnaire_actions(state, cx)
                    .children(mac::questionnaire_previous(state, cx))
                    .children(mac::questionnaire_next(state, cx))
                    .children(mac::questionnaire_submit(state, cx))).w(rems(36.)).max_w_full()).child(note(self.question_result.clone(), cx));
            }
            "stepper" => content = content.child(Stepper::new("setup-steps").selected_index(self.step)
                .items([StepperItem::new().child("Create workspace"), StepperItem::new().child("Choose appearance"), StepperItem::new().child("Ready")])
                .on_click(cx.listener(|this, index, _, cx| { this.extended.step = *index; cx.notify(); })).w_full())
                .child(note(format!("Step {} of 3", self.step + 1), cx)),
            "list" => {
                let mut rows = div().w(rems(30.)).flex().flex_col().gap(rems(0.125));
                for (index, label) in ["Project brief", "Meeting notes", "Design references"].into_iter().enumerate() {
                    rows = rows.child(mac::list_item(("list-item", index), cx).debug_selector(move || format!("list-row-{index}")).selected(index == self.selected_row).child(div().flex().items_center().gap(rems(0.625)).child(icon(IconName::File)).child(div().debug_selector(move || format!("list-label-{index}")).child(label)))
                        .on_click(cx.listener(move |this, _, _, cx| { this.extended.selected_row = index; cx.notify(); })));
                }
                content = content.child(rows).child(note(format!("Selected document {}", self.selected_row + 1), cx));
            }
            "data_table" => content = content.child(div().w_full().h(rems(25.)).child(DataTable::new(&self.table).stripe(true).bordered(true))),
            "notification" => content = content.child(div().flex().gap(rems(0.75))
                .child(button("notify-success", "Show notification", ButtonVariant::Primary, cx).on_click(|_, window, cx| { window.push_notification(Notification::success("Workspace saved to iCloud"), cx); }))
                .child(button("notify-warning", "Show warning", ButtonVariant::Secondary, cx).on_click(|_, window, cx| { window.push_notification(Notification::warning("Your storage is almost full"), cx); }))),
            "bubble" => content = content.child(Bubble::new().content(mac::bubble_content(MessageAlignment::Start, cx).child("The project brief is ready. Can you review it this afternoon?")))
                .child(Bubble::new().alignment(MessageAlignment::End).content(mac::bubble_content(MessageAlignment::End, cx).child("Yes, I’ll leave my comments in the shared document."))),
            "message" => content = content.child(conversation_message(0, cx)).child(conversation_message(1, cx)),
            "marker" => content = content.child(Marker::new().content(MarkerContent::new().text("Today, 9:41 AM"))).child(conversation_message(0, cx))
                .child(Marker::new().loading(true).content(MarkerContent::new().text("Preparing response"))),
            "attachment" => {
                if self.attachment_visible {
                    let failed = self.attachment_failed;
                    content = content.child(mac::attachment(cx).id("document-attachment").w(rems(29.)).status(if failed { AttachmentStatus::Failed } else { AttachmentStatus::Complete })
                        .content(AttachmentContent::new().title(AttachmentTitle::new("Design review.pdf")).description(AttachmentDescription::new(if failed { "Upload failed. Try again." } else { "2.4 MB · PDF document" })))
                        .on_retry(cx.listener(|this, _, _, cx| { this.extended.attachment_failed = false; cx.notify(); }))
                        .on_remove(cx.listener(|this, _, _, cx| { this.extended.attachment_visible = false; cx.notify(); })));
                }
                content = content.child(button("attachment-reset", "Attach document", ButtonVariant::Secondary, cx).on_click(cx.listener(|this, _, _, cx| { this.extended.attachment_visible = true; this.extended.attachment_failed = true; cx.notify(); })));
            }
            "message_scroller" => {
                content = content.child(MessageScroller::new("conversation-scroll", self.messages.clone(), |index, _, cx| {
                    conversation_message(index, cx).into_any_element()
                }).w_full().h(rems(23.)))
                    .child(button("append-message", "Add a message", ButtonVariant::Primary, cx).on_click(cx.listener(|this, _, _, cx| { this.extended.message_count += 1; this.extended.messages.update(cx, |state, cx| { state.append(1, cx); }); cx.notify(); })));
            }
            "speech" => content = content.child(div().flex().items_center().gap(rems(1.)).child(SpeechButton::new(&self.speech).show_when_unsupported(true)).child(SpeechWaveform::new(&self.speech).w(rems(20.)).h(rems(3.))))
                .child(note(if self.transcript.is_empty() { "Enable the speech Cargo feature to use the macOS recognizer. Recognition begins when you press the microphone.".into() } else { self.transcript.clone() }, cx)),
            "theme" => {
                content = content.child(div().text_size(rems(1.25)).font_weight(FontWeight::SEMIBOLD).child(t.name.clone()));
                let swatches = [("Accent", t.accent), ("Background", t.background), ("Surface", t.surface), ("Text", t.foreground), ("Secondary", t.secondary), ("Success", t.success), ("Warning", t.warning), ("Danger", t.danger)];
                content = content.child(div().flex().flex_wrap().gap(rems(1.)).children(swatches.map(|(name, color)| div().flex().flex_col().gap(rems(0.5)).child(div().w(rems(6.)).h(rems(4.)).rounded(rems(0.75)).bg(color).border_1().border_color(t.border)).child(name))))
                    .child(note("Use the toolbar menu to switch between System, Light, and Dark appearance.", cx));
            }
            "root" => content = content.child(GroupBox::new().title("Native window host").fill().w_full().child("Root coordinates keyboard navigation, overlays, tooltips, and selectable text.")
                .child(div().mt(rems(1.)).flex().gap(rems(0.5)).child(button("root-notification", "Show overlay", ButtonVariant::Primary, cx).on_click(|_, window, cx| { window.push_notification("This notification is hosted by Root", cx); }))
                    .child(Clipboard::new("root-clipboard").value("Root::new(view, window, cx)")))),
            "focus_trap" => content = content.child(div().id("focus-trap-preview").focus_trap("focus-trap-preview", &self.focus).w(rems(30.)).p(rems(1.)).rounded(rems(0.75)).border_1().border_color(t.border)
                .child(div().flex().gap(rems(0.5)).child(button("trap-one", "First control", ButtonVariant::Secondary, cx)).child(button("trap-two", "Second control", ButtonVariant::Primary, cx)).child(button("trap-three", "Third control", ButtonVariant::Secondary, cx))))
                .child(note("Click a control, then press Tab or Shift-Tab to cycle within this group.", cx)),
            _ => unreachable!("Missing extended gallery page: {page}"),
        }
        content
    }
}

fn note(text: impl Into<gpui_kit::SharedString>, cx: &App) -> gpui_kit::Div {
    div()
        .text_color(cx.macos().secondary)
        .text_size(rems(0.8125))
        .child(text.into())
}

fn conversation_message(index: usize, cx: &App) -> Message {
    let outgoing = index % 2 == 1;
    let alignment = if outgoing {
        MessageAlignment::End
    } else {
        MessageAlignment::Start
    };
    mac::message(alignment, cx)
        .id(("conversation-message", index))
        .role(gpui_kit::accesskit::Role::ListItem)
        .header(
            MessageHeader::new()
                .text_size(rems(0.6875))
                .text_color(cx.macos().secondary)
                .child(if outgoing { "You" } else { "Alex" }),
        )
        .content(
            MessageContent::new().bubble(Bubble::new().alignment(alignment).content(
                mac::bubble_content(alignment, cx).child(if outgoing {
                    format!(
                        "Review {} is complete. The new controls look great.",
                        index / 2 + 1
                    )
                } else {
                    format!(
                        "Design review {} is ready. Take a look when you have a moment.",
                        index / 2 + 1
                    )
                }),
            )),
        )
        .footer(
            MessageFooter::new()
                .text_size(rems(0.6875))
                .text_color(cx.macos().secondary)
                .child(if outgoing { "Delivered" } else { "9:41 AM" }),
        )
}

const DOCUMENT_PREVIEWS: [(&str, &str); 3] = [
    ("Project brief.pdf", "2 pages · Shared with Studio"),
    ("Weekend itinerary.pdf", "1 page · On My Mac"),
    ("Quarterly budget.pdf", "3 pages · iCloud Drive"),
];

fn document_preview(index: usize) -> std::sync::Arc<gpui_kit::Image> {
    use std::sync::{Arc, LazyLock};
    static IMAGES: LazyLock<[Arc<gpui_kit::Image>; 3]> = LazyLock::new(|| {
        [
            include_bytes!("assets/project-brief.svg").as_slice(),
            include_bytes!("assets/weekend-itinerary.svg").as_slice(),
            include_bytes!("assets/quarterly-budget.svg").as_slice(),
        ]
        .map(|bytes| {
            Arc::new(gpui_kit::Image::from_bytes(
                gpui_kit::ImageFormat::Svg,
                bytes.to_vec(),
            ))
        })
    });
    IMAGES[index].clone()
}

struct FilesTable {
    rows: Vec<[String; 3]>,
}
impl FilesTable {
    fn new() -> Self {
        Self {
            rows: (0..120)
                .map(|index| {
                    [
                        format!("Document {:03}", index + 1),
                        if index % 3 == 0 {
                            "iCloud"
                        } else {
                            "On My Mac"
                        }
                        .into(),
                        format!("{} KB", 24 + index * 8),
                    ]
                })
                .collect(),
        }
    }
}
impl TableDelegate for FilesTable {
    fn columns_count(&self, _: &App) -> usize {
        3
    }
    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }
    fn column(&self, col: usize, _: &App) -> Column {
        [
            Column::new("name", "Name").width(260.).sortable(),
            Column::new("location", "Location").width(160.),
            Column::new("size", "Size").width(120.).sortable(),
        ][col]
            .clone()
    }
    fn render_td(
        &mut self,
        row: usize,
        col: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        self.rows[row][col].clone()
    }
    fn perform_sort(
        &mut self,
        column: usize,
        sort: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        self.rows.sort_by(|a, b| a[column].cmp(&b[column]));
        if sort == ColumnSort::Descending {
            self.rows.reverse();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::test::TestQueryExt as _;
    use gpui_kit::{SharedString, TestAppContext};

    #[gpui_kit::test]
    fn command_search_and_keyboard_confirmation_open_the_selected_component(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_macos::init);
        let (view, cx) = cx.add_window_view(Gallery::new);
        view.update(cx, |this, cx| {
            this.page = "command";
            cx.notify();
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let command = view.read(cx).extended.command.clone();
            command.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.simulate_input("Calendar");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(view.read(cx).extended.command.read(cx).matched_count(), 1);
        });
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(view.read(cx).page, "calendar");
        });
    }

    #[gpui_kit::test]
    fn masking_a_password_preserves_the_regular_search_and_input_fields(cx: &mut TestAppContext) {
        cx.update(gpui_macos::init);
        let (view, cx) = cx.add_window_view(Gallery::new);
        for page in ["secure_field", "search_field", "input_group", "input"] {
            view.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }
        cx.update(|_, cx| {
            let gallery = view.read(cx);
            assert_eq!(
                gallery.extended.password.read(cx).value().as_ref(),
                "studio-workspace"
            );
            assert_eq!(gallery.extended.url.read(cx).value().as_ref(), "studio");
            assert!(gallery.extended.search.read(cx).value().is_empty());
            assert!(gallery.input.read(cx).value().is_empty());
        });
    }

    #[gpui_kit::test]
    fn setup_assistant_retains_answers_and_submits_through_kit_controls(cx: &mut TestAppContext) {
        cx.update(gpui_macos::init);
        let (view, cx) = cx.add_window_view(Gallery::new);
        view.update(cx, |this, cx| {
            this.page = "questionnaire";
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let state = cx.read(|cx| view.read(cx).extended.questionnaire.clone());
        let light = Box::leak(
            format!("mac-questionnaire-{}-appearance-light", state.entity_id()).into_boxed_str(),
        );
        let compact = Box::leak(
            format!("mac-questionnaire-{}-density-compact", state.entity_id()).into_boxed_str(),
        );
        let point = cx
            .debug_bounds(light)
            .expect("native radio indicator is visible")
            .center();
        cx.simulate_click(point, Default::default());
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(600));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(state.read(cx).current_item().unwrap().as_ref(), "density");
            assert_eq!(
                state.read(cx).answer("appearance").unwrap().choices(),
                &[SharedString::from("light")]
            );
        });
        let point = cx
            .debug_bounds(compact)
            .expect("second question is visible")
            .center();
        cx.simulate_click(point, Default::default());
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(600));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(
                view.read(cx).extended.question_result,
                "Saved 2 appearance preferences"
            );
            assert_eq!(
                state.read(cx).answer("density").unwrap().choices(),
                &[SharedString::from("compact")]
            );
        });
        for page in ["overview", "questionnaire"] {
            view.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }
        assert_eq!(
            cx.read(|cx| state
                .read(cx)
                .answer("appearance")
                .unwrap()
                .choices()
                .to_vec()),
            vec![SharedString::from("light")]
        );
    }

    #[gpui_kit::test]
    fn setup_actions_have_accessible_names_native_geometry_and_inactive_fill(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_macos::init(cx);
            cx.set_reduce_motion(true);
        });
        let (view, cx) = cx.add_window_view(Gallery::new);
        view.update(cx, |this, cx| {
            this.page = "questionnaire";
            cx.notify();
        });
        cx.deactivate_window();
        let continue_point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let action = window.find_by_label("Continue");
            assert_eq!(action.role(), Some(gpui_kit::Role::Button));
            assert_eq!(action.bounds().size.height, px(24.));
            assert!(window.try_find_by_label("Back").is_none());
            assert!(window.try_find_by_label("Done").is_none());
            let bounds = action.bounds().scale(window.scale_factor());
            let fill = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.bounds == bounds && quad.background.as_solid().is_some())
                .expect("Continue paints its native bezel")
                .background
                .as_solid()
                .unwrap();
            assert_eq!(
                fill,
                cx.macos().bezel_fill(),
                "Inactive Continue uses the translucent Bezel (spec 6.2.3)"
            );
            action.bounds().center()
        });
        cx.simulate_click(continue_point, Default::default());
        let back_point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            let state = view.read(cx).extended.questionnaire.read(cx);
            assert_eq!(state.current_item().unwrap().as_ref(), "density");
            assert!(window.try_find_by_label("Continue").is_none());
            assert_eq!(window.find_by_label("Done").bounds().size.height, px(24.));
            window.find_by_label("Back").bounds().center()
        });
        cx.simulate_click(back_point, Default::default());
        let continue_point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert_eq!(
                view.read(cx)
                    .extended
                    .questionnaire
                    .read(cx)
                    .current_item()
                    .unwrap()
                    .as_ref(),
                "appearance"
            );
            window.find_by_label("Continue").bounds().center()
        });
        cx.simulate_click(continue_point, Default::default());
        let submit_point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.find_by_label("Done").bounds().center()
        });
        cx.simulate_click(submit_point, Default::default());
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                !view.read(cx).extended.questionnaire.read(cx).is_complete(),
                "Kit validation still requires an answer or an explicit Skip"
            );
        });
        // Validation focuses the first answer. An arrow selects the following
        // answer without the pointer's automatic confirmation timer.
        cx.simulate_keystrokes("down");
        let submit_point = cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                !view
                    .read(cx)
                    .extended
                    .questionnaire
                    .read(cx)
                    .answer("density")
                    .unwrap()
                    .is_empty()
            );
            window.find_by_label("Done").bounds().center()
        });
        cx.simulate_click(submit_point, Default::default());
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(view.read(cx).extended.questionnaire.read(cx).is_complete());
            assert_eq!(
                view.read(cx).extended.question_result,
                "Saved 2 appearance preferences"
            );
        });
    }

    #[gpui_kit::test]
    fn document_preview_controls_enforce_boundaries_and_keep_selection(cx: &mut TestAppContext) {
        cx.update(gpui_macos::init);
        let (view, cx) = cx.add_window_view(Gallery::new);
        view.update(cx, |this, cx| {
            this.page = "carousel";
            cx.notify();
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let assert_contained = |cx: &mut gpui_kit::VisualTestContext,
                                pane: &'static str,
                                image: &'static str,
                                expected_height: gpui_kit::Pixels| {
            let pane = cx.debug_bounds(pane).expect("document pane");
            let image = cx.debug_bounds(image).expect("bounded document image");
            assert_eq!(image.size.height, expected_height);
            assert!(image.left() >= pane.left() && image.top() >= pane.top());
            assert!(
                image.right() <= pane.right() && image.bottom() <= pane.bottom(),
                "Full contain image box must stay inside the preview pane: {image:?} in {pane:?}"
            );
        };
        for (selector, expected) in [
            ("document-preview-previous", 0),
            ("document-preview-next", 1),
            ("document-preview-next", 2),
            ("document-preview-next", 2),
            ("document-preview-previous", 1),
        ] {
            let point = cx.debug_bounds(selector).unwrap().center();
            cx.simulate_click(point, Default::default());
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
                assert_eq!(
                    view.read(cx).extended.carousel.read(cx).selected_index(),
                    Some(expected)
                );
            });
            let (pane, image) = [
                ("document-preview-pane-0", "document-preview-image-0"),
                ("document-preview-pane-1", "document-preview-image-1"),
                ("document-preview-pane-2", "document-preview-image-2"),
            ][expected];
            assert_contained(cx, pane, image, px(288.));
        }
        for page in ["image", "carousel"] {
            view.update(cx, |this, cx| {
                this.page = page;
                cx.notify();
            });
            cx.update(|window, cx| window.draw(cx).clear(cx));
            if page == "image" {
                for (pane, image) in [
                    ("document-thumbnail-pane-0", "document-thumbnail-image-0"),
                    ("document-thumbnail-pane-1", "document-thumbnail-image-1"),
                    ("document-thumbnail-pane-2", "document-thumbnail-image-2"),
                ] {
                    assert_contained(cx, pane, image, px(176.));
                }
            }
        }
        assert_eq!(
            cx.read(|cx| view.read(cx).extended.carousel.read(cx).selected_index()),
            Some(1)
        );
    }
}
