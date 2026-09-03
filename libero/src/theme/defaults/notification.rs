use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Placement, Size, SizeCss};

/// The stack's width. A notification is as wide as its stack.
pub const NOTIFICATION_WIDTH: CssVar = CssVar::new("--lsx-notification-width");
/// Between two notifications in one stack.
pub const NOTIFICATION_GAP: CssVar = CssVar::new("--lsx-notification-gap");
/// From the viewport's edge to the stack.
pub const NOTIFICATION_OFFSET: CssVar = CssVar::new("--lsx-notification-offset");
/// The entry and the exit, as a CSS `<time>`.
pub const NOTIFICATION_TRANSITION: CssVar = CssVar::new("--lsx-notification-transition");

/// The entry and the exit, appended to the stylesheet beside the other
/// keyframes. Keyframes rather than a transition, because an entry needs no
/// "mounted, not yet visible" render to animate from, and no `transitionend`
/// to finish on - the exit's end is a timer, so reduced motion (no animation
/// at all) and a renderer without animations end it just the same.
///
/// `visibility` interpolates discretely and stays `visible` for the whole run,
/// so the leaving state's own `visibility: hidden` lands when the fade ends and
/// takes the notification out of the accessibility tree with it.
pub const NOTIFICATION_KEYFRAMES: &str = concat!(
    "@keyframes lsx-notification-in{from{opacity:0;transform:translateY(8px);}}",
    "@keyframes lsx-notification-out{from{opacity:1;visibility:visible;}to{opacity:0;visibility:hidden;}}",
);
pub const NOTIFICATION_IN: &str = "lsx-notification-in";
pub const NOTIFICATION_OUT: &str = "lsx-notification-out";

/// When a notification closes on its own.
///
/// An enum and not an `Option<u32>`, because a notification's options need
/// three answers - the host's default, never, or after so long - and an
/// `Option<Option<u32>>` would spell them as a riddle. The options hold an
/// `Option<AutoClose>`, where `None` is the host's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoClose {
    /// Stays until it is closed by hand or by `hide`.
    Never,
    /// Milliseconds, counted while nothing in the stack is hovered or focused.
    After(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationDefaults {
    /// The stack a notification joins when neither the host nor the
    /// notification names one.
    pub position: Placement,
    pub auto_close: AutoClose,
    /// How many notifications one stack shows at once. The rest wait, in
    /// order, and appear as the shown ones close.
    pub limit: usize,
    pub width: &'static str,
    pub gap: Size,
    pub offset: Size,
    /// Milliseconds. Read in Rust too: it is how long a closing notification
    /// stays mounted.
    pub transition_duration: u32,
    /// The default template's close button. An English literal, like
    /// `AlertDefaults`'s (todo 28).
    pub close_label: &'static str,
}

impl ToCssDeclarations for NotificationDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            NOTIFICATION_WIDTH.declare(self.width),
            NOTIFICATION_GAP.declare(SizeCss::SPACING.value(self.gap)),
            NOTIFICATION_OFFSET.declare(SizeCss::SPACING.value(self.offset)),
            NOTIFICATION_TRANSITION.declare(format!("{}ms", self.transition_duration)),
        ]
    }
}
