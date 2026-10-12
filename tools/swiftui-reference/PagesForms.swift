import AppKit
import SwiftUI

// Forms pages (checkbox, switch and select live in Pages.swift).
// Each page mirrors its gallery render function (examples/gallery/).

extension PageContent {
    func formsPage(_ slug: String) -> AnyView? {
        switch slug {
        case "input": AnyView(InputPage())
        case "textarea": AnyView(TextareaPage())
        case "number_input": AnyView(NumberInputPage())
        case "combobox": AnyView(ComboboxPage())
        case "calendar": AnyView(CalendarPage().pinnedDateEnvironment())
        case "date_picker": AnyView(DatePickerPage().pinnedDateEnvironment())
        case "time_field": AnyView(TimeFieldPage().pinnedDateEnvironment())
        case "color_picker": AnyView(ColorPickerPage())
        case "slider": AnyView(SliderPage())
        case "radio": AnyView(RadioPage())
        case "search_field": AnyView(SearchFieldPage())
        case "secure_field": AnyView(SecureFieldPage())
        default: nil
        }
    }
}

// MARK: - Determinism

/// Pinned clock shared with the gallery: 2026-01-15 09:41, en_US, Gregorian, UTC.
private enum PinnedClock {
    static let timeZone = TimeZone(identifier: "UTC")!
    static let locale = Locale(identifier: "en_US")
    static let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = timeZone
        calendar.locale = locale
        return calendar
    }()
    static let date: Date = calendar.date(
        from: DateComponents(year: 2026, month: 1, day: 15, hour: 9, minute: 41)
    )!
}

private extension View {
    func pinnedDateEnvironment() -> some View {
        environment(\.locale, PinnedClock.locale)
            .environment(\.timeZone, PinnedClock.timeZone)
            .environment(\.calendar, PinnedClock.calendar)
    }
}

/// Secondary-colored helper text (the gallery's `note` and `text_color(secondary)` divs).
private struct Note: View {
    let text: String
    init(_ text: String) { self.text = text }
    var body: some View { Text(text).foregroundStyle(.secondary) }
}

/// The gallery's single-line input frame: 0.375 rem padding and corner radius,
/// 1.5 rem minimum height, 0.25 rem gap between prefix, editor and suffix.
/// SwiftUI's rounded-border style has no prefix/suffix slots, so the bezel is
/// composed around a plain TextField.
private struct AffixedField<Prefix: View, Suffix: View>: View {
    let placeholder: String
    @Binding var text: String
    @ViewBuilder var prefix: Prefix
    @ViewBuilder var suffix: Suffix

    var body: some View {
        HStack(spacing: 4) {
            prefix
            TextField(placeholder, text: $text)
                .textFieldStyle(.plain)
                .labelsHidden()
            suffix
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 3)
        .frame(minHeight: 24)
        .background(
            RoundedRectangle(cornerRadius: 6).fill(Color(nsColor: .textBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 6).strokeBorder(Color(nsColor: .separatorColor))
        )
    }
}

// MARK: - Input (render_input_page)

private struct InputPage: View {
    @State private var name = ""
    /// Shares the gallery's `number` state, which defaults to "1".
    @State private var quantity = "1"
    @State private var count = 0

    /// 23.75 rem.
    private let fieldWidth: CGFloat = 380

    var body: some View {
        Text("Workspace name")
        AffixedField(placeholder: "Project name", text: $name) {
            Image(systemName: "gearshape")
                .font(.system(size: 14))
                .foregroundStyle(.secondary)
        } suffix: {
            EmptyView()
        }
        .frame(maxWidth: fieldWidth)
        .accessibilityIdentifier("workspace-name")

        Text("Quantity")
        AffixedField(placeholder: "", text: $quantity) {
            EmptyView()
        } suffix: {
            Text("shares").foregroundStyle(.secondary)
        }
        .frame(maxWidth: fieldWidth)
        .accessibilityIdentifier("quantity-with-unit")

        HStack(spacing: 8) {
            Button("Read value") { count = name.count }
                .buttonStyle(.borderedProminent)
                .accessibilityIdentifier("submit-input")
            Button("Reset value") {
                name = ""
                count = 0
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("reset-input")
        }
        Text("Submitted length: \(count) characters")
            .accessibilityIdentifier("submitted-length")
    }
}

// MARK: - Textarea (render_textarea_page)

private struct TextareaPage: View {
    @State private var notes = ""

    var body: some View {
        Text("Workspace notes")
        // TextEditor has neither bezel nor placeholder; compose both. The
        // gallery's textarea is 4 rows (6 rem minimum height), 32.5 rem wide max.
        TextEditor(text: $notes)
            .scrollContentBackground(.hidden)
            .padding(.horizontal, 1)
            .padding(.vertical, 5)
            .overlay(alignment: .topLeading) {
                if notes.isEmpty {
                    Text("Add notes")
                        .foregroundStyle(.placeholder)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 5)
                        .allowsHitTesting(false)
                }
            }
            .frame(height: 96)
            .frame(maxWidth: 520)
            .background(
                RoundedRectangle(cornerRadius: 6).fill(Color(nsColor: .textBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 6).strokeBorder(Color(nsColor: .separatorColor))
            )
            .accessibilityIdentifier("notes")
        Text("Supports multiple lines, selection, clipboard and IME")
    }
}

// MARK: - Number input (render_number_input_page)

private struct NumberInputPage: View {
    @State private var quantity = 1

    var body: some View {
        Text("Quantity")
        // 7.5 rem wide, value right-aligned, steppers on the trailing edge.
        HStack(spacing: 4) {
            TextField("Quantity", value: $quantity, format: .number)
                .textFieldStyle(.roundedBorder)
                .multilineTextAlignment(.trailing)
                .labelsHidden()
            Stepper("Quantity", value: $quantity)
                .labelsHidden()
        }
        .frame(width: 120)
        .accessibilityIdentifier("quantity")
        Text("Arrow Up / Arrow Down  adjust by 1")
    }
}

// MARK: - Combobox (render_select_page, combobox branch)

private struct ComboboxPage: View {
    private let items = ["Personal workspace", "Team workspace", "Archived workspace", "Sandbox"]

    @State private var selected: String?

    /// 22.5 rem column.
    private let columnWidth: CGFloat = 360

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Find a workspace")
            ComboBox(items: items, placeholder: "Find a workspace…", value: nil, enabled: true) {
                selected = $0
            }
            .frame(maxWidth: .infinity)
            .accessibilityIdentifier("workspace-choice")
            Note("Archived workspaces cannot be selected.")
            Text("Selected: \(selected ?? "None")")
                .padding(.top, 12)
                .accessibilityIdentifier("selected-choice")
        }
        .frame(width: columnWidth, alignment: .leading)

        Divider()
            .padding(.top, 18)

        VStack(alignment: .leading, spacing: 8) {
            Text("Managed workspace")
            ComboBox(items: items, placeholder: "Find a workspace…", value: "Team workspace", enabled: false) { _ in }
                .frame(maxWidth: .infinity)
                .disabled(true)
                .accessibilityIdentifier("managed-choice")
            Note("This setting is managed by your organization.")
        }
        .frame(width: columnWidth, alignment: .leading)
    }
}

/// NSComboBox bridge: SwiftUI has no combo box on macOS.
private struct ComboBox: NSViewRepresentable {
    let items: [String]
    let placeholder: String
    let value: String?
    let enabled: Bool
    let onSelect: (String) -> Void

    func makeCoordinator() -> Coordinator { Coordinator(onSelect: onSelect) }

    func makeNSView(context: Context) -> NSComboBox {
        let box = NSComboBox()
        box.addItems(withObjectValues: items)
        box.placeholderString = placeholder
        box.completes = true
        box.usesDataSource = false
        box.numberOfVisibleItems = items.count
        box.stringValue = value ?? ""
        box.isEnabled = enabled
        box.delegate = context.coordinator
        box.setContentHuggingPriority(.defaultLow, for: .horizontal)
        return box
    }

    func updateNSView(_ box: NSComboBox, context: Context) {
        context.coordinator.onSelect = onSelect
        box.isEnabled = enabled
    }

    @MainActor
    final class Coordinator: NSObject, NSComboBoxDelegate {
        var onSelect: (String) -> Void
        init(onSelect: @escaping (String) -> Void) { self.onSelect = onSelect }

        func comboBoxSelectionDidChange(_ notification: Notification) {
            guard let box = notification.object as? NSComboBox,
                  let value = box.objectValueOfSelectedItem as? String else { return }
            onSelect(value)
        }
    }
}

// MARK: - Calendar (render_calendar_page)

private struct CalendarPage: View {
    /// Gallery side must select the same pinned day (see brief).
    @State private var date: Date? = PinnedClock.date

    var body: some View {
        Text("Schedule a workspace review")
        // The graphical picker always shows a selection; "Clear date" only
        // clears the summary line. Weekend disabling has no NSDatePicker API.
        DatePicker(
            "Review date",
            selection: Binding(get: { date ?? PinnedClock.date }, set: { date = $0 }),
            displayedComponents: .date
        )
        .datePickerStyle(.graphical)
        .labelsHidden()
        .fixedSize()
        .accessibilityIdentifier("review-calendar")
        Note("Choose a weekday. Weekends are unavailable.")
        Text(summary)
            .accessibilityIdentifier("selected-date")
        Button("Clear date") { date = nil }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("clear-date")
    }

    /// chrono "%A, %B %e, %Y".
    private var summary: String {
        guard let date else { return "No date selected" }
        let formatter = DateFormatter()
        formatter.locale = PinnedClock.locale
        formatter.timeZone = PinnedClock.timeZone
        formatter.calendar = PinnedClock.calendar
        formatter.dateFormat = "EEEE, MMMM d, yyyy"
        return formatter.string(from: date)
    }
}

// MARK: - Date picker (render_date_picker_page)

private struct DatePickerPage: View {
    @State private var date = PinnedClock.date

    var body: some View {
        Text("Review date")
        // Compact style: a date field whose click opens the calendar popover,
        // matching the gallery's trigger plus calendar popup.
        DatePicker("Review date", selection: $date, displayedComponents: .date)
            .datePickerStyle(.compact)
            .labelsHidden()
            .fixedSize()
            .accessibilityIdentifier("review-date")
        Note("Select a day to confirm. Escape cancels the calendar.")
    }
}

// MARK: - Time field (extended.rs "time_field")

private struct TimeFieldPage: View {
    @State private var time = PinnedClock.date

    var body: some View {
        // The gallery's TimeField uses a 24-hour cycle (HourCycle::H23).
        DatePicker("Meeting time", selection: $time, displayedComponents: .hourAndMinute)
            .datePickerStyle(.field)
            .labelsHidden()
            .fixedSize()
            .environment(\.locale, Locale(identifier: "en_US@hours=h23"))
            .accessibilityIdentifier("time-field")
        Note("Meeting starts at \(formatted)")
            .accessibilityIdentifier("time-note")
    }

    private var formatted: String {
        let parts = PinnedClock.calendar.dateComponents([.hour, .minute], from: time)
        return String(format: "%02d:%02d", parts.hour ?? 0, parts.minute ?? 0)
    }
}

// MARK: - Color picker (render_color_picker_page)

private struct ColorPickerPage: View {
    /// The gallery defaults to the theme accent.
    @State private var color = Color(nsColor: .controlAccentColor)

    var body: some View {
        Text("Project label color")
        ColorPicker("Choose color", selection: $color, supportsOpacity: false)
            .fixedSize()
            .accessibilityIdentifier("project-color")
        HStack(spacing: 10) {
            Rectangle()
                .fill(color)
                .frame(width: 20, height: 20)
            Text("Website refresh")
        }
        .accessibilityIdentifier("color-preview")
        Note("The swatch shows the committed label color. Escape discards an unconfirmed Hex edit.")
    }
}

// MARK: - Slider (render_slider_page)

private struct SliderPage: View {
    @State private var volume: Double = 40
    @State private var lower: Double = 20
    @State private var upper: Double = 80

    var body: some View {
        Text("Volume: \(Int(volume))")
            .accessibilityIdentifier("volume-value")
        // Continuous slider with snapping in the binding: a `step:` slider on
        // macOS draws tick marks, which the gallery slider does not have.
        Slider(value: snapped($volume), in: 0...100)
            .labelsHidden()
            .accessibilityIdentifier("volume")
        Text("Range: \(Int(lower))..\(Int(upper))")
            .accessibilityIdentifier("range-value")
        RangeSlider(lower: $lower, upper: $upper, bounds: 0...100, step: 5)
            .accessibilityIdentifier("range")
        Text("Disabled")
        Slider(value: .constant(60), in: 0...100)
            .labelsHidden()
            .disabled(true)
            .accessibilityIdentifier("disabled-slider")
        Text("Arrow keys / h l  adjust · Home / End  bounds · Tab  next thumb")
    }

    private func snapped(_ value: Binding<Double>) -> Binding<Double> {
        Binding(get: { value.wrappedValue }, set: { value.wrappedValue = ($0 / 5).rounded() * 5 })
    }
}

/// macOS has no two-thumb slider. Composed from shapes approximating the
/// system slider track, accent fill and knobs.
private struct RangeSlider: View {
    @Binding var lower: Double
    @Binding var upper: Double
    let bounds: ClosedRange<Double>
    let step: Double

    /// Measured from the native macOS 26 slider: 6 pt full-width track,
    /// 22 x 19 pt knob whose center travels between half-knob insets.
    private let knobWidth: CGFloat = 22
    private let knobHeight: CGFloat = 19
    private let trackHeight: CGFloat = 6

    var body: some View {
        GeometryReader { proxy in
            let usable = max(proxy.size.width - knobWidth, 1)
            let span = bounds.upperBound - bounds.lowerBound
            let lowX = knobWidth / 2 + CGFloat((lower - bounds.lowerBound) / span) * usable
            let highX = knobWidth / 2 + CGFloat((upper - bounds.lowerBound) / span) * usable
            let midY = proxy.size.height / 2
            ZStack(alignment: .topLeading) {
                Capsule()
                    .fill(Color(nsColor: .quaternaryLabelColor))
                    .frame(width: proxy.size.width, height: trackHeight)
                    .position(x: proxy.size.width / 2, y: midY)
                Rectangle()
                    .fill(Color.accentColor)
                    .frame(width: highX - lowX, height: trackHeight)
                    .position(x: (lowX + highX) / 2, y: midY)
                knob.position(x: lowX, y: midY)
                    .gesture(drag(usable: usable, span: span) { value in
                        lower = min(value, upper)
                    })
                knob.position(x: highX, y: midY)
                    .gesture(drag(usable: usable, span: span) { value in
                        upper = max(value, lower)
                    })
            }
        }
        .frame(height: 24)
    }

    private var knob: some View {
        Capsule()
            .fill(.white)
            .frame(width: knobWidth, height: knobHeight)
            .shadow(color: .black.opacity(0.3), radius: 1, y: 0.5)
            .overlay(Capsule().strokeBorder(Color.black.opacity(0.08), lineWidth: 0.5))
    }

    private func drag(usable: CGFloat, span: Double, apply: @escaping (Double) -> Void) -> some Gesture {
        DragGesture(minimumDistance: 0, coordinateSpace: .local).onChanged { gesture in
            let fraction = Double(min(max((gesture.location.x - knobWidth / 2) / usable, 0), 1))
            let raw = bounds.lowerBound + fraction * span
            apply((raw / step).rounded() * step)
        }
    }
}

// MARK: - Radio (render_radio_page)

private struct RadioPage: View {
    @State private var choice = 0

    private let options = ["Compact", "Comfortable", "Spacious"]
    private let rows = [
        ("Project brief", "Updated today"),
        ("Meeting notes", "Updated yesterday"),
        ("Release checklist", "Draft"),
    ]
    /// Preview row heights per density: 1.75 / 2.25 / 2.75 rem.
    private let rowHeights: [CGFloat] = [28, 36, 44]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("List density")
            Picker("List density", selection: $choice) {
                ForEach(options.indices, id: \.self) { index in
                    Text(options[index]).tag(index)
                }
            }
            .pickerStyle(.radioGroup)
            .labelsHidden()
            .accessibilityIdentifier("list-density")
        }
        VStack(alignment: .leading, spacing: 8) {
            Text("Preview")
            VStack(spacing: 0) {
                ForEach(rows.indices, id: \.self) { index in
                    if index > 0 { Divider().overlay(Color(nsColor: .separatorColor)) }
                    HStack {
                        Text(rows[index].0)
                        Spacer()
                        Text(rows[index].1).foregroundStyle(.secondary)
                    }
                    .padding(.horizontal, 10)
                    .frame(height: rowHeights[choice])
                    .accessibilityIdentifier("density-preview-\(index)")
                }
            }
            .overlay(Rectangle().strokeBorder(Color(nsColor: .separatorColor)))
            // 26.25 rem maximum width.
            .frame(maxWidth: 420)
        }
    }
}

// MARK: - Search field (extended.rs "search_field")

private struct SearchFieldPage: View {
    var body: some View {
        // `.searchable` is toolbar-bound; an in-content search field is NSSearchField.
        SearchField(placeholder: "Search documents")
            .frame(width: 448)
            .accessibilityIdentifier("native-search")
        Note("Type a query. Standard selection, clipboard, and IME editing work in the native field.")
    }
}

private struct SearchField: NSViewRepresentable {
    let placeholder: String

    func makeNSView(context: Context) -> NSSearchField {
        let field = NSSearchField()
        field.placeholderString = placeholder
        field.setContentHuggingPriority(.defaultLow, for: .horizontal)
        return field
    }

    func updateNSView(_ field: NSSearchField, context: Context) {}
}

// MARK: - Secure field (extended.rs "secure_field")

private struct SecureFieldPage: View {
    @State private var password = "studio-workspace"

    var body: some View {
        SecureField("Password", text: $password)
            .textFieldStyle(.roundedBorder)
            .frame(width: 448)
            .accessibilityIdentifier("native-password")
        Note("The field masks its value while retaining native editing.")
    }
}
