//! The debug warnings for props that do not work together, said once on mount.

use super::TableProps;
use crate::utils::warn;

pub(super) fn warn_misuse<T: Clone + PartialEq + 'static>(props: &TableProps<T>) {
    let tree = props.parent_key.is_set();
    let groups_too = props.row_grouping.is_some() || !props.default_row_grouping.is_empty();
    if props.selectable && !props.row_key.is_set() {
        warn(
            "Table: `selectable` without `row_key` keys the selection by row index, so it \
             moves to other rows when `data` changes.",
        );
    }
    if props.row_detail.is_set() && !props.row_key.is_set() {
        warn(
            "Table: `row_detail` without `row_key` keys the open details by row index, so \
             they move to other rows when `data` changes.",
        );
    }
    if props.virtual_row_height.is_some() && props.max_height.is_none() {
        warn("Table: `virtual_row_height` without `max_height` renders every row.");
    }
    if props.onbottomreached.is_some() && props.max_height.is_none() {
        warn(
            "Table: `onbottomreached` without `max_height` never fires: the page scrolls, not the table.",
        );
    }
    if props.sticky_header.is_some() && props.max_height.is_some() {
        warn(
            "Table: `sticky_header` with `max_height` does nothing: that header stays put already.",
        );
    }
    if props.sticky_header.is_some() && props.scroll && props.max_height.is_none() {
        warn(
            "Table: `sticky_header` with `scroll` does nothing: the scroll area holds the \
             header, so it cannot stick to the page.",
        );
    }
    if props.virtual_row_height.is_some() && props.row_detail.is_set() {
        warn(
            "Table: `virtual_row_height` with `row_detail` renders every row: a detail \
             row breaks the one row height.",
        );
    }
    let name_column = props.columns.iter().position(|column| column.row_header);
    let name = props.columns.get(name_column.unwrap_or(0));
    let unnamed = |row| name.is_some_and(|column| (column.text)(row).is_empty());
    if let Some(row) = props
        .data
        .first()
        .filter(|row| props.selectable && unnamed(row))
    {
        let key = props.row_key.call(row).unwrap_or_else(|| "0".to_string());
        warn(&format!(
            "Table: `selectable` row \"{key}\" has no text in its name column (the row \
             header, else the first cell), so its checkbox is named by the row key. Give \
             the column text.",
        ));
    }
    if props.onrowreorder.is_some() && !props.row_key.is_set() {
        warn(
            "Table: `onrowreorder` without `row_key` keys the rows by index, so a moved row \
             is rebuilt and its handle loses the focus.",
        );
    }
    if props.resizable_columns && !props.column_menu {
        warn(
            "Table: `resizable_columns` without `column_menu` resizes by drag only, out of \
             reach of the keyboard and of a pointer that cannot drag (WCAG 2.5.7).",
        );
    }
    let editable = props.columns.iter().any(|column| column.edit.is_some());
    if editable && props.onrowupdate.is_none() {
        warn("Table: an editable column without `onrowupdate` drops every edit.");
    }
    if tree {
        let ignored = [
            (
                !props.row_key.is_set(),
                "without `row_key` matches parents by row index",
            ),
            (groups_too, "ignores `row_grouping`"),
            (props.row_detail.is_set(), "ignores `row_detail`"),
            (props.onrowreorder.is_some(), "does not reorder rows"),
        ];
        for (_, what) in ignored.into_iter().filter(|(on, _)| *on) {
            warn(&format!("Table: `parent_key` {what}."));
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use super::super::{
        super::column::{Column, column},
        Table,
    };
    use crate::{context::LiberoProvider, utils::warnings_of};

    #[derive(Clone, PartialEq)]
    struct Row {
        name: String,
    }

    fn people() -> Vec<Row> {
        vec![Row { name: "Ana".into() }]
    }

    fn name_column() -> Column<Row> {
        column("Name")
            .value(|row: &Row| row.name.clone())
            .row_header()
    }

    fn editable_column() -> Column<Row> {
        column("Name")
            .value(|row: &Row| row.name.clone())
            .editable(|row: &mut Row, name: String| row.name = name)
    }

    /// The warnings a `Table` of `people()` with these props says on mount.
    macro_rules! table {
        ($($props:tt)*) => {
            warnings_of(|| {
                rsx! {
                    LiberoProvider {
                        Table::<Row> {
                            caption: "People",
                            data: people(),
                            columns: vec![name_column()],
                            $($props)*
                        }
                    }
                }
            })
        };
    }

    fn says(warnings: &[String], what: &str) -> bool {
        warnings.iter().any(|warning| warning.contains(what))
    }

    #[test]
    fn a_plain_table_says_nothing() {
        assert_eq!(table!(), Vec::<String>::new());
    }

    #[test]
    fn selectable_without_a_row_key_warns() {
        let what = "`selectable` without `row_key`";
        assert!(says(&table!(selectable: true), what));
        let keyed = table!(selectable: true, row_key: |row: &Row| row.name.clone());
        assert!(!says(&keyed, what), "{keyed:?}");
    }

    #[test]
    fn a_row_detail_without_a_row_key_warns() {
        let what = "`row_detail` without `row_key`";
        assert!(says(&table!(row_detail: |_: &Row| None::<Element>), what));
        let keyed = table!(
            row_detail: |_: &Row| None::<Element>,
            row_key: |row: &Row| row.name.clone()
        );
        assert!(!says(&keyed, what), "{keyed:?}");
    }

    #[test]
    fn a_windowed_table_without_a_height_cap_warns() {
        let what = "`virtual_row_height` without `max_height`";
        assert!(says(&table!(virtual_row_height: 40.0), what));
        let capped = table!(virtual_row_height: 40.0, max_height: "20rem");
        assert!(!says(&capped, what), "{capped:?}");
    }

    #[test]
    fn onbottomreached_without_a_height_cap_warns() {
        let what = "`onbottomreached` without `max_height`";
        assert!(says(&table!(onbottomreached: |()| {}), what));
        let capped = table!(onbottomreached: |()| {}, max_height: "20rem");
        assert!(!says(&capped, what), "{capped:?}");
    }

    #[test]
    fn a_page_sticky_header_in_a_box_warns() {
        let what = "`sticky_header` with";
        assert!(says(&table!(sticky_header: "0", max_height: "20rem"), what));
        assert!(says(&table!(sticky_header: "0", scroll: true), what));
        let page = table!(sticky_header: "0");
        assert!(!says(&page, what), "{page:?}");
    }

    #[test]
    fn a_windowed_table_with_a_detail_warns() {
        let what = "`virtual_row_height` with `row_detail`";
        let both = table!(
            virtual_row_height: 40.0,
            max_height: "20rem",
            row_detail: |_: &Row| None::<Element>
        );
        assert!(says(&both, what));
        let windowed = table!(virtual_row_height: 40.0, max_height: "20rem");
        assert!(!says(&windowed, what), "{windowed:?}");
    }

    #[test]
    fn a_selectable_row_without_a_name_warns() {
        let what = "has no text";
        let unnamed = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    Table::<Row> {
                        caption: "People",
                        data: vec![Row { name: String::new() }],
                        columns: vec![name_column()],
                        selectable: true,
                        row_key: |row: &Row| row.name.clone(),
                    }
                }
            }
        });
        assert!(says(&unnamed, what), "{unnamed:?}");
        let named = table!(selectable: true, row_key: |row: &Row| row.name.clone());
        assert!(!says(&named, what), "{named:?}");
    }

    #[test]
    fn row_reorder_without_a_row_key_warns() {
        let what = "`onrowreorder` without `row_key`";
        assert!(says(&table!(onrowreorder: |_| {}), what));
        let keyed = table!(onrowreorder: |_| {}, row_key: |row: &Row| row.name.clone());
        assert!(!says(&keyed, what), "{keyed:?}");
    }

    #[test]
    fn resizable_columns_without_a_column_menu_warn() {
        let what = "`resizable_columns` without `column_menu`";
        assert!(says(&table!(resizable_columns: true), what));
        let menu = table!(resizable_columns: true, column_menu: true);
        assert!(!says(&menu, what), "{menu:?}");
    }

    #[test]
    fn an_editable_column_without_onrowupdate_warns() {
        let what = "editable column without `onrowupdate`";
        let dropped = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    Table::<Row> {
                        caption: "People",
                        data: people(),
                        columns: vec![editable_column()],
                    }
                }
            }
        });
        assert!(says(&dropped, what), "{dropped:?}");
        let handled = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    Table::<Row> {
                        caption: "People",
                        data: people(),
                        columns: vec![editable_column()],
                        onrowupdate: |_| {},
                    }
                }
            }
        });
        assert!(!says(&handled, what), "{handled:?}");
    }

    #[test]
    fn a_tree_says_what_it_ignores() {
        let parent = table!(parent_key: |_: &Row| None::<String>);
        assert!(
            says(&parent, "`parent_key` without `row_key`"),
            "{parent:?}"
        );
        let tree = table!(
            parent_key: |_: &Row| None::<String>,
            row_key: |row: &Row| row.name.clone()
        );
        assert!(!says(&tree, "`parent_key`"), "{tree:?}");
        let grouped = table!(
            parent_key: |_: &Row| None::<String>,
            row_key: |row: &Row| row.name.clone(),
            default_row_grouping: vec!["Name".to_string()]
        );
        assert!(
            says(&grouped, "`parent_key` ignores `row_grouping`"),
            "{grouped:?}"
        );
        let controlled = table!(
            parent_key: |_: &Row| None::<String>,
            row_key: |row: &Row| row.name.clone(),
            row_grouping: Vec::<String>::new()
        );
        assert!(
            says(&controlled, "`parent_key` ignores `row_grouping`"),
            "{controlled:?}"
        );
        let detailed = table!(
            parent_key: |_: &Row| None::<String>,
            row_key: |row: &Row| row.name.clone(),
            row_detail: |_: &Row| None::<Element>
        );
        assert!(
            says(&detailed, "`parent_key` ignores `row_detail`"),
            "{detailed:?}"
        );
        let reordered = table!(
            parent_key: |_: &Row| None::<String>,
            row_key: |row: &Row| row.name.clone(),
            onrowreorder: |_| {}
        );
        assert!(
            says(&reordered, "`parent_key` does not reorder rows"),
            "{reordered:?}"
        );
    }
}
