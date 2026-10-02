/// A `FloatingWindow`'s handles and its title-bar menu. Its close button reads
/// [`CommonLabels::close`](super::CommonLabels::close).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatingWindowLabels {
    /// Names the title bar, which is the keyboard move handle.
    pub move_handle: &'static str,
    /// Names the group of move buttons, apart from the handle.
    pub move_steps: &'static str,
    /// The move handle's description: how to move it.
    pub move_hint: &'static str,
    /// Names the corner resize handle.
    pub resize_handle: &'static str,
    /// Names the group of resize buttons, apart from the handle.
    pub resize_steps: &'static str,
    /// The resize handle's value text: `{width}` and `{height}` in pixels.
    pub size: &'static str,
    /// Names the title bar's menu button.
    pub menu: &'static str,
    /// The menu item that shows the move buttons.
    pub move_item: &'static str,
    /// The menu item that shows the resize buttons.
    pub resize_item: &'static str,
    /// The menu item that puts the window back where and how it opened.
    pub reset_item: &'static str,
    pub move_up: &'static str,
    pub move_down: &'static str,
    pub move_left: &'static str,
    pub move_right: &'static str,
    pub narrower: &'static str,
    pub wider: &'static str,
    pub shorter: &'static str,
    pub taller: &'static str,
    /// Hides the move or resize buttons again.
    pub done: &'static str,
}

impl FloatingWindowLabels {
    pub const ENGLISH: Self = Self {
        move_handle: "Move window",
        move_steps: "Move window by steps",
        move_hint: "Use arrow keys to move the window",
        resize_handle: "Resize window",
        resize_steps: "Resize window by steps",
        size: "{width} by {height} pixels",
        menu: "Window menu",
        move_item: "Move",
        resize_item: "Resize",
        reset_item: "Reset position and size",
        move_up: "Move up",
        move_down: "Move down",
        move_left: "Move left",
        move_right: "Move right",
        narrower: "Narrower",
        wider: "Wider",
        shorter: "Shorter",
        taller: "Taller",
        done: "Done",
    };

    pub const GERMAN: Self = Self {
        move_handle: "Fenster verschieben",
        move_steps: "Fenster schrittweise verschieben",
        move_hint: "Mit den Pfeiltasten das Fenster verschieben",
        resize_handle: "Fenstergröße ändern",
        resize_steps: "Fenstergröße schrittweise ändern",
        size: "{width} mal {height} Pixel",
        menu: "Fenstermenü",
        move_item: "Verschieben",
        resize_item: "Größe ändern",
        reset_item: "Position und Größe zurücksetzen",
        move_up: "Nach oben",
        move_down: "Nach unten",
        move_left: "Nach links",
        move_right: "Nach rechts",
        narrower: "Schmaler",
        wider: "Breiter",
        shorter: "Niedriger",
        taller: "Höher",
        done: "Fertig",
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2012: the step row is a group of its own, not a second handle.
    #[test]
    fn the_step_groups_are_named_apart_from_the_handles() {
        for labels in [FloatingWindowLabels::ENGLISH, FloatingWindowLabels::GERMAN] {
            assert_ne!(labels.move_steps, labels.move_handle);
            assert_ne!(labels.resize_steps, labels.resize_handle);
        }
    }
}
