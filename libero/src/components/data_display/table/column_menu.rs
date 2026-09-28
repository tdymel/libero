use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::SortDirection,
    column_filter::{CellTest, filter_of},
    core::{ActiveSort, TableSort},
    filter_popover::{FilterPopover, FilterTarget, FilteredButton},
    pinning::{PinSide, PinnedColumns},
    resize::MenuWidth,
    use_table::StateSlice,
};
use crate::{
    components::{
        common::{Glyph, Input, Parts},
        overlay::{Menu, MenuEntry, MenuItem, MenuPart, use_menu},
    },
    context::IconSlot,
    hooks::{Align, use_element},
    localization::TableLabels,
    theme::Size,
};

/// One column as the menus see it.
#[derive(Clone, PartialEq)]
pub(super) struct MenuColumn {
    pub header: String,
    pub sortable: bool,
    pub hideable: bool,
    pub hidden: bool,
}

/// A header's menu: sort, filter, move, width, pin, hide, and the columns to show.
#[component]
pub(super) fn ColumnMenu(
    index: usize,
    columns: Rc<[MenuColumn]>,
    active: Rc<ActiveSort>,
    sort: StateSlice<Vec<TableSort>>,
    multi_sort: bool,
    hidden: StateSlice<Vec<String>>,
    pinned: StateSlice<PinnedColumns>,
    /// The column order after a move towards the start and towards the end;
    /// `None` at that edge.
    moves: (Option<Vec<String>>, Option<Vec<String>>),
    order: StateSlice<Vec<String>>,
    /// With `resizable_columns`, unless the column opted out.
    width: Option<MenuWidth>,
    labels: TableLabels,
    size: Size,
    parts: Input<Parts<MenuPart>>,
    /// Set for a filterable column: the menu offers its filter popover.
    filter: Option<FilterTarget>,
) -> Element {
    let menu = use_menu();
    let trigger = use_element();
    let mut filter_open = use_signal(|| false);
    let column = &columns[index];
    let shown = columns.iter().filter(|column| !column.hidden).count();
    let mut items: Vec<MenuEntry> = Vec::new();
    if column.sortable {
        let current = active
            .iter()
            .find(|(column, _)| *column == index)
            .map(|(_, direction)| *direction);
        let set = |direction: Option<SortDirection>| {
            let (columns, active) = (columns.clone(), active.clone());
            move |_| sort.set(sorted_by(&columns, &active, index, direction, false))
        };
        items.push(
            MenuItem::new(labels.sort_ascending)
                .disabled(current == Some(SortDirection::Ascending))
                .onselect(set(Some(SortDirection::Ascending)))
                .into(),
        );
        items.push(
            MenuItem::new(labels.sort_descending)
                .disabled(current == Some(SortDirection::Descending))
                .onselect(set(Some(SortDirection::Descending)))
                .into(),
        );
        if multi_sort && current.is_none() && !active.is_empty() {
            let (columns, active) = (columns.clone(), active.clone());
            items.push(
                MenuItem::new(labels.add_to_sort)
                    .onselect(move |_| {
                        sort.set(sorted_by(
                            &columns,
                            &active,
                            index,
                            Some(SortDirection::Ascending),
                            true,
                        ))
                    })
                    .into(),
            );
        }
        if current.is_some() {
            items.push(MenuItem::new(labels.unsort).onselect(set(None)).into());
        }
        items.push(MenuEntry::Separator);
    }
    // Sort, filter, move, width, pin, hide.
    if filter.is_some() {
        items.push(
            MenuItem::new(labels.filter)
                .onselect(move |_| filter_open.set(true))
                .into(),
        );
        items.push(MenuEntry::Separator);
    }
    let side = pinned.read().side(&column.header);
    // A pinned column moves by its pin; the labels are physical, as the reader sees them.
    if side.is_none() {
        let (backward, forward) = moves;
        let (left, right) = match trigger.is_rtl() {
            true => (forward, backward),
            false => (backward, forward),
        };
        for (label, next) in [
            (labels.move_column_left, left),
            (labels.move_column_right, right),
        ] {
            items.push(
                MenuItem::new(label)
                    .disabled(next.is_none())
                    .onselect(move |_| {
                        if let Some(next) = next.clone() {
                            order.set(next);
                        }
                    })
                    .into(),
            );
        }
    }
    if let Some(width) = width {
        items.extend(width.items(&column.header, labels));
    }
    // The side it is on is left out.
    for (label, to) in [
        (labels.pin_start, Some(PinSide::Start)),
        (labels.pin_end, Some(PinSide::End)),
        (labels.unpin, None),
    ] {
        if to != side {
            let header = column.header.clone();
            items.push(
                MenuItem::new(label)
                    .onselect(move |_| pinned.set(pinned.read().with(&header, to)))
                    .into(),
            );
        }
    }
    let hideable: Vec<usize> = (0..columns.len())
        .filter(|&column| columns[column].hideable)
        .collect();
    if !items.is_empty() && !hideable.is_empty() {
        items.push(MenuEntry::Separator);
    }
    if column.hideable {
        let header = column.header.clone();
        items.push(
            MenuItem::new(labels.hide_column)
                .disabled(shown <= 1)
                .onselect(move |_| hidden.set(toggle_column(&hidden.read(), &header, false)))
                .into(),
        );
    }
    if !hideable.is_empty() {
        let toggles = hideable
            .into_iter()
            .map(|column| {
                let MenuColumn {
                    header,
                    hidden: off,
                    ..
                } = columns[column].clone();
                MenuItem::new(header.clone())
                    .checkbox(!off)
                    // The last shown column stays.
                    .disabled(!off && shown <= 1)
                    .keep_open()
                    .onselect(move |_| hidden.set(toggle_column(&hidden.read(), &header, off)))
                    .into()
            })
            .collect();
        items.push(MenuItem::new(labels.columns).submenu(toggles).into());
    }
    let mut attributes = menu.a11y_attributes();
    attributes.push(Attribute::new(
        "aria-label",
        (labels.column_menu)(&column.header),
        None,
        false,
    ));
    let filtered = filter.as_ref().is_some_and(|target| {
        filter_of(&target.slice.read(), &target.column)
            .is_some_and(|item| CellTest::new(item, target.kind).is_some())
    });
    rsx! {
        if filtered {
            FilteredButton { column: column.header.clone(), labels, open: filter_open }
        }
        Menu {
            state: menu,
            items,
            align: Align::End,
            size,
            parts,
            button {
                r#type: "button",
                "data-column-menu": true,
                onmounted: trigger.mount(),
                ..attributes,
                Glyph { slot: IconSlot::More, icon: lucide::ellipsis_vertical::outlined }
            }
        }
        if let Some(target) = filter {
            FilterPopover { target, anchor: trigger, open: filter_open }
        }
    }
}

/// The sort after a menu pick on column `index`: `direction` in its place if
/// sorted, else alone, or after the others with `add`; `None` drops it.
pub(super) fn sorted_by(
    columns: &[MenuColumn],
    active: &ActiveSort,
    index: usize,
    direction: Option<SortDirection>,
    add: bool,
) -> Vec<TableSort> {
    let entry = |(column, direction): (usize, SortDirection)| {
        TableSort::new(&columns[column].header, direction)
    };
    let sorted = active.iter().any(|(column, _)| *column == index);
    if !sorted && !add {
        return direction
            .map(|direction| entry((index, direction)))
            .into_iter()
            .collect();
    }
    active
        .iter()
        .filter_map(|&(column, current)| match column == index {
            true => direction.map(|direction| (column, direction)),
            false => Some((column, current)),
        })
        .chain(
            direction
                .filter(|_| !sorted)
                .map(|direction| (index, direction)),
        )
        .map(entry)
        .collect()
}

/// `hidden` with `header` shown or hidden, the others in their order.
pub(super) fn toggle_column(hidden: &[String], header: &str, show: bool) -> Vec<String> {
    let mut next: Vec<String> = hidden.iter().filter(|h| *h != header).cloned().collect();
    if !show {
        next.push(header.to_string());
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use SortDirection::{Ascending, Descending};

    fn columns(headers: &[&str]) -> Vec<MenuColumn> {
        headers
            .iter()
            .map(|header| MenuColumn {
                header: header.to_string(),
                sortable: true,
                hideable: true,
                hidden: false,
            })
            .collect()
    }

    #[test]
    fn a_menu_sort_replaces_unless_added_or_already_sorted() {
        let columns = columns(&["A", "B", "C"]);
        let active = vec![(0, Ascending), (1, Descending)];

        assert_eq!(
            sorted_by(&columns, &active, 2, Some(Descending), false),
            vec![TableSort::new("C", Descending)]
        );
        assert_eq!(
            sorted_by(&columns, &active, 2, Some(Ascending), true),
            vec![
                TableSort::new("A", Ascending),
                TableSort::new("B", Descending),
                TableSort::new("C", Ascending)
            ]
        );
        assert_eq!(
            sorted_by(&columns, &active, 1, Some(Ascending), false),
            vec![
                TableSort::new("A", Ascending),
                TableSort::new("B", Ascending)
            ]
        );
        assert_eq!(
            sorted_by(&columns, &active, 0, None, false),
            vec![TableSort::new("B", Descending)]
        );
    }

    #[test]
    fn a_column_toggles_in_and_out_of_the_hidden_list() {
        let hidden = vec!["A".to_string()];

        assert_eq!(toggle_column(&hidden, "B", false), vec!["A", "B"]);
        assert_eq!(toggle_column(&hidden, "A", true), Vec::<String>::new());
        assert_eq!(toggle_column(&hidden, "A", false), vec!["A"]);
    }
}
