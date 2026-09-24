/// Every string `Table` puts in front of a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct TableLabels {
    /// The full-width row of a table with no data, when its `empty` is unset.
    pub no_rows: &'static str,
    /// The header checkbox of a `selectable` table.
    pub select_all: &'static str,
    /// A row's checkbox, from the row header's text, else the first cell's.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.select_row)("Ada"), "Select Ada");
    /// assert_eq!((TableLabels::GERMAN.select_row)("Ada"), "Ada auswählen");
    /// ```
    pub select_row: fn(&str) -> String,
    /// Announced after a selection change. A fn, for plural forms.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.selected)(1), "1 row selected");
    /// assert_eq!((TableLabels::GERMAN.selected)(3), "3 Zeilen ausgewählt");
    /// ```
    pub selected: fn(usize) -> String,
    /// A sorted header's place among several sorted columns. `{n}` is 1 for the
    /// column that sorts first.
    pub sort_order: &'static str,
}

/// `TableLabels::ENGLISH.select_row`. A named fn, so every copy compares equal.
fn english_select_row(row: &str) -> String {
    format!("Select {row}")
}

/// `TableLabels::GERMAN.select_row`.
fn german_select_row(row: &str) -> String {
    format!("{row} auswählen")
}

/// `TableLabels::ENGLISH.selected`.
fn english_selected(count: usize) -> String {
    match count {
        1 => "1 row selected".to_string(),
        _ => format!("{count} rows selected"),
    }
}

/// `TableLabels::GERMAN.selected`.
fn german_selected(count: usize) -> String {
    match count {
        1 => "1 Zeile ausgewählt".to_string(),
        _ => format!("{count} Zeilen ausgewählt"),
    }
}

impl TableLabels {
    pub const ENGLISH: Self = Self {
        no_rows: "No rows",
        select_all: "Select all rows",
        select_row: english_select_row,
        selected: english_selected,
        sort_order: "sort order {n}",
    };

    pub const GERMAN: Self = Self {
        no_rows: "Keine Zeilen",
        select_all: "Alle Zeilen auswählen",
        select_row: german_select_row,
        selected: german_selected,
        sort_order: "Sortierreihenfolge {n}",
    };
}
