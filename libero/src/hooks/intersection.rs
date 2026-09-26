use std::{cell::RefCell, rc::Rc};

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use super::{ElementHandle, use_element};
use crate::platform::{
    ContentSubscription, INTERSECT_ATTR, next_observe_tag, observes_by_tag, on_intersection,
};

/// What [`use_intersection`] takes.
#[derive(Clone, PartialEq)]
pub struct IntersectionOptions {
    /// The element whose box clips the target; the viewport when `None`. A WebView
    /// needs `root.attributes()` spread on it, else it falls back to the viewport.
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

/// An observed element: give `on_mounted` to its `onmounted`, spread
/// `attributes` on it, read `entry`.
#[derive(Clone)]
pub struct Intersection {
    /// The `onmounted` handler of the observed element.
    pub on_mounted: Callback<MountedEvent>,
    /// Spread on the observed element (`..seen.attributes`). A WebView finds the
    /// element by it and observes nothing without; empty on the web and Blitz.
    pub attributes: Vec<Attribute>,
    /// `None` until the first observation, and always `None` where nothing can
    /// observe: Blitz, a server render.
    pub entry: ReadSignal<Option<IntersectionEntry>>,
}

type Slot = Rc<RefCell<Option<Box<dyn ContentSubscription>>>>;

/// Reports how much of an element is visible inside its root.
///
/// It is built on the browser's `IntersectionObserver`, in a WebView (desktop,
/// Android) too, where the element must carry `attributes`. Where there is
/// neither, `entry` stays `None` and the element counts as never intersecting.
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
///     div {
///         onmounted: move |event| seen.on_mounted.call(event),
///         ..seen.attributes,
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
    let tag = use_hook(|| observes_by_tag().then(next_observe_tag));
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
                    let root_tag = root.and_then(|root| root.tag());
                    if cfg!(debug_assertions) && tag.is_some() && root.is_some() && root_tag.is_none() {
                        crate::utils::warn(
                            "use_intersection: the root carries no `attributes()`, so a WebView observes against the viewport",
                        );
                    }
                    on_intersection(
                        &mounted,
                        root_mounted.as_ref(),
                        (tag.unwrap_or_default(), root_tag),
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
    let attributes = tag
        .map(|tag| {
            Attribute::new(
                INTERSECT_ATTR,
                AttributeValue::Text(tag.to_string()),
                None,
                false,
            )
        })
        .into_iter()
        .collect();
    Intersection {
        on_mounted,
        attributes,
        entry: entry.into(),
    }
}

/// What [`use_in_viewport`] gives: [`use_intersection`] with its defaults,
/// reduced to a bool.
#[derive(Clone)]
pub struct InViewport {
    /// The `onmounted` handler of the observed element.
    pub on_mounted: Callback<MountedEvent>,
    /// Spread on the observed element, see [`Intersection::attributes`].
    pub attributes: Vec<Attribute>,
    /// `false` where nothing can observe.
    pub visible: ReadSignal<bool>,
}

/// Whether an element is in the viewport, as a bool.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_in_viewport;
/// # fn app() -> Element {
/// let seen = use_in_viewport();
///
/// rsx! {
///     div {
///         onmounted: move |event| seen.on_mounted.call(event),
///         ..seen.attributes,
///         if (seen.visible)() {
///             "On screen"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-intersection>
pub fn use_in_viewport() -> InViewport {
    let Intersection {
        on_mounted,
        attributes,
        entry,
    } = use_intersection(IntersectionOptions::default());
    let visible = use_memo(move || entry().is_some_and(|entry| entry.is_intersecting));
    InViewport {
        on_mounted,
        attributes,
        visible: visible.into(),
    }
}
