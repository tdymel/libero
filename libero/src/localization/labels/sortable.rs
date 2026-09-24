/// A `Sortable`'s handle, move buttons and announcements. In the templates `{n}`
/// is a position from 1, `{m}` the item count, `{label}` the item's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortableLabels {
    pub handle: &'static str,
    /// How to move an item by keyboard, read with the handle.
    pub instructions: &'static str,
    /// Names an item without a `label` by where it was lifted.
    pub item: &'static str,
    pub lifted: &'static str,
    pub moved: &'static str,
    pub dropped: &'static str,
    pub cancelled: &'static str,
    /// The single-pointer buttons; "backward"/"forward" name a row's, as in `ScrollerLabels`.
    pub move_up: &'static str,
    pub move_down: &'static str,
    pub move_backward: &'static str,
    pub move_forward: &'static str,
}

impl SortableLabels {
    pub const ENGLISH: Self = Self {
        handle: "Reorder",
        instructions: "Press Space to lift the item, the arrow keys to move it, Space again to drop it, Escape to cancel.",
        item: "Item {n}",
        lifted: "Lifted {label}, position {n} of {m}.",
        moved: "{label} moved to position {n} of {m}.",
        dropped: "Dropped {label} at position {n} of {m}.",
        cancelled: "Cancelled. {label} back at position {n} of {m}.",
        move_up: "Move up",
        move_down: "Move down",
        move_backward: "Move backward",
        move_forward: "Move forward",
    };

    pub const GERMAN: Self = Self {
        handle: "Neu anordnen",
        instructions: "Leertaste hebt das Element an, die Pfeiltasten verschieben es, erneut Leertaste legt es ab, Escape bricht ab.",
        item: "Element {n}",
        lifted: "{label} angehoben, Position {n} von {m}.",
        moved: "{label} an Position {n} von {m} verschoben.",
        dropped: "{label} an Position {n} von {m} abgelegt.",
        cancelled: "Abgebrochen. {label} wieder an Position {n} von {m}.",
        move_up: "Nach oben",
        move_down: "Nach unten",
        move_backward: "Nach vorne",
        move_forward: "Nach hinten",
    };
}
