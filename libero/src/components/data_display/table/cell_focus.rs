//! The grid's key map and steps, pure: positions in, positions out.

use dioxus::prelude::{Key, NamedKey};

use super::{header_groups::HeaderCell, pipeline::VisibleRow};
use crate::platform::logical_arrow;

/// A row of the grid: a header row by level, top first, or a body row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GridRow {
    Head(usize),
    Body(VisibleRow),
}

/// Where the grid's tab stop sits: a row, and a cell of it by header index, which
/// a hide or move of other columns leaves alone. `None` is the row itself. A
/// header cell is named by the level it starts at and its first column's index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FocusTarget {
    pub row: GridRow,
    pub col: Option<usize>,
}

/// A cell by position: its row among the shown ones, the first shown column it covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pos {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Move {
    Left,
    Right,
    Up,
    Down,
    RowStart,
    RowEnd,
    GridStart,
    GridEnd,
    PageUp(usize),
    PageDown(usize),
}

/// The move a key asks for, `page` rows for PageUp and PageDown. `ctrl` takes Home
/// and End to the grid's corners and leaves every other key alone.
pub(super) fn key_move(key: Key, ctrl: bool, rtl: bool, page: usize) -> Option<Move> {
    let page = page.max(1);
    Some(match (logical_arrow(key, rtl), ctrl) {
        (Key::Named(NamedKey::ArrowLeft), false) => Move::Left,
        (Key::Named(NamedKey::ArrowRight), false) => Move::Right,
        (Key::Named(NamedKey::ArrowUp), false) => Move::Up,
        (Key::Named(NamedKey::ArrowDown), false) => Move::Down,
        (Key::Named(NamedKey::Home), false) => Move::RowStart,
        (Key::Named(NamedKey::End), false) => Move::RowEnd,
        (Key::Named(NamedKey::Home), true) => Move::GridStart,
        (Key::Named(NamedKey::End), true) => Move::GridEnd,
        (Key::Named(NamedKey::PageUp), false) => Move::PageUp(page),
        (Key::Named(NamedKey::PageDown), false) => Move::PageDown(page),
        _ => return None,
    })
}

/// A header cell as the arrows see it: its first shown column, its width, and the
/// level it starts at, as a column's header spans down to the last level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HeadCell {
    pub start: usize,
    pub span: usize,
    pub level: usize,
}

/// Each header level's cells by start, a column's header in every level it spans.
pub(super) fn head_levels(rows: &[Vec<HeaderCell>], layout: &[usize]) -> Vec<Vec<HeadCell>> {
    let mut levels = vec![Vec::new(); rows.len()];
    for (level, cells) in rows.iter().enumerate() {
        for cell in cells {
            let (start, span, down) = match *cell {
                HeaderCell::Group { start, span, .. } => (start, span.max(1), 1),
                HeaderCell::Column { index, rowspan } => {
                    let start = layout.iter().position(|&shown| shown == index);
                    (start.unwrap_or(0), 1, rowspan.max(1))
                }
            };
            for below in levels.iter_mut().skip(level).take(down) {
                below.push(HeadCell { start, span, level });
            }
        }
    }
    for cells in &mut levels {
        cells.sort_by_key(|cell| cell.start);
    }
    levels
}

/// The first column of the cell covering column `col`, in a row of cells `spans` wide.
pub(super) fn owner(spans: &[usize], col: usize) -> usize {
    let mut start = 0;
    for &span in spans {
        if col < start + span.max(1) {
            return start;
        }
        start += span.max(1);
    }
    // Past the row's end: its last cell.
    start - spans.last().map_or(0, |&span| span.max(1))
}

/// Where `by` takes the cell at `at` among `rows` rows, whose cells `spans(row)`
/// lists by width. Stops at the edges, no wrap (APG data grid).
pub(super) fn step(at: Pos, by: Move, rows: usize, spans: impl Fn(usize) -> Vec<usize>) -> Pos {
    if rows == 0 {
        return at;
    }
    let last_row = rows - 1;
    let in_row = |row: usize, col: usize| Pos {
        row,
        col: owner(&spans(row), col),
    };
    let width = |row: usize| spans(row).iter().map(|&span| span.max(1)).sum::<usize>();
    let here = spans(at.row);
    let start = owner(&here, at.col);
    match by {
        Move::Left => in_row(at.row, start.saturating_sub(1)),
        Move::Right => {
            let span = here
                .iter()
                .scan(0, |next, &span| {
                    let first = *next;
                    *next += span.max(1);
                    Some((first, span.max(1)))
                })
                .find(|&(first, _)| first == start)
                .map_or(1, |(_, span)| span);
            in_row(at.row, (start + span).min(width(at.row).saturating_sub(1)))
        }
        Move::Up => in_row(at.row.saturating_sub(1), at.col),
        Move::Down => in_row((at.row + 1).min(last_row), at.col),
        Move::RowStart => in_row(at.row, 0),
        Move::RowEnd => in_row(at.row, width(at.row).saturating_sub(1)),
        Move::GridStart => in_row(0, 0),
        Move::GridEnd => in_row(last_row, width(last_row).saturating_sub(1)),
        Move::PageUp(page) => in_row(at.row.saturating_sub(page), at.col),
        Move::PageDown(page) => in_row((at.row + page).min(last_row), at.col),
    }
}

/// A shown tree row as the keys see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TreeNode {
    /// Its parent's place among the shown rows; `None` for a root.
    pub parent: Option<usize>,
    /// Whether its children show; `None` for a leaf.
    pub open: Option<bool>,
}

/// What a key does on a focused tree row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TreeAct {
    /// Focus the shown row at this place.
    Row(usize),
    FirstCell,
    /// Open (`true`) or close the row.
    Toggle(bool),
    Stay,
}

/// The APG treegrid keys on the row at `at` among the shown `nodes`; `cells`
/// when its cells take focus too. Without them Right steps into an open row's children.
pub(super) fn tree_step(at: usize, by: Move, nodes: &[TreeNode], cells: bool) -> TreeAct {
    let Some(node) = nodes.get(at) else {
        return TreeAct::Stay;
    };
    let last = nodes.len() - 1;
    // An open row whose children have not loaded yet has none to step to.
    let child = nodes
        .get(at + 1)
        .is_some_and(|next| next.parent == Some(at));
    match by {
        Move::Right => match node.open {
            Some(false) => TreeAct::Toggle(true),
            _ if cells => TreeAct::FirstCell,
            Some(true) if child => TreeAct::Row(at + 1),
            _ => TreeAct::Stay,
        },
        Move::Left => match (node.open, node.parent) {
            (Some(true), _) => TreeAct::Toggle(false),
            (_, Some(parent)) => TreeAct::Row(parent),
            _ => TreeAct::Stay,
        },
        Move::Up => TreeAct::Row(at.saturating_sub(1)),
        Move::Down => TreeAct::Row((at + 1).min(last)),
        Move::RowStart | Move::GridStart => TreeAct::Row(0),
        Move::RowEnd | Move::GridEnd => TreeAct::Row(last),
        Move::PageUp(page) => TreeAct::Row(at.saturating_sub(page)),
        Move::PageDown(page) => TreeAct::Row((at + page).min(last)),
    }
}

/// The stop's cell among the shown `order` and `layout` (header indices in display
/// order), else the first cell: its row or column is gone. `None` without a cell.
pub(super) fn resolve(
    stop: Option<FocusTarget>,
    order: &[GridRow],
    layout: &[usize],
    spans: impl Fn(usize) -> Vec<usize>,
) -> Option<Pos> {
    if order.is_empty() || layout.is_empty() {
        return None;
    }
    let (row, col) = stop
        .and_then(|stop| {
            let row = order.iter().position(|&row| row == stop.row)?;
            let col = match stop.col {
                Some(col) => layout.iter().position(|&shown| shown == col)?,
                None => 0,
            };
            Some((row, col))
        })
        .unwrap_or((0, 0));
    Some(Pos {
        row,
        col: owner(&spans(row), col),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(row: usize, col: usize) -> Pos {
        Pos { row, col }
    }

    /// Three rows of four single cells.
    fn plain(_: usize) -> Vec<usize> {
        vec![1, 1, 1, 1]
    }

    #[test]
    fn an_arrow_stops_at_the_edge() {
        assert_eq!(step(at(0, 0), Move::Left, 3, plain), at(0, 0));
        assert_eq!(step(at(0, 0), Move::Up, 3, plain), at(0, 0));
        assert_eq!(step(at(2, 3), Move::Right, 3, plain), at(2, 3));
        assert_eq!(step(at(2, 3), Move::Down, 3, plain), at(2, 3));
        assert_eq!(step(at(1, 1), Move::Right, 3, plain), at(1, 2));
        assert_eq!(step(at(1, 1), Move::Down, 3, plain), at(2, 1));
    }

    #[test]
    fn home_and_end_stay_in_the_row() {
        assert_eq!(step(at(1, 2), Move::RowStart, 3, plain), at(1, 0));
        assert_eq!(step(at(1, 1), Move::RowEnd, 3, plain), at(1, 3));
    }

    #[test]
    fn ctrl_home_and_end_reach_the_grid_corners() {
        assert_eq!(
            key_move(Key::Named(NamedKey::Home), true, false, 1),
            Some(Move::GridStart)
        );
        assert_eq!(
            key_move(Key::Named(NamedKey::End), true, false, 1),
            Some(Move::GridEnd)
        );
        assert_eq!(
            key_move(Key::Named(NamedKey::ArrowDown), true, false, 1),
            None
        );
        assert_eq!(step(at(1, 2), Move::GridStart, 3, plain), at(0, 0));
        assert_eq!(step(at(1, 2), Move::GridEnd, 3, plain), at(2, 3));
    }

    #[test]
    fn page_down_moves_by_the_given_rows() {
        assert_eq!(
            key_move(Key::Named(NamedKey::PageDown), false, false, 0),
            Some(Move::PageDown(1))
        );
        assert_eq!(step(at(0, 1), Move::PageDown(5), 20, plain), at(5, 1));
        assert_eq!(step(at(18, 1), Move::PageDown(5), 20, plain), at(19, 1));
        assert_eq!(step(at(3, 1), Move::PageUp(5), 20, plain), at(0, 1));
    }

    #[test]
    fn rtl_swaps_sideways_arrows() {
        assert_eq!(
            key_move(Key::Named(NamedKey::ArrowLeft), false, true, 1),
            Some(Move::Right)
        );
        assert_eq!(
            key_move(Key::Named(NamedKey::ArrowRight), false, true, 1),
            Some(Move::Left)
        );
        assert_eq!(
            key_move(Key::Named(NamedKey::ArrowUp), false, true, 1),
            Some(Move::Up)
        );
        assert_eq!(
            key_move(Key::Named(NamedKey::Home), false, true, 1),
            Some(Move::RowStart)
        );
    }

    /// Row 1 spans its first three columns in one cell.
    #[test]
    fn a_spanned_cell_owns_its_columns() {
        let spans = |row: usize| {
            if row == 1 {
                vec![3, 1]
            } else {
                vec![1, 1, 1, 1]
            }
        };
        assert_eq!(step(at(0, 2), Move::Down, 3, spans), at(1, 0));
        assert_eq!(step(at(1, 0), Move::Right, 3, spans), at(1, 3));
        assert_eq!(step(at(1, 3), Move::Left, 3, spans), at(1, 0));
        assert_eq!(step(at(1, 3), Move::Up, 3, spans), at(0, 3));
        assert_eq!(owner(&[3, 1], 2), 0);
        assert_eq!(owner(&[3, 1], 9), 3);
    }

    #[test]
    fn a_lost_stop_falls_back_to_the_first_cell() {
        let order = [2, 0, 1].map(|index| GridRow::Body(VisibleRow::Data(index)));
        let layout = [4, 1, 3];
        let stop = |row, col| {
            Some(FocusTarget {
                row: GridRow::Body(VisibleRow::Data(row)),
                col: Some(col),
            })
        };
        let three = |_| vec![1, 1, 1];
        assert_eq!(resolve(stop(0, 3), &order, &layout, three), Some(at(1, 2)));
        assert_eq!(resolve(stop(9, 3), &order, &layout, three), Some(at(0, 0)));
        assert_eq!(resolve(stop(0, 7), &order, &layout, three), Some(at(0, 0)));
        assert_eq!(resolve(None, &order, &layout, three), Some(at(0, 0)));
        assert_eq!(resolve(None, &[], &layout, three), None);
    }

    /// a (open) > b (closed), c (leaf); d a closed root.
    fn tree() -> Vec<TreeNode> {
        let node = |parent, open| TreeNode { parent, open };
        vec![
            node(None, Some(true)),
            node(Some(0), Some(false)),
            node(Some(0), None),
            node(None, Some(false)),
        ]
    }

    #[test]
    fn right_opens_a_closed_row_then_steps_in() {
        let nodes = tree();
        assert_eq!(
            tree_step(1, Move::Right, &nodes, false),
            TreeAct::Toggle(true)
        );
        assert_eq!(tree_step(0, Move::Right, &nodes, false), TreeAct::Row(1));
        assert_eq!(tree_step(2, Move::Right, &nodes, false), TreeAct::Stay);
        assert_eq!(tree_step(0, Move::Right, &nodes, true), TreeAct::FirstCell);
        assert_eq!(tree_step(2, Move::Right, &nodes, true), TreeAct::FirstCell);
        assert_eq!(
            tree_step(3, Move::Right, &nodes, true),
            TreeAct::Toggle(true)
        );
        // Open, its children not loaded yet: the next row is a sibling.
        let mut waiting = tree();
        waiting[1].open = Some(true);
        assert_eq!(tree_step(1, Move::Right, &waiting, false), TreeAct::Stay);
    }

    #[test]
    fn left_closes_an_open_row_else_goes_to_its_parent() {
        let nodes = tree();
        assert_eq!(
            tree_step(0, Move::Left, &nodes, false),
            TreeAct::Toggle(false)
        );
        assert_eq!(tree_step(1, Move::Left, &nodes, false), TreeAct::Row(0));
        assert_eq!(tree_step(2, Move::Left, &nodes, true), TreeAct::Row(0));
        assert_eq!(tree_step(3, Move::Left, &nodes, false), TreeAct::Stay);
    }

    #[test]
    fn the_vertical_keys_move_between_rows_without_wrap() {
        let nodes = tree();
        assert_eq!(tree_step(0, Move::Up, &nodes, false), TreeAct::Row(0));
        assert_eq!(tree_step(3, Move::Down, &nodes, false), TreeAct::Row(3));
        assert_eq!(tree_step(1, Move::Down, &nodes, false), TreeAct::Row(2));
        assert_eq!(tree_step(2, Move::RowStart, &nodes, false), TreeAct::Row(0));
        assert_eq!(tree_step(1, Move::RowEnd, &nodes, true), TreeAct::Row(3));
        assert_eq!(
            tree_step(0, Move::PageDown(2), &nodes, false),
            TreeAct::Row(2)
        );
        assert_eq!(
            tree_step(1, Move::PageUp(5), &nodes, false),
            TreeAct::Row(0)
        );
    }

    /// "Name" over columns 7 and 8, column 5 beside it spanning both levels.
    #[test]
    fn a_column_header_lives_in_every_level_it_spans() {
        let rows = vec![
            vec![
                HeaderCell::Column {
                    index: 5,
                    rowspan: 2,
                },
                HeaderCell::Group {
                    name: "Name".into(),
                    start: 1,
                    span: 2,
                },
            ],
            vec![
                HeaderCell::Column {
                    index: 7,
                    rowspan: 1,
                },
                HeaderCell::Column {
                    index: 8,
                    rowspan: 1,
                },
            ],
        ];
        let cell = |start, span, level| HeadCell { start, span, level };
        assert_eq!(
            head_levels(&rows, &[5, 7, 8]),
            vec![
                vec![cell(0, 1, 0), cell(1, 2, 0)],
                vec![cell(0, 1, 0), cell(1, 1, 1), cell(2, 1, 1)],
            ]
        );
    }
}
