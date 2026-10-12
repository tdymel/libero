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
    pub series: &'static str,
    /// A series with no value at all.
    pub series_empty: &'static str,
    /// The start of a pie's summary.
    pub total: &'static str,
    pub slice: &'static str,
    /// The plot and the summary of a chart with nothing to draw.
    pub no_data: &'static str,
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
}

impl ChartLabels {
    pub const ENGLISH: Self = Self {
        line: "Line chart",
        bar: "Bar chart",
        pie: "Pie chart",
        range: "{kind} from {first} to {last}.",
        series: "{name}: {min} to {max}, highest at {x}.",
        series_empty: "{name}: no values.",
        total: "{kind}, total {total}.",
        slice: "{name}: {value}, {share}.",
        no_data: "No data",
        loading: "Loading chart",
        chart: "chart",
        keys: "Arrow keys move between points, Enter selects one.",
        legend: "Series",
        slices: "Slices",
        point: "{name}, {x}: {value}",
    };

    pub const GERMAN: Self = Self {
        line: "Liniendiagramm",
        bar: "Balkendiagramm",
        pie: "Kreisdiagramm",
        range: "{kind} von {first} bis {last}.",
        series: "{name}: {min} bis {max}, am höchsten bei {x}.",
        series_empty: "{name}: keine Werte.",
        total: "{kind}, insgesamt {total}.",
        slice: "{name}: {value}, {share}.",
        no_data: "Keine Daten",
        loading: "Diagramm wird geladen",
        chart: "Diagramm",
        keys: "Pfeiltasten wechseln zwischen den Punkten, Eingabe wählt einen aus.",
        legend: "Reihen",
        slices: "Segmente",
        point: "{name}, {x}: {value}",
    };
}
