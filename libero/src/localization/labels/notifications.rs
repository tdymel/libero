/// The `Notifications` host's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationsLabels {
    /// Names the region around the stacks. The hotkey goes to `aria-keyshortcuts`,
    /// so a phone reader does not hear a key it cannot press.
    pub region: &'static str,
}

impl NotificationsLabels {
    pub const ENGLISH: Self = Self {
        region: "Notifications",
    };

    pub const GERMAN: Self = Self {
        region: "Benachrichtigungen",
    };
}
