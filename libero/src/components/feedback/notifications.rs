use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use dioxus::{
    core::{Runtime, current_scope_id},
    prelude::*,
};

use crate::{
    components::{
        common::{FOCUSABLE_SELECTOR, HtmlTag, Input, States, Variant, focus_ring_sx},
        feedback::Alert,
        layout::{Box, Float, use_box},
    },
    hooks::{use_focus_within, use_localization, use_portal_slot, use_theme},
    localization::fill,
    platform::{self, ElementApi, KeyChord, KeySubscription, TimerSubscription, keyboard, timer},
    sx::{REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        AutoClose, NOTIFICATION_GAP, NOTIFICATION_IN, NOTIFICATION_OFFSET, NOTIFICATION_OUT,
        NOTIFICATION_TRANSITION, NOTIFICATION_WIDTH, Placement, Size, SizeCss,
        Z_INDEX_NOTIFICATION,
    },
    utils::warn,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// Every mounted host's hotkey, by host, so one press moves focus once:
    /// into the newest notification across all hosts (todo 670).
    static HOTKEYS: RefCell<Vec<(u64, NotificationStore, Key)>> = const { RefCell::new(Vec::new()) };
}

/// One stack. It lets the pointer through: only its notifications take clicks.
static STACK_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .width(format!(
            "min({}, calc(100% - 2 * {}))",
            NOTIFICATION_WIDTH.value(),
            NOTIFICATION_OFFSET.value()
        ))
        .pointer_events("none")
});

/// A contained host's box: the stacks' positioned ancestor.
static CONTAINED_SX: StaticSx = StaticSx::new(|| sx().position("relative"));

/// The landmark around every stack. It covers the host, so a stack anchors to
/// the host even where `position` resolves against the parent (Blitz, todo 682).
static LANDMARK_SX: StaticSx =
    StaticSx::new(|| sx().position("absolute").inset("0").pointer_events("none"));

/// One live region, always rendered. Spaced here, not by a `gap`, so an empty
/// one takes no space.
static REGION_SX: StaticSx = StaticSx::new(|| {
    sx().selector(
        "&:not(:empty) + &:not(:empty)",
        sx().margin_top(NOTIFICATION_GAP.value()),
    )
});

static LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(NOTIFICATION_GAP.value())
        .margin("0")
        .padding("0")
        .list_style("none")
});

static ITEM_SX: StaticSx = StaticSx::new(|| {
    let animation =
        |name: &str, easing: &str| format!("{name} {} {easing}", NOTIFICATION_TRANSITION.value());

    sx().pointer_events("auto")
        .selector("&:focus-visible", focus_ring_sx())
        .animation(animation(NOTIFICATION_IN, "ease-out"))
        .media(REDUCED_MOTION, sx().animation("none"))
        // Declared, not only animated to: it holds until the unmount, and is
        // where reduced motion lands at once.
        .when(
            "leaving",
            sx().opacity("0")
                .visibility("hidden")
                .pointer_events("none")
                .animation(animation(NOTIFICATION_OUT, "ease-in"))
                .media(REDUCED_MOTION, sx().animation("none")),
        )
});

/// Floats over the page, so it takes back the shadow `Alert` drops.
static DEFAULT_TEMPLATE_SX: StaticSx =
    StaticSx::new(|| sx().box_shadow(SizeCss::SHADOW.value(Size::Md)));

/// Names one notification, for [`NotificationHandle::update`] and
/// [`NotificationHandle::hide`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

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
    /// Offer a close control; see [`NotificationScope::closable`].
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

/// What the default template shows: an [`Alert`].
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

type Draw = Rc<dyn Fn() -> Element>;

/// A notification with its type erased, so the store, the timers and the host
/// compile once whatever `T`s an app uses.
struct Entry {
    id: NotificationId,
    /// The `T`. Its own signal, so an `update` redraws only this template.
    args: Signal<std::boxed::Box<dyn Any>>,
    /// The template with `T` still known.
    draw: Draw,
    placement: Option<Placement>,
    /// Stays in its first stack when the host's `placement` changes: a move
    /// would remount, re-announce and restart the timer (todo 577).
    drawn_in: Cell<Option<Placement>>,
    auto_close: Option<AutoClose>,
    live: NotificationLive,
    /// The exit is running; removed when it ends.
    leaving: bool,
    /// Has been on screen. A queued one has no exit to run.
    shown: Cell<bool>,
}

impl Drop for Entry {
    /// The root owns `args`, so nothing else would ever drop it.
    fn drop(&mut self) {
        self.args.manually_drop();
    }
}

/// The queue. The app's lives in the root scope, so a notification survives its
/// caller navigating away. A contained host owns one of its own.
#[derive(Clone, Copy, PartialEq)]
struct NotificationStore {
    entries: Signal<Vec<Entry>>,
    /// Hover or focus on any notification pauses every timer.
    hovered: Signal<Option<NotificationId>>,
    focused: Signal<Option<NotificationId>>,
    /// Each stack's ids in document order, as the host last drew them.
    drawn: CopyValue<Vec<Vec<NotificationId>>>,
    /// Each drawn notification's element, to hand focus on to.
    elements: CopyValue<HashMap<NotificationId, Rc<MountedData>>>,
    /// What held focus before it entered the notifications (todo 423).
    return_to: CopyValue<Option<Rc<dyn ElementApi>>>,
    /// `return_to` sits inside this contained host, and goes when it does.
    return_in_host: CopyValue<bool>,
    /// Set while the store moves focus itself, so that move is no entry.
    handing_off: CopyValue<bool>,
    /// `show` may run from a timer callback outside every runtime, and creates
    /// a signal. Weak: the runtime owns the store.
    runtime: CopyValue<Weak<Runtime>>,
    /// The scope every signal of the store, and of each entry, belongs to.
    owner: ScopeId,
    /// Names a contained host's box in a selector.
    id: u64,
}

impl NotificationStore {
    fn new(owner: ScopeId) -> Self {
        Self {
            entries: Signal::new_in_scope(Vec::new(), owner),
            hovered: Signal::new_in_scope(None, owner),
            focused: Signal::new_in_scope(None, owner),
            drawn: CopyValue::new_in_scope(Vec::new(), owner),
            elements: CopyValue::new_in_scope(HashMap::new(), owner),
            return_to: CopyValue::new_in_scope(None, owner),
            return_in_host: CopyValue::new_in_scope(false, owner),
            handing_off: CopyValue::new_in_scope(false, owner),
            runtime: CopyValue::new_in_scope(Rc::downgrade(&Runtime::current()), owner),
            owner,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    fn owned_signal<V: 'static>(&self, value: V) -> Signal<V> {
        let runtime = self
            .runtime
            .peek()
            .upgrade()
            .expect("the notification store outlived its runtime");
        runtime.in_scope(self.owner, || Signal::new_in_scope(value, self.owner))
    }

    /// A handle may outlive a contained host's store.
    fn alive(&self) -> bool {
        self.entries.try_peek().is_ok()
    }

    /// A contained host's box, `None` for the app's host.
    fn host_selector(&self) -> Option<String> {
        (self.owner != ScopeId::ROOT).then(|| format!("[data-notifications-host=\"{}\"]", self.id))
    }

    fn paused(&self) -> bool {
        self.hovered.read().is_some() || self.focused.read().is_some()
    }

    /// Starts the exit of one that is showing, and drops one that is queued.
    fn hide(&self, id: NotificationId) {
        if !self.alive() {
            return;
        }
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
        drop(entries);
        if *self.focused.peek() == Some(id) {
            self.hand_focus_on(id);
        }
    }

    /// Focus leaves a closing notification for the next one in its stack, the
    /// previous one if it was the last, else for where it came from.
    fn hand_focus_on(&self, id: NotificationId) {
        let entries = self.entries.peek();
        let open = |other: &&NotificationId| {
            entries
                .iter()
                .any(|entry| entry.id == **other && !entry.leaving)
        };
        let drawn = self.drawn.peek();
        let elements = self.elements.peek();
        let stack = drawn
            .iter()
            .find(|stack| stack.contains(&id))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let at = stack.iter().position(|other| *other == id).unwrap_or(0);
        let next = stack.get(at + 1..).unwrap_or_default().iter();
        let previous = stack[..at].iter().rev();
        let target = next
            .chain(previous)
            .filter(open)
            .filter_map(|other| elements.get(other))
            .find_map(|item| {
                let item = platform::element(item);
                item.query_selector(r#"[data-slot="close"]"#)
                    .or_else(|_| item.query_selector(FOCUSABLE_SELECTOR))
                    .ok()
            })
            .map(Rc::from)
            .or_else(|| self.return_target());
        self.focus(target);
    }

    /// Where focus came from, if that element is still in the document.
    fn return_target(&self) -> Option<Rc<dyn ElementApi>> {
        let return_to = self.return_to.peek().clone();
        return_to.filter(|element| element.is_connected())
    }

    fn focus(&self, target: Option<Rc<dyn ElementApi>>) {
        let Some(target) = target else {
            return;
        };
        let Some(runtime) = self.runtime.peek().upgrade() else {
            return;
        };
        // Spawned: this may run inside the close button's click dispatch.
        let mut handing_off = self.handing_off;
        handing_off.set(true);
        runtime.in_scope(self.owner, || {
            spawn(async move {
                let _ = target.focus();
                handing_off.set(false);
            })
        });
    }

    fn remove(&self, id: NotificationId) {
        if !self.alive() {
            return;
        }
        let mut entries = self.entries;
        entries.write().retain(|entry| entry.id != id);
    }

    /// The newest notification on screen and not closing. Ids only grow.
    fn newest(&self) -> Option<NotificationId> {
        let entries = self.entries.try_peek().ok()?;
        let drawn = self.drawn.peek();
        drawn
            .iter()
            .flatten()
            .filter(|id| {
                entries
                    .iter()
                    .any(|entry| entry.id == **id && !entry.leaving)
            })
            .max_by_key(|id| id.0)
            .copied()
    }

    /// The hotkey's move: into the newest notification's first focusable,
    /// else onto the notification itself.
    fn focus_newest(&self) {
        let Some(id) = self.newest() else {
            return;
        };
        let Some(item) = self.elements.peek().get(&id).cloned() else {
            return;
        };
        let item = platform::element(&item);
        let _ = match item.query_selector(FOCUSABLE_SELECTOR) {
            Ok(inner) => inner.focus(),
            Err(_) => item.focus(),
        };
    }
}

/// The nearest contained host's store, else the app's. The app's lives in the
/// root scope, so it outlives the component that first asked.
fn use_notification_store() -> NotificationStore {
    use_hook(|| {
        try_consume_context::<NotificationStore>().unwrap_or_else(|| {
            dioxus::core::provide_root_context(NotificationStore::new(ScopeId::ROOT))
        })
    })
}

/// A template's view of the notification it draws.
pub struct NotificationScope<T: 'static> {
    id: NotificationId,
    store: NotificationStore,
    args: Signal<std::boxed::Box<dyn Any>>,
    closable: bool,
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
        let draw: Draw = Rc::new(move || {
            template(NotificationScope {
                id,
                store,
                args,
                closable,
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
/// Needs a [`Notifications`] host, rendered once.
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

/// The host where notifications render. Render it once; a `contained` one has
/// its own queue.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Notifications, use_notifications};
/// # fn app() -> Element {
/// # rsx! {
/// Notifications { contained: true,
///     SaveButton {} // its `use_notifications()` shows them in this box
/// }
/// # } }
/// # #[component] fn SaveButton() -> Element { rsx! {} }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/notifications>
#[component]
pub fn Notifications(
    /// The default stack. A change moves only notifications shown after it.
    #[props(default, into)]
    placement: Input<Placement>,
    /// Shown at once per stack; the rest wait.
    #[props(default)]
    limit: Option<usize>,
    /// Unless a notification says otherwise.
    #[props(default)]
    auto_close: Option<AutoClose>,
    /// Draws inside its own box, with a queue of its own. Read once, on mount.
    #[props(default)]
    contained: bool,
    /// Focuses the newest notification from anywhere.
    #[props(default = Key::F8)]
    hotkey: Key,
    /// Rendered inside a contained host, before its stacks.
    #[props(default)]
    children: Option<Element>,
) -> Element {
    let theme = use_theme();
    let localization = use_localization();
    let store = use_hook(|| {
        if contained {
            provide_context(NotificationStore::new(current_scope_id()))
        } else {
            try_consume_context::<NotificationStore>().unwrap_or_else(|| {
                dioxus::core::provide_root_context(NotificationStore::new(ScopeId::ROOT))
            })
        }
    });
    use_hotkey(store, hotkey.clone());
    let key_name = match &hotkey {
        Key::Character(text) => text.to_uppercase(),
        key => key.to_string(),
    };
    let region_label = fill(localization.notifications.region, &[("key", &key_name)]);

    let host_placement = placement.copied_or(theme.notifications.placement);
    let limit = limit.unwrap_or(theme.notifications.limit);
    let auto_close = auto_close.unwrap_or(theme.notifications.auto_close);
    let exit_ms = theme.notifications.transition_duration;

    let entries = store.entries.read();
    // Every placement, always: a live region must exist before content is added,
    // or nothing is announced (todo 447).
    let mut drawn = Vec::new();
    let stacks = Placement::ALL.iter().map(|&placement| {
        let items = entries
            .iter()
            .filter(|entry| {
                let stack = entry.drawn_in.get().or(entry.placement);
                stack.unwrap_or(host_placement) == placement
            })
            .take(limit)
            .inspect(|entry| entry.drawn_in.set(Some(placement)))
            .map(|entry| ItemProps {
                store,
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
        let (assertive, polite): (Vec<_>, Vec<_>) = items
            .into_iter()
            .partition(|item| item.live == NotificationLive::Assertive);
        drawn.push(
            assertive
                .iter()
                .chain(&polite)
                .map(|item| item.id)
                .collect(),
        );

        rsx! {
            NotificationStack {
                key: "{placement.as_str()}",
                placement,
                fixed: !contained,
                assertive,
                polite,
            }
        }
    });
    let stacks = stacks.collect::<Vec<_>>();
    let idle = drawn.iter().all(Vec::is_empty);
    let mut stored = store.drawn;
    stored.set(drawn);
    // One landmark for every stack: nine would crowd the landmark list.
    let content = rsx! {
        Box {
            framework_sx: &LANDMARK_SX,
            role: "region",
            "aria-label": region_label,
            {stacks.into_iter()}
        }
    };
    drop(entries);

    let slot = use_portal_slot();
    if contained {
        slot.show(None);
        return rsx! {
            Box { framework_sx: &CONTAINED_SX,
                "data-notifications-host": store.id.to_string(),
                {children}
                {content}
            }
        };
    }
    // The empty regions stay mounted; the outlet need not follow a scroll for them.
    slot.show_idle(content, idle);
    rsx! {}
}

/// `hotkey` focuses the newest notification (todo 575). A letter is not taken
/// from a text field; any other key is heard from anywhere.
fn use_hotkey(store: NotificationStore, hotkey: Key) {
    // The key callback runs outside every scope on the web; an effect moves focus.
    let tick = use_signal(|| 0u64);
    let slot: Rc<RefCell<Option<std::boxed::Box<dyn KeySubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    let host = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_drop({
        let slot = slot.clone();
        move || {
            slot.borrow_mut().take();
            HOTKEYS.with_borrow_mut(|hosts| hosts.retain(|(other, ..)| *other != host));
        }
    });

    let listening = slot.clone();
    use_effect(use_reactive!(|hotkey| {
        listening.borrow_mut().take();
        HOTKEYS.with_borrow_mut(|hosts| {
            hosts.retain(|(other, ..)| *other != host);
            hosts.push((host, store, hotkey.clone()));
        });
        let Some(api) = keyboard() else {
            return;
        };
        let typed = matches!(hotkey, Key::Character(_));
        let key = hotkey.clone();
        let callback = std::boxed::Box::new(move |chord: KeyChord| {
            let modifiers = chord.modifiers;
            // Nothing on screen, or another host holds a newer one: not ours.
            if !is_hotkey(&chord.key, &key)
                || modifiers.ctrl()
                || modifiers.alt()
                || modifiers.meta()
                || hotkey_owner(&chord.key) != Some(host)
            {
                return false;
            }
            if !chord.repeat {
                let mut tick = tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            true
        });
        *listening.borrow_mut() = Some(match typed {
            true => api.on_key(callback),
            false => api.on_key_unfiltered(callback),
        });
    }));

    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let pressed = tick();
        if pressed == *seen.peek() {
            return;
        }
        seen.set(pressed);
        store.focus_newest();
    });
}

fn is_hotkey(pressed: &Key, key: &Key) -> bool {
    match (pressed, key) {
        (Key::Character(pressed), Key::Character(key)) => pressed.eq_ignore_ascii_case(key),
        (pressed, key) => pressed == key,
    }
}

/// The host that answers `pressed`: the one drawing the newest notification
/// among those listening for it. The first registered wins a shared store.
fn hotkey_owner(pressed: &Key) -> Option<u64> {
    let hosts = HOTKEYS.with_borrow(|hosts| hosts.clone());
    hosts
        .into_iter()
        .filter(|(_, store, key)| is_hotkey(pressed, key) && store.alive())
        .filter_map(|(host, store, _)| Some((store.newest()?, host)))
        .fold(
            None,
            |owner: Option<(NotificationId, u64)>, (id, host)| match owner {
                Some((newest, _)) if newest.0 >= id.0 => owner,
                _ => Some((id, host)),
            },
        )
        .map(|(_, host)| host)
}

#[derive(Props, Clone, PartialEq)]
struct StackProps {
    placement: Placement,
    fixed: bool,
    assertive: Vec<ItemProps>,
    polite: Vec<ItemProps>,
}

/// Its own scope: a list write redraws only the stack it changed.
fn NotificationStack(props: StackProps) -> Element {
    let placement = props.placement;
    let regions = [("assertive", props.assertive), ("polite", props.polite)];

    rsx! {
        Float {
            fixed: props.fixed,
            placement: Input::Value(placement),
            offset_x: edge_offset(placement, Axis::Horizontal),
            offset_y: edge_offset(placement, Axis::Vertical),
            z_index: Z_INDEX_NOTIFICATION.value(),
            sx: &STACK_SX,
            for (live, items) in regions {
                Box {
                    key: "{live}",
                    framework_sx: &REGION_SX,
                    "aria-live": live,
                    if !items.is_empty() {
                        Box {
                            component: HtmlTag::Ol,
                            framework_sx: &LIST_SX,
                            // Both: Safari with VoiceOver drops list
                            // semantics from a `list-style: none` list.
                            role: "list",
                            for item in items {
                                NotificationItem {
                                    key: "{item.id.0}",
                                    store: item.store,
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
            }
        }
    }
}

enum Axis {
    Horizontal,
    Vertical,
}

/// `Float`'s offsets are a translate: negative from an end or bottom edge.
fn edge_offset(placement: Placement, axis: Axis) -> String {
    use Placement::*;
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

/// Compared by pointer: a closure has no other equality.
#[derive(Clone)]
struct DrawRef(Draw);

impl PartialEq for DrawRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Props, Clone, PartialEq)]
struct ItemProps {
    /// Passed, not looked up: a lookup depends on where the item renders.
    store: NotificationStore,
    id: NotificationId,
    draw: DrawRef,
    /// Resolved against the host: `None` stays until closed.
    auto_close: Option<u32>,
    leaving: bool,
    exit_ms: u32,
    live: NotificationLive,
}

/// The last focusable before the contained host `host` selects, `item` being
/// inside it.
fn focusable_before(
    item: &Rc<MountedData>,
    host: &str,
) -> Result<Option<std::boxed::Box<dyn ElementApi>>, crate::platform::PlatformError> {
    platform::element(item).previous_focusable(&format!("{FOCUSABLE_SELECTOR}:not({host} *)"))
}

fn NotificationItem(props: ItemProps) -> Element {
    let store = props.store;
    let id = props.id;

    use_hook(|| {
        if let Some(entry) = store.entries.peek().iter().find(|entry| entry.id == id) {
            entry.shown.set(true);
        }
    });

    // Dropping a subscription cancels it. Held here, so an unmount takes its timer.
    let subscription =
        use_hook(|| Rc::new(RefCell::new(None::<std::boxed::Box<dyn TimerSubscription>>)));

    let leaving = props.leaving;
    let auto_close = props.auto_close;
    let exit_ms = props.exit_ms;
    let armed = subscription.clone();
    use_effect(use_reactive!(|(leaving, auto_close, exit_ms)| {
        // Read in the effect, not the render: a pause re-arms timers without a redraw.
        let paused = store.paused();
        let mut armed = armed.borrow_mut();
        *armed = None;

        // The callbacks only write signals: on the web they run with no runtime.
        let (delay, then): (u32, fn(NotificationStore, NotificationId)) = match auto_close {
            _ if leaving => (exit_ms, |store, id| store.remove(id)),
            // Resumes with the full time, not the remainder.
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

    // Its own element, held here: the store's copy goes with a contained host.
    let own = use_hook(|| Rc::new(RefCell::new(None::<Rc<MountedData>>)));
    let dropped = own.clone();
    // What precedes a contained host, found as focus enters. Blitz holds its
    // document through the unmount, so the drop cannot ask then.
    let before_host = use_hook(|| Rc::new(RefCell::new(None::<std::boxed::Box<dyn ElementApi>>)));
    let remembered = before_host.clone();
    use_drop(move || {
        subscription.borrow_mut().take();
        // Removed under the pointer or focus, it never sees `mouseleave`/`focusout`.
        let (mut hovered, mut focused) = (store.hovered, store.focused);
        if *hovered.peek() == Some(id) {
            hovered.set(None);
        }
        if *focused.peek() == Some(id) {
            focused.set(None);
            // Now, not spawned: a contained host going too takes the scope.
            let return_to = store.return_to.try_peek().ok().and_then(|to| to.clone());
            let return_to = return_to.filter(|to| to.is_connected());
            // An entry that outlives its item: the host itself is going.
            let host_going = store.host_selector().filter(|_| {
                store
                    .entries
                    .try_peek()
                    .map_or(true, |entries| entries.iter().any(|entry| entry.id == id))
            });
            let in_host = store
                .return_in_host
                .try_peek()
                .map_or(true, |in_host| *in_host);
            let before = host_going
                .filter(|_| in_host || return_to.is_none())
                .and_then(|host| {
                    let own = dropped.borrow().clone()?;
                    match focusable_before(&own, &host) {
                        Ok(before) => before,
                        Err(_) => remembered.borrow_mut().take(),
                    }
                });
            // Nothing before the host, or no way to ask: where focus came from.
            match (before, return_to) {
                (Some(before), _) => {
                    let _ = before.focus();
                }
                (None, Some(target)) => {
                    let _ = target.focus();
                }
                (None, None) => {}
            }
        }
        let mut elements = store.elements;
        elements.write().remove(&id);
    });
    let item = move || store.elements.peek().get(&id).cloned();
    let focus = use_focus_within(
        move || vec![item()],
        move |change| {
            let mut focused = store.focused;
            if !change.within {
                if *focused.peek() == Some(id) {
                    focused.set(None);
                }
                return;
            }
            if let (Some(host), Some(mounted)) = (store.host_selector(), item()) {
                *before_host.borrow_mut() = focusable_before(&mounted, &host).ok().flatten();
            }
            if !*store.handing_off.peek()
                && let Some(from) = change.entered_from("[data-notification]")
            {
                // The host as the boundary answers `None` for a `from` inside it.
                let in_host = store
                    .host_selector()
                    .is_some_and(|host| from.is_some() && change.entered_from(&host).is_none());
                let (mut return_in_host, mut return_to) = (store.return_in_host, store.return_to);
                return_in_host.set(in_host);
                return_to.set(from.map(Rc::from));
            }
            focused.set(Some(id))
        },
    );

    let states: Input<States> = States::default().with("leaving", leaving).into();
    let mut hovered = store.hovered;
    let mut elements = store.elements;

    use_box()
        .framework_sx(&ITEM_SX)
        .states(&states)
        .prepare()
        .attr("data-notification", true)
        // The hotkey's target when nothing inside takes focus.
        .attr("tabindex", "-1")
        .event("onmounted", move |event: Event<MountedData>| {
            own.replace(Some(event.data()));
            elements.write().insert(id, event.data());
        })
        .event("onmouseenter", move |_: Event<MouseData>| {
            hovered.set(Some(id))
        })
        .event("onmouseleave", move |_: Event<MouseData>| {
            if *hovered.peek() == Some(id) {
                hovered.set(None);
            }
        })
        .event("onfocusin", focus.focusin(0))
        .event("onfocusout", focus.focusout(0))
        .render(HtmlTag::Li, Vec::new(), (props.draw.0)())
}
