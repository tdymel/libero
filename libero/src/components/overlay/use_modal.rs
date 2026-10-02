use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use dioxus::prelude::*;

use crate::{
    components::overlay::Modal,
    context::{ModalContext, ModalHost},
    hooks::{FocusReturn, use_focus_return, use_portal_slot},
};

pub(crate) fn use_modal_z_index() -> i32 {
    let host = use_context::<ModalHost>();
    let id = use_hook(|| host.open());
    use_drop(move || host.close(id));
    host.z_index(id)
}

/// Closes the enclosing modal, for content that cannot capture its [`ModalScope`].
pub fn use_modal_close() -> Callback<()> {
    let modal = use_context::<ModalContext>();
    use_callback(move |()| modal.close())
}

/// One opening's state without `S`, so [`Opening`] need not carry it.
struct Resolution<R: 'static> {
    /// Bumped per open. Anything holding an older one is stale and inert.
    generation: u64,
    outcome: Option<Option<R>>,
    handlers: Vec<Box<dyn FnMut(Option<R>)>>,
    wakers: Vec<Waker>,
    /// Where focus came from, to hand it back on close.
    focus_return: FocusReturn,
}

impl<R: 'static> Resolution<R> {
    fn new(focus_return: FocusReturn) -> Self {
        Self {
            generation: 0,
            outcome: None,
            handlers: Vec::new(),
            wakers: Vec::new(),
            focus_return,
        }
    }
}

/// What settling one opening hands back: who to tell, and where focus returns.
struct Settled<R: 'static> {
    handlers: Vec<Box<dyn FnMut(Option<R>)>>,
    wakers: Vec<Waker>,
    focus_return: FocusReturn,
}

impl<R: Clone + 'static> Settled<R> {
    /// Records `value` for `generation` if still live; `None` when stale or settled.
    fn take(
        mut resolution: Signal<Resolution<R>>,
        generation: u64,
        value: &Option<R>,
    ) -> Option<Self> {
        let mut resolution = resolution.try_write().ok()?;
        if resolution.generation != generation || resolution.outcome.is_some() {
            return None;
        }
        resolution.outcome = Some(value.clone());
        Some(Self {
            handlers: std::mem::take(&mut resolution.handlers),
            wakers: std::mem::take(&mut resolution.wakers),
            focus_return: resolution.focus_return,
        })
    }

    fn notify(self, value: Option<R>) {
        for mut handler in self.handlers {
            handler(value.clone());
        }
        for waker in self.wakers {
            waker.wake();
        }
    }
}

/// Settles `generation` if still live, so a stale [`Opening`] is inert. Handlers
/// run after the lock is released, so one may reopen this modal.
fn finish<R: Clone + 'static>(
    resolution: Signal<Resolution<R>>,
    closer: Callback<()>,
    generation: u64,
    value: Option<R>,
) {
    let Some(settled) = Settled::take(resolution, generation, &value) else {
        return;
    };
    closer.call(());
    settled.focus_return.restore();
    settled.notify(value);
}

/// The modal's own view of itself: its arguments, and the two ways to end it.
pub struct ModalScope<S: 'static, R: 'static = ()> {
    args: Signal<Option<S>>,
    resolution: Signal<Resolution<R>>,
    closer: Callback<()>,
    generation: u64,
}

impl<S: 'static, R: 'static> Clone for ModalScope<S, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static, R: 'static> Copy for ModalScope<S, R> {}

impl<S: Clone + 'static, R: 'static> ModalScope<S, R> {
    /// The arguments this opening was given.
    ///
    /// # Panics
    ///
    /// Once this opening has ended: Escape, the backdrop, [`close`](Self::close),
    /// [`resolve`](Self::resolve) and a superseding `open_with` end it at once. A
    /// task that outlives the opening reads [`try_args`](Self::try_args) instead.
    pub fn args(&self) -> S {
        self.try_args().expect("ModalScope outside its opening")
    }

    /// The arguments this opening was given, or `None` once it has ended,
    /// including when a later opening superseded it.
    pub fn try_args(&self) -> Option<S> {
        let live = self
            .resolution
            .try_peek()
            .is_ok_and(|resolution| resolution.generation == self.generation);
        live.then(|| self.args.try_read().ok()?.clone()).flatten()
    }
}

impl<S: 'static, R: Clone + 'static> ModalScope<S, R> {
    /// Closes with no result - the same outcome as Escape or the backdrop.
    pub fn close(&self) {
        finish(self.resolution, self.closer, self.generation, None);
    }

    /// Closes, handing `value` to the caller's handler or awaited `Opening`.
    pub fn resolve(&self, value: R) {
        finish(self.resolution, self.closer, self.generation, Some(value));
    }
}

/// One opening of a modal: attach a handler, `.await` it, or close it. A stale
/// one is inert.
pub struct Opening<R: 'static = ()> {
    resolution: Signal<Resolution<R>>,
    closer: Callback<()>,
    generation: u64,
}

impl<R: 'static> Clone for Opening<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: 'static> Copy for Opening<R> {}

impl<R: Clone + 'static> Opening<R> {
    /// Runs `handler` when this opening settles - `None` if it was dismissed.
    pub fn onresult(self, mut handler: impl FnMut(Option<R>) + 'static) -> Self {
        let mut signal = self.resolution;
        // A stale opening settled as a dismissal the moment it was superseded,
        // or its owner unmounted.
        let settled = match signal.try_peek() {
            Ok(resolution) if resolution.generation == self.generation => {
                resolution.outcome.clone()
            }
            _ => Some(None),
        };

        match settled {
            Some(outcome) => handler(outcome),
            None => signal.write().handlers.push(Box::new(handler)),
        }
        self
    }

    /// Closes this opening, if it is still the one showing.
    pub fn close(&self) {
        finish(self.resolution, self.closer, self.generation, None);
    }
}

pub struct OpeningFuture<R: 'static> {
    opening: Opening<R>,
}

impl<R: Clone + 'static> Future for OpeningFuture<R> {
    type Output = Option<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut signal = self.opening.resolution;
        // Gone with its owner, which settled it as a dismissal.
        let Ok(mut resolution) = signal.try_write() else {
            return Poll::Ready(None);
        };
        if resolution.generation != self.opening.generation {
            return Poll::Ready(None);
        }
        if let Some(outcome) = resolution.outcome.clone() {
            return Poll::Ready(outcome);
        }
        if !resolution.wakers.iter().any(|w| w.will_wake(cx.waker())) {
            resolution.wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }
}

impl<R: Clone + 'static> IntoFuture for Opening<R> {
    type Output = Option<R>;
    type IntoFuture = OpeningFuture<R>;

    fn into_future(self) -> Self::IntoFuture {
        OpeningFuture { opening: self }
    }
}

/// Opens the modal registered by [`use_modal`], from anywhere below the hook.
pub struct ModalHandle<S: 'static, R: 'static = ()> {
    args: Signal<Option<S>>,
    resolution: Signal<Resolution<R>>,
    closer: Callback<()>,
    /// Publishes the modal's slot, or clears it once closed.
    show: Callback<()>,
}

impl<S: 'static, R: 'static> Clone for ModalHandle<S, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static, R: 'static> Copy for ModalHandle<S, R> {}

impl<S: 'static, R: Clone + 'static> ModalHandle<S, R> {
    /// Opens with `args`, superseding whatever this modal was showing.
    pub fn open_with(&self, args: impl Into<S>) -> Opening<R> {
        let mut signal = self.resolution;
        let (generation, focus_return, replaced) = {
            let mut resolution = signal.write();
            resolution.generation += 1;
            resolution.outcome = None;
            resolution.handlers.clear();
            let replaced = std::mem::take(&mut resolution.wakers);
            (resolution.generation, resolution.focus_return, replaced)
        };
        // A task awaiting the replaced opening polls again and finds it stale.
        for waker in replaced {
            waker.wake();
        }
        // Still in the trigger's handler, so the active element is the trigger.
        // Skipped when already open: it would remember a control inside (todo 37).
        let opening = self.args.peek().is_none();
        if opening {
            // A WebView keeps it page-side (959).
            focus_return.remember_focused();
        }

        let mut slot = self.args;
        slot.set(Some(args.into()));
        // Already open: the slot reads `args` and redraws itself.
        if opening {
            self.show.call(());
        }

        Opening {
            resolution: self.resolution,
            closer: self.closer,
            generation,
        }
    }

    /// Closes whatever this modal is currently showing.
    pub fn close(&self) {
        let generation = self.resolution.peek().generation;
        finish(self.resolution, self.closer, generation, None);
    }

    pub fn is_open(&self) -> bool {
        self.args.read().is_some()
    }

    /// [`is_open`](Self::is_open) without subscribing, for callbacks that run outside a scope.
    pub(crate) fn is_open_untracked(&self) -> bool {
        self.args.peek().is_some()
    }
}

impl<S: Default + 'static, R: Clone + 'static> ModalHandle<S, R> {
    /// Opens with default arguments.
    pub fn open(&self) -> Opening<R> {
        self.open_with(S::default())
    }
}

/// Registers `render` as a modal and returns the handle that opens it.
///
/// Call it in a component that outlives every trigger: the modal portals from there.
/// If that component unmounts while open, the modal goes too: the opening settles as dismissed.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Dialog};
/// # use libero::hooks::{ModalScope, use_modal};
/// # fn app() -> Element {
/// # #[derive(Clone, PartialEq)] struct Confirm { message: String }
/// # impl From<&str> for Confirm { fn from(m: &str) -> Self { Confirm { message: m.into() } } }
/// let confirm = use_modal(|s: ModalScope<Confirm, bool>| rsx! {
///     Dialog { title: "{s.args().message}",
///         Button { onclick: move |_| s.resolve(true), "Delete" } }
/// });
/// confirm.open_with("Delete this file?").onresult(move |deleted| { let _ = deleted; });
/// # rsx! {}
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/modal>
pub fn use_modal<S, R>(
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
where
    S: Clone + 'static,
    R: Clone + 'static,
{
    let args = use_signal(|| None::<S>);
    let focus_return = use_focus_return();
    let resolution = use_signal(move || Resolution::<R>::new(focus_return));
    // `use_callback`, so a stored handle sees fresh captures, not the mount's.
    let slot = use_portal_slot();
    let closer = use_callback(move |()| {
        let mut args = args;
        args.set(None);
        slot.show(None);
    });
    let render = use_callback(render);

    // Runs in `ModalSlot`'s scope, so `args` and whatever `render` reads
    // subscribe it, not this caller: opening redraws the modal alone.
    let draw = use_callback(move |()| {
        // `peek`: it changes with `args`; reading would redraw per handler attached.
        let generation = resolution.peek().generation;
        args.read().is_some().then(|| {
            let scope = ModalScope {
                args,
                resolution,
                closer,
                generation,
            };
            rsx! {
                Modal {
                    onclose: move |_| finish(resolution, closer, generation, None),
                    {render.call(scope)}
                }
            }
        })
    });
    // Bumped per publish, so the slot redraws when this caller does (fresh
    // captures in `render`), and skips when the outlet redraws for another.
    let mut drawn = use_hook(|| CopyValue::new(0_u64));
    let show = use_callback(move |()| {
        let open = args.peek().is_some();
        let content = open.then(|| {
            let tick = drawn();
            drawn.set(tick + 1);
            rsx! { ModalSlot { draw, tick } }
        });
        slot.show(content);
    });
    show.call(());

    // The owner unmounting takes the modal with it: settle the opening as a
    // dismissal, so an awaiting task ends, and hand focus back.
    use_drop(move || {
        if args.try_peek().is_ok_and(|args| args.is_none()) {
            return;
        }
        let Ok(generation) = resolution
            .try_peek()
            .map(|resolution| resolution.generation)
        else {
            return;
        };
        if let Some(settled) = Settled::take(resolution, generation, &None) {
            settled.focus_return.restore_detached();
            settled.notify(None);
        }
    });

    ModalHandle {
        args,
        resolution,
        closer,
        show,
    }
}

/// The open modal's own scope, drawn in the portal outlet.
#[component]
fn ModalSlot(draw: Callback<(), Option<Element>>, tick: u64) -> Element {
    let _ = tick;
    draw.call(()).unwrap_or_else(VNode::empty)
}
