//! Debounced and throttled values and callbacks, on the shared [`Scheduled`]
//! timer. A change lands from an effect, so a value follows a render after its
//! source.

use dioxus::prelude::*;

use super::timers::{Slot, run_slot, use_latest_ms, use_scheduled};

/// `value`, delayed until it has stopped changing for `ms`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_debounced_value;
/// # fn app() -> Element {
/// let mut query = use_signal(String::new);
/// let settled = use_debounced_value(query.into(), 300);
///
/// rsx! {
///     input { oninput: move |event| query.set(event.value()) }
///     p { "Searching for {settled}" }
/// }
/// # }
/// ```
///
/// The first value shows at once. A change and its undo inside `ms` never
/// shows. Without a timer, on a server render, the value never follows.
pub fn use_debounced_value<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    ms: u64,
) -> ReadSignal<T> {
    let ms = use_latest_ms(ms);
    let mut settled = use_signal(|| value.peek().clone());
    let delay = use_scheduled(move |_| {
        let next = value.peek().clone();
        if *settled.peek() != next {
            settled.set(next);
        }
    });
    use_effect(move || {
        let next = value();
        match *settled.peek() == next {
            true => delay.cancel(),
            false => delay.after(*ms.peek()),
        }
    });
    settled.into()
}

/// A callback that runs `ms` after its last call, with that call's argument.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_debounced_callback;
/// # fn app() -> Element {
/// let mut searched = use_signal(String::new);
/// let search = use_debounced_callback(move |text: String| searched.set(text), 300);
///
/// rsx! {
///     input { oninput: move |event| search.call(event.value()) }
///     p { "Searched for {searched}" }
/// }
/// # }
/// ```
///
/// A pending call is dropped when the component unmounts.
pub fn use_debounced_callback<A: 'static>(
    callback: impl FnMut(A) + 'static,
    ms: u64,
) -> Callback<A> {
    let ms = use_latest_ms(ms);
    let mut latest = use_hook(|| CopyValue::new(None::<Slot<A>>));
    latest.set(Some(Box::new(callback)));
    let mut held = use_hook(|| CopyValue::new(None::<A>));
    let delay = use_scheduled(move |_| {
        let arg = held.write().take();
        if let Some(arg) = arg {
            run_slot(latest, arg);
        }
    });
    use_callback(move |arg: A| {
        held.set(Some(arg));
        delay.after(*ms.peek());
    })
}

/// `value`, updated at most once per `ms`: a change shows at once when the
/// window is quiet, otherwise at the window's end.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_throttled_value;
/// # fn app() -> Element {
/// let mut pointer = use_signal(|| 0);
/// let shown = use_throttled_value(pointer.into(), 100);
///
/// rsx! {
///     div { onmousemove: move |event| pointer.set(event.client_coordinates().x as i32),
///         "x: {shown}"
///     }
/// }
/// # }
/// ```
///
/// Without a timer, on a server render, a change after the first waits for
/// nothing and never shows.
pub fn use_throttled_value<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    ms: u64,
) -> ReadSignal<T> {
    let ms = use_latest_ms(ms);
    let mut shown = use_signal(|| value.peek().clone());
    let mut open = use_hook(|| CopyValue::new(false));
    let window = use_scheduled(move |window| {
        let next = value.peek().clone();
        let changed = *shown.peek() != next;
        match changed {
            true => {
                shown.set(next);
                window.after(*ms.peek());
            }
            false => open.set(false),
        }
    });
    use_effect(move || {
        let next = value();
        if *shown.peek() == next || *open.peek() {
            return;
        }
        shown.set(next);
        open.set(true);
        window.after(*ms.peek());
    });
    shown.into()
}

/// A callback that runs at once, then at most once per `ms`: calls inside the
/// window collapse into one at its end, with the last argument.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_throttled_callback;
/// # fn app() -> Element {
/// let mut moves = use_signal(|| 0);
/// let track = use_throttled_callback(move |()| moves += 1, 100);
///
/// rsx! {
///     div { onmousemove: move |_| track.call(()), "{moves} moves counted" }
/// }
/// # }
/// ```
///
/// A pending trailing call is dropped when the component unmounts.
pub fn use_throttled_callback<A: 'static>(
    callback: impl FnMut(A) + 'static,
    ms: u64,
) -> Callback<A> {
    let ms = use_latest_ms(ms);
    let mut latest = use_hook(|| CopyValue::new(None::<Slot<A>>));
    latest.set(Some(Box::new(callback)));
    let mut held = use_hook(|| CopyValue::new(None::<A>));
    let mut open = use_hook(|| CopyValue::new(false));
    let window = use_scheduled(move |window| {
        let arg = held.write().take();
        match arg {
            Some(arg) => {
                run_slot(latest, arg);
                window.after(*ms.peek());
            }
            None => open.set(false),
        }
    });
    use_callback(move |arg: A| {
        if *open.peek() {
            held.set(Some(arg));
            return;
        }
        open.set(true);
        run_slot(latest, arg);
        window.after(*ms.peek());
    })
}
