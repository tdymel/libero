use std::{any::Any, cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::common::{Input, Variant},
    sx::ThemeAwareValue,
    theme::{AutoClose, Placement},
};

/// Names one notification, for [`NotificationHandle::update`](super::NotificationHandle::update)
/// and [`NotificationHandle::hide`](super::NotificationHandle::hide).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotificationId(pub(super) u64);

/// Which of the two live regions a notification is announced from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotificationLive {
    /// Announced when the reader is idle.
    #[default]
    Polite,
    /// Interrupts the reader.
    Assertive,
}

/// Per notification. Every field defaults to the host's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationOptions {
    /// The stack it joins. `None` is the `Notifications` host's `placement`.
    pub placement: Option<Placement>,
    /// `None` is the host's `auto_close`.
    pub auto_close: Option<AutoClose>,
    /// Offer a close control; see [`NotificationScope::closable`](super::NotificationScope::closable).
    pub closable: bool,
    pub live: NotificationLive,
}

impl Default for NotificationOptions {
    fn default() -> Self {
        Self {
            placement: None,
            auto_close: None,
            closable: true,
            live: NotificationLive::default(),
        }
    }
}

/// What the default template shows: an [`Alert`](crate::components::Alert).
#[derive(Clone, PartialEq, Default)]
pub struct NotificationData {
    pub title: Option<String>,
    /// Empty renders no message slot.
    pub message: String,
    /// `Alert`'s `color`.
    pub color: Input<ThemeAwareValue>,
    /// `Alert`'s `variant`.
    pub variant: Input<Variant>,
    /// A glyph only, without event handlers: it may outlive the scope that built it.
    pub icon: Option<Element>,
}

impl From<&str> for NotificationData {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

impl From<String> for NotificationData {
    fn from(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
}

/// Called with whether the notification stays until closed.
pub(super) type Draw = Rc<dyn Fn(bool) -> Element>;

/// A notification with its type erased, so the store, the timers and the host
/// compile once whatever `T`s an app uses.
pub(super) struct Entry {
    pub(super) id: NotificationId,
    /// The `T`. Its own signal, so an `update` redraws only this template.
    pub(super) args: Signal<std::boxed::Box<dyn Any>>,
    /// The template with `T` still known.
    pub(super) draw: Draw,
    pub(super) placement: Option<Placement>,
    /// Stays in its first stack when the host's `placement` changes: a move
    /// would remount, re-announce and restart the timer (todo 577).
    pub(super) drawn_in: Cell<Option<Placement>>,
    pub(super) auto_close: Option<AutoClose>,
    pub(super) live: NotificationLive,
    /// The exit is running; removed when it ends.
    pub(super) leaving: bool,
    /// Has been on screen. A queued one has no exit to run.
    pub(super) shown: Cell<bool>,
}

impl Drop for Entry {
    /// The root owns `args`, so nothing else would ever drop it.
    fn drop(&mut self) {
        self.args.manually_drop();
    }
}
