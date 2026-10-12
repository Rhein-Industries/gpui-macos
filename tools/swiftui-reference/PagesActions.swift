import AppKit
import Charts
import SwiftUI

// Actions, Navigation (except menu) and Charts pages.
// Each page mirrors its gallery render function (examples/gallery/).

extension PageContent {
    func actionsNavigationChartsPage(_ slug: String) -> AnyView? {
        switch slug {
        case "button_group": AnyView(ButtonGroupPage())
        case "dropdown_button": AnyView(DropdownButtonPage())
        case "link": AnyView(LinkPage())
        case "toggle": AnyView(TogglePage())
        case "toggle_group": AnyView(ToggleGroupPage())
        case "tabs": AnyView(TabsPage())
        case "collapsible": AnyView(CollapsiblePage())
        case "sidebar": AnyView(SidebarDemoPage())
        case "line_chart": AnyView(LineChartPage())
        case "bar_chart": AnyView(BarChartPage())
        case "area_chart": AnyView(AreaChartPage())
        case "pie_chart": AnyView(PieChartPage())
        default: nil
        }
    }
}

// MARK: - Shared

/// Gallery divider: foreground at 25/255 opacity.
private let dividerColor = Color.primary.opacity(25.0 / 255.0)

/// A row with a bottom divider, as the gallery draws list rows inside pages.
private struct DividedRow: View {
    let title: String
    let detail: String
    var top: CGFloat = 0
    var bottom: CGFloat = 10

    var body: some View {
        HStack(spacing: 14) {
            Text(title)
            Spacer(minLength: 8)
            Text(detail).foregroundStyle(.secondary)
        }
        .padding(.top, top)
        .padding(.bottom, bottom)
        .overlay(alignment: .bottom) {
            Rectangle().fill(dividerColor).frame(height: 1)
        }
        .frame(maxWidth: .infinity)
    }
}

// MARK: - Button group (render_button_group_page)

private struct ButtonGroupPage: View {
    private let labels = ["Top", "Right", "Bottom", "Left"]
    @State private var position = 0

    var body: some View {
        Text("Toolbar position")
        // The gallery's button group spans the content column.
        Picker("Toolbar position", selection: $position) {
            ForEach(labels.indices, id: \.self) { index in
                Text(labels[index]).tag(index)
            }
        }
        .pickerStyle(.segmented)
        .labelsHidden()
        .buttonSizing(.flexible)
        .frame(maxWidth: .infinity)
        .accessibilityIdentifier("toolbar-position")
        Text("Toolbar is placed at the \(labels[position].lowercased()).")
            .foregroundStyle(.secondary)
            .accessibilityIdentifier("toolbar-position-status")
        Text("Left / Right move the cursor · Return / Space choose")
            .foregroundStyle(.secondary)
    }
}

// MARK: - Dropdown button (extended.rs "dropdown_button")

private struct DropdownButtonPage: View {
    @State private var result = "Choose a command"

    var body: some View {
        // Primary trigger with a chevron that opens the actions menu.
        Menu("Export") {
            Button("Export as PDF") { result = "PDF exported" }
            Button("Export as Markdown") { result = "Markdown exported" }
            Button("Export selection") {}.disabled(true)
        }
        .menuStyle(.button)
        .buttonStyle(.borderedProminent)
        .menuIndicator(.visible)
        .fixedSize()
        .accessibilityIdentifier("export-trigger")
        Text(result)
            .foregroundStyle(.secondary)
            .accessibilityIdentifier("export-result")
    }
}

// MARK: - Link (render_link_page)

private struct LinkPage: View {
    private let url = URL(string: "https://gpui-kit.com")!

    var body: some View {
        // Gallery links carry 0.25 rem horizontal and 0.375 rem vertical padding.
        Link("Open GPUI Kit documentation", destination: url)
            .padding(.horizontal, 4)
            .padding(.vertical, 6)
            .accessibilityIdentifier("manual")
        Link("Unavailable link", destination: url)
            .disabled(true)
            .padding(.horizontal, 4)
            .padding(.vertical, 6)
            .accessibilityIdentifier("disabled-link")
    }
}

// MARK: - Toggle (render_toggle_page)

private struct TogglePage: View {
    @State private var pressed = false

    var body: some View {
        // Label first, then the star symbol, as in the gallery.
        Toggle(isOn: $pressed) {
            HStack(spacing: 8) {
                Text("Favorite panel")
                Image(systemName: "star")
            }
        }
        .toggleStyle(.button)
        .accessibilityLabel("Favorite panel")
        .accessibilityIdentifier("toggle")
    }
}

// MARK: - Toggle group (render_toggle_group)

private struct ToggleGroupPage: View {
    private let labels = ["Draft", "In review", "Published"]
    private let articles: [(title: String, status: Int)] = [
        ("Autumn release notes", 0),
        ("Getting started", 1),
        ("Keyboard shortcuts", 2),
        ("Workspace migration", 0),
    ]
    @State private var filters = [true, true, false]

    private var shown: [(index: Int, title: String, status: Int)] {
        articles.enumerated()
            .filter { filters[$0.element.status] }
            .map { ($0.offset, $0.element.title, $0.element.status) }
    }

    var body: some View {
        Text("Article status")
        // SwiftUI has no multi-select segmented control; the gallery's toggle
        // group is a row of independent toggle buttons 0.375 rem apart.
        HStack(spacing: 6) {
            ForEach(labels.indices, id: \.self) { index in
                Toggle(labels[index], isOn: $filters[index])
                    .toggleStyle(.button)
                    .accessibilityIdentifier("article-filter-\(index)")
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Article status filters")
        Text("Select any combination. Turn every filter off to show no articles.")
            .foregroundStyle(.secondary)
        ForEach(shown, id: \.index) { article in
            DividedRow(title: article.title, detail: labels[article.status], top: 10)
                .accessibilityIdentifier("filtered-article-\(article.index)")
        }
        Text("\(shown.count) articles shown")
            .foregroundStyle(.secondary)
            .accessibilityIdentifier("articles-shown")
        if shown.isEmpty {
            Button("Show all statuses") { filters = [true, true, true] }
                .accessibilityIdentifier("show-all-articles")
        }
    }
}

// MARK: - Tabs (render_tabs_example)

private struct TabsPage: View {
    private let labels = ["Overview", "Activity", "Settings"]
    @State private var tab = 0
    @State private var workspaceName = "Personal"
    @State private var sync = true

    var body: some View {
        // The gallery's tab list hugs its tabs at the leading edge above the panel.
        Picker("Workspace pages", selection: $tab) {
            ForEach(labels.indices, id: \.self) { index in
                Text(labels[index]).tag(index)
            }
        }
        .pickerStyle(.segmented)
        .labelsHidden()
        .fixedSize()
        .accessibilityIdentifier("workspace-tabs")

        VStack(alignment: .leading, spacing: 14) {
            switch tab {
            case 0:
                Text("Personal workspace").bold()
                Text("Your files and projects, organized in one place.").foregroundStyle(.secondary)
                DividedRow(title: "Documents", detail: "Notes and reference material")
                DividedRow(title: "Projects", detail: "Active work and experiments")
                DividedRow(title: "Archive", detail: "Completed projects")
            case 1:
                Text("Recent activity").bold()
                Text("Sample workspace history").foregroundStyle(.secondary)
                DividedRow(title: "Updated project notes", detail: "Today · 09:42")
                DividedRow(title: "Added a reference document", detail: "Today · 09:15")
                DividedRow(title: "Archived a completed project", detail: "Yesterday · 16:30")
            default:
                Text("Workspace name")
                TextField("Workspace name", text: $workspaceName)
                    .textFieldStyle(.roundedBorder)
                    .labelsHidden()
                    .frame(maxWidth: 360)
                HStack(spacing: 8) {
                    Toggle("Sync workspace", isOn: $sync)
                        .toggleStyle(.switch)
                        .labelsHidden()
                        .accessibilityIdentifier("tab-sync")
                    Text("Sync workspace")
                }
                Text("Changes apply immediately and remain available when you switch tabs.")
                    .foregroundStyle(.secondary)
            }
        }
        .padding(14)
        .frame(maxWidth: .infinity, minHeight: 240, alignment: .topLeading)
        .background(Color.primary.opacity(0.04))
        .accessibilityIdentifier("workspace-tab-panel")
    }
}

// MARK: - Collapsible (render_collapsible_page)

private struct CollapsiblePage: View {
    @State private var open = false
    @State private var sync = true

    var body: some View {
        DisclosureGroup("Advanced settings", isExpanded: $open) {
            VStack(alignment: .leading, spacing: 14) {
                Text("Workspace synchronization")
                HStack(spacing: 8) {
                    Toggle("Sync workspace", isOn: $sync)
                        .toggleStyle(.switch)
                        .labelsHidden()
                        .accessibilityIdentifier("collapse-sync")
                    Text("Sync workspace")
                }
                Text("Your selection is preserved when this section is collapsed.")
                    .foregroundStyle(.secondary)
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.primary.opacity(0.04))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .accessibilityIdentifier("collapse-trigger")
    }
}

// MARK: - Sidebar (extended.rs "sidebar")

private struct SidebarDemoPage: View {
    private let rows = [("Recents", "clock"), ("Documents", "doc"), ("Shared", "person.2")]
    @State private var selection: Int? = 0

    var body: some View {
        // 13.75 rem wide, 19 rem tall, 0.75 rem corner radius.
        List(selection: $selection) {
            Section("Favorites") {
                ForEach(rows.indices, id: \.self) { index in
                    Label(rows[index].0, systemImage: rows[index].1)
                        .tag(index)
                        .accessibilityIdentifier("native-sidebar-row-\(index)")
                }
            }
        }
        .listStyle(.sidebar)
        .scrollDisabled(true)
        .frame(width: 220, height: 304)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .accessibilityIdentifier("sidebar-preview")
    }
}

// MARK: - Charts (charts.rs)

/// The gallery's theme series colors (system blue, purple, teal, orange, green).
private let seriesColors: [Color] = [
    Color(nsColor: .systemBlue),
    Color(nsColor: .systemPurple),
    Color(nsColor: .systemTeal),
    Color(nsColor: .systemOrange),
    Color(nsColor: .systemGreen),
]

/// Chart region: full width, 15 rem tall.
private let chartHeight: CGFloat = 240
/// The x-axis label gutter under the plot: 10 pt text + 4 x 2 pt gaps.
private let xAxisGutter: CGFloat = 18
/// Plot height above the x-axis label gutter.
private let plotHeight: CGFloat = chartHeight - xAxisGutter
/// Axis and value labels are 10 pt.
private let chartLabelFont = Font.system(size: 10)
/// Secondary label color; Swift Charts would otherwise dim `.secondary` further.
private let secondaryText = Color(nsColor: .secondaryLabelColor)
/// Gallery axis line: the theme border, foreground at 12 % opacity.
private let axisColor = Color.primary.opacity(0.12)

/// Value-axis top for a point chart whose largest value sits `padding` points
/// below the top of the plot, so grid lines land where the gallery draws them.
private func paddedTop(_ maximum: Double, padding: CGFloat) -> Double {
    maximum * Double(plotHeight / (plotHeight - padding))
}

/// The gallery grid: a line at each quarter of the plot height, plus the
/// baseline in the axis color.
@AxisContentBuilder
private func quarterGrid(top: Double) -> some AxisContent {
    AxisMarks(values: [top * 0.25, top * 0.5, top * 0.75, top]) { _ in
        AxisGridLine(stroke: StrokeStyle(lineWidth: 1)).foregroundStyle(dividerColor)
    }
    AxisMarks(values: [0.0]) { _ in
        AxisGridLine(stroke: StrokeStyle(lineWidth: 1)).foregroundStyle(axisColor)
    }
}

/// Category labels under a point chart's plot: every `margin`-th point, the
/// first leading-aligned, the last trailing-aligned and the rest centered,
/// 6 pt below the baseline as in the gallery. Swift Charts ignores
/// `AxisValueLabel(anchor:)` on macOS and drops the trailing label, so the
/// labels are drawn from the chart proxy into the reserved gutter instead.
private struct PointAxisLabels: ViewModifier {
    let labels: [String]
    let margin: Int

    func body(content: Content) -> some View {
        content
            .chartXAxis(.hidden)
            .chartOverlay { proxy in
                GeometryReader { geometry in
                    let plot = proxy.plotFrame.map { geometry[$0] } ?? .zero
                    let marks: [(String, CGFloat, UnitPoint)] = labels.indices
                        .filter { ($0 + 1) % margin == 0 }
                        .compactMap { index in
                            guard let x = proxy.position(forX: index) else { return nil }
                            let anchor: UnitPoint = index == 0
                                ? .topLeading : (index == labels.count - 1 ? .topTrailing : .top)
                            return (labels[index], plot.minX + x, anchor)
                        }
                    Canvas { context, _ in
                        for (text, x, anchor) in marks {
                            context.draw(
                                Text(text).font(chartLabelFont).foregroundStyle(secondaryText),
                                at: CGPoint(x: x, y: plot.maxY + 6),
                                anchor: anchor
                            )
                        }
                    }
                    .frame(width: geometry.size.width, height: geometry.size.height + xAxisGutter)
                }
                .allowsHitTesting(false)
            }
            .padding(.bottom, xAxisGutter)
    }
}

/// Series names beside square swatches.
private struct ChartLegend: View {
    let series: [(String, Color)]

    var body: some View {
        HStack(spacing: 14) {
            ForEach(series, id: \.0) { name, color in
                HStack(spacing: 6) {
                    Rectangle().fill(color).frame(width: 8, height: 8)
                    Text(name).foregroundStyle(.secondary)
                }
            }
        }
    }
}

// MARK: Line chart

private struct LineChartPage: View {
    private let memory: [(String, Double)] = [
        ("08:00", 5.2), ("09:00", 6.8), ("10:00", 8.1), ("11:00", 9.4),
        ("12:00", 8.7), ("13:00", 7.9), ("14:00", 10.2), ("15:00", 11.6),
        ("16:00", 11.1), ("17:00", 9.8), ("18:00", 7.4), ("19:00", 6.1),
    ]

    var body: some View {
        let top = paddedTop(memory.map(\.1).max() ?? 1, padding: 10)
        Text("Memory in use today, in GB").foregroundStyle(.secondary)
        Chart {
            ForEach(memory.indices, id: \.self) { index in
                LineMark(x: .value("Hour", index), y: .value("Memory (GB)", memory[index].1))
                    .interpolationMethod(.catmullRom)
                    .lineStyle(StrokeStyle(lineWidth: 2))
                PointMark(x: .value("Hour", index), y: .value("Memory (GB)", memory[index].1))
                    .symbolSize(64)
            }
        }
        .foregroundStyle(seriesColors[0])
        .chartXScale(domain: 0...(memory.count - 1))
        .chartYScale(domain: 0...top)
        .chartYAxis { quarterGrid(top: top) }
        .frame(height: plotHeight)
        .modifier(PointAxisLabels(labels: memory.map(\.0), margin: 2))
        .accessibilityIdentifier("memory-line")
    }
}

// MARK: Bar chart

private struct BarChartPage: View {
    private let updates: [(String, Int)] = [
        ("Mon", 42), ("Tue", 18), ("Wed", 27), ("Thu", 9), ("Fri", 64), ("Sat", 12), ("Sun", 5),
    ]

    var body: some View {
        // Value labels keep one 12 pt text line clear above the tallest bar.
        let top = paddedTop(Double(updates.map(\.1).max() ?? 1), padding: 12)
        Text("Documents updated each day this week").foregroundStyle(.secondary)
        Chart {
            ForEach(updates, id: \.0) { day, count in
                BarMark(x: .value("Day", day), y: .value("Documents", count), width: .fixed(30))
                    .foregroundStyle(seriesColors[0])
                    .annotation(position: .top, spacing: 2) {
                        Text("\(count)").font(chartLabelFont).foregroundStyle(.primary)
                    }
            }
        }
        .chartYScale(domain: 0...top)
        .chartXAxis {
            AxisMarks { value in
                AxisValueLabel {
                    Text(value.as(String.self) ?? "").font(chartLabelFont).foregroundStyle(secondaryText)
                }
            }
        }
        .chartYAxis { quarterGrid(top: top) }
        .frame(height: chartHeight)
        .accessibilityIdentifier("updates-bar")
    }
}

// MARK: Area chart

private struct AreaChartPage: View {
    private let traffic: [(String, Double, Double)] = [
        ("10:00", 12.4, 1.8), ("10:05", 18.9, 2.6), ("10:10", 42.1, 3.9), ("10:15", 38.6, 5.2),
        ("10:20", 21.3, 4.1), ("10:25", 9.7, 1.2), ("10:30", 15.2, 7.8), ("10:35", 27.8, 9.4),
        ("10:40", 33.5, 6.3), ("10:45", 19.1, 2.2),
    ]

    var body: some View {
        let top = paddedTop(traffic.map(\.1).max() ?? 1, padding: 10)
        Text("Network throughput on Wi-Fi, in MB/s").foregroundStyle(.secondary)
        ChartLegend(series: [("Download", seriesColors[0]), ("Upload", seriesColors[1])])
        Chart {
            // Each series fills from the baseline (not stacked) under a 1 pt stroke.
            ForEach(Array(zip(["Download", "Upload"], seriesColors)), id: \.0) { name, color in
                ForEach(traffic.indices, id: \.self) { index in
                    let value = name == "Download" ? traffic[index].1 : traffic[index].2
                    AreaMark(
                        x: .value("Time", index),
                        yStart: .value("Rate", 0),
                        yEnd: .value("Rate", value),
                        series: .value("Series", name)
                    )
                    .interpolationMethod(.catmullRom)
                    .foregroundStyle(color.opacity(0.2))
                    LineMark(x: .value("Time", index), y: .value("Rate", value), series: .value("Series", name))
                        .interpolationMethod(.catmullRom)
                        .lineStyle(StrokeStyle(lineWidth: 1))
                        .foregroundStyle(color)
                }
            }
        }
        .chartXScale(domain: 0...(traffic.count - 1))
        .chartYScale(domain: 0...top)
        .chartYAxis { quarterGrid(top: top) }
        .chartLegend(.hidden)
        .frame(height: plotHeight)
        .modifier(PointAxisLabels(labels: traffic.map(\.0), margin: 2))
        .accessibilityIdentifier("network-area")
    }
}

// MARK: Pie chart

private struct PieChartPage: View {
    private let usage: [(String, Double)] = [
        ("Movies", 182), ("Projects", 96), ("Photos", 64), ("Caches", 38), ("Other", 22),
    ]
    private let innerRadius: CGFloat = 56
    private let outerRadius: CGFloat = 88
    /// Leader lines end 15 pt past the ring.
    private let labelRadius: CGFloat = 88 + 15

    var body: some View {
        Text("Disk usage by directory, 402 GB of 512 GB").foregroundStyle(.secondary)
        Chart {
            ForEach(usage.indices, id: \.self) { index in
                SectorMark(
                    angle: .value("Used (GB)", usage[index].1),
                    innerRadius: .fixed(innerRadius),
                    outerRadius: .fixed(outerRadius),
                    angularInset: 0.9
                )
                .foregroundStyle(seriesColors[index % seriesColors.count])
            }
        }
        .chartLegend(.hidden)
        .overlay { labels }
        .frame(height: chartHeight)
        .accessibilityIdentifier("disk-pie")
    }

    private struct Placed {
        var edge: CGPoint
        var anchor: CGPoint
        var y: CGFloat
        let text: String
    }

    /// Leader-line labels outside the ring, laid out as the gallery's pie chart
    /// does: ring edge to label anchor, then a horizontal pull to the label radius.
    private var labels: some View {
        Canvas { context, size in
            let center = CGPoint(x: size.width / 2, y: size.height / 2)
            let total = usage.map(\.1).reduce(0, +)
            var right: [Placed] = []
            var left: [Placed] = []
            var start = 0.0
            for (name, gb) in usage {
                let end = start + gb / total * 2 * .pi
                let mid = (start + end) / 2 - .pi / 2
                start = end
                let unit = CGPoint(x: cos(mid), y: sin(mid))
                let anchor = CGPoint(x: unit.x * labelRadius, y: unit.y * labelRadius)
                let placed = Placed(
                    edge: CGPoint(x: unit.x * outerRadius, y: unit.y * outerRadius),
                    anchor: anchor,
                    y: anchor.y,
                    text: "\(name) \(Int(gb.rounded())) GB"
                )
                if anchor.x > 0 { right.append(placed) } else { left.append(placed) }
            }
            let textHeight: CGFloat = 12
            for (side, items) in [(CGFloat(1), spread(right, half: center.y, gap: textHeight)),
                                  (CGFloat(-1), spread(left, half: center.y, gap: textHeight))]
            {
                for item in items {
                    var path = Path()
                    path.move(to: CGPoint(x: center.x + item.edge.x, y: center.y + item.edge.y))
                    path.addLine(to: CGPoint(x: center.x + item.anchor.x, y: center.y + item.y))
                    path.addLine(to: CGPoint(x: center.x + side * labelRadius, y: center.y + item.y))
                    context.stroke(path, with: .color(axisColor), lineWidth: 1)
                    let text = context.resolve(Text(item.text).font(chartLabelFont).foregroundStyle(.primary))
                    context.draw(
                        text,
                        at: CGPoint(x: center.x + side * (labelRadius + 4), y: center.y + item.y),
                        anchor: side > 0 ? .leading : .trailing
                    )
                }
            }
        }
        .allowsHitTesting(false)
    }

    /// Keep neighbours at least one text line apart within the chart height.
    private func spread(_ input: [Placed], half: CGFloat, gap: CGFloat) -> [Placed] {
        var items = input.sorted { $0.y < $1.y }
        guard !items.isEmpty else { return items }
        for i in 1..<items.count where items[i].y < items[i - 1].y + gap {
            items[i].y = items[i - 1].y + gap
        }
        items[items.count - 1].y = min(items[items.count - 1].y, half - gap / 2)
        for i in stride(from: items.count - 2, through: 0, by: -1) where items[i].y > items[i + 1].y - gap {
            items[i].y = items[i + 1].y - gap
        }
        items[0].y = max(items[0].y, -half + gap / 2)
        return items
    }
}
