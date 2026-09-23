use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::{ElementHandle, use_element};
use crate::platform::{ContentSubscription, on_intersection};

/// What [`use_intersection`] takes.
#[derive(Clone, PartialEq)]
pub struct IntersectionOptions {
    /// The element whose box clips the target; the viewport when `None`.
    pub root: Option<ElementHandle>,
    /// CSS margin that grows or shrinks the root's box, like `"100px 0px"`.
    pub root_margin: String,
    /// Visible ratios (0 to 1) at which the entry updates.
    pub thresholds: Vec<f64>,
    /// Stops observing once the element intersected, so `entry` keeps that state.
    pub once: bool,
}

impl Default for IntersectionOptions {
    fn default() -> Self {
        Self {
            root: None,
            root_margin: "0px".to_string(),
            thresholds: vec![0.0],
            once: false,
        }
    }
}

/// How much of the observed element is visible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectionEntry {
    pub is_intersecting: bool,
    /// The visible share of the element's box, 0 to 1.
    pub ratio: f64,
}

/// An observed element: give it to `onmounted`, read `entry`.
#[derive(Clone, Copy)]
pub struct Intersection {
    /// The `onmounted` handler of the observed element.
    pub on_mounted: Callback<MountedEvent>,
    /// `None` until the first observation, and always `None` where nothing can
    /// observe: Blitz, a WebView, a server render.
    pub entry: ReadSignal<Option<IntersectionEntry>>,
}

type Slot = Rc<RefCell<Option<Box<dyn ContentSubscription>>>>;

/// Reports how much of an element is visible inside its root.
///
/// It is built on the browser's `IntersectionObserver`. Where there is none,
/// `entry` stays `None` and the element counts as never intersecting.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{IntersectionOptions, use_intersection};
/// # fn app() -> Element {
/// let seen = use_intersection(IntersectionOptions {
///     once: true,
///     ..Default::default()
/// });
/// let revealed = seen.entry.read().is_some_and(|entry| entry.is_intersecting);
///
/// rsx! {
///     div { onmounted: move |event| seen.on_mounted.call(event),
///         if revealed {
///             "Now visible"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-intersection>
pub fn use_intersection(options: IntersectionOptions) -> Intersection {
    let IntersectionOptions {
        root,
        root_margin,
        thresholds,
        once,
    } = options;
    let target = use_element();
    let entry = use_signal(|| None::<IntersectionEntry>);
    let slot: Slot = use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });

    use_effect({
        let slot = slot.clone();
        use_reactive!(|root, root_margin, thresholds, once| {
            let _ = target.mount_token();
            let _ = root.and_then(|root| root.mount_token());
            // Dropped first, so a changed option never leaves two observers.
            slot.borrow_mut().take();
            let done = once && entry.peek().is_some_and(|entry| entry.is_intersecting);
            let watching = (!done)
                .then(|| target.mounted())
                .flatten()
                .and_then(|mounted| {
                    let root_mounted = match root {
                        Some(root) => Some(root.mounted()?),
                        None => None,
                    };
                    on_intersection(
                        &mounted,
                        root_mounted.as_ref(),
                        &root_margin,
                        &thresholds,
                        Box::new(move |is_intersecting, ratio| {
                            let next = Some(IntersectionEntry {
                                is_intersecting,
                                ratio,
                            });
                            let mut entry = entry;
                            if *entry.peek() != next {
                                entry.set(next);
                            }
                        }),
                    )
                });
            *slot.borrow_mut() = watching;
        })
    });
    use_effect({
        let slot = slot.clone();
        use_reactive!(|once| {
            if once && entry().is_some_and(|entry| entry.is_intersecting) {
                slot.borrow_mut().take();
            }
        })
    });

    let on_mounted = use_callback(move |event: MountedEvent| target.mount()(event));
    Intersection {
        on_mounted,
        entry: entry.into(),
    }
}

/// Whether an element is in the viewport: [`use_intersection`] with its
/// defaults, reduced to a bool. `false` where nothing can observe.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_in_viewport;
/// # fn app() -> Element {
/// let (on_mounted, visible) = use_in_viewport();
///
/// rsx! {
///     div { onmounted: move |event| on_mounted.call(event),
///         if visible() {
///             "On screen"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-intersection>
pub fn use_in_viewport() -> (Callback<MountedEvent>, ReadSignal<bool>) {
    let Intersection { on_mounted, entry } = use_intersection(IntersectionOptions::default());
    let visible = use_memo(move || entry().is_some_and(|entry| entry.is_intersecting));
    (on_mounted, visible.into())
}
