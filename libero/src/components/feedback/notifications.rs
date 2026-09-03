use std::{
    any::Any,
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        feedback::Alert,
        layout::{Box, Float, use_box},
    },
    hooks::{use_portal, use_theme},
    platform::{TimerSubscription, timer},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        AutoClose, NOTIFICATION_GAP, NOTIFICATION_IN, NOTIFICATION_OFFSET, NOTIFICATION_OUT,
        NOTIFICATION_TRANSITION, NOTIFICATION_WIDTH, Placement, Size, SizeCss,
        Z_INDEX_NOTIFICATION,
    },
    utils::warn,
};

const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// One stack, anchored to a viewport corner or edge. It lets the pointer
/// through: only the notifications in it take clicks.
static STACK_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width(format!(
            "min({}, calc(100vw - 2 * {}))",
            NOTIFICATION_WIDTH.value(),
            NOTIFICATION_OFFSET.value()
        ))
        .pointer_events("none")
});

/// One live region. Both are always rendered, even empty, and the space
/// between the two comes from here rather than from a `gap` on the stack, so
/// an empty list takes none.
static LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(NOTIFICATION_GAP.value())
        .margin("0")
        .padding("0")
        .list_style("none")
        .selector(
            "&:not(:empty) + &:not(:empty)",
            sx().margin_top(NOTIFICATION_GAP.value()),
        )
});

static ITEM_SX: StaticSx = StaticSx::new(|| {
    let animation =
        |name: &str, easing: &str| format!("{name} {} {easing}", NOTIFICATION_TRANSITION.value());

    sx().pointer_events("auto")
        .animation(animation(NOTIFICATION_IN, "ease-out"))
        .media(REDUCED_MOTION, sx().animation("none"))
        // The end state is declared, not only animated to, so it holds for the
        // few ms between the animation's end and the unmount - and is where a
        // reduced-motion reader lands at once.
        .when(
            "leaving",
            sx().opacity("0")
                .visibility("hidden")
                .pointer_events("none")
                .animation(animation(NOTIFICATION_OUT, "ease-in"))
                .media(REDUCED_MOTION, sx().animation("none")),
        )
});

/// The default template's surface floats over the page, so it takes back the
/// shadow `Alert` drops for sitting in the flow.
static DEFAULT_TEMPLATE_SX: StaticSx =
    StaticSx::new(|| sx().box_shadow(SizeCss::SHADOW.value(Size::Md)));

/// Names one notification, for [`NotificationHandle::update`] and
/// [`NotificationHandle::hide`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

/// Which of the two live regions a notification is announced from.
///
/// Not derived from a colour: a custom `T` has no colour the library can read,
/// and an error that is not urgent should not interrupt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotificationLive {
    /// Announced when the reader is idle. Right for nearly everything.
    #[default]
    Polite,
    /// Interrupts the reader. For what cannot wait.
    Assertive,
}

/// Per notification. Every field defaults to the host's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationOptions {
    /// The stack it joins. `None` is the `Notifications` host's `position`.
    pub position: Option<Placement>,
    /// `None` is the host's `auto_close`.
    pub auto_close: Option<AutoClose>,
    /// Whether the template offers a close control. The default template
    /// reads it; a custom one reads it from [`NotificationScope::closable`].
    pub closable: bool,
    pub live: NotificationLive,
}

impl Default for NotificationOptions {
    fn default() -> Self {
        Self {
            position: None,
            auto_close: None,
            closable: true,
            live: NotificationLive::default(),
        }
    }
}

/// What the default template shows: an [`Alert`].
#[derive(Clone, PartialEq, Default)]
pub struct NotificationData {
    pub title: Option<String>,
    /// Empty renders no message slot, so a title-only notification has no
    /// `aria-describedby` pointing at nothing.
    pub message: String,
    /// `Alert`'s `color`; unset is `theme.alert.color`.
    pub color: Input<ThemeAwareValue>,
    /// A glyph only. It is rendered by the host long after the scope that
    /// built it may be gone, so it must not carry event handlers.
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

type Draw = Rc<dyn Fn(NotificationId) -> Element>;

/// A notification with its type erased, so the store, the timers and the host
/// compile once whatever `T`s an app uses.
struct Entry {
    id: NotificationId,
    /// The `T`, so [`NotificationHandle::update`] can replace it and
    /// [`NotificationScope::args`] can read it. Only ever downcast to the `T`
    /// of the handle that stored it.
    args: std::boxed::Box<dyn Any>,
    /// The template with `T` still known, called by the host in its own scope.
    draw: Draw,
    position: Option<Placement>,
    auto_close: Option<AutoClose>,
    closable: bool,
    live: NotificationLive,
    /// Closing: the exit is running, and it is removed when that ends.
    leaving: bool,
    /// Has been on screen. One that never was - queued past the limit - has
    /// no exit to run and is removed at once.
    shown: Cell<bool>,
}

/// The queue. Lives in the root scope, so it outlives every component that
/// raised a notification: a notification survives its caller navigating away.
#[derive(Clone, Copy)]
struct NotificationStore {
    entries: Signal<Vec<Entry>>,
    /// Which notification the pointer is on, and which one holds focus. Either
    /// pauses every timer - hovering to read one must not let the rest vanish,
    /// and a close button someone tabbed to must not disappear under them.
    hovered: Signal<Option<NotificationId>>,
    focused: Signal<Option<NotificationId>>,
}

impl NotificationStore {
    fn paused(&self) -> bool {
        self.hovered.read().is_some() || self.focused.read().is_some()
    }

    /// Starts the exit of one that is showing, and drops one that is queued.
    fn hide(&self, id: NotificationId) {
        let mut entries = self.entries;
        let mut entries = entries.write();
        let Some(index) = entries.iter().position(|entry| entry.id == id) else {
            return;
        };
        if entries[index].shown.get() {
            entries[index].leaving = true;
        } else {
            entries.remove(index);
        }
    }

    fn remove(&self, id: NotificationId) {
        let mut entries = self.entries;
        entries.write().retain(|entry| entry.id != id);
    }
}

/// The one store, created in the root scope by whichever caller asks first.
///
/// Root, not the caller's scope: the store and every signal in it must outlive
/// the component that first asked, and timer callbacks write to it from
/// outside every scope.
fn use_notification_store() -> NotificationStore {
    use_hook(|| {
        try_consume_context::<NotificationStore>().unwrap_or_else(|| {
            dioxus::core::provide_root_context(NotificationStore {
                entries: Signal::new_in_scope(Vec::new(), ScopeId::ROOT),
                hovered: Signal::new_in_scope(None, ScopeId::ROOT),
                focused: Signal::new_in_scope(None, ScopeId::ROOT),
            })
        })
    })
}

/// A template's view of the notification it draws. `Copy`, so several
/// handlers in one template can each hold it.
pub struct NotificationScope<T: 'static> {
    id: NotificationId,
    store: NotificationStore,
    args: PhantomData<fn() -> T>,
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
        self.store
            .entries
            .read()
            .iter()
            .find(|entry| entry.id == self.id)
            .is_some_and(|entry| entry.closable)
    }
}

impl<T: Clone + 'static> NotificationScope<T> {
    /// The data it was shown with, or last updated to.
    pub fn args(&self) -> T {
        self.store
            .entries
            .read()
            .iter()
            .find(|entry| entry.id == self.id)
            .and_then(|entry| entry.args.downcast_ref::<T>())
            .cloned()
            .expect("NotificationScope used after its notification was removed")
    }
}

/// Shows, updates and hides notifications drawn by one template. `Copy`, so
/// it can be handed to any handler, task or child.
///
/// **`show` queues; it does not promise the notification is visible.** Each
/// stack shows at most the host's `limit`, and the rest wait their turn in
/// order - their timers start only once they are on screen.
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
        let template = self.template;
        let draw: Draw = Rc::new(move |id| {
            template(NotificationScope {
                id,
                store,
                args: PhantomData,
            })
        });

        let mut entries = self.store.entries;
        entries.write().push(Entry {
            id,
            args: std::boxed::Box::new(args.into()),
            draw,
            position: options.position,
            auto_close: options.auto_close,
            closable: options.closable,
            live: options.live,
            leaving: false,
            shown: Cell::new(false),
        });
        id
    }

    /// Replaces the data a notification shows - "Uploading..." becoming
    /// "Uploaded". Its place in the stack and its timer are untouched. Does
    /// nothing once it is gone.
    pub fn update(&self, id: NotificationId, args: impl Into<T>) {
        let mut entries = self.store.entries;
        let mut entries = entries.write();
        let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        match entry.args.downcast_mut::<T>() {
            Some(slot) => *slot = args.into(),
            None => warn("NotificationHandle::update: that id belongs to another template"),
        }
    }

    /// Closes one, with its exit. Does nothing once it is gone.
    pub fn hide(&self, id: NotificationId) {
        self.store.hide(id);
    }

    /// Removes every notification, of every template, at once.
    pub fn clear(&self) {
        let mut entries = self.store.entries;
        entries.write().clear();
    }
}

/// Notifications drawn as an [`Alert`], over [`NotificationData`].
///
/// ```ignore
/// let notify = use_notifications();
/// notify.show("Saved.");
/// notify.show(NotificationData {
///     title: Some("Upload failed".into()),
///     color: "error".into(),
///     ..Default::default()
/// });
/// ```
///
/// Nothing appears unless the app renders a [`Notifications`] host, once.
pub fn use_notifications() -> NotificationHandle<NotificationData> {
    use_notifications_with(default_template)
}

/// Notifications drawn by your own template, over your own `T`.
///
/// The template is a `fn`, not a closure that captures: a notification
/// outlives the component that raised it, so anything captured from that
/// component could be dropped while the notification still draws with it.
/// Everything a template needs travels in `T`, and a non-capturing closure
/// coerces to the `fn`.
///
/// ```ignore
/// let uploads = use_notifications_with(|s: NotificationScope<Upload>| rsx! {
///     Paper { Text { "{s.args().file}" } ProgressBar { value: s.args().percent } }
/// });
/// let id = uploads.show(Upload { file: "archive.zip".into(), percent: 0.0 });
/// uploads.update(id, Upload { file: "archive.zip".into(), percent: 40.0 });
/// ```
///
/// The template is called in the host's scope on every render, so it may call
/// hooks, the same ones every time.
pub fn use_notifications_with<T: 'static>(
    template: fn(NotificationScope<T>) -> Element,
) -> NotificationHandle<T> {
    NotificationHandle {
        store: use_notification_store(),
        template,
    }
}

fn default_template(s: NotificationScope<NotificationData>) -> Element {
    let theme = use_theme();
    let data = s.args();
    // `Alert` gives a message slot and `aria-describedby` to any children at
    // all, so an empty message passes none rather than an empty text node.
    let message = if data.message.is_empty() {
        VNode::empty()
    } else {
        rsx! { "{data.message}" }
    };

    rsx! {
        Alert {
            // Not `alert`: the list around it is already the live region, and
            // a live region inside another is announced twice or not at all.
            role: "group",
            title: data.title,
            color: data.color,
            icon: data.icon,
            onclose: s.closable().then(|| EventHandler::new(move |()| s.close())),
            close_label: theme.notification.close_label,
            sx: &DEFAULT_TEMPLATE_SX,
            children: message,
        }
    }
}

/// Where the stacks and their notifications render. **Render it once**, near
/// the root: it is the one outlet for every [`use_notifications`] handle, and
/// a second one would draw every notification twice.
///
/// It is portaled, so where it sits in the tree does not matter, and each
/// stack is a `Float { fixed: true }`, so it stays in its corner while the
/// page scrolls.
///
/// No keyboard behaviour of its own and no `Escape`: nothing here ever takes
/// focus. A close button is reached by `Tab` in document order.
#[component]
pub fn Notifications(
    /// The stack a notification joins unless it names its own. Defaults to
    /// `theme.notification.position`.
    #[props(default, into)]
    position: Input<Placement>,
    /// Shown at once per stack; the rest wait. Defaults to
    /// `theme.notification.limit`.
    #[props(default)]
    limit: Option<usize>,
    /// Unless a notification says otherwise. Defaults to
    /// `theme.notification.auto_close`.
    #[props(default)]
    auto_close: Option<AutoClose>,
) -> Element {
    let theme = use_theme();
    let store = use_notification_store();

    let position = position.copied_or(theme.notification.position);
    let limit = limit.unwrap_or(theme.notification.limit);
    let auto_close = auto_close.unwrap_or(theme.notification.auto_close);
    let exit_ms = theme.notification.transition_duration;

    let entries = store.entries.read();
    // Every position, always: both live regions of a stack have to be in the
    // document before anything is added to them, or nothing is announced. So
    // a stack cannot appear with its first notification.
    let stacks = Placement::ALL.iter().map(|&placement| {
        let items = entries
            .iter()
            .filter(|entry| entry.position.unwrap_or(position) == placement)
            .take(limit)
            .map(|entry| ItemProps {
                id: entry.id,
                draw: DrawRef(entry.draw.clone()),
                auto_close: match entry.auto_close.unwrap_or(auto_close) {
                    AutoClose::Never => None,
                    AutoClose::After(ms) => Some(ms),
                },
                leaving: entry.leaving,
                exit_ms,
                live: entry.live,
            })
            .collect::<Vec<_>>();
        let assertive = items
            .iter()
            .filter(|item| item.live == NotificationLive::Assertive)
            .cloned();
        let polite = items
            .iter()
            .filter(|item| item.live == NotificationLive::Polite)
            .cloned();

        rsx! {
            Float {
                key: "{placement.as_str()}",
                fixed: true,
                placement: Input::Value(placement),
                offset_x: edge_offset(placement, Axis::Horizontal),
                offset_y: edge_offset(placement, Axis::Vertical),
                z_index: Z_INDEX_NOTIFICATION.value(),
                sx: &STACK_SX,
                Box {
                    component: HtmlTag::Ol,
                    framework_sx: &LIST_SX,
                    "aria-live": "assertive",
                    for item in assertive {
                        NotificationItem {
                            key: "{item.id.0}",
                            id: item.id,
                            draw: item.draw,
                            auto_close: item.auto_close,
                            leaving: item.leaving,
                            exit_ms: item.exit_ms,
                            live: item.live,
                        }
                    }
                }
                Box {
                    component: HtmlTag::Ol,
                    framework_sx: &LIST_SX,
                    "aria-live": "polite",
                    for item in polite {
                        NotificationItem {
                            key: "{item.id.0}",
                            id: item.id,
                            draw: item.draw,
                            auto_close: item.auto_close,
                            leaving: item.leaving,
                            exit_ms: item.exit_ms,
                            live: item.live,
                        }
                    }
                }
            }
        }
    });
    let content = rsx! {
        {stacks}
    };
    drop(entries);

    use_portal(Some(content));
    rsx! {}
}

enum Axis {
    Horizontal,
    Vertical,
}

/// `Float`'s offsets are a translate, so pushing a stack in from an end or
/// bottom edge is a negative one. A centred axis takes none.
fn edge_offset(placement: Placement, axis: Axis) -> String {
    use Placement::*;
    // `Some(true)` for the start or top edge, `Some(false)` for the end or
    // bottom one.
    let from_start = match axis {
        Axis::Horizontal => match placement {
            TopStart | CenterStart | BottomStart => Some(true),
            TopEnd | CenterEnd | BottomEnd => Some(false),
            TopCenter | CenterCenter | BottomCenter => None,
        },
        Axis::Vertical => match placement {
            TopStart | TopCenter | TopEnd => Some(true),
            BottomStart | BottomCenter | BottomEnd => Some(false),
            CenterStart | CenterCenter | CenterEnd => None,
        },
    };
    match from_start {
        Some(true) => NOTIFICATION_OFFSET.value(),
        Some(false) => format!("calc(-1 * {})", NOTIFICATION_OFFSET.value()),
        None => "0px".to_string(),
    }
}

/// Compared by pointer: the same `Rc` is the same template over the same
/// entry, and a closure has no other equality.
#[derive(Clone)]
struct DrawRef(Draw);

impl PartialEq for DrawRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Props, Clone, PartialEq)]
struct ItemProps {
    id: NotificationId,
    draw: DrawRef,
    /// Resolved against the host: `None` stays until closed.
    auto_close: Option<u32>,
    leaving: bool,
    exit_ms: u32,
    live: NotificationLive,
}

#[allow(non_snake_case)]
fn NotificationItem(props: ItemProps) -> Element {
    let store = use_notification_store();
    let id = props.id;

    use_hook(|| {
        if let Some(entry) = store.entries.peek().iter().find(|entry| entry.id == id) {
            entry.shown.set(true);
        }
    });

    // Dropping a subscription cancels it, so replacing this is the whole of
    // "stop the old timer". Held here, not in the store: an item that
    // unmounts takes its timer with it.
    let subscription =
        use_hook(|| Rc::new(RefCell::new(None::<std::boxed::Box<dyn TimerSubscription>>)));

    let paused = store.paused();
    let leaving = props.leaving;
    let auto_close = props.auto_close;
    let exit_ms = props.exit_ms;
    let armed = subscription.clone();
    use_effect(use_reactive!(|(paused, leaving, auto_close, exit_ms)| {
        let mut armed = armed.borrow_mut();
        *armed = None;

        // The callbacks write the store's signals and nothing else: on the
        // web they run with no dioxus runtime at all.
        let (delay, then): (u32, fn(NotificationStore, NotificationId)) = match auto_close {
            _ if leaving => (exit_ms, |store, id| store.remove(id)),
            // Resumed with the full time, not the remainder: someone who
            // stopped to read it gets the whole of it again.
            Some(_) if paused => return,
            Some(ms) => (ms, |store, id| store.hide(id)),
            None => return,
        };
        if let Some(api) = timer() {
            *armed = Some(api.after(
                Duration::from_millis(delay.into()),
                std::boxed::Box::new(move || then(store, id)),
            ));
        }
    }));

    use_drop(move || {
        subscription.borrow_mut().take();
        // Removed from under the pointer or with focus inside, it never sees
        // its `mouseleave`/`focusout`, and every timer would stay paused.
        let (mut hovered, mut focused) = (store.hovered, store.focused);
        if *hovered.peek() == Some(id) {
            hovered.set(None);
        }
        if *focused.peek() == Some(id) {
            focused.set(None);
        }
    });

    let states: Input<States> = States::default().with("leaving", leaving).into();
    let (mut hovered, mut focused) = (store.hovered, store.focused);

    use_box()
        .framework_sx(&ITEM_SX)
        .states(&states)
        .prepare()
        .event("onmouseenter", move |_: Event<MouseData>| {
            hovered.set(Some(id))
        })
        .event("onmouseleave", move |_: Event<MouseData>| {
            if *hovered.peek() == Some(id) {
                hovered.set(None);
            }
        })
        .event("onfocusin", move |_: Event<FocusData>| {
            focused.set(Some(id))
        })
        .event("onfocusout", move |_: Event<FocusData>| {
            if *focused.peek() == Some(id) {
                focused.set(None);
            }
        })
        .render(HtmlTag::Li, Vec::new(), (props.draw.0)(id))
}
