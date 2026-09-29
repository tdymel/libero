use crate::theme::Size;

/// How a `ColumnFilter` compares a cell. Text operators ignore case, `Equals`
/// too; number operators compare the value, not the formatted text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FilterOperator {
    Contains,
    DoesNotContain,
    Equals,
    NotEquals,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    IsEmpty,
    IsNotEmpty,
    /// A boolean column's value, `"true"` or `"false"`.
    Is,
    /// Date operators: the value is an ISO day, `2024-03-09`.
    Before,
    After,
    OnOrBefore,
    OnOrAfter,
    /// From `value` to `value_to`, both days kept, in either order; an empty end is open.
    Between,
}

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
    /// A header's column menu button, from the header text.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.column_menu)("Age"), "Age column options");
    /// ```
    pub column_menu: fn(&str) -> String,
    pub sort_ascending: &'static str,
    pub sort_descending: &'static str,
    /// Drops the column from the sort.
    pub unsort: &'static str,
    /// Sorts by the column after the sorted ones, with `multi_sort`.
    pub add_to_sort: &'static str,
    pub hide_column: &'static str,
    /// The submenu that shows and hides columns.
    pub columns: &'static str,
    /// The quick-filter field's label.
    pub search: &'static str,
    /// The full-width row when the quick filter leaves no rows.
    pub no_results: &'static str,
    /// Announced once typing in the quick filter settles. A fn, for plural forms.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.results)(1), "1 row");
    /// assert_eq!((TableLabels::GERMAN.results)(12), "12 Zeilen");
    /// ```
    pub results: fn(usize) -> String,
    /// The progress bar over a `loading` table that still shows rows.
    pub loading: &'static str,
    /// Holds the column at the table's start edge, the left in a left-to-right page.
    pub pin_start: &'static str,
    pub pin_end: &'static str,
    pub unpin: &'static str,
    /// Moves the column one place left on screen, whichever the text direction.
    pub move_column_left: &'static str,
    pub move_column_right: &'static str,
    /// Steps a resizable column's width; the drag-free way to resize it.
    pub widen_column: &'static str,
    pub narrow_column: &'static str,
    /// Drops a resized column's width, back to its own.
    pub reset_column_width: &'static str,
    /// Announced after a menu Widen or Narrow step: the header and the new width in px.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.column_width)("Name", 240.0), "Name: 240 px");
    /// ```
    pub column_width: fn(&str, f64) -> String,
    /// Announced after a menu Reset width, from the header text.
    pub column_width_reset: fn(&str) -> String,
    /// A header's resize grip, a separator the arrow keys move, from the header text.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.resize_column)("Name"), "Resize Name");
    /// assert_eq!((TableLabels::GERMAN.resize_column)("Name"), "Breite von Name ändern");
    /// ```
    pub resize_column: fn(&str) -> String,
    /// The header of the column of row reorder handles, read out only.
    pub reorder: &'static str,
    /// The header of the column of detail toggles, read out only.
    pub details: &'static str,
    /// A row's detail toggle, from the row header's text, else the first cell's.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.row_details)("Ada"), "Details for Ada");
    /// ```
    pub row_details: fn(&str) -> String,
    /// The column menu entry that opens the column's filter.
    pub filter: &'static str,
    /// The filter popover and a header filter field, from the header text.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.filter_column)("Age"), "Filter Age");
    /// ```
    pub filter_column: fn(&str) -> String,
    /// The button a filtered header shows, which opens its filter.
    pub filtered: fn(&str) -> String,
    /// The filter's operator picker.
    pub operator: &'static str,
    /// The filter's value field.
    pub value: &'static str,
    pub clear_filter: &'static str,
    /// A boolean filter that keeps every row.
    pub any: &'static str,
    /// A boolean filter's `true` and `false`.
    pub yes: &'static str,
    pub no: &'static str,
    /// A `Between` date filter's first and last day fields.
    pub date_from: &'static str,
    pub date_to: &'static str,
    /// An operator's name in the operator picker.
    ///
    /// ```
    /// use libero::{components::FilterOperator, localization::TableLabels};
    ///
    /// assert_eq!((TableLabels::ENGLISH.operator_name)(FilterOperator::GreaterThan), "Greater than");
    /// ```
    pub operator_name: fn(FilterOperator) -> &'static str,
    /// The toolbar's `TableDensityButton`, which picks the row height.
    pub density: &'static str,
    /// A density's name in its menu: small, medium and large.
    ///
    /// ```
    /// use libero::{localization::TableLabels, theme::Size};
    ///
    /// assert_eq!((TableLabels::ENGLISH.density_name)(Size::Sm), "Compact");
    /// ```
    pub density_name: fn(Size) -> &'static str,
    /// The toolbar's `TableExportButton`.
    pub export: &'static str,
}

fn english_filter_column(column: &str) -> String {
    format!("Filter {column}")
}

fn german_filter_column(column: &str) -> String {
    format!("{column} filtern")
}

fn english_filtered(column: &str) -> String {
    format!("{column} is filtered")
}

fn german_filtered(column: &str) -> String {
    format!("{column} ist gefiltert")
}

fn english_operator_name(operator: FilterOperator) -> &'static str {
    use FilterOperator::*;
    match operator {
        Contains => "Contains",
        DoesNotContain => "Does not contain",
        Equals => "Equals",
        NotEquals => "Does not equal",
        StartsWith => "Starts with",
        EndsWith => "Ends with",
        GreaterThan => "Greater than",
        GreaterOrEqual => "Greater than or equal",
        LessThan => "Less than",
        LessOrEqual => "Less than or equal",
        IsEmpty => "Is empty",
        IsNotEmpty => "Is not empty",
        Is => "Is",
        Before => "Before",
        After => "After",
        OnOrBefore => "On or before",
        OnOrAfter => "On or after",
        Between => "Between",
    }
}

fn german_operator_name(operator: FilterOperator) -> &'static str {
    use FilterOperator::*;
    match operator {
        Contains => "Enthält",
        DoesNotContain => "Enthält nicht",
        Equals => "Ist gleich",
        NotEquals => "Ist ungleich",
        StartsWith => "Beginnt mit",
        EndsWith => "Endet mit",
        GreaterThan => "Größer als",
        GreaterOrEqual => "Größer oder gleich",
        LessThan => "Kleiner als",
        LessOrEqual => "Kleiner oder gleich",
        IsEmpty => "Ist leer",
        IsNotEmpty => "Ist nicht leer",
        Is => "Ist",
        Before => "Vor",
        After => "Nach",
        OnOrBefore => "Am oder vor",
        OnOrAfter => "Am oder nach",
        Between => "Zwischen",
    }
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

fn english_results(count: usize) -> String {
    match count {
        1 => "1 row".to_string(),
        _ => format!("{count} rows"),
    }
}

fn german_results(count: usize) -> String {
    match count {
        1 => "1 Zeile".to_string(),
        _ => format!("{count} Zeilen"),
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

fn english_column_menu(column: &str) -> String {
    format!("{column} column options")
}

fn german_column_menu(column: &str) -> String {
    format!("Optionen für Spalte {column}")
}

fn column_width(column: &str, width: f64) -> String {
    format!("{column}: {width} px")
}

fn english_column_width_reset(column: &str) -> String {
    format!("{column}: width reset")
}

fn german_column_width_reset(column: &str) -> String {
    format!("{column}: Breite zurückgesetzt")
}

fn english_resize_column(column: &str) -> String {
    format!("Resize {column}")
}

fn german_resize_column(column: &str) -> String {
    format!("Breite von {column} ändern")
}

fn english_density_name(size: Size) -> &'static str {
    match size {
        Size::Xs | Size::Sm => "Compact",
        Size::Md => "Standard",
        _ => "Comfortable",
    }
}

fn german_density_name(size: Size) -> &'static str {
    match size {
        Size::Xs | Size::Sm => "Kompakt",
        Size::Md => "Standard",
        _ => "Komfortabel",
    }
}

fn english_row_details(row: &str) -> String {
    format!("Details for {row}")
}

fn german_row_details(row: &str) -> String {
    format!("Details zu {row}")
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
        column_menu: english_column_menu,
        sort_ascending: "Sort ascending",
        sort_descending: "Sort descending",
        unsort: "Unsort",
        add_to_sort: "Add to sort",
        hide_column: "Hide column",
        columns: "Columns",
        search: "Search",
        no_results: "No matching rows",
        results: english_results,
        loading: "Loading rows",
        pin_start: "Pin to start",
        pin_end: "Pin to end",
        unpin: "Unpin",
        move_column_left: "Move left",
        move_column_right: "Move right",
        widen_column: "Widen column",
        narrow_column: "Narrow column",
        reset_column_width: "Reset width",
        column_width,
        column_width_reset: english_column_width_reset,
        resize_column: english_resize_column,
        reorder: "Reorder",
        details: "Details",
        row_details: english_row_details,
        filter: "Filter",
        filter_column: english_filter_column,
        filtered: english_filtered,
        operator: "Operator",
        value: "Value",
        clear_filter: "Clear filter",
        any: "Any",
        yes: "Yes",
        no: "No",
        date_from: "From",
        date_to: "To",
        operator_name: english_operator_name,
        density: "Density",
        density_name: english_density_name,
        export: "Export",
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
        column_menu: german_column_menu,
        sort_ascending: "Aufsteigend sortieren",
        sort_descending: "Absteigend sortieren",
        unsort: "Sortierung aufheben",
        add_to_sort: "Zur Sortierung hinzufügen",
        hide_column: "Spalte ausblenden",
        columns: "Spalten",
        search: "Suchen",
        no_results: "Keine passenden Zeilen",
        results: german_results,
        loading: "Zeilen werden geladen",
        pin_start: "Am Anfang fixieren",
        pin_end: "Am Ende fixieren",
        unpin: "Fixierung lösen",
        move_column_left: "Nach links",
        move_column_right: "Nach rechts",
        widen_column: "Spalte verbreitern",
        narrow_column: "Spalte verschmälern",
        reset_column_width: "Breite zurücksetzen",
        column_width,
        column_width_reset: german_column_width_reset,
        resize_column: german_resize_column,
        reorder: "Neu anordnen",
        details: "Details",
        row_details: german_row_details,
        filter: "Filtern",
        filter_column: german_filter_column,
        filtered: german_filtered,
        operator: "Operator",
        value: "Wert",
        clear_filter: "Filter entfernen",
        any: "Alle",
        yes: "Ja",
        no: "Nein",
        date_from: "Von",
        date_to: "Bis",
        operator_name: german_operator_name,
        density: "Zeilenhöhe",
        density_name: german_density_name,
        export: "Exportieren",
    };
}
