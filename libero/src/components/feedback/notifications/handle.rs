use std::{any::Any, cell::Cell, marker::PhantomData, rc::Rc, sync::atomic::Ordering};

use dioxus::prelude::*;

use super::{
    data::{Draw, Entry, NotificationData, NotificationId, NotificationOptions},
    store::{NEXT_ID, NotificationStore, use_notification_store},
};
use crate::{
    components::feedback::Alert,
    sx::{StaticSx, sx},
    theme::{Size, SizeCss},
    utils::warn,
};

/// Floats over the page, so it takes back the shadow `Alert` drops.
static DEFAULT_TEMPLATE_SX: StaticSx =
    StaticSx::new(|| sx().box_shadow(SizeCss::SHADOW.value(Size::Md)));

/// A template's view of the notification it draws.
pub struct NotificationScope<T: 'static> {
    id: NotificationId,
    store: NotificationStore,
    args: Signal<std::boxed::Box<dyn Any>>,
    closable: bool,
    /// Resolves to `AutoClose::Never`: only a close control or `hide` ends it.
    stays: bool,
    ty: PhantomData<fn() -> T>,
}

impl<T: 'static> Clone for NotificationScope<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for NotificationScope<T> {}

impl<T: 'static> NotificationScope<T> {
    pub fn id(&self) -> NotificationId {
        self.id
    }

    /// Starts this notification's exit.
    pub fn close(&self) {
        self.store.hide(self.id);
    }

    /// [`NotificationOptions::closable`]: whether to draw a close control.
    pub fn closable(&self) -> bool {
        self.closable
    }
}

impl<T: Clone + 'static> NotificationScope<T> {
    /// The data it was shown with, or last updated to.
    pub fn args(&self) -> T {
        self.args
            .try_read()
            .ok()
            .and_then(|args| args.downcast_ref::<T>().cloned())
            .expect("NotificationScope used after its notification was removed")
    }
}

/// Shows, updates and hides notifications drawn by one template.
///
/// `show` queues: past the host's `limit`, a notification waits its turn.
pub struct NotificationHandle<T: 'static> {
    store: NotificationStore,
    template: fn(NotificationScope<T>) -> Element,
}

impl<T: 'static> Clone for NotificationHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for NotificationHandle<T> {}

impl<T: 'static> NotificationHandle<T> {
    pub fn show(&self, args: impl Into<T>) -> NotificationId {
        self.show_with(args, NotificationOptions::default())
    }

    pub fn show_with(&self, args: impl Into<T>, options: NotificationOptions) -> NotificationId {
        let id = NotificationId(NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let store = self.store;
        if !store.alive() {
            return id;
        }
        let template = self.template;
        let args =
            store.owned_signal(std::boxed::Box::new(args.into()) as std::boxed::Box<dyn Any>);
        let closable = options.closable;
        let draw: Draw = Rc::new(move |stays| {
            template(NotificationScope {
                id,
                store,
                args,
                closable,
                stays,
                ty: PhantomData,
            })
        });

        let mut entries = self.store.entries;
        entries.write().push(Entry {
            id,
            args,
            draw,
            placement: options.placement,
            drawn_in: Cell::new(None),
            auto_close: options.auto_close,
            live: options.live,
            leaving: false,
            shown: Cell::new(false),
        });
        id
    }

    /// Replaces a notification's data; its place and timer are untouched.
    pub fn update(&self, id: NotificationId, args: impl Into<T>) {
        // `peek`: the list is unchanged, so nothing drawing it should redraw.
        let Ok(entries) = self.store.entries.try_peek() else {
            return;
        };
        let Some(mut slot) = entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.args)
        else {
            return;
        };
        drop(entries);
        match slot.write().downcast_mut::<T>() {
            Some(slot) => *slot = args.into(),
            None => warn("NotificationHandle::update: that id belongs to another template"),
        }
    }

    /// Closes one, with its exit.
    pub fn hide(&self, id: NotificationId) {
        self.store.hide(id);
    }

    /// Removes every notification, of every template, at once.
    pub fn clear(&self) {
        if !self.store.alive() {
            return;
        }
        let mut entries = self.store.entries;
        entries.write().clear();
        // No notification is left to hand focus on to (todo 440).
        if self.store.focused.peek().is_some() {
            self.store.focus(self.store.return_target());
        }
    }
}

/// Notifications drawn as an [`Alert`], over [`NotificationData`].
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{NotificationData, use_notifications};
/// # fn app() -> Element {
/// let notify = use_notifications();
/// notify.show("Saved.");
/// notify.show(NotificationData {
///     title: Some("Upload failed".into()),
///     color: "error".into(),
///     ..Default::default()
/// });
/// # rsx! {}
/// # }
/// ```
///
/// Needs a [`Notifications`](super::Notifications) host, rendered once.
pub fn use_notifications() -> NotificationHandle<NotificationData> {
    use_notifications_with(default_template)
}

/// Notifications drawn by your own template, over your own `T`.
///
/// A `fn`, not a capturing closure: a notification outlives its caller, so
/// everything the template needs travels in `T`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{NotificationScope, Paper, ProgressBar, Text, use_notifications_with};
/// # fn app() -> Element {
/// # #[derive(Clone, PartialEq)] struct Upload { file: String, percent: f64 }
/// let uploads = use_notifications_with(|s: NotificationScope<Upload>| rsx! {
///     Paper { Text { "{s.args().file}" } ProgressBar { value: s.args().percent } }
/// });
/// let id = uploads.show(Upload { file: "archive.zip".into(), percent: 0.0 });
/// uploads.update(id, Upload { file: "archive.zip".into(), percent: 40.0 });
/// # rsx! {}
/// # }
/// ```
///
/// The template runs in the notification's own scope, so it may call hooks.
pub fn use_notifications_with<T: 'static>(
    template: fn(NotificationScope<T>) -> Element,
) -> NotificationHandle<T> {
    NotificationHandle {
        store: use_notification_store(),
        template,
    }
}

fn default_template(s: NotificationScope<NotificationData>) -> Element {
    use_hook(|| {
        if !s.closable && s.stays {
            warn(
                "Notifications: `closable: false` with `AutoClose::Never` leaves the user no way \
                 to close it; only `hide` does.",
            );
        }
    });
    // All but the message text, so a text-only update redraws just `NotificationMessage`.
    let chrome = use_memo(move || {
        let data = s.args();
        let has_message = !data.message.is_empty();
        (data.title, data.color, data.variant, data.icon, has_message)
    });
    let (title, color, variant, icon, has_message) = chrome();
    // An empty text node would still get a message slot and `aria-describedby`.
    let message = if has_message {
        rsx! { NotificationMessage { args: s.args } }
    } else {
        VNode::empty()
    };

    rsx! {
        Alert {
            // Not `alert`: a live region inside another is announced twice or not at all.
            role: "group",
            title,
            color,
            variant,
            icon,
            onclose: s.closable().then(|| EventHandler::new(move |()| s.close())),
            sx: &DEFAULT_TEMPLATE_SX,
            children: message,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct MessageProps {
    args: Signal<std::boxed::Box<dyn Any>>,
}

/// The default template's message text, the one scope an update of it redraws.
fn NotificationMessage(props: MessageProps) -> Element {
    let args = props.args.read();
    let message = args
        .downcast_ref::<NotificationData>()
        .map(|data| data.message.as_str())
        .unwrap_or_default();
    rsx! { "{message}" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LiberoProvider, components::feedback::Notifications, theme::AutoClose, utils::take_warnings,
    };

    thread_local! {
        static OPTIONS: Cell<NotificationOptions> = Cell::new(NotificationOptions::default());
    }

    fn warns(closable: bool, auto_close: AutoClose) -> bool {
        OPTIONS.set(NotificationOptions {
            closable,
            auto_close: Some(auto_close),
            ..Default::default()
        });
        take_warnings();
        let mut dom = VirtualDom::new(|| {
            let notify = use_notifications();
            use_hook(|| notify.show_with("Saved.", OPTIONS.get()));
            rsx! { LiberoProvider { Notifications {} } }
        });
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        take_warnings()
            .iter()
            .any(|warning| warning.starts_with("Notifications: `closable: false`"))
    }

    #[test]
    fn a_notification_nobody_can_close_warns() {
        assert!(warns(false, AutoClose::Never));
        assert!(!warns(true, AutoClose::Never));
        assert!(!warns(false, AutoClose::After(4000)));
    }
}
