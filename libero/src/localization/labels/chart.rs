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
    pub funnel: &'static str,
    pub scatter: &'static str,
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
    /// A funnel's stage: `{value}` and `{share}`, its share of the first stage.
    pub stage: &'static str,
    /// A stage when the first has no value to share.
    pub stage_value: &'static str,
    pub heatmap: &'static str,
    /// The start of a heatmap's summary: `{rows}`, `{columns}`, its lowest and highest value.
    pub heatmap_range: &'static str,
    /// A heatmap's `{high}` and `{low}` cell, each as `point` writes it.
    pub cell_extremes: &'static str,
    /// A heatmap's row in its summary: `{name}` and its `{cells}`, each `cell` or `cell_empty`.
    pub row: &'static str,
    pub cell: &'static str,
    pub cell_empty: &'static str,
    /// The key of an empty cell under a heatmap's scale.
    pub no_value: &'static str,
    /// Ends a large heatmap's summary in place of its rows.
    pub cells_by_keys: &'static str,
    pub candlestick: &'static str,
    /// A candlestick chart's series name when it has none.
    pub price: &'static str,
    /// A candle's prices, `{open}`, `{high}`, `{low}`, `{close}`, and its `{direction}`.
    pub candle: &'static str,
    pub up: &'static str,
    pub down: &'static str,
    pub unchanged: &'static str,
    /// The keys of a rising and a falling candle under the plot.
    pub up_key: &'static str,
    pub down_key: &'static str,
    /// The start of a candlestick chart's summary: `{count}` periods, `{first}` to `{last}`.
    pub candle_range: &'static str,
    /// The first `{open}` and the last `{close}`, and the `{change}` between them.
    pub candle_change: &'static str,
    /// The highest `{high}` on `{high_x}` and the lowest `{low}` on `{low_x}`.
    pub candle_extremes: &'static str,
    /// How many candles rose (`{up}`), fell (`{down}`) and stayed (`{flat}`).
    pub candle_count: &'static str,
    /// In a pie's summary while the legend hides a slice: its shares count the hidden ones.
    pub shares_of_all: &'static str,
    /// The plot and the summary of a chart with nothing to draw.
    pub no_data: &'static str,
    /// The plot and the summary of a chart whose legend hides every series or slice.
    pub all_hidden: &'static str,
    /// The summary while the chart is loading, said once when it starts.
    pub loading: &'static str,
    /// What a screen reader calls the plot the keys walk.
    pub chart: &'static str,
    /// How to walk the plot's points, read after its name.
    pub keys: &'static str,
    /// The legend's name: its buttons show and hide series.
    pub legend: &'static str,
    /// A pie legend's name: its buttons show and hide slices.
    pub slices: &'static str,
    /// A funnel legend's name.
    pub stages: &'static str,
    /// A point as the keys announce it: `{name}` its series, `{x}` and `{value}` its own.
    pub point: &'static str,
    /// A scatter point as the keys announce it: `{value}` is its `scatter_value`.
    pub point_free: &'static str,
    /// A scatter point's `{x}` and `{y}`, in its tooltip and as the keys say it.
    pub scatter_value: &'static str,
    /// The same for a bubble, with its `{size}`.
    pub bubble_value: &'static str,
    /// A scatter series in the summary: `{count}` points, x from `{xmin}` to `{xmax}`, y from
    /// `{min}` to `{max}`.
    pub scatter_series: &'static str,
    /// After a bubble series' sentence: its sizes from `{min}` to `{max}`.
    pub bubble_sizes: &'static str,
    /// A scatter series' trend line in the summary: `{kind}` one of `trends`, `{direction}`
    /// one of `directions`, as it runs left to right.
    pub trend: &'static str,
    /// Linear, exponential, logarithmic and polynomial.
    pub trends: [&'static str; 4],
    /// Rising, falling and flat.
    pub directions: [&'static str; 3],
    /// The x axis' `{title}`, in the summary.
    pub x_axis: &'static str,
    /// The y axis' `{title}`, in the summary.
    pub y_axis: &'static str,
    /// The right y axis' `{title}`, in the summary.
    pub y2_axis: &'static str,
    /// The summary of a line chart with a right y axis: `{names}` the series it measures.
    pub secondary: &'static str,
    /// The summary of a bar chart with lines: `{names}` the series drawn as lines.
    pub lines: &'static str,
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
    /// Written after a long axis' ticks in millions, billions and trillions.
    pub compact: [&'static str; 3],
}

impl ChartLabels {
    pub const ENGLISH: Self = Self {
        line: "Line chart",
        bar: "Bar chart",
        pie: "Pie chart",
        funnel: "Funnel chart",
        scatter: "Scatter chart",
        range: "{kind} from {first} to {last}.",
        range_one: "{kind} for {first}.",
        series: "{name}: {min} to {max}, highest at {x}.",
        series_empty: "{name}: no values.",
        total: "{kind}, total {total}.",
        slice: "{name}: {value}, {share}.",
        slice_empty: "{name}: no value.",
        stage: "{name}: {value}, {share} of the first.",
        stage_value: "{name}: {value}.",
        heatmap: "Heatmap",
        heatmap_range: "{kind} of {rows} rows by {columns} columns, values {min} to {max}.",
        cell_extremes: "Highest: {high}. Lowest: {low}.",
        row: "{name}: {cells}.",
        cell: "{x} {value}",
        cell_empty: "{x} no value",
        no_value: "No value",
        cells_by_keys: "Use the arrow keys to read each cell.",
        candlestick: "Candlestick chart",
        price: "Price",
        candle: "open {open}, high {high}, low {low}, close {close}, {direction}",
        up: "up",
        down: "down",
        unchanged: "unchanged",
        up_key: "Up",
        down_key: "Down",
        candle_range: "{kind} of {count} periods from {first} to {last}.",
        candle_change: "Opened at {open}, closed at {close}: {change}.",
        candle_extremes: "Highest: {high} on {high_x}. Lowest: {low} on {low_x}.",
        candle_count: "{up} up, {down} down, {flat} unchanged.",
        shares_of_all: "Shares count the hidden slices too.",
        no_data: "No data",
        all_hidden: "All series are hidden.",
        loading: "Loading chart",
        chart: "chart",
        keys: "Arrow keys move between points, Enter selects one.",
        legend: "Series",
        slices: "Slices",
        stages: "Stages",
        point: "{name}, {x}: {value}",
        point_free: "{name}: {value}",
        scatter_value: "x {x}, y {y}",
        bubble_value: "x {x}, y {y}, size {size}",
        scatter_series: "{name}: {count} points, x from {xmin} to {xmax}, y from {min} to {max}.",
        bubble_sizes: "{name}: sizes from {min} to {max}.",
        trend: "{name}: {kind} trend, {direction}.",
        trends: ["linear", "exponential", "logarithmic", "polynomial"],
        directions: ["rising", "falling", "flat"],
        x_axis: "X axis: {title}.",
        y_axis: "Y axis: {title}.",
        y2_axis: "Right y axis: {title}.",
        secondary: "On the right y axis: {names}.",
        lines: "Drawn as a line over the bars: {names}.",
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
        compact: ["M", "B", "T"],
    };

    pub const GERMAN: Self = Self {
        line: "Liniendiagramm",
        bar: "Balkendiagramm",
        pie: "Kreisdiagramm",
        funnel: "Trichterdiagramm",
        scatter: "Streudiagramm",
        range: "{kind} von {first} bis {last}.",
        range_one: "{kind} für {first}.",
        series: "{name}: {min} bis {max}, am höchsten bei {x}.",
        series_empty: "{name}: keine Werte.",
        total: "{kind}, insgesamt {total}.",
        slice: "{name}: {value}, {share}.",
        slice_empty: "{name}: kein Wert.",
        stage: "{name}: {value}, {share} der ersten Stufe.",
        stage_value: "{name}: {value}.",
        heatmap: "Heatmap",
        heatmap_range: "{kind} mit {rows} Zeilen und {columns} Spalten, Werte von {min} bis {max}.",
        cell_extremes: "Höchster Wert: {high}. Niedrigster Wert: {low}.",
        row: "{name}: {cells}.",
        cell: "{x} {value}",
        cell_empty: "{x} kein Wert",
        no_value: "Kein Wert",
        cells_by_keys: "Mit den Pfeiltasten lässt sich jede Zelle lesen.",
        candlestick: "Kerzendiagramm",
        price: "Kurs",
        candle: "Eröffnung {open}, Hoch {high}, Tief {low}, Schluss {close}, {direction}",
        up: "gestiegen",
        down: "gefallen",
        unchanged: "unverändert",
        up_key: "Gestiegen",
        down_key: "Gefallen",
        candle_range: "{kind} mit {count} Perioden von {first} bis {last}.",
        candle_change: "Eröffnet bei {open}, geschlossen bei {close}: {change}.",
        candle_extremes: "Höchster Kurs: {high} am {high_x}. Tiefster Kurs: {low} am {low_x}.",
        candle_count: "{up} gestiegen, {down} gefallen, {flat} unverändert.",
        shares_of_all: "Die Anteile zählen die ausgeblendeten Segmente mit.",
        no_data: "Keine Daten",
        all_hidden: "Alle Reihen sind ausgeblendet.",
        loading: "Diagramm wird geladen",
        chart: "Diagramm",
        keys: "Pfeiltasten wechseln zwischen den Punkten, Eingabe wählt einen aus.",
        legend: "Reihen",
        slices: "Segmente",
        stages: "Stufen",
        point: "{name}, {x}: {value}",
        point_free: "{name}: {value}",
        scatter_value: "x {x}, y {y}",
        bubble_value: "x {x}, y {y}, Größe {size}",
        scatter_series: "{name}: {count} Punkte, x von {xmin} bis {xmax}, y von {min} bis {max}.",
        bubble_sizes: "{name}: Größen von {min} bis {max}.",
        trend: "{name}: {kind}er Trend, {direction}.",
        trends: ["linear", "exponentiell", "logarithmisch", "polynomial"],
        directions: ["steigend", "fallend", "flach"],
        x_axis: "X-Achse: {title}.",
        y_axis: "Y-Achse: {title}.",
        y2_axis: "Rechte Y-Achse: {title}.",
        secondary: "Auf der rechten Y-Achse: {names}.",
        lines: "Als Linie über den Balken: {names}.",
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
        compact: ["\u{a0}Mio.", "\u{a0}Mrd.", "\u{a0}Bio."],
    };
}
