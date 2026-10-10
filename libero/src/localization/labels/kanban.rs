/// A `Kanban` card's Move to menu. In `moved` `{label}` is the card's name,
/// `{column}` the column's, `{n}` its position from 1 and `{m}` the card count.
/// The handle, move buttons and drag announcements are [`SortableLabels`](super::SortableLabels).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KanbanLabels {
    /// The menu's trigger; `{label}` names its card.
    pub move_to: &'static str,
    pub moved: &'static str,
    /// Replaces `SortableLabels::instructions`: the keys of a card lifted on a board.
    pub instructions: &'static str,
    /// Left or Right where the board has no column on that side; `{label}` names the card.
    pub no_column: &'static str,
    /// Replaces `SortableLabels::touch_instructions` on a touch screen: the move
    /// buttons and the Move to menu.
    pub touch_instructions: &'static str,
    /// Replaces it on a touch screen with `move_buttons` off: only the Move to menu is left.
    pub touch_move_to: &'static str,
}

impl KanbanLabels {
    pub const ENGLISH: Self = Self {
        move_to: "Move {label} to column",
        moved: "{label} moved to {column}, position {n} of {m}.",
        instructions: "Press Space to lift the card, Up and Down to move it in its column, Left and Right to put it in the next column, Space again to drop it, Escape to cancel.",
        no_column: "{label} has no column on that side.",
        touch_instructions: "Use the move buttons to reorder the card, or the Move to column button to put it in another column.",
        touch_move_to: "Use the Move to column button to put the card in another column.",
    };

    pub const GERMAN: Self = Self {
        move_to: "{label} in Spalte verschieben",
        moved: "{label} nach {column} verschoben, Position {n} von {m}.",
        instructions: "Leertaste hebt die Karte an, Pfeil nach oben und unten verschieben sie in ihrer Spalte, Pfeil nach links und rechts setzen sie in die nächste Spalte, erneut Leertaste legt sie ab, Escape bricht ab.",
        no_column: "{label} hat auf dieser Seite keine Spalte.",
        touch_instructions: "Die Verschiebe-Schaltflächen ordnen die Karte neu an, die Schaltfläche „In Spalte verschieben“ setzt sie in eine andere Spalte.",
        touch_move_to: "Die Schaltfläche „In Spalte verschieben“ setzt die Karte in eine andere Spalte.",
    };
}
