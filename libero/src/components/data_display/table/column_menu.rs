use std::{collections::BTreeMap, rc::Rc};

use dioxus::{
    core::{ScopeId, current_scope_id},
    prelude::*,
};
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::SortDirection,
    column_filter::{CellTest, filter_of},
    core::{ActiveSort, TableSort, UserSort},
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
    hooks::{Align, ElementHandle, use_formats, use_press_marker},
    localization::TableLabels,
    platform::{ElementApi, next_observe_tag, observes_by_tag},
    theme::Size,
};

/// One column as the menus see it.
#[derive(Clone, PartialEq)]
pub(super) struct MenuColumn {
    pub header: String,
    /// Its header cell's key: a menu's focus follows the column, not its index.
    pub key: String,
    pub sortable: bool,
    pub hideable: bool,
    pub hidden: bool,
    /// Offered as a row grouping level.
    pub groupable: bool,
}

/// Focus after a menu pick that moved its header in the DOM (a pin) or removed
/// it (Hide): the trigger, or a neighbour's. Owned by the table.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuFocus {
    /// The menu buttons by column key.
    triggers: CopyValue<BTreeMap<String, ElementHandle>>,
    owed: Signal<Option<String>>,
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
        let Some(key) = (focus.owed)() else {
            return;
        };
        focus.owed.set(None);
        if let Some(trigger) = focus.triggers.peek().get(&key) {
            let _ = trigger.focus();
        }
    });
    focus
}

impl MenuFocus {
    fn trigger(self, key: &str) -> ElementHandle {
        let mut triggers = self.triggers;
        *triggers.write().entry(key.to_string()).or_insert_with(|| {
            ElementHandle::new_in_scope(self.owner).retag(observes_by_tag().then(next_observe_tag))
        })
    }

    /// Opens column `key`'s menu as a click on its button would, else focuses the button.
    pub(super) fn open(self, key: &str) {
        if let Some(trigger) = self.triggers.peek().get(key)
            && trigger.click().is_err()
        {
            let _ = trigger.focus();
        }
    }

    fn owe(mut self, key: String) {
        self.owed.set(Some(key));
    }
}

/// A menu pin, said once its column's width is known: the room may release it at once.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PinNews(CopyValue<Option<(String, PinSide)>>);

pub(super) fn use_pin_news() -> PinNews {
    PinNews(use_hook(|| CopyValue::new(None)))
}

impl PinNews {
    fn owe(mut self, header: String, side: PinSide, pinned: StateSlice<PinnedColumns>) {
        self.0.set(Some((header.clone(), side)));
        // A pin the host refused lapses with the event's renders: no later pin vouches for it.
        // A refusing host renders nothing: this render runs the check below now.
        dioxus::core::needs_update();
        let mut news = self.0;
        dioxus::core::queue_effect(move || {
            if pinned.peek().side(&header) != Some(side)
                && let Ok(mut owed) = news.try_write()
                && owed.as_ref() == Some(&(header, side))
            {
                *owed = None;
            }
        });
    }

    /// The owed pin and whether `kept` holds it, once `known` has its column's width.
    /// A pin `requested` no longer holds is dropped unsaid.
    pub(super) fn settle(
        mut self,
        requested: &PinnedColumns,
        kept: &PinnedColumns,
        known: impl Fn(&str) -> bool,
    ) -> Option<(String, PinSide, bool)> {
        let owed = self.0.peek().clone();
        let (header, side) = owed?;
        if requested.side(&header) != Some(side) {
            self.0.set(None);
            return None;
        }
        if !known(&header) {
            return None;
        }
        self.0.set(None);
        let held = kept.side(&header) == Some(side);
        Some((header, side, held))
    }
}

/// Says Unpin once the event's renders are done, if the host kept it, as [`PinNews`] waits for a pin.
fn say_unpin(
    header: String,
    pinned: StateSlice<PinnedColumns>,
    announcer: Announcer,
    labels: TableLabels,
) {
    // A refusing host renders nothing: this render runs the check below now.
    dioxus::core::needs_update();
    dioxus::core::queue_effect(move || {
        if pinned.peek().side(&header).is_none() {
            announcer.say((labels.column_unpinned)(&header));
        }
    });
}

/// The column order after one move, and the column's new place.
pub(super) type ColumnMove = Option<(Vec<String>, usize)>;

/// A header's menu: sort, filter, group, move, width, pin, hide, and the columns to show.
#[component]
pub(super) fn ColumnMenu(
    index: usize,
    columns: Rc<[MenuColumn]>,
    active: Rc<ActiveSort>,
    sort: UserSort,
    multi_sort: bool,
    hidden: StateSlice<Vec<String>>,
    pinned: StateSlice<PinnedColumns>,
    /// The side the layout holds the column at: a pin the room released is none.
    side: Option<PinSide>,
    news: PinNews,
    grouping: StateSlice<Vec<String>>,
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
    /// In a grid the `th` is the stop and opens this on Ctrl+Enter: the button leaves the Tab order.
    grid: bool,
    /// The shown column whose menu button takes focus after Hide.
    neighbour: Option<usize>,
    /// Says a move, pin or hide: the menu closes on a layout the reader cannot see (todo 1428).
    announcer: Announcer,
) -> Element {
    let menu = use_menu();
    let trigger = focus.trigger(&columns[index].key);
    let mut filter_open = use_signal(|| false);
    let filter_marker = use_press_marker();
    let filtered_button = use_hook(ElementHandle::new);
    let mut opener = use_signal(move || trigger);
    let column = &columns[index];
    let own_key = column.key.clone();
    let neighbour_key = neighbour.map(|neighbour| columns[neighbour].key.clone());
    let shown = columns.iter().filter(|column| !column.hidden).count();
    let current = active
        .iter()
        .find(|(column, _)| *column == index)
        .map(|(_, direction)| *direction);
    // A released pin is still requested: Unpin drops it, so it does not return with the room.
    let requested = pinned.read().side(&column.header);
    // Rows a pick adds (Unpin's moves, Unsort) wait for the close: Blitz panics on a
    // button made and dropped in one render (form owner, todo 3534).
    let mut held = use_hook(|| CopyValue::new(None));
    let shape = *held.peek();
    let (current, side, requested) = match (menu.is_open(), shape) {
        (true, Some(shape)) => shape,
        _ => {
            held.set(Some((current, side, requested)));
            (current, side, requested)
        }
    };
    let mut items: Vec<MenuEntry> = Vec::new();
    if column.sortable {
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
    // Sort, filter, group, move, width, pin, hide.
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
                    None => {
                        opener.set(trigger);
                        filter_open.set(true);
                    }
                })
                .into(),
        );
        items.push(MenuEntry::Separator);
    }
    if column.groupable {
        let header = column.header.clone();
        let grouped = grouping.read().contains(&header);
        let (label, said) = match grouped {
            true => (labels.ungroup, labels.column_ungrouped),
            false => (labels.group_by, labels.column_grouped),
        };
        items.push(
            MenuItem::new(label(&header))
                .onselect(move |_| {
                    grouping.set(regrouped(&grouping.read(), &header, !grouped));
                    announcer.say(said(&header));
                })
                .into(),
        );
        items.push(MenuEntry::Separator);
    }
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
    // The side it is on is left out; a pin is said once the table kept or released it.
    for (label, to) in [
        (labels.pin_start, Some(PinSide::Start)),
        (labels.pin_end, Some(PinSide::End)),
        (labels.unpin, None),
    ] {
        let offered = match to {
            Some(_) => to != side,
            None => requested.is_some(),
        };
        if offered {
            let (header, own_key) = (column.header.clone(), own_key.clone());
            items.push(
                MenuItem::new(label)
                    .onselect(move |_| {
                        pinned.set(pinned.read().with(&header, to));
                        focus.owe(own_key.clone());
                        match to {
                            Some(to) => news.owe(header.clone(), to, pinned),
                            None => say_unpin(header.clone(), pinned, announcer, labels),
                        }
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
        let (header, neighbour_key) = (column.header.clone(), neighbour_key.clone());
        let item = MenuItem::new(labels.hide_column);
        let item = match shown <= 1 {
            true => item.description(labels.last_column),
            false => item,
        };
        items.push(
            item.disabled(shown <= 1)
                .onselect(move |_| {
                    hidden.set(toggle_column(&hidden.read(), &header, false));
                    if let Some(neighbour) = &neighbour_key {
                        focus.owe(neighbour.clone());
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
                let neighbour_key = neighbour_key.clone();
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
                        if column == index
                            && !off
                            && let Some(neighbour) = &neighbour_key
                        {
                            focus.owe(neighbour.clone());
                        }
                        if !off {
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
    let formats = use_formats();
    let filtered = filter.as_ref().is_some_and(|target| {
        filter_of(&target.slice.read(), &target.column)
            .is_some_and(|item| CellTest::new(item, target.kind, formats).is_some())
    });
    let marked: Vec<Attribute> = filter_marker
        .attribute()
        .filter(|_| filter_open())
        .into_iter()
        .collect();
    rsx! {
        if filtered {
            FilteredButton {
                column: column.header.clone(),
                labels,
                open: filter_open,
                handle: filtered_button,
                opener,
                onopen: open_panel,
                grid,
            }
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
                tabindex: grid.then_some("-1"),
                onmounted: trigger.mount(),
                ..attributes,
                ..trigger.attributes(),
                ..marked,
                Glyph { slot: IconSlot::More, icon: lucide::ellipsis_vertical::outlined }
            }
        }
        if let Some(target) = filter.filter(|_| panel.is_none()) {
            FilterPopover {
                target,
                anchor: trigger,
                opener,
                open: filter_open,
                marker: filter_marker,
            }
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

/// `grouping` with `header` as its innermost level, or without it.
pub(super) fn regrouped(grouping: &[String], header: &str, group: bool) -> Vec<String> {
    let mut next: Vec<String> = grouping.iter().filter(|h| *h != header).cloned().collect();
    if group {
        next.push(header.to_string());
    }
    next
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
                key: header.to_string(),
                sortable: true,
                hideable: true,
                hidden: false,
                groupable: true,
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

    /// 4416: a pin the host refused is dropped unsaid, so the host's own pin later is not said
    /// either; a pin the host stored and the room released is said as released.
    #[test]
    fn a_refused_pin_is_dropped_unsaid() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let owed = || PinNews(CopyValue::new(Some(("Name".to_string(), PinSide::Start))));
            let (none, name) = (
                PinnedColumns::default(),
                PinnedColumns::default().start(["Name"]),
            );

            let refused = owed();
            assert_eq!(refused.settle(&none, &none, |_| false), None);
            assert_eq!(refused.settle(&name, &name, |_| true), None);

            let unmeasured = owed();
            assert_eq!(unmeasured.settle(&name, &none, |_| false), None);
            assert_eq!(
                unmeasured.settle(&name, &none, |_| true),
                Some(("Name".to_string(), PinSide::Start, false))
            );
        });
    }

    #[test]
    fn a_column_toggles_in_and_out_of_the_hidden_list() {
        let hidden = vec!["A".to_string()];

        assert_eq!(toggle_column(&hidden, "B", false), vec!["A", "B"]);
        assert_eq!(toggle_column(&hidden, "A", true), Vec::<String>::new());
        assert_eq!(toggle_column(&hidden, "A", false), vec!["A"]);
    }

    #[test]
    fn group_by_nests_innermost_and_ungroup_keeps_the_rest_in_order() {
        let grouping = vec!["A".to_string(), "B".to_string()];

        assert_eq!(regrouped(&grouping, "C", true), vec!["A", "B", "C"]);
        assert_eq!(regrouped(&grouping, "A", false), vec!["B"]);
        assert_eq!(regrouped(&grouping, "A", true), vec!["B", "A"]);
    }
}
