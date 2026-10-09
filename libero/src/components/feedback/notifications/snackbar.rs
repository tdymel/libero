use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::handle::{DEFAULT_TEMPLATE_SX, NotificationScope};
use crate::components::{buttons::Button, feedback::Alert};

/// A snackbar's one button. Pressing it runs `onclick`, then closes the notification.
#[derive(Clone)]
pub struct SnackbarAction {
    pub label: String,
    /// An `Rc`, not an `EventHandler`: it must outlive the scope that built it.
    pub onclick: Rc<RefCell<dyn FnMut()>>,
}

impl PartialEq for SnackbarAction {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && Rc::ptr_eq(&self.onclick, &other.onclick)
    }
}

/// What [`snackbar`] shows: a message and at most one action.
#[derive(Clone, PartialEq, Default)]
pub struct SnackbarData {
    pub message: String,
    pub action: Option<SnackbarAction>,
}

impl SnackbarData {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
        }
    }

    /// Adds the button, such as "Undo".
    pub fn action(mut self, label: impl Into<String>, onclick: impl FnMut() + 'static) -> Self {
        self.action = Some(SnackbarAction {
            label: label.into(),
            onclick: Rc::new(RefCell::new(onclick)),
        });
        self
    }
}

impl From<&str> for SnackbarData {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<String> for SnackbarData {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

/// A short message with one action, drawn as an [`Alert`].
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{NotificationOptions, SnackbarData, snackbar, use_notifications_with};
/// # use libero::theme::AutoClose;
/// # fn app() -> Element {
/// let snacks = use_notifications_with(snackbar);
/// snacks.show_with(
///     SnackbarData::new("Message archived.").action("Undo", || { /* restore it */ }),
///     NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() },
/// );
/// # rsx! {}
/// # }
/// ```
///
/// Needs a [`Notifications`](super::Notifications) host. Keep it open with `AutoClose::Never`
/// when the action is the only way back: a keyboard reader needs `F8` to reach it. The handler
/// runs outside the component that showed it, so it writes state owned near the root.
pub fn snackbar(s: NotificationScope<SnackbarData>) -> Element {
    let SnackbarData { message, action } = s.args();
    let actions = action.map(|SnackbarAction { label, onclick }| {
        rsx! {
            Button {
                variant: "standard",
                color: "currentColor",
                size: "xs",
                onclick: move |_| {
                    (onclick.borrow_mut())();
                    s.close();
                },
                "{label}"
            }
        }
    });
    // An empty text node would still get a message slot and `aria-describedby`.
    let message = if message.is_empty() {
        VNode::empty()
    } else {
        rsx! { "{message}" }
    };

    rsx! {
        Alert {
            // Not `alert`: a live region inside another is announced twice or not at all.
            role: "group",
            actions,
            onclose: s.closable().then(|| EventHandler::new(move |()| s.close())),
            sx: &DEFAULT_TEMPLATE_SX,
            {message}
        }
    }
}
