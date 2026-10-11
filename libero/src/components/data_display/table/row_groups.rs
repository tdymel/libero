use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    column::Column,
    footer::aggregate_values,
    memo::{Memo, same_rows},
    pinning::CellPin,
    pipeline::{GroupInfo, VisibleRow, nest_rows},
};
use crate::{
    components::common::Glyph, context::IconSlot, localization::TableLabels,
    theme::TABLE_GROUP_INDENT,
};

/// The grouped columns: each header's first column, unknown and repeated headers skipped.
pub(super) fn group_levels<T>(columns: &[Column<T>], grouping: &[String]) -> Vec<usize> {
    let mut levels = Vec::new();
    for header in grouping {
        if let Some(index) = columns.iter().position(|column| column.header == *header)
            && !levels.contains(&index)
        {
            levels.push(index);
        }
    }
    levels
}

/// Each grouped column's cell texts, by `data` index.
pub(super) type GroupTexts = Rc<Vec<Vec<String>>>;

type TextsInput<T> = (Rc<Vec<T>>, Vec<Column<T>>, Vec<usize>);
type RowsInput = (GroupTexts, Vec<VisibleRow>, Vec<String>, &'static str);

/// The groups across renders: the texts are read again only when the rows or the
/// grouped columns change, the groups when the rows shown or the open groups do.
pub(super) struct GroupedRows<T> {
    texts: Memo<TextsInput<T>, GroupTexts>,
    rows: Memo<RowsInput, (Vec<VisibleRow>, Rc<[GroupInfo]>)>,
}

impl<T> Default for GroupedRows<T> {
    fn default() -> Self {
        Self {
            texts: Memo::default(),
            rows: Memo::default(),
        }
    }
}

impl<T: PartialEq> GroupedRows<T> {
    pub fn texts(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        levels: &[usize],
    ) -> GroupTexts {
        let fresh = |(known, cols, at): &TextsInput<T>| {
            at == levels && cols.as_slice() == columns && same_rows(known, data)
        };
        let compute = || {
            let input = (data.clone(), columns.to_vec(), levels.to_vec());
            let texts = levels
                .iter()
                .map(|&index| data.iter().map(|row| (columns[index].text)(row)).collect())
                .collect();
            (input, Rc::new(texts))
        };
        self.texts.get(fresh, compute).clone()
    }

    /// The `leaves` in their groups, each open group's rows after its row.
    pub fn rows(
        &mut self,
        texts: GroupTexts,
        leaves: &[VisibleRow],
        open: &[String],
        empty: &'static str,
    ) -> (Vec<VisibleRow>, Rc<[GroupInfo]>) {
        let fresh = |(known, rows, ids, label): &RowsInput| {
            Rc::ptr_eq(known, &texts) && rows == leaves && ids == open && *label == empty
        };
        let compute = || {
            let (rows, groups) =
                nest_rows(leaves, &texts, &|id| open.iter().any(|o| o == id), empty);
            let input = (texts.clone(), leaves.to_vec(), open.to_vec(), empty);
            (input, (rows, groups.into()))
        };
        self.rows.get(fresh, compute).clone()
    }
}

/// A group's aggregates by header index.
pub(super) type GroupValues = Rc<[Option<f64>]>;

type KnownValues = HashMap<String, (Vec<usize>, GroupValues)>;
type ValuesInput<T> = (Rc<Vec<T>>, Vec<Column<T>>);

/// The groups' aggregates across renders, by group id: a group keeps its values
/// while its rows stay, so opening a group computes none but its new subgroups'.
pub(super) struct GroupAggregates<T> {
    of: Option<ValuesInput<T>>,
    known: KnownValues,
}

impl<T> Default for GroupAggregates<T> {
    fn default() -> Self {
        Self {
            of: None,
            known: HashMap::new(),
        }
    }
}

impl<T: PartialEq> GroupAggregates<T> {
    /// Each group's aggregates, in `groups` order; `None` when no column aggregates.
    pub fn values(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        groups: &[GroupInfo],
    ) -> Option<Vec<GroupValues>> {
        if groups.is_empty() || columns.iter().all(|column| column.aggregate.is_none()) {
            *self = Self::default();
            return None;
        }
        let fresh = self.of.as_ref().is_some_and(|(known, cols)| {
            cols.as_slice() == columns && (Rc::ptr_eq(known, data) || known == data)
        });
        if !fresh {
            self.known.clear();
            self.of = Some((data.clone(), columns.to_vec()));
        }
        let mut known = std::mem::take(&mut self.known);
        let values = groups
            .iter()
            .map(|group| {
                let values = match known.remove(&group.id) {
                    Some((leaves, values)) if leaves == group.leaves => values,
                    _ => aggregate_values(columns, data, &group.leaves).into(),
                };
                self.known
                    .insert(group.id.clone(), (group.leaves.clone(), values.clone()));
                values
            })
            .collect();
        Some(values)
    }
}

/// The group rows' cells: the lead columns spanned by one, then one per shown column.
#[derive(Clone, PartialEq)]
pub(super) struct GroupCells {
    pub leads: usize,
    /// Each shown column's pin, in display order.
    pub pins: Rc<[Option<CellPin>]>,
    /// Each shown column's `data-align`, in display order.
    pub aligns: Rc<[Option<&'static str>]>,
    pub labels: TableLabels,
    /// Opens or shuts the group of an id, named by a label.
    pub toggle: Callback<(String, String, bool)>,
}

/// A group row: a toggle naming the group and its row count in the first shown
/// column, then each shown column's aggregate over the group's rows, if any. Its
/// own scope, so a table render that leaves the group as it was skips it.
#[component]
pub(super) fn GroupRow(
    id: String,
    label: String,
    count: usize,
    depth: usize,
    open: bool,
    /// An aggregate's name and text per shown column; empty without aggregates.
    #[props(default)]
    aggregates: Rc<[Option<(&'static str, String)>]>,
    cells: GroupCells,
    /// With `cell_focus` on a page: its `aria-rowindex`.
    #[props(default)]
    rowindex: Option<usize>,
) -> Element {
    let GroupCells {
        leads,
        pins,
        aligns,
        labels,
        toggle,
    } = cells;
    let name = (labels.group_toggle)(&label, count);
    let indent = format!(
        "margin-inline-start:calc({depth} * {})",
        TABLE_GROUP_INDENT.value()
    );
    let cells = pins.iter().enumerate().map(|(position, pin)| {
        let button = (position == 0).then(|| {
            let (id, label) = (id.clone(), label.clone());
            rsx! {
                button {
                    r#type: "button",
                    "data-group-toggle": true,
                    aria_label: name.clone(),
                    aria_expanded: "{open}",
                    style: indent.clone(),
                    onclick: move |_| toggle((id.clone(), label.clone(), !open)),
                    Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                    span { "data-group-label": true, "{label}" }
                    span { "data-group-count": true, "({count})" }
                }
            }
        });
        let aggregate = aggregates.get(position).cloned().flatten();
        rsx! {
            td {
                key: "{position}",
                "data-align": aligns.get(position).copied().flatten(),
                "data-pin": pin.as_ref().map(|pin| pin.side.as_str()),
                "data-pin-edge": pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                style: pin.as_ref().map(CellPin::style),
                {button}
                if let Some((name, text)) = aggregate {
                    span { "data-group-aggregate": true,
                        span { "data-footer-name": true, "{name}" }
                        "{text}"
                    }
                }
            }
        }
    });
    rsx! {
        tr { "data-row-group": true, "data-level": "{depth}", aria_rowindex: rowindex.map(|at| at.to_string()),
            if leads > 0 {
                td {
                    key: "lead{leads}",
                    "data-group-lead": true,
                    colspan: (leads > 1).then(|| leads.to_string()),
                }
            }
            {cells}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};

    #[derive(Clone, PartialEq)]
    struct Row {
        team: &'static str,
        city: &'static str,
    }

    fn columns() -> Vec<Column<Row>> {
        vec![
            column("Team").value(|row: &Row| row.team.to_string()),
            column("City").value(|row: &Row| row.city.to_string()),
        ]
    }

    #[test]
    fn levels_name_known_columns_once_in_grouping_order() {
        let grouping = ["City", "Nope", "Team", "City"].map(String::from);
        assert_eq!(group_levels(&columns(), &grouping), [1, 0]);
    }

    #[test]
    fn texts_and_groups_are_reused_while_their_inputs_stay() {
        let data = Rc::new(vec![
            Row {
                team: "a",
                city: "x",
            },
            Row {
                team: "b",
                city: "x",
            },
        ]);
        let mut cache = GroupedRows::default();
        let texts = cache.texts(&data, &columns(), &[1]);
        assert_eq!(*texts, [["x", "x"]]);
        assert!(Rc::ptr_eq(&texts, &cache.texts(&data, &columns(), &[1])));
        let leaves = [VisibleRow::Data(0), VisibleRow::Data(1)];
        let (rows, groups) = cache.rows(texts.clone(), &leaves, &[], "-");
        assert_eq!(rows, [VisibleRow::Group(0)]);
        let (_, again) = cache.rows(texts.clone(), &leaves, &[], "-");
        assert!(Rc::ptr_eq(&groups, &again));
        let (rows, _) = cache.rows(texts, &leaves, &["x".into()], "-");
        assert_eq!(rows.len(), 3);
    }

    fn group(id: &str, leaves: &[usize]) -> GroupInfo {
        GroupInfo {
            path: vec![id.to_string()],
            depth: 0,
            label: id.to_string(),
            leaves: leaves.to_vec(),
            id: id.to_string(),
            open: false,
        }
    }

    #[test]
    fn group_aggregates_sum_each_groups_rows_and_are_reused_by_id() {
        use std::cell::Cell;

        use crate::localization::Aggregate;

        let calls = Rc::new(Cell::new(0));
        let counted = calls.clone();
        let columns = vec![
            column("N")
                .value(move |n: &u32| {
                    counted.set(counted.get() + 1);
                    *n
                })
                .aggregate(Aggregate::Sum),
            column("Text").value(|n: &u32| n.to_string()),
        ];
        let data = Rc::new(vec![1, 2, 3, 4]);
        let mut cache = GroupAggregates::default();
        let groups = [group("a", &[0, 2]), group("b", &[1, 3])];
        let values = cache.values(&data, &columns, &groups).unwrap();
        assert_eq!(
            (&*values[0], &*values[1]),
            (&[Some(4.0), None][..], &[Some(6.0), None][..])
        );
        assert_eq!(calls.get(), 4);
        // A new group list with "a" unchanged: only "c" is summed.
        let again = cache
            .values(&data, &columns, &[group("a", &[0, 2]), group("c", &[3])])
            .unwrap();
        assert!(Rc::ptr_eq(&values[0], &again[0]));
        assert_eq!((again[1][0], calls.get()), (Some(4.0), 5));
        assert!(cache.values(&data, &columns[1..], &groups).is_none());
    }
}
