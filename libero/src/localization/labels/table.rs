/// Every string `Table` puts in front of a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableLabels {
    /// The full-width row of a table with no data, when its `empty` is unset.
    pub no_rows: &'static str,
}

impl TableLabels {
    pub const ENGLISH: Self = Self { no_rows: "No rows" };

    pub const GERMAN: Self = Self {
        no_rows: "Keine Zeilen",
    };
}
