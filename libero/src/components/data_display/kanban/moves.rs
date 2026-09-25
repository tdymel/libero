/// Which card moved where: from slot `from` of column `from_column` to slot
/// `to` of column `to_column`, each from 0.
///
/// ```rust
/// # use libero::components::KanbanMove;
/// let mut board = vec![vec!["a", "b"], vec!["c"]];
/// KanbanMove { from_column: 0, from: 0, to_column: 1, to: 1 }.apply(&mut board);
/// assert_eq!(board, [vec!["b"], vec!["c", "a"]]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KanbanMove {
    pub from_column: usize,
    pub from: usize,
    pub to_column: usize,
    pub to: usize,
}

impl KanbanMove {
    /// Moves the card in `columns`. Out of range, it changes nothing; `to` past
    /// the end lands last.
    pub fn apply<T>(&self, columns: &mut [Vec<T>]) {
        let in_range = columns
            .get(self.from_column)
            .is_some_and(|cards| self.from < cards.len())
            && self.to_column < columns.len();
        if !in_range {
            return;
        }
        let card = columns[self.from_column].remove(self.from);
        let target = &mut columns[self.to_column];
        target.insert(self.to.min(target.len()), card);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Vec<Vec<char>> {
        vec![vec!['a', 'b', 'c'], vec!['d'], vec![]]
    }

    fn moved(from_column: usize, from: usize, to_column: usize, to: usize) -> Vec<Vec<char>> {
        let mut columns = board();
        KanbanMove {
            from_column,
            from,
            to_column,
            to,
        }
        .apply(&mut columns);
        columns
    }

    #[test]
    fn a_move_inside_a_column_reorders_it() {
        assert_eq!(moved(0, 0, 0, 2), [vec!['b', 'c', 'a'], vec!['d'], vec![]]);
        assert_eq!(moved(0, 2, 0, 0), [vec!['c', 'a', 'b'], vec!['d'], vec![]]);
    }

    #[test]
    fn a_move_across_columns_lands_at_the_slot() {
        assert_eq!(moved(0, 1, 1, 0), [vec!['a', 'c'], vec!['b', 'd'], vec![]]);
        assert_eq!(moved(0, 1, 1, 1), [vec!['a', 'c'], vec!['d', 'b'], vec![]]);
    }

    #[test]
    fn a_move_into_an_empty_column_or_past_the_end_lands_last() {
        assert_eq!(moved(1, 0, 2, 0), [vec!['a', 'b', 'c'], vec![], vec!['d']]);
        assert_eq!(
            moved(1, 0, 0, 9),
            [vec!['a', 'b', 'c', 'd'], vec![], vec![]]
        );
    }

    #[test]
    fn a_move_out_of_range_changes_nothing() {
        assert_eq!(moved(1, 1, 0, 0), board());
        assert_eq!(moved(3, 0, 0, 0), board());
        assert_eq!(moved(0, 0, 3, 0), board());
    }
}
