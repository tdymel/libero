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
    /// The page-size picker's label.
    pub rows_per_page: &'static str,
    /// The shown rows: first and last row number, and the rows over all pages.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.range)(11, 20, 95), "11–20 of 95");
    /// ```
    pub range: fn(usize, usize, usize) -> String,
    /// The page buttons' landmark, for a table without a caption.
    pub pages: &'static str,
    /// The same landmark, named after the table's caption.
    pub pages_of: fn(&str) -> String,
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

fn english_range(from: usize, to: usize, total: usize) -> String {
    format!("{from}–{to} of {total}")
}

fn german_range(from: usize, to: usize, total: usize) -> String {
    format!("{from}–{to} von {total}")
}

fn english_pages_of(table: &str) -> String {
    format!("Pages of {table}")
}

fn german_pages_of(table: &str) -> String {
    format!("Seiten von {table}")
}

impl TableLabels {
    pub const ENGLISH: Self = Self {
        no_rows: "No rows",
        select_all: "Select all rows",
        select_row: english_select_row,
        selected: english_selected,
        sort_order: "sort order {n}",
        rows_per_page: "Rows per page",
        range: english_range,
        pages: "Table pages",
        pages_of: english_pages_of,
    };

    pub const GERMAN: Self = Self {
        no_rows: "Keine Zeilen",
        select_all: "Alle Zeilen auswählen",
        select_row: german_select_row,
        selected: german_selected,
        sort_order: "Sortierreihenfolge {n}",
        rows_per_page: "Zeilen pro Seite",
        range: german_range,
        pages: "Tabellenseiten",
        pages_of: german_pages_of,
    };
}
