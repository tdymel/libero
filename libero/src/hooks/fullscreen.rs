use std::{cell::RefCell, rc::Rc};

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use crate::hooks::{ElementHandle, FocusWithin, listener, use_back, use_focus_within};
use crate::platform::{self, FullscreenApi, FullscreenSubscription, focused_attribute, next_task};

/// Set on the element while it fills the screen: `native` or `drawn`.
pub(crate) const FULLSCREEN_ATTR: &str = "data-fullscreen";

/// Puts one element in fullscreen: the Fullscreen API where the page has it and
/// grants it, else a drawn fullscreen, which the element's own CSS draws.
#[derive(Clone, Copy, PartialEq)]
pub struct FullscreenHandle {
    element: ElementHandle,
    available: Signal<bool>,
    active: Signal<bool>,
    drawn: Signal<bool>,
    focus: FocusWithin,
}

impl FullscreenHandle {
    fn api(&self) -> Option<Box<dyn FullscreenApi>> {
        platform::fullscreen(&self.element.mounted()?, self.element.tag())
    }

    /// Native or drawn; reactive.
    pub fn is_fullscreen(&self) -> bool {
        (self.active)() || (self.drawn)()
    }

    /// Only the drawn fullscreen; reactive.
    pub fn is_drawn(&self) -> bool {
        (self.drawn)()
    }

    /// Asks for native fullscreen; drawn where the platform has none or refuses.
    pub fn enter(&self) {
        let mut drawn = self.drawn;
        if *self.active.peek() || *drawn.peek() {
            return;
        }
        match self.api().filter(|_| *self.available.peek()) {
            // Refused (a headless browser, a WebView without fullscreen): drawn.
            Some(api) => {
                spawn(async move {
                    if api.enter().await.is_err() {
                        drawn.set(true);
                    }
                });
            }
            None => drawn.set(true),
        }
    }

    /// Leaves either fullscreen.
    pub fn exit(&self) {
        let mut drawn = self.drawn;
        if *self.active.peek() {
            if let Some(api) = self.api() {
                let _ = api.exit();
            }
        } else if *drawn.peek() {
            drawn.set(false);
        }
    }

    pub fn toggle(&self) {
        if *self.active.peek() || *self.drawn.peek() {
            self.exit();
        } else {
            self.enter();
        }
    }

    /// Spread on the element: its [`ElementHandle::attributes`], `data-fullscreen`,
    /// and the listeners that leave the drawn fullscreen on Escape or focus leaving.
    pub fn attributes(&self) -> Vec<Attribute> {
        self.attributes_with_focusin(|_| {})
    }

    /// [`attributes`](Self::attributes), its `focusin` also calling `extra`: an element keeps one listener per event.
    pub(crate) fn attributes_with_focusin(
        &self,
        mut extra: impl FnMut(&Event<FocusData>) + 'static,
    ) -> Vec<Attribute> {
        let mut focusin = self.focus.focusin(0);
        let state = match ((self.active)(), (self.drawn)()) {
            (_, true) => Some("drawn"),
            (true, false) => Some("native"),
            _ => None,
        };
        let handle = *self;
        let mut attributes = self.element.attributes();
        attributes.extend(state.map(|state| {
            Attribute::new(
                FULLSCREEN_ATTR,
                AttributeValue::Text(state.into()),
                None,
                false,
            )
        }));
        // A click on nothing focusable inside lands focus here, not on the page.
        if state == Some("drawn") {
            attributes.push(Attribute::new(
                "tabindex",
                AttributeValue::Text("-1".into()),
                None,
                false,
            ));
        }
        attributes.extend([
            listener("onkeydown", move |event: Event<KeyboardData>| {
                if *handle.drawn.peek() && event.key() == Key::Escape {
                    event.prevent_default();
                    handle.exit();
                }
            }),
            listener("onfocusin", move |event: Event<FocusData>| {
                extra(&event);
                focusin(event);
            }),
            listener("onfocusout", self.focus.focusout(0)),
        ]);
        attributes
    }
}

/// Puts `element` in fullscreen, natively or drawn. Spread
/// [`attributes`](FullscreenHandle::attributes) on it and mount it.
///
/// Where the platform has no Fullscreen API or refuses it (a headless browser, an
/// iframe without `allowfullscreen`), the handle draws it: it sets `data-fullscreen="drawn"`, and
/// your CSS makes the element a fixed box over the page. Escape and focus
/// leaving the element leave it, so the covered page is never reachable.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{use_element, use_fullscreen};
/// # fn app() -> Element {
/// let chart = use_element();
/// let fullscreen = use_fullscreen(chart);
///
/// rsx! {
///     style { "#chart[data-fullscreen='drawn'] {{ position: fixed; inset: 0; z-index: 1000; }}" }
///     div { id: "chart", onmounted: chart.mount(), ..fullscreen.attributes(),
///         button { onclick: move |_| fullscreen.toggle(),
///             if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-fullscreen>
pub fn use_fullscreen(element: ElementHandle) -> FullscreenHandle {
    let mut drawn = use_signal(|| false);
    // A `focusin` in between only moved focus inside; a WebView's trails over
    // IPC, so there the page says where focus landed.
    let moves = use_signal(|| 0u64);
    let focus = use_focus_within(
        move || vec![element.mounted()],
        move |change| {
            let mut moves = moves;
            if change.within {
                moves += 1;
                return;
            }
            if !*drawn.peek() {
                return;
            }
            let seen = *moves.peek();
            let landed = focused_attribute(FULLSCREEN_ATTR);
            spawn(async move {
                next_task().await;
                let left = match landed.await {
                    Ok(state) => state.is_none(),
                    Err(_) => *moves.peek() == seen,
                };
                if left && *drawn.peek() {
                    drawn.set(false);
                }
            });
        },
    );
    let handle = FullscreenHandle {
        element,
        available: use_signal(|| false),
        active: use_signal(|| false),
        drawn,
        focus,
    };
    // Android's Back leaves the fullscreen rather than the app (1275).
    use_back(
        handle.is_fullscreen(),
        use_callback(move |()| handle.exit()),
    );
    let slot: Rc<RefCell<Option<Box<dyn FullscreenSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(move || {
        let Some(_) = element.mount_token() else {
            return;
        };
        slot.borrow_mut().take();
        let (available, active) = (handle.available, handle.active);
        *slot.borrow_mut() = handle.api().map(|api| {
            api.watch(Box::new(move |state| {
                let (mut available, mut active) = (available, active);
                if *available.peek() != state.available {
                    available.set(state.available);
                }
                if *active.peek() != state.active {
                    active.set(state.active);
                }
            }))
        });
    });
    handle
}
