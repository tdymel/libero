use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::{CellAlign, SortDirection, SortKey},
    column::{Column, ColumnDefaults},
    use_table::StateSlice,
};
use crate::{
    components::{accessibility::VisuallyHidden, common::Glyph},
    context::IconSlot,
    localization::fill,
};

/// One entry of a [`Table`](super::Table)'s sort: a column, named by its header text.
///
/// ```rust
/// # use libero::components::{SortDirection, TableSort};
/// let by_age = vec![TableSort::new("Age", SortDirection::Descending)];
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TableSort {
    pub column: String,
    pub direction: SortDirection,
}

impl TableSort {
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }
}

/// A per-row closure for a [`Table`](super::Table) prop, as `row_key`, or none
/// by default. Compares equal to any other, as a column's closures do.
///
/// ```rust
/// # use libero::components::RowFn;
/// # struct User { id: u64 }
/// let key: RowFn<User, String> = (|u: &User| u.id.to_string()).into();
/// ```
pub struct RowFn<T, R>(Option<RowClosure<T, R>>);

type RowClosure<T, R> = Rc<dyn Fn(&T) -> R>;

impl<T, R> RowFn<T, R> {
    /// `None` when unset.
    pub(super) fn call(&self, row: &T) -> Option<R> {
        self.0.as_ref().map(|f| f(row))
    }

    pub(super) fn is_set(&self) -> bool {
        self.0.is_some()
    }
}

impl<T, R, F: Fn(&T) -> R + 'static> From<F> for RowFn<T, R> {
    fn from(f: F) -> Self {
        Self(Some(Rc::new(f)))
    }
}

// Hand-written: a derive would demand `T: Default`, `T: Clone` and `R: Clone`.
impl<T, R> Default for RowFn<T, R> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T, R> Clone for RowFn<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, R> PartialEq for RowFn<T, R> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// What the non-generic body needs from a `Column<T>`, once `T` is gone.
pub(super) struct HeaderSpec {
    pub header: String,
    pub align: CellAlign,
    pub sortable: bool,
    pub row_header: bool,
    /// The header cell's inline `width`/`min-width`, which size the column.
    pub style: Option<String>,
    /// A caller's header body, which replaces the text.
    pub body: Option<Element>,
}

pub(super) fn header_specs<T>(columns: &[Column<T>], defaults: &ColumnDefaults) -> Vec<HeaderSpec> {
    columns
        .iter()
        .map(|column| {
            let resolved = column.resolve(defaults);
            HeaderSpec {
                header: column.header.clone(),
                align: resolved.align,
                sortable: column.sortable,
                row_header: column.row_header,
                style: width_style(resolved.width, resolved.min_width),
                body: column.header_render.as_ref().map(|render| render()),
            }
        })
        .collect()
}

// Not `<col>`: Blitz ignores its width (blitz-table-probe). Border-box, as
// Blitz reads a cell's width.
fn width_style(width: Option<&str>, min_width: Option<&str>) -> Option<String> {
    if width.is_none() && min_width.is_none() {
        return None;
    }
    let mut style = String::from("box-sizing:border-box;");
    if let Some(width) = width {
        style.push_str(&format!("width:{width};"));
    }
    if let Some(min_width) = min_width {
        style.push_str(&format!("min-width:{min_width};"));
    }
    Some(style)
}

/// The sorted columns by header index, in priority order.
pub(super) type ActiveSort = Vec<(usize, SortDirection)>;

/// `data`'s indices in display order: source order unless a column is sorted.
pub(super) fn row_order<T>(
    data: &[T],
    columns: &[Column<T>],
    active: &[(usize, SortDirection)],
) -> Vec<usize> {
    if active.is_empty() {
        return (0..data.len()).collect();
    }
    let keys: Vec<(Vec<SortKey>, SortDirection)> = active
        .iter()
        .map(|&(index, direction)| {
            let sort_key = &columns[index].sort_key;
            (data.iter().map(|row| sort_key(row)).collect(), direction)
        })
        .collect();
    sorted_order(&keys)
}

/// What a click on header `index` asks for: ascending, descending, then unsorted.
/// `add` keeps the other sorted columns and appends a new one last.
pub(super) fn next_sort(
    active: &[(usize, SortDirection)],
    headers: &[HeaderSpec],
    index: usize,
    add: bool,
) -> Vec<TableSort> {
    let current = active
        .iter()
        .find(|(column, _)| *column == index)
        .map(|(_, direction)| *direction);
    let next = match current {
        None => Some(SortDirection::Ascending),
        Some(SortDirection::Ascending) => Some(SortDirection::Descending),
        Some(SortDirection::Descending) => None,
    };
    let entry = |(column, direction): (usize, SortDirection)| {
        TableSort::new(&headers[column].header, direction)
    };
    if !add {
        return next
            .map(|direction| entry((index, direction)))
            .into_iter()
            .collect();
    }
    active
        .iter()
        .filter_map(|&(column, direction)| match column == index {
            true => next.map(|next| (column, next)),
            false => Some((column, direction)),
        })
        .chain(
            current
                .is_none()
                .then_some((index, SortDirection::Ascending)),
        )
        .map(entry)
        .collect()
}

/// A row's cells and its DOM key: the caller's `row_key`, else its index in
/// `data`. A cell is its text, or a caller's body.
pub(super) struct RowSpec {
    pub key: String,
    /// The row's `data-state`.
    pub states: Option<String>,
    pub attributes: Vec<Attribute>,
    pub cells: Vec<(String, Option<Element>)>,
    /// With `selectable`: whether it is selected, and its checkbox cell.
    pub selected: Option<bool>,
    pub select: Option<Element>,
}

/// Row indices in sorted order, one key column per sorted column, the first
/// deciding. Stable, so equal keys keep source order, and `Empty` sinks to the
/// bottom either way.
pub(super) fn sorted_order(keys: &[(Vec<SortKey>, SortDirection)]) -> Vec<usize> {
    let len = keys.first().map_or(0, |(column, _)| column.len());
    let mut order: Vec<usize> = (0..len).collect();
    order.sort_by(|&a, &b| {
        keys.iter()
            .map(|(column, direction)| {
                let (a, b) = (&column[a], &column[b]);
                match (a.is_empty(), b.is_empty()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) => match direction {
                        SortDirection::Ascending => a.compare(b),
                        SortDirection::Descending => a.compare(b).reverse(),
                    },
                }
            })
            .find(|ordering| ordering.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    order
}

/// The sorted columns, keyed by header text so reordering can't move them.
/// With duplicate headers the first wins; entries naming no sortable header are
/// skipped. Without `multi`, only the first entry left sorts.
pub(super) fn active_sort(headers: &[HeaderSpec], sort: &[TableSort], multi: bool) -> ActiveSort {
    let mut active = ActiveSort::new();
    for sort in sort {
        let Some(index) = headers
            .iter()
            .position(|spec| spec.sortable && spec.header == sort.column)
        else {
            continue;
        };
        if !active.iter().any(|(column, _)| *column == index) {
            active.push((index, sort.direction));
        }
        if !multi {
            break;
        }
    }
    active
}

fn align_attr(align: CellAlign) -> Option<&'static str> {
    (align != CellAlign::Start).then(|| align.as_str())
}

/// The caption's text and the id the scroll region names itself by.
pub(super) struct CaptionSpec {
    pub text: String,
    pub id: String,
}

/// Everything inside the `<table>`, with `T` gone.
pub(super) struct BodySpec {
    pub caption: Option<CaptionSpec>,
    pub headers: Vec<HeaderSpec>,
    pub rows: Vec<RowSpec>,
    /// The empty row's body: the caller's `empty`, else the localized text.
    pub empty: Element,
    pub active: ActiveSort,
    pub sort: StateSlice<Vec<TableSort>>,
    /// Set with `multi_sort`: whether the press before a header click was a touch.
    pub touch: Option<CopyValue<bool>>,
    pub sort_order: &'static str,
    /// The select-all header cell, with `selectable`.
    pub select_all: Option<Element>,
}

/// Whether a header click adds its column to the others: a modifier, or a touch,
/// which has none. The touch mark is spent by the click.
fn adds_column(touch: Option<CopyValue<bool>>, event: &MouseEvent) -> bool {
    let Some(mut touch) = touch else {
        return false;
    };
    let touched = *touch.read();
    touch.set(false);
    let modifiers = event.modifiers();
    touched || modifiers.shift() || modifiers.ctrl() || modifiers.meta()
}

fn header_body(spec: &HeaderSpec) -> Element {
    match &spec.body {
        Some(body) => body.clone(),
        None => rsx! { "{spec.header}" },
    }
}

pub(super) fn render_body(body: BodySpec) -> Element {
    let BodySpec {
        caption,
        headers,
        rows,
        empty,
        active,
        sort,
        touch,
        sort_order,
        select_all,
    } = body;
    let columns = headers.len().max(1) + usize::from(select_all.is_some());
    let empty = rows.is_empty().then_some(empty);
    // The order shows only when it tells something: with two or more sorted columns.
    let ranked = active.len() > 1;
    let active = Rc::new(active);
    let headers = Rc::new(headers);
    rsx! {
        if let Some(spec) = caption {
            caption { id: spec.id, "{spec.text}" }
        }
        thead {
            tr {
                {select_all}
                for (index , spec) in headers.iter().enumerate() {
                    th {
                        key: "{index}",
                        scope: "col",
                        "data-align": align_attr(spec.align),
                        "data-sortable": spec.sortable.then_some(true),
                        style: spec.style.clone(),
                        // On the sorted columns only (APG): a "none" on every
                        // other one is read out as "not sorted" at each.
                        aria_sort: active
                            .iter()
                            .find(|(column, _)| *column == index)
                            .map(|(_, direction)| direction.aria_value()),
                        if spec.sortable {
                            button {
                                r#type: "button",
                                onpointerdown: move |event: PointerEvent| {
                                    if let Some(mut touch) = touch {
                                        touch.set(event.pointer_type() != "mouse");
                                    }
                                },
                                onclick: {
                                    let (active, headers) = (active.clone(), headers.clone());
                                    move |event: MouseEvent| {
                                        let add = adds_column(touch, &event);
                                        sort.set(next_sort(&active, &headers, index, add));
                                    }
                                },
                                {header_body(spec)}
                                // Always rendered, so sorting can't change the width;
                                // `aria-sort` shows and flips it.
                                Glyph { slot: IconSlot::ArrowDown, icon: lucide::arrow_down::outlined }
                                if let Some(order) = active
                                    .iter()
                                    .position(|(column, _)| *column == index)
                                    .filter(|_| ranked)
                                    .map(|order| order + 1)
                                {
                                    span { "data-sort-order": true, aria_hidden: "true", "{order}" }
                                    VisuallyHidden { {fill(sort_order, &[("n", &order)])} }
                                }
                            }
                        } else {
                            {header_body(spec)}
                        }
                    }
                }
            }
        }
        tbody {
            if let Some(empty) = empty {
                tr { "data-empty": true,
                    td { colspan: "{columns}", {empty} }
                }
            }
            for RowSpec { key , states , attributes , cells , selected , select } in rows {
                tr {
                    key: "{key}",
                    "data-state": states,
                    aria_selected: selected.map(|selected| selected.to_string()),
                    ..attributes,
                    {select}
                    for (index , (text , body)) in cells.into_iter().enumerate() {
                        if headers.get(index).is_some_and(|spec| spec.row_header) {
                            th {
                                key: "{index}",
                                scope: "row",
                                "data-align": headers.get(index).and_then(|spec| align_attr(spec.align)),
                                "{text}"
                                {body}
                            }
                        } else {
                            td {
                                key: "{index}",
                                "data-align": headers.get(index).and_then(|spec| align_attr(spec.align)),
                                // Text inline: a nested node per cell costs ~1 us a sort.
                                "{text}"
                                {body}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};
    use SortDirection::{Ascending, Descending};

    fn keys(values: &[Option<f64>]) -> Vec<SortKey> {
        values
            .iter()
            .map(|value| value.map_or(SortKey::Empty, SortKey::num))
            .collect()
    }

    fn sorted(keys: &[SortKey], direction: SortDirection) -> Vec<usize> {
        sorted_order(&[(keys.to_vec(), direction)])
    }

    #[test]
    fn sorts_numbers_both_ways() {
        let keys = keys(&[Some(3.0), Some(1.0), Some(2.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![1, 2, 0]);
        assert_eq!(sorted(&keys, Descending), vec![0, 2, 1]);
    }

    #[test]
    fn empty_keys_sink_in_both_directions() {
        let keys = keys(&[None, Some(2.0), Some(1.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![2, 1, 0]);
        assert_eq!(sorted(&keys, Descending), vec![1, 2, 0]);
    }

    #[test]
    fn equal_keys_keep_source_order() {
        let keys = keys(&[Some(1.0), Some(1.0), Some(1.0)]);

        assert_eq!(sorted(&keys, Ascending), vec![0, 1, 2]);
        assert_eq!(sorted(&keys, Descending), vec![0, 1, 2]);
    }

    #[test]
    fn a_second_column_breaks_ties_of_the_first() {
        let first = keys(&[Some(1.0), Some(2.0), Some(1.0), Some(2.0)]);
        let second = keys(&[Some(5.0), Some(6.0), Some(7.0), None]);

        let order = sorted_order(&[(first.clone(), Ascending), (second.clone(), Descending)]);
        assert_eq!(order, vec![2, 0, 1, 3]);
        let order = sorted_order(&[(first, Descending), (second, Ascending)]);
        assert_eq!(order, vec![1, 3, 0, 2]);
    }

    fn headers(names: &[&str]) -> Vec<HeaderSpec> {
        names
            .iter()
            .map(|name| HeaderSpec {
                header: name.to_string(),
                align: CellAlign::Start,
                sortable: true,
                row_header: false,
                style: None,
                body: None,
            })
            .collect()
    }

    #[test]
    fn the_sort_follows_its_header_not_its_position() {
        let sort = [TableSort::new("Age", Descending)];

        let active = active_sort(&headers(&["Name", "Age"]), &sort, false);
        assert_eq!(active, [(1, Descending)]);

        let reordered = active_sort(&headers(&["Age", "Name"]), &sort, false);
        assert_eq!(reordered, [(0, Descending)]);

        assert!(active_sort(&headers(&["Name"]), &sort, false).is_empty());
    }

    #[test]
    fn duplicate_headers_sort_the_first_sortable_match() {
        let sort = [TableSort::new("Age", Ascending)];
        let mut specs = headers(&["Age", "Name", "Age"]);

        assert_eq!(active_sort(&specs, &sort, false), [(0, Ascending)]);
        specs[0].sortable = false;
        assert_eq!(active_sort(&specs, &sort, false), [(2, Ascending)]);
    }

    #[test]
    fn the_first_entry_naming_a_sortable_header_sorts() {
        let sort = [
            TableSort::new("Height", Ascending),
            TableSort::new("Name", Descending),
            TableSort::new("Age", Ascending),
        ];

        let active = active_sort(&headers(&["Age", "Name"]), &sort, false);
        assert_eq!(active, [(1, Descending)]);
        assert!(active_sort(&headers(&["Age"]), &[], false).is_empty());
    }

    #[test]
    fn multi_keeps_every_named_column_once_in_order() {
        let sort = [
            TableSort::new("Name", Descending),
            TableSort::new("Height", Ascending),
            TableSort::new("Age", Ascending),
            TableSort::new("Name", Ascending),
        ];

        let active = active_sort(&headers(&["Age", "Name"]), &sort, true);
        assert_eq!(active, [(1, Descending), (0, Ascending)]);
    }

    #[test]
    fn text_sorts_case_insensitively() {
        let keys = vec![SortKey::text("beta"), SortKey::text("Alpha")];

        assert_eq!(sorted(&keys, Ascending), vec![1, 0]);
    }

    #[test]
    fn a_header_click_cycles_ascending_descending_unsorted() {
        let specs = headers(&["Name", "Age"]);
        let asc = next_sort(&[], &specs, 1, false);
        assert_eq!(asc, [TableSort::new("Age", Ascending)]);

        let desc = next_sort(&[(1, Ascending)], &specs, 1, false);
        assert_eq!(desc, [TableSort::new("Age", Descending)]);

        assert!(next_sort(&[(1, Descending)], &specs, 1, false).is_empty());
    }

    #[test]
    fn another_header_starts_ascending() {
        let next = next_sort(&[(0, Descending)], &headers(&["Name", "Age"]), 1, false);

        assert_eq!(next, [TableSort::new("Age", Ascending)]);
    }

    #[test]
    fn a_plain_click_on_a_secondary_column_sorts_by_it_alone() {
        let next = next_sort(
            &[(0, Ascending), (1, Ascending)],
            &headers(&["Name", "Age"]),
            1,
            false,
        );

        assert_eq!(next, [TableSort::new("Age", Descending)]);
    }

    #[test]
    fn an_added_click_appends_cycles_and_removes_in_place() {
        let specs = headers(&["Name", "Age", "City"]);

        let appended = next_sort(&[(0, Descending)], &specs, 2, true);
        assert_eq!(
            appended,
            [
                TableSort::new("Name", Descending),
                TableSort::new("City", Ascending)
            ]
        );
        let flipped = next_sort(&[(0, Ascending), (2, Ascending)], &specs, 0, true);
        assert_eq!(
            flipped,
            [
                TableSort::new("Name", Descending),
                TableSort::new("City", Ascending)
            ]
        );
        let removed = next_sort(&[(0, Descending), (2, Ascending)], &specs, 0, true);
        assert_eq!(removed, [TableSort::new("City", Ascending)]);
    }

    fn ages() -> (Vec<u32>, Vec<Column<u32>>) {
        let columns = vec![
            column("Name").value(|age: &u32| format!("n{age}")),
            column("Age").value(|age: &u32| *age).sortable(),
        ];
        (vec![30, 10, 20], columns)
    }

    #[test]
    fn rows_keep_source_order_until_sorted() {
        let (data, columns) = ages();

        assert_eq!(row_order(&data, &columns, &[]), vec![0, 1, 2]);
        assert_eq!(row_order(&data, &columns, &[(1, Ascending)]), vec![1, 2, 0]);
    }

    #[test]
    fn a_sort_on_an_unknown_or_unsortable_header_is_inactive() {
        let (_, columns) = ages();
        let headers = header_specs(&columns, &ColumnDefaults::new());

        let unknown = [TableSort::new("Height", Ascending)];
        assert!(active_sort(&headers, &unknown, true).is_empty());
        let unsortable = [TableSort::new("Name", Ascending)];
        assert!(active_sort(&headers, &unsortable, true).is_empty());
    }

    #[test]
    fn widths_become_the_header_cells_style() {
        assert_eq!(width_style(None, None), None);
        assert_eq!(
            width_style(Some("6rem"), None).as_deref(),
            Some("box-sizing:border-box;width:6rem;")
        );
        assert_eq!(
            width_style(Some("50%"), Some("4rem")).as_deref(),
            Some("box-sizing:border-box;width:50%;min-width:4rem;")
        );
    }
}
