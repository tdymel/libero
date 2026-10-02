use std::{collections::BTreeMap, rc::Rc};

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::SortDirection,
    column_filter::{CellTest, filter_of},
    core::{ActiveSort, TableSort},
    filter_panel::PanelControl,
    filter_popover::{FilterPopover, FilterTarget, FilteredButton},
    pinning::{PinSide, PinnedColumns},
    resize::MenuWidth,
    use_table::StateSlice,
};
use crate::{
    components::{
        accessibility::Announcer,
        common::{Glyph, Input, Parts},
        overlay::{Menu, MenuEntry, MenuItem, MenuPart, use_menu},
    },
    context::IconSlot,
    hooks::{Align, ElementHandle},
    localization::TableLabels,
    platform::ElementApi,
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

/// Focus after a menu pick that moved its header in the DOM (a pin) or removed
/// it (Hide): the trigger, or a neighbour's. Owned by the table.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuFocus {
    /// The menu buttons by header index.
    triggers: CopyValue<BTreeMap<usize, ElementHandle>>,
    owed: Signal<Option<usize>>,
    owner: ScopeId,
}

pub(super) fn use_menu_focus() -> MenuFocus {
    let mut focus = MenuFocus {
        triggers: use_hook(|| CopyValue::new(BTreeMap::new())),
        owed: use_signal(|| None),
        owner: current_scope_id(),
    };
    // After the render that moved or dropped the header: the node is in place.
    use_effect(move || {
        let Some(index) = (focus.owed)() else {
            return;
        };
        focus.owed.set(None);
        if let Some(trigger) = focus.triggers.peek().get(&index) {
            let _ = trigger.focus();
        }
    });
    focus
}

impl MenuFocus {
    fn trigger(self, index: usize) -> ElementHandle {
        let mut triggers = self.triggers;
        *triggers
            .write()
            .entry(index)
            .or_insert_with(|| ElementHandle::new_in_scope(self.owner))
    }

    fn owe(mut self, index: usize) {
        self.owed.set(Some(index));
    }
}

/// The column order after one move, and the column's new place.
pub(super) type ColumnMove = Option<(Vec<String>, usize)>;

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
    /// The column order after a move towards the start and towards the end, with the
    /// column's place among the shown ones from 1; `None` at that edge.
    moves: (ColumnMove, ColumnMove),
    order: StateSlice<Vec<String>>,
    /// With `resizable_columns`, unless the column opted out.
    width: Option<MenuWidth>,
    labels: TableLabels,
    size: Size,
    parts: Input<Parts<MenuPart>>,
    /// Set for a filterable column: the menu offers its filter popover.
    filter: Option<FilterTarget>,
    /// Set with `filter_panel`: Filter opens the panel on this column's line.
    #[props(default)]
    panel: Option<PanelControl>,
    focus: MenuFocus,
    /// The shown column whose menu button takes focus after Hide.
    neighbour: Option<usize>,
    /// Says a move, pin or hide: the menu closes on a layout the reader cannot see (todo 1428).
    announcer: Announcer,
) -> Element {
    let menu = use_menu();
    let trigger = focus.trigger(index);
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
    let target = filter.clone();
    let open_panel = use_callback(move |()| {
        if let (Some(panel), Some(target)) = (panel, &target) {
            panel.open_for(target);
        }
    });
    let open_panel = (panel.is_some() && filter.is_some()).then_some(open_panel);
    if filter.is_some() {
        items.push(
            MenuItem::new(labels.filter)
                .onselect(move |_| match open_panel {
                    Some(open_panel) => open_panel.call(()),
                    None => filter_open.set(true),
                })
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
            let header = column.header.clone();
            items.push(
                MenuItem::new(label)
                    .disabled(next.is_none())
                    .onselect(move |_| {
                        if let Some((next, place)) = next.clone() {
                            order.set(next);
                            announcer.say((labels.column_moved)(&header, place, shown));
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
    for (label, to, said) in [
        (
            labels.pin_start,
            Some(PinSide::Start),
            labels.column_pinned_start,
        ),
        (labels.pin_end, Some(PinSide::End), labels.column_pinned_end),
        (labels.unpin, None, labels.column_unpinned),
    ] {
        if to != side {
            let header = column.header.clone();
            items.push(
                MenuItem::new(label)
                    .onselect(move |_| {
                        pinned.set(pinned.read().with(&header, to));
                        focus.owe(index);
                        announcer.say(said(&header));
                    })
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
        let item = MenuItem::new(labels.hide_column);
        let item = match shown <= 1 {
            true => item.description(labels.last_column),
            false => item,
        };
        items.push(
            item.disabled(shown <= 1)
                .onselect(move |_| {
                    hidden.set(toggle_column(&hidden.read(), &header, false));
                    if let Some(neighbour) = neighbour {
                        focus.owe(neighbour);
                    }
                    announcer.say((labels.column_hidden)(&header));
                })
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
                // The last shown column stays, and says so.
                let last = !off && shown <= 1;
                let item = MenuItem::new(header.clone());
                let item = match last {
                    true => item.description(labels.last_column),
                    false => item,
                };
                item.checkbox(!off)
                    .disabled(last)
                    .keep_open()
                    .onselect(move |_| {
                        hidden.set(toggle_column(&hidden.read(), &header, off));
                        // Hiding its own column unmounts this menu, as Hide does (todo 1490):
                        // the checkbox is gone before its state is read.
                        if column == index && !off {
                            if let Some(neighbour) = neighbour {
                                focus.owe(neighbour);
                            }
                            announcer.say((labels.column_hidden)(&header));
                        }
                    })
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
            FilteredButton { column: column.header.clone(), labels, open: filter_open, onopen: open_panel }
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
        if let Some(target) = filter.filter(|_| panel.is_none()) {
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
