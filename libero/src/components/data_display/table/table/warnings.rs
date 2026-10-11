//! The debug warnings for props that do not work together, said once on mount.

use super::TableProps;
use crate::utils::warn;

/// `tree` with `parent_key`, `groups_too` with a row grouping set or seeded.
pub(super) fn warn_misuse<T: Clone + PartialEq + 'static>(
    props: &TableProps<T>,
    tree: bool,
    groups_too: bool,
) {
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
