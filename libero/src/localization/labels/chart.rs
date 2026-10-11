/// What a chart says beyond its own data. In `range` `{kind}` is the chart's kind,
/// `{first}` and `{last}` the ends of its x axis; in `series` `{name}` is a series,
/// `{min}` and `{max}` its lowest and highest value, `{x}` where the highest is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartLabels {
    /// A line chart's kind, the start of its summary.
    pub line: &'static str,
    pub range: &'static str,
    pub series: &'static str,
    /// A series with no value at all.
    pub series_empty: &'static str,
    /// The plot and the summary of a chart with nothing to draw.
    pub no_data: &'static str,
    /// The summary while the chart is loading.
    pub loading: &'static str,
    /// The data table's caption without a `title`.
    pub table: &'static str,
    /// The header of the table's first column, the x values.
    pub category: &'static str,
}

impl ChartLabels {
    pub const ENGLISH: Self = Self {
        line: "Line chart",
        range: "{kind} from {first} to {last}.",
        series: "{name}: {min} to {max}, highest at {x}.",
        series_empty: "{name}: no values.",
        no_data: "No data",
        loading: "Loading chart",
        table: "Chart data",
        category: "Category",
    };

    pub const GERMAN: Self = Self {
        line: "Liniendiagramm",
        range: "{kind} von {first} bis {last}.",
        series: "{name}: {min} bis {max}, am höchsten bei {x}.",
        series_empty: "{name}: keine Werte.",
        no_data: "Keine Daten",
        loading: "Diagramm wird geladen",
        table: "Diagrammdaten",
        category: "Kategorie",
    };
}
