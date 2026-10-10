use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    column::Column,
    pinning::CellPin,
    pipeline::{GroupInfo, VisibleRow, group_rows},
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
    texts_of: Option<TextsInput<T>>,
    texts: GroupTexts,
    rows_of: Option<RowsInput>,
    rows: (Vec<VisibleRow>, Rc<[GroupInfo]>),
}

impl<T> Default for GroupedRows<T> {
    fn default() -> Self {
        Self {
            texts_of: None,
            texts: Rc::default(),
            rows_of: None,
            rows: (Vec::new(), Rc::from([])),
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
        let fresh = self.texts_of.as_ref().is_some_and(|(known, cols, at)| {
            at == levels && cols.as_slice() == columns && (Rc::ptr_eq(known, data) || known == data)
        });
        if !fresh {
            self.texts = Rc::new(
                levels
                    .iter()
                    .map(|&index| data.iter().map(|row| (columns[index].text)(row)).collect())
                    .collect(),
            );
            self.texts_of = Some((data.clone(), columns.to_vec(), levels.to_vec()));
        }
        self.texts.clone()
    }

    /// The `leaves` in their groups, each open group's rows after its row.
    pub fn rows(
        &mut self,
        texts: GroupTexts,
        leaves: &[VisibleRow],
        open: &[String],
        empty: &'static str,
    ) -> (Vec<VisibleRow>, Rc<[GroupInfo]>) {
        let fresh = self
            .rows_of
            .as_ref()
            .is_some_and(|(known, rows, ids, label)| {
                Rc::ptr_eq(known, &texts) && rows == leaves && ids == open && *label == empty
            });
        if !fresh {
            let (rows, groups) =
                group_rows(leaves, &texts, &|id| open.iter().any(|o| o == id), empty);
            self.rows = (rows, groups.into());
            self.rows_of = Some((texts, leaves.to_vec(), open.to_vec(), empty));
        }
        self.rows.clone()
    }
}

/// The group rows' cells: the lead columns spanned by one, then one per shown column.
#[derive(Clone, PartialEq)]
pub(super) struct GroupCells {
    pub leads: usize,
    /// Each shown column's pin, in display order.
    pub pins: Rc<[Option<CellPin>]>,
    pub labels: TableLabels,
    /// Opens or shuts the group of an id, named by a label.
    pub toggle: Callback<(String, String, bool)>,
}

/// A group row: a toggle naming the group and its row count in the first shown
/// column. Its own scope, so a table render that leaves the group as it was skips it.
#[component]
pub(super) fn GroupRow(
    id: String,
    label: String,
    count: usize,
    depth: usize,
    open: bool,
    cells: GroupCells,
) -> Element {
    let GroupCells {
        leads,
        pins,
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
        rsx! {
            td {
                key: "{position}",
                "data-pin": pin.as_ref().map(|pin| pin.side.as_str()),
                "data-pin-edge": pin.as_ref().filter(|pin| pin.edge).map(|_| true),
                style: pin.as_ref().map(CellPin::style),
                {button}
            }
        }
    });
    rsx! {
        tr { "data-row-group": true, "data-level": "{depth}",
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
}
