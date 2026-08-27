use dioxus::prelude::*;

use crate::components::OptionLabel;

/// Everything the caller's control needs to be one. `Combobox` renders no
/// field of its own, so the target owns its whole look.
#[derive(Clone)]
pub struct ComboboxTarget {
    /// The selected options, richly. Empty when nothing is selected; length
    /// one today, longer once multi-select lands - which is why it is a `Vec`
    /// rather than an `Option`.
    pub labels: Vec<OptionLabel>,
    pub opened: bool,
    pub disabled: bool,
    /// `id`, `aria-haspopup`, `aria-expanded`, `aria-controls`. Inert strings
    /// only - spread them with `attributes: t.aria`.
    pub aria: Vec<Attribute>,
    /// Opens and closes.
    pub onclick: EventHandler<MouseEvent>,
    /// Enter, Space and ArrowDown open.
    pub onkeydown: EventHandler<KeyboardEvent>,
    /// Selects nothing - for a clear button.
    pub onclear: EventHandler<MouseEvent>,
}
