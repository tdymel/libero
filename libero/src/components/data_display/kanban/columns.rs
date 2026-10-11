/// Which column moved where on the board: from position `from` to `to`, each from 0.
/// Apply it to every list kept by column: the names and the cards.
///
/// ```rust
/// # use libero::components::KanbanColumnMove;
/// let mut names = vec!["To do", "Doing", "Done"];
/// let mut cards = vec![vec!["a"], vec![], vec!["b"]];
/// let step = KanbanColumnMove { from: 0, to: 2 };
/// step.apply(&mut names);
/// step.apply(&mut cards);
/// assert_eq!(names, ["Doing", "Done", "To do"]);
/// assert_eq!(cards, [vec![], vec!["b"], vec!["a"]]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KanbanColumnMove {
    pub from: usize,
    pub to: usize,
}

impl KanbanColumnMove {
    /// Moves `columns[from]` to `to`, shifting the ones between. Out of range, it changes nothing.
    pub fn apply<T>(&self, columns: &mut [T]) {
        let (from, to) = (self.from, self.to);
        if from >= columns.len() || to >= columns.len() {
            return;
        }
        match from < to {
            true => columns[from..=to].rotate_left(1),
            false => columns[to..=from].rotate_right(1),
        }
    }
}

/// A column move told to the app: its words wait until the column shows at its new place,
/// so a move the app refuses is never said (3196).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ColumnLanding {
    step: Option<KanbanColumnMove>,
    landed: bool,
    held: Option<String>,
}

impl ColumnLanding {
    /// The app was told of `step`: the sortable's next words wait for it.
    pub(super) fn expect(&mut self, step: KanbanColumnMove) {
        *self = Self {
            step: Some(step),
            ..Self::default()
        };
    }

    /// The sortable's words: what to say now, `None` while they wait for the move to show.
    pub(super) fn words(&mut self, said: String) -> Option<String> {
        if self.step.is_some() && !self.landed && self.held.is_none() {
            self.held = Some(said);
            return None;
        }
        *self = Self::default();
        Some(said)
    }

    /// A column went from position `from` to `to`: the held words, if that is the move.
    pub(super) fn moved(&mut self, from: usize, to: usize) -> Option<String> {
        if self.step != Some(KanbanColumnMove { from, to }) {
            return None;
        }
        let said = self.held.take();
        match said {
            Some(_) => *self = Self::default(),
            None => self.landed = true,
        }
        said
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: KanbanColumnMove = KanbanColumnMove { from: 0, to: 2 };

    #[test]
    fn a_move_is_said_once_the_column_shows_there() {
        let mut landing = ColumnLanding::default();
        landing.expect(STEP);
        assert_eq!(landing.words("moved".into()), None);
        assert_eq!(landing.moved(1, 0), None);
        assert_eq!(landing.moved(0, 2), Some("moved".into()));
        assert_eq!(landing, ColumnLanding::default());
    }

    #[test]
    fn a_column_shown_before_the_words_lets_them_through() {
        let mut landing = ColumnLanding::default();
        landing.expect(STEP);
        assert_eq!(landing.moved(0, 2), None);
        assert_eq!(landing.words("moved".into()), Some("moved".into()));
    }

    #[test]
    fn a_refused_move_is_never_said_and_the_next_words_are() {
        let mut landing = ColumnLanding::default();
        landing.expect(STEP);
        assert_eq!(landing.words("moved".into()), None);
        assert_eq!(landing.words("lifted".into()), Some("lifted".into()));
        assert_eq!(landing.moved(0, 2), None);
    }

    #[test]
    fn words_without_a_move_are_said_at_once() {
        assert_eq!(
            ColumnLanding::default().words("lifted".into()),
            Some("lifted".into())
        );
    }

    fn moved(from: usize, to: usize) -> Vec<char> {
        let mut columns = vec!['a', 'b', 'c', 'd'];
        KanbanColumnMove { from, to }.apply(&mut columns);
        columns
    }

    #[test]
    fn a_column_moved_later_shifts_the_ones_between_back() {
        assert_eq!(moved(0, 2), ['b', 'c', 'a', 'd']);
        assert_eq!(moved(1, 3), ['a', 'c', 'd', 'b']);
    }

    #[test]
    fn a_column_moved_earlier_shifts_the_ones_between_on() {
        assert_eq!(moved(3, 0), ['d', 'a', 'b', 'c']);
        assert_eq!(moved(2, 1), ['a', 'c', 'b', 'd']);
    }

    #[test]
    fn a_move_in_place_or_out_of_range_changes_nothing() {
        assert_eq!(moved(1, 1), ['a', 'b', 'c', 'd']);
        assert_eq!(moved(4, 0), ['a', 'b', 'c', 'd']);
        assert_eq!(moved(0, 4), ['a', 'b', 'c', 'd']);
    }

    #[test]
    fn the_names_and_the_cards_move_alike() {
        let step = KanbanColumnMove { from: 2, to: 0 };
        let (mut names, mut cards) = (vec!["x", "y", "z"], vec![vec![1], vec![], vec![3, 4]]);
        step.apply(&mut names);
        step.apply(&mut cards);
        assert_eq!(names, ["z", "x", "y"]);
        assert_eq!(cards, [vec![3, 4], vec![1], vec![]]);
    }
}
