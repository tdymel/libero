use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use dioxus::prelude::*;

use crate::{
    components::Modal,
    context::{ModalContext, ModalHost},
    hooks::{FocusReturn, use_focus_return, use_portal},
};

pub(crate) fn use_modal_z_index() -> i32 {
    let host = use_context::<ModalHost>();
    let id = use_hook(|| host.open());
    use_drop(move || host.close(id));
    host.z_index(id)
}

/// Closes the modal this content is rendered in. For a component factored out
/// of the render closure, which cannot capture its [`ModalScope`].
pub fn use_modal_close() -> Callback<()> {
    let modal = use_context::<ModalContext>();
    use_callback(move |()| modal.close())
}

/// Everything about one opening that does not mention the argument type, so
/// [`Opening`] can outlive it without carrying `S`.
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

/// Settles `generation` if it is still the live one, otherwise does nothing -
/// which is what makes a stale [`Opening`] inert.
///
/// Handlers run after the write lock is released, so one of them may open this
/// same modal again.
fn finish<R: Clone + 'static>(
    mut resolution: Signal<Resolution<R>>,
    closer: Callback<()>,
    generation: u64,
    value: Option<R>,
) {
    let (handlers, wakers, focus_return) = {
        let mut resolution = resolution.write();
        if resolution.generation != generation || resolution.outcome.is_some() {
            return;
        }
        resolution.outcome = Some(value.clone());
        (
            std::mem::take(&mut resolution.handlers),
            std::mem::take(&mut resolution.wakers),
            resolution.focus_return,
        )
    };

    closer.call(());
    focus_return.restore();

    for mut handler in handlers {
        handler(value.clone());
    }
    for waker in wakers {
        waker.wake();
    }
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
    pub fn args(&self) -> S {
        self.args
            .read()
            .clone()
            .expect("ModalScope outside an open modal")
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

/// One opening of a modal. Attach per-open consequences to it, `.await` it, or
/// close it. Keeping it past its opening is safe: a stale handle is inert, it
/// never reaches whichever modal is open later.
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
        // A stale opening settled as a dismissal the moment it was superseded.
        let settled = {
            let resolution = signal.peek();
            if resolution.generation == self.generation {
                resolution.outcome.clone()
            } else {
                Some(None)
            }
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
        let mut resolution = signal.write();
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
        let (generation, focus_return) = {
            let mut resolution = signal.write();
            resolution.generation += 1;
            resolution.outcome = None;
            resolution.handlers.clear();
            resolution.wakers.clear();
            (resolution.generation, resolution.focus_return)
        };
        // Out of the guard, still inside the trigger's own handler - which is
        // what makes the active element the one the user acted on.
        //
        // Skipped when this modal is *already* open, because then the active
        // element is a control inside the overlay that is about to be torn
        // down, and remembering it would clobber the trigger that is still the
        // right answer ([[todos]] item 37). `peek`: `open_with` is called from
        // handlers, and nothing here should subscribe.
        if self.args.peek().is_none() {
            focus_return.remember_active();
        }

        let mut slot = self.args;
        slot.set(Some(args.into()));

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
}

impl<S: Default + 'static, R: Clone + 'static> ModalHandle<S, R> {
    /// Opens with default arguments.
    pub fn open(&self) -> Opening<R> {
        self.open_with(S::default())
    }
}

/// Registers `render` as a modal and returns the handle that opens it.
///
/// The modal is portaled from here, so this must be called in a component that
/// outlives every trigger. Arguments shared by every opening are simply
/// captured by `render`. The handle is `Copy`; a dialog wanting one shared
/// instance can `use_context_provider` it in its own hook.
///
/// ```ignore
/// let confirm = use_modal(|s: ModalScope<Confirm, bool>| rsx! {
///     Dialog { title: "{s.args().message}",
///         Button { onclick: move |_| s.resolve(true), "Delete" } }
/// });
/// confirm.open_with("Delete this file?").onresult(move |r| { .. });
/// ```
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
    // `use_callback`, not the closure directly: rsx rebuilds `render` every
    // render, and a context-provided handle is stored once - the swap keeps
    // captured values live instead of frozen at mount.
    let closer = use_callback(move |()| {
        let mut args = args;
        args.set(None);
    });
    let render = use_callback(render);

    let handle = ModalHandle {
        args,
        resolution,
        closer,
    };

    // `peek`: the generation only ever changes together with `args`, which is
    // already subscribed below, and reading it would re-render on every
    // handler attached.
    let generation = resolution.peek().generation;
    let content = args.read().is_some().then(|| {
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
    });
    use_portal(content);

    handle
}
