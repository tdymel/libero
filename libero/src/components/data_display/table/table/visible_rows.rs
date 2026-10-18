//! The row pipeline: key, filter, sort, group or nest, then page the rows.

use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use super::super::{
    column::Column,
    column_filter::CellTest,
    core::{ActiveSort, SortedRows},
    csv::shown_csv,
    filter::{FilteredRows, query_words},
    footer::{FooterCells, FooterRows},
    memo::Memo,
    paging::{TablePager, clamp_page, page_rows},
    pipeline::{GroupInfo, KnownRows, VisibleRow, cut_page, keyed_rows, nest_rows, visible_rows},
    row_groups::{GroupAggregates, GroupedRows, group_levels},
    toolbar::TableTools,
    tree::{TreeLevel, tree_parents, tree_rows, with_ancestors},
    use_table::TableState,
};
use super::TableProps;
use crate::{localization::labels::table::TableLabels, theme::Size, utils::warn};

/// A tree's parents by `data` index, kept while the row keys stay.
type ParentsMemo = Memo<Rc<[String]>, Rc<[usize]>>;

/// Each stage's last inputs and output, so an unchanged stage is not redone.
pub(super) struct RowCaches<T: 'static> {
    known: CopyValue<KnownRows<T>>,
    filtered: CopyValue<FilteredRows<T>>,
    parents: CopyValue<ParentsMemo>,
    sorted: CopyValue<SortedRows<T>>,
    footer: CopyValue<FooterRows<T>>,
    grouped: CopyValue<GroupedRows<T>>,
    pub aggregates: CopyValue<GroupAggregates<T>>,
    group_seeded: CopyValue<bool>,
    nodes_seeded: CopyValue<bool>,
}

// Hand-written: a derive would demand `T: Copy`.
impl<T> Clone for RowCaches<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RowCaches<T> {}

pub(super) fn use_row_caches<T: 'static>() -> RowCaches<T> {
    RowCaches {
        sorted: use_hook(|| CopyValue::new(SortedRows::<T>::default())),
        filtered: use_hook(|| CopyValue::new(FilteredRows::<T>::default())),
        footer: use_hook(|| CopyValue::new(FooterRows::<T>::default())),
        grouped: use_hook(|| CopyValue::new(GroupedRows::<T>::default())),
        aggregates: use_hook(|| CopyValue::new(GroupAggregates::<T>::default())),
        group_seeded: use_hook(|| CopyValue::new(false)),
        parents: use_hook(|| CopyValue::new(ParentsMemo::default())),
        nodes_seeded: use_hook(|| CopyValue::new(false)),
        known: use_hook(|| CopyValue::new(KnownRows::<T>::None)),
    }
}

/// The keyed rows and what the filters keep of them.
pub(super) struct KeptRows<T> {
    pub data: Rc<Vec<T>>,
    pub keys: Rc<[String]>,
    /// The quick filter's words.
    pub words: Vec<String>,
    /// By `data` index; `None` keeps every row.
    pub kept: Option<Rc<Vec<bool>>>,
    /// A filtered tree's ancestors of kept rows, open so the kept rows show.
    pub opened: Option<Vec<bool>>,
    /// Each row's parent by `data` index, itself for a root; `None` without `parent_key`.
    pub parents: Option<Rc<[usize]>>,
}

/// Keys `props.data`, taking it, then filters it unless `manual_filter`.
pub(super) fn keep_rows<T: Clone + PartialEq + 'static>(
    props: &mut TableProps<T>,
    caches: RowCaches<T>,
    state: TableState,
    columns: &[Column<T>],
    hidden: impl Fn(usize) -> bool,
    tests: &[(usize, CellTest)],
) -> KeptRows<T> {
    let RowCaches {
        mut known,
        mut filtered,
        parents: mut tree_parents_memo,
        ..
    } = caches;
    let tree = props.parent_key.is_set();
    let (data, keys) = keyed_rows(
        &known.peek(),
        std::mem::take(&mut props.data),
        props.row_key.is_set(),
        |row| props.row_key.call(row),
    );
    known.set(Some((data.clone(), props.row_key.is_set(), keys.clone())));
    // The quick filter searches the shown, filterable columns.
    let words = query_words(&state.quick_filter.read());
    let searched: Vec<usize> = columns
        .iter()
        .enumerate()
        .filter(|(index, column)| column.filterable && !hidden(*index))
        .map(|(index, _)| index)
        .collect();
    let kept = match props.manual_filter {
        true => None,
        false => filtered.write().kept(
            &data,
            columns,
            &searched,
            &words,
            tests,
            state.filter_logic.read(),
        ),
    };
    let parents = tree.then(|| {
        let fresh = |known: &Rc<[String]>| Rc::ptr_eq(known, &keys);
        let compute = || {
            let named: Vec<Option<String>> = data
                .iter()
                .map(|row| props.parent_key.call(row).flatten())
                .collect();
            let (parent, unknown) = tree_parents(&keys, &named);
            if unknown > 0 {
                warn(&format!(
                    "Table: {unknown} rows name a `parent_key` no row has, so they show as roots."
                ));
            }
            (keys.clone(), Rc::<[usize]>::from(parent))
        };
        tree_parents_memo.write().get(fresh, compute).clone()
    });
    // A filtered tree keeps the kept rows' ancestors, open so the kept rows show.
    let (kept, opened) = match (kept, &parents) {
        (Some(kept), Some(parents)) => {
            let (shown, opened) = with_ancestors(&kept, parents);
            (Some(Rc::new(shown)), Some(opened))
        }
        (kept, _) => (kept, None),
    };
    KeptRows {
        data,
        keys,
        words,
        kept,
        opened,
        parents,
    }
}

/// The kept rows in their shown order, before paging.
pub(super) struct OrderedRows {
    pub order: Vec<VisibleRow>,
    /// The rows passing the filters, group rows not counted.
    pub results: usize,
    pub footer: Option<FooterCells>,
    pub grouped: bool,
    pub groups: Rc<[GroupInfo]>,
    /// A tree's levels by `data` index.
    pub tree_levels: Option<Rc<[Option<TreeLevel>]>>,
}

/// Sorts the kept rows, then groups or nests them; a `toolbar`'s export takes them before.
#[allow(clippy::too_many_arguments)]
pub(super) fn order_rows<T: Clone + PartialEq + 'static>(
    props: &TableProps<T>,
    caches: RowCaches<T>,
    state: TableState,
    rows: &KeptRows<T>,
    columns: &Rc<Vec<Column<T>>>,
    active: &ActiveSort,
    (layout, labels, lang): (&[usize], TableLabels, &'static str),
    tools: TableTools,
) -> OrderedRows {
    let RowCaches {
        mut sorted,
        mut footer,
        mut grouped,
        mut group_seeded,
        mut nodes_seeded,
        ..
    } = caches;
    let KeptRows {
        data,
        keys,
        kept,
        opened,
        parents,
        ..
    } = rows;
    let tree = parents.is_some();
    // Filter, sort, then page; a `manual_*` stage is the caller's. Filtering
    // the sorted order keeps each stage's cache apart: a keystroke never re-sorts.
    let sorted_order = match props.manual_sort {
        true => (0..data.len()).collect(),
        false => sorted.write().order(data, columns, active, lang),
    };
    let kept_rows = kept.as_deref().map(Vec::as_slice);
    let mut order = visible_rows(sorted_order, kept_rows);
    // The rows passing the filters, group rows not counted.
    let results = order.len();
    let footer = footer.write().cells(data, columns, kept_rows, &labels);
    // Groups nest the sorted rows; the footer above and an export count the rows alone.
    let levels = match tree {
        true => Vec::new(),
        false => group_levels(columns, &state.row_grouping.read()),
    };
    let is_grouped = !levels.is_empty();
    let texts = is_grouped.then(|| grouped.write().texts(data, columns, &levels));
    let leaves = texts.as_ref().map(|_| order.clone());
    if props.toolbar.is_some() {
        let (data, columns, shown, order, texts, parents) = (
            data.clone(),
            columns.clone(),
            layout.to_vec(),
            order.clone(),
            texts.clone(),
            parents.clone(),
        );
        let guard = props.csv_formula_guard;
        let mut export = tools.export;
        export.set(Some(Rc::new(move || {
            let rows = match (&texts, &parents) {
                (Some(texts), _) => nest_rows(&order, texts, &|_| true, "").0,
                (_, Some(parents)) => {
                    let leaves: Vec<usize> = order.iter().filter_map(|row| row.data()).collect();
                    tree_rows(&leaves, parents, &|_| true, &|_| false).0
                }
                _ => order.clone(),
            };
            (
                shown_csv(&columns, &shown, &data, &rows, guard),
                order.len(),
            )
        })));
    }
    let groups = match (texts, leaves) {
        (Some(texts), Some(leaves)) => {
            // The first levels open the first time there are rows to group.
            if !*group_seeded.peek() && !data.is_empty() {
                group_seeded.set(true);
                let depth = props.default_group_expansion;
                if depth > 0 && !state.expanded_groups.is_controlled() {
                    let every: Vec<VisibleRow> = (0..data.len()).map(VisibleRow::Data).collect();
                    let mut open = state.expanded_groups.peek();
                    for group in nest_rows(&every, &texts, &|_| true, "").1 {
                        if group.depth < depth && !open.contains(&group.id) {
                            open.push(group.id);
                        }
                    }
                    state.expanded_groups.seed(open);
                }
            }
            let open = state.expanded_groups.read();
            let (rows, groups) = grouped
                .write()
                .rows(texts, &leaves, &open, labels.group_empty);
            order = rows;
            groups
        }
        _ => Rc::from([]),
    };
    // A tree nests the sorted rows under their parents, the first levels open once.
    let tree_levels = parents.as_ref().map(|parents| {
        if !*nodes_seeded.peek() && !data.is_empty() {
            nodes_seeded.set(true);
            let depth = props.default_node_expansion;
            if depth > 0 && !state.expanded_nodes.is_controlled() {
                let every: Vec<usize> = (0..data.len()).collect();
                let mut open = state.expanded_nodes.peek();
                let levels = tree_rows(&every, parents, &|_| true, &|_| false).1;
                for (index, level) in levels.iter().enumerate() {
                    let opens =
                        level.is_some_and(|level| level.depth < depth && level.children > 0);
                    if opens && !open.contains(&keys[index]) {
                        open.push(keys[index].clone());
                    }
                }
                state.expanded_nodes.seed(open);
            }
        }
        let open: HashSet<String> = state.expanded_nodes.read().iter().cloned().collect();
        let leaves: Vec<usize> = order.iter().filter_map(|row| row.data()).collect();
        let forced = |row: usize| opened.as_ref().is_some_and(|opened| opened[row]);
        let lazy = |row: usize| props.row_has_children.call(&data[row]) == Some(true);
        let (rows, levels) = tree_rows(
            &leaves,
            parents,
            &|row| forced(row) || open.contains(&keys[row]),
            &lazy,
        );
        order = rows;
        levels
    });
    OrderedRows {
        order,
        results,
        footer,
        grouped: is_grouped,
        groups,
        tree_levels,
    }
}

/// Cuts `order` to the page shown, with its pager, unless `manual_pagination`. Also
/// the rows on the pages before and every row: a windowed table counts the other pages' too (APG grid, todo 1421).
pub(super) fn page_window<T: Clone + PartialEq + 'static>(
    props: &mut TableProps<T>,
    state: TableState,
    paginated: bool,
    size: Size,
    order: &mut Vec<VisibleRow>,
) -> (Option<Element>, (usize, usize)) {
    if !paginated {
        return (None, (0, order.len()));
    }
    // Group rows take places on a page as the rows do.
    let total = match props.manual_pagination {
        true => props.row_count.unwrap_or(order.len()),
        false => order.len(),
    };
    let page_size = state.page_size.read().max(1);
    let page = clamp_page(total, state.page.read(), page_size);
    let shown = page_rows(total, page, page_size);
    if !props.manual_pagination {
        cut_page(order, shown.clone());
    }
    let counted = (shown.start, total.max(shown.start + order.len()));
    let pager = rsx! {
        TablePager {
            state,
            total,
            page,
            page_size,
            page_sizes: std::mem::take(&mut props.page_sizes),
            caption: props.caption.clone(),
            size,
        }
    };
    (Some(pager), counted)
}
