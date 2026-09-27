/// A `Kanban` card's Move to menu. In `moved` `{label}` is the card's name,
/// `{column}` the column's, `{n}` its position from 1 and `{m}` the card count.
/// The handle, move buttons and drag announcements are [`SortableLabels`](super::SortableLabels).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KanbanLabels {
    /// The menu's trigger; `{label}` names its card.
    pub move_to: &'static str,
    pub moved: &'static str,
}

impl KanbanLabels {
    pub const ENGLISH: Self = Self {
        move_to: "Move {label} to column",
        moved: "{label} moved to {column}, position {n} of {m}.",
    };

    pub const GERMAN: Self = Self {
        move_to: "{label} in Spalte verschieben",
        moved: "{label} nach {column} verschoben, Position {n} von {m}.",
    };
}
