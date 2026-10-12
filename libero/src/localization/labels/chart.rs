/// What a chart says beyond its own data. In `range` `{kind}` is the chart's kind,
/// `{first}` and `{last}` the ends of its x axis; in `series` `{name}` is a series,
/// `{min}` and `{max}` its lowest and highest value, `{x}` where the highest is. A pie's
/// `total` and `slice` say `{total}` of all slices, a slice's `{value}` and `{share}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartLabels {
    /// A line chart's kind, the start of its summary.
    pub line: &'static str,
    pub bar: &'static str,
    pub pie: &'static str,
    pub range: &'static str,
    /// `range` of an x axis with one value, `{first}`.
    pub range_one: &'static str,
    pub series: &'static str,
    /// A series with no value at all.
    pub series_empty: &'static str,
    /// The start of a pie's summary.
    pub total: &'static str,
    pub slice: &'static str,
    /// A slice whose value is missing or not a number.
    pub slice_empty: &'static str,
    /// The plot and the summary of a chart with nothing to draw.
    pub no_data: &'static str,
    /// The plot and the summary of a chart whose legend hides every series or slice.
    pub all_hidden: &'static str,
    /// The summary while the chart is loading.
    pub loading: &'static str,
    /// What a screen reader calls the plot the keys walk.
    pub chart: &'static str,
    /// How to walk the plot's points, read after its name.
    pub keys: &'static str,
    /// The legend's name: its buttons show and hide series.
    pub legend: &'static str,
    /// A pie legend's name: its buttons show and hide slices.
    pub slices: &'static str,
    /// A point as the keys announce it: `{name}` its series, `{x}` and `{value}` its own.
    pub point: &'static str,
    /// The x axis' `{title}`, in the summary.
    pub x_axis: &'static str,
    /// The y axis' `{title}`, in the summary.
    pub y_axis: &'static str,
    /// The right y axis' `{title}`, in the summary.
    pub y2_axis: &'static str,
    /// The summary of a line chart with a right y axis: `{names}` the series it measures.
    pub secondary: &'static str,
    /// The summary of a stacked line chart: what its heights mean.
    pub stacked: &'static str,
    /// After `stacked` when a line is on the right y axis, which never stacks.
    pub secondary_unstacked: &'static str,
    /// Read after `keys` on a zoomable plot.
    pub zoom_keys: &'static str,
    /// Read after `keys` on a plot with a brush.
    pub brush_keys: &'static str,
    /// The zoom's reset button.
    pub reset_zoom: &'static str,
    /// After a zoom or a pan: `{first}` and `{last}` the shown ends of the x axis.
    pub showing: &'static str,
    /// The brush's band: `{first}` and `{last}` its ends.
    pub selected: &'static str,
    pub selection_cleared: &'static str,
    pub save_svg: &'static str,
    pub save_png: &'static str,
    /// After an export button: `{name}` the file's name.
    pub saved: &'static str,
    pub save_failed: &'static str,
    /// A mark line's own name, unless given one: at the average, the lowest, the highest
    /// value, or at a fixed one.
    pub average: &'static str,
    pub lowest: &'static str,
    pub highest: &'static str,
    pub mark: &'static str,
    /// Written by a mark line: `{label}` its name, `{value}` where it sits.
    pub mark_line: &'static str,
    /// A mark line in the summary, `{name}` its series.
    pub mark_line_summary: &'static str,
    /// A series' error ranges in the summary: `{low}` and `{high}` their lowest and highest ends.
    pub band: &'static str,
    /// A marked point in the summary: `{label}` is `lowest` or `highest`, `{x}` where it is.
    pub mark_point: &'static str,
}

impl ChartLabels {
    pub const ENGLISH: Self = Self {
        line: "Line chart",
        bar: "Bar chart",
        pie: "Pie chart",
        range: "{kind} from {first} to {last}.",
        range_one: "{kind} for {first}.",
        series: "{name}: {min} to {max}, highest at {x}.",
        series_empty: "{name}: no values.",
        total: "{kind}, total {total}.",
        slice: "{name}: {value}, {share}.",
        slice_empty: "{name}: no value.",
        no_data: "No data",
        all_hidden: "All series are hidden.",
        loading: "Loading chart",
        chart: "chart",
        keys: "Arrow keys move between points, Enter selects one.",
        legend: "Series",
        slices: "Slices",
        point: "{name}, {x}: {value}",
        x_axis: "X axis: {title}.",
        y_axis: "Y axis: {title}.",
        y2_axis: "Right y axis: {title}.",
        secondary: "On the right y axis: {names}.",
        stacked: "Stacked: each line adds to the ones before it, the top one is the total.",
        secondary_unstacked: "Lines on the right y axis are not stacked.",
        zoom_keys: "Plus and minus zoom, 0 resets the zoom.",
        brush_keys: "Shift with an arrow key selects a range, Escape clears it.",
        reset_zoom: "Reset zoom",
        showing: "Showing {first} to {last}",
        selected: "Selected {first} to {last}",
        selection_cleared: "Selection cleared",
        save_svg: "Save as SVG",
        save_png: "Save as PNG",
        saved: "Saved {name}",
        save_failed: "Could not save {name}",
        average: "Average",
        lowest: "Lowest",
        highest: "Highest",
        mark: "Mark",
        mark_line: "{label}: {value}",
        mark_line_summary: "{name}: {label} line at {value}.",
        mark_point: "{name}: {label} marked at {x}, {value}.",
        band: "{name}: each value has a range, {low} to {high} in all.",
    };

    pub const GERMAN: Self = Self {
        line: "Liniendiagramm",
        bar: "Balkendiagramm",
        pie: "Kreisdiagramm",
        range: "{kind} von {first} bis {last}.",
        range_one: "{kind} für {first}.",
        series: "{name}: {min} bis {max}, am höchsten bei {x}.",
        series_empty: "{name}: keine Werte.",
        total: "{kind}, insgesamt {total}.",
        slice: "{name}: {value}, {share}.",
        slice_empty: "{name}: kein Wert.",
        no_data: "Keine Daten",
        all_hidden: "Alle Reihen sind ausgeblendet.",
        loading: "Diagramm wird geladen",
        chart: "Diagramm",
        keys: "Pfeiltasten wechseln zwischen den Punkten, Eingabe wählt einen aus.",
        legend: "Reihen",
        slices: "Segmente",
        point: "{name}, {x}: {value}",
        x_axis: "X-Achse: {title}.",
        y_axis: "Y-Achse: {title}.",
        y2_axis: "Rechte Y-Achse: {title}.",
        secondary: "Auf der rechten Y-Achse: {names}.",
        stacked: "Gestapelt: jede Linie addiert sich zu den vorigen, die oberste ist die Summe.",
        secondary_unstacked: "Linien auf der rechten Y-Achse sind nicht gestapelt.",
        zoom_keys: "Plus und Minus zoomen, 0 setzt den Zoom zurück.",
        brush_keys: "Umschalt mit einer Pfeiltaste wählt einen Bereich, Escape hebt ihn auf.",
        reset_zoom: "Zoom zurücksetzen",
        showing: "Zeigt {first} bis {last}",
        selected: "{first} bis {last} ausgewählt",
        selection_cleared: "Auswahl aufgehoben",
        save_svg: "Als SVG speichern",
        save_png: "Als PNG speichern",
        saved: "{name} gespeichert",
        save_failed: "{name} konnte nicht gespeichert werden",
        average: "Durchschnitt",
        lowest: "Tiefstwert",
        highest: "Höchstwert",
        mark: "Markierung",
        mark_line: "{label}: {value}",
        mark_line_summary: "{name}: Linie {label} bei {value}.",
        mark_point: "{name}: {label} markiert bei {x}, {value}.",
        band: "{name}: jeder Wert hat einen Bereich, insgesamt {low} bis {high}.",
    };
}
