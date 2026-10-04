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

/// Keyframes, not a transition: no "mounted, not yet visible" render. Compositor-only
/// properties; the item's delayed `visibility` flip leaves the a11y tree at the fade's end.
pub const NOTIFICATION_KEYFRAMES: &str = concat!(
    "@keyframes lsx-notification-in{from{opacity:0;transform:translateY(8px);}}",
    "@keyframes lsx-notification-out{from{opacity:1;}to{opacity:0;}}",
);
pub const NOTIFICATION_IN: &str = "lsx-notification-in";
pub const NOTIFICATION_OUT: &str = "lsx-notification-out";

/// When a notification closes on its own. Options hold an `Option<AutoClose>`,
/// where `None` is the host's default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoClose {
    /// Stays until it is closed by hand or by `hide`.
    Never,
    /// Milliseconds, counted while nothing in the stack is hovered or focused.
    After(u32),
}

/// Theme defaults for `Notifications`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationsDefaults {
    /// The stack a notification joins when neither host nor notification names one.
    pub placement: Placement,
    pub auto_close: AutoClose,
    /// Shown at once per stack; the rest queue in order.
    pub limit: usize,
    pub width: &'static str,
    pub gap: Size,
    pub offset: Size,
    /// Milliseconds. Also how long a closing notification stays mounted.
    pub transition_duration: u32,
}

impl NotificationsDefaults {
    pub const DEFAULT: Self = Self {
        // The corner least likely to cover a page's header and primary actions.
        placement: Placement::BottomEnd,
        // Above the 5 s floor common guidance gives a short message (todo 576).
        auto_close: AutoClose::After(6000),
        limit: 5,
        width: "360px",
        gap: Size::Sm,
        offset: Size::Md,
        transition_duration: 200,
    };
}

impl ToCssDeclarations for NotificationsDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            NOTIFICATION_WIDTH.declare(self.width),
            NOTIFICATION_GAP.declare(SizeCss::SPACING.value(self.gap)),
            NOTIFICATION_OFFSET.declare(SizeCss::SPACING.value(self.offset)),
            NOTIFICATION_TRANSITION.declare(format!("{}ms", self.transition_duration)),
        ]
    }
}
