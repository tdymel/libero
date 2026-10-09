use crate::theme::Size;

/// How a table's column filters join: a row stays when it passes all of them,
/// or any one. The quick filter applies on top either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FilterLogic {
    #[default]
    And,
    Or,
}

/// How a column's footer cell sums up the rows that pass the filters. `Sum`,
/// `Avg`, `Min` and `Max` read the numeric cells; `Count` counts the filled ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Aggregate {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

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
    /// Describes the disabled Hide column and checkbox of the last shown column.
    pub last_column: &'static str,
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
    /// Announced after a menu Move left or right: the header, its place among the
    /// shown columns from 1, and their count.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.column_moved)("Name", 2, 5), "Name moved to column 2 of 5");
    /// assert_eq!((TableLabels::GERMAN.column_moved)("Name", 2, 5), "Name an Spalte 2 von 5 verschoben");
    /// ```
    pub column_moved: fn(&str, usize, usize) -> String,
    /// Announced after a menu pin or unpin, from the header text.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.column_pinned_start)("Name"), "Name pinned to start");
    /// assert_eq!((TableLabels::GERMAN.column_unpinned)("Name"), "Fixierung von Name gelöst");
    /// ```
    pub column_pinned_start: fn(&str) -> String,
    pub column_pinned_end: fn(&str) -> String,
    pub column_unpinned: fn(&str) -> String,
    /// Announced after the column menu hides its own column, from the header text.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.column_hidden)("Name"), "Name hidden");
    /// ```
    pub column_hidden: fn(&str) -> String,
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
    /// Describes the row reorder controls while a sort or filter turns them off.
    pub reorder_unavailable: &'static str,
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
    /// Announced after an export, from the rows it holds. A fn, for plural forms.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.exported)(12), "Exported 12 rows");
    /// assert_eq!((TableLabels::GERMAN.exported)(1), "1 Zeile exportiert");
    /// ```
    pub exported: fn(usize) -> String,
    /// The `filter_panel` button's text and the panel's name.
    pub filters: &'static str,
    /// The `filter_panel` button's name, from the active filters' count.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::ENGLISH.active_filters)(2), "Filters, 2 active");
    /// assert_eq!((TableLabels::ENGLISH.active_filters)(0), "Filters");
    /// ```
    pub active_filters: fn(usize) -> String,
    pub add_filter: &'static str,
    /// The filter panel's pick of how its filters join.
    pub logic: &'static str,
    pub logic_name: fn(FilterLogic) -> &'static str,
    /// The name a footer cell shows before its value.
    ///
    /// ```
    /// use libero::localization::{Aggregate, TableLabels};
    ///
    /// assert_eq!((TableLabels::ENGLISH.aggregate_name)(Aggregate::Avg), "Average");
    /// assert_eq!((TableLabels::GERMAN.aggregate_name)(Aggregate::Sum), "Summe");
    /// ```
    pub aggregate_name: fn(Aggregate) -> &'static str,
    /// A panel line's controls, from its column's header.
    ///
    /// ```
    /// use libero::localization::TableLabels;
    ///
    /// assert_eq!((TableLabels::GERMAN.filter_line_value)("Name"), "Name: Wert");
    /// assert_eq!((TableLabels::ENGLISH.remove_filter)("Name"), "Remove Name filter");
    /// ```
    pub filter_line_column: fn(&str) -> String,
    pub filter_line_operator: fn(&str) -> String,
    pub filter_line_value: fn(&str) -> String,
    pub remove_filter: fn(&str) -> String,
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

fn english_column_moved(column: &str, place: usize, count: usize) -> String {
    format!("{column} moved to column {place} of {count}")
}

fn german_column_moved(column: &str, place: usize, count: usize) -> String {
    format!("{column} an Spalte {place} von {count} verschoben")
}

fn english_pinned_start(column: &str) -> String {
    format!("{column} pinned to start")
}

fn german_pinned_start(column: &str) -> String {
    format!("{column} am Anfang fixiert")
}

fn english_pinned_end(column: &str) -> String {
    format!("{column} pinned to end")
}

fn german_pinned_end(column: &str) -> String {
    format!("{column} am Ende fixiert")
}

fn english_unpinned(column: &str) -> String {
    format!("{column} unpinned")
}

fn german_unpinned(column: &str) -> String {
    format!("Fixierung von {column} gelöst")
}

fn english_column_hidden(column: &str) -> String {
    format!("{column} hidden")
}

fn german_column_hidden(column: &str) -> String {
    format!("{column} ausgeblendet")
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

fn english_exported(count: usize) -> String {
    match count {
        1 => "Exported 1 row".to_string(),
        _ => format!("Exported {count} rows"),
    }
}

fn german_exported(count: usize) -> String {
    match count {
        1 => "1 Zeile exportiert".to_string(),
        _ => format!("{count} Zeilen exportiert"),
    }
}

fn english_active_filters(count: usize) -> String {
    match count {
        0 => "Filters".to_string(),
        _ => format!("Filters, {count} active"),
    }
}

fn german_active_filters(count: usize) -> String {
    match count {
        0 => "Filter".to_string(),
        _ => format!("Filter, {count} aktiv"),
    }
}

fn english_logic_name(logic: FilterLogic) -> &'static str {
    match logic {
        FilterLogic::And => "All filters",
        FilterLogic::Or => "Any filter",
    }
}

fn german_logic_name(logic: FilterLogic) -> &'static str {
    match logic {
        FilterLogic::And => "Alle Filter",
        FilterLogic::Or => "Ein beliebiger Filter",
    }
}

fn english_line_column(column: &str) -> String {
    format!("{column}: column")
}

fn german_line_column(column: &str) -> String {
    format!("{column}: Spalte")
}

fn line_operator(column: &str) -> String {
    format!("{column}: operator")
}

fn german_line_operator(column: &str) -> String {
    format!("{column}: Operator")
}

fn english_line_value(column: &str) -> String {
    format!("{column}: value")
}

fn german_line_value(column: &str) -> String {
    format!("{column}: Wert")
}

fn english_remove_filter(column: &str) -> String {
    format!("Remove {column} filter")
}

fn german_remove_filter(column: &str) -> String {
    format!("{column}-Filter entfernen")
}

fn english_aggregate_name(aggregate: Aggregate) -> &'static str {
    match aggregate {
        Aggregate::Sum => "Sum",
        Aggregate::Avg => "Average",
        Aggregate::Min => "Minimum",
        Aggregate::Max => "Maximum",
        Aggregate::Count => "Count",
    }
}

fn german_aggregate_name(aggregate: Aggregate) -> &'static str {
    match aggregate {
        Aggregate::Sum => "Summe",
        Aggregate::Avg => "Durchschnitt",
        Aggregate::Min => "Minimum",
        Aggregate::Max => "Maximum",
        Aggregate::Count => "Anzahl",
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
        last_column: "One column stays shown",
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
        column_moved: english_column_moved,
        column_pinned_start: english_pinned_start,
        column_pinned_end: english_pinned_end,
        column_unpinned: english_unpinned,
        column_hidden: english_column_hidden,
        resize_column: english_resize_column,
        reorder: "Reorder",
        reorder_unavailable: "Clear the sort and filters to reorder rows",
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
        exported: english_exported,
        filters: "Filters",
        active_filters: english_active_filters,
        add_filter: "Add filter",
        logic: "Match",
        logic_name: english_logic_name,
        aggregate_name: english_aggregate_name,
        filter_line_column: english_line_column,
        filter_line_operator: line_operator,
        filter_line_value: english_line_value,
        remove_filter: english_remove_filter,
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
        last_column: "Eine Spalte bleibt sichtbar",
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
        column_moved: german_column_moved,
        column_pinned_start: german_pinned_start,
        column_pinned_end: german_pinned_end,
        column_unpinned: german_unpinned,
        column_hidden: german_column_hidden,
        resize_column: german_resize_column,
        reorder: "Neu anordnen",
        reorder_unavailable: "Sortierung und Filter aufheben, um Zeilen neu anzuordnen",
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
        exported: german_exported,
        filters: "Filter",
        active_filters: german_active_filters,
        add_filter: "Filter hinzufügen",
        logic: "Übereinstimmung",
        logic_name: german_logic_name,
        aggregate_name: german_aggregate_name,
        filter_line_column: german_line_column,
        filter_line_operator: german_line_operator,
        filter_line_value: german_line_value,
        remove_filter: german_remove_filter,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1400: every panel line control names its column, and the button its count.
    #[test]
    fn the_filter_panel_labels_name_their_column_and_count() {
        for labels in [TableLabels::ENGLISH, TableLabels::GERMAN] {
            for name in [
                labels.filter_line_column,
                labels.filter_line_operator,
                labels.filter_line_value,
                labels.remove_filter,
            ] {
                assert!(name("Stock").contains("Stock"), "{}", name("Stock"));
            }
            assert!((labels.active_filters)(3).contains('3'));
            assert!((labels.active_filters)(3).starts_with(labels.filters));
            assert_eq!((labels.active_filters)(0), labels.filters);
        }
    }

    /// 1428: each column menu announcement names its column, a move its place too.
    #[test]
    fn the_column_menu_announcements_name_their_column() {
        for labels in [TableLabels::ENGLISH, TableLabels::GERMAN] {
            for said in [
                labels.column_pinned_start,
                labels.column_pinned_end,
                labels.column_unpinned,
                labels.column_hidden,
            ] {
                assert!(said("Stock").contains("Stock"), "{}", said("Stock"));
            }
            let moved = (labels.column_moved)("Stock", 3, 7);
            assert!(
                moved.contains("Stock") && moved.contains('3') && moved.contains('7'),
                "{moved}"
            );
        }
    }
}
