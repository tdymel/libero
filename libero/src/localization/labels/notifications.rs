/// The `Notifications` host's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationsLabels {
    /// Names the region around the stacks: `{key}` is the key that focuses
    /// the newest notification.
    pub region: &'static str,
}

impl NotificationsLabels {
    pub const ENGLISH: Self = Self {
        region: "Notifications ({key})",
    };

    pub const GERMAN: Self = Self {
        region: "Benachrichtigungen ({key})",
    };
}
