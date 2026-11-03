use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    platform::{MediaQuerySubscription, hydrating, media_query},
    tokens::Size,
};

/// The viewport's breakpoint: `None` under `xs`, else the widest [`Size`] whose
/// `min-width` matches. One per outermost [`LiberoProvider`](crate::LiberoProvider),
/// shared by nested ones; watched only once a responsive value asks.
#[derive(Clone, Copy)]
pub(crate) struct Viewport {
    /// `None` until known: on a server, while hydrating, before the first answer.
    breakpoint: Signal<Option<Option<Size>>>,
    watch: CopyValue<ViewportWatch>,
}

#[derive(Default)]
struct ViewportWatch {
    /// A hydrating client may not answer before mount.
    ready: bool,
    wanted: bool,
    /// The six `min-width` watches, or empty once forced or where nothing measures.
    started: Option<Vec<Box<dyn MediaQuerySubscription>>>,
}

impl Viewport {
    fn new() -> Self {
        Self {
            breakpoint: Signal::new(None),
            watch: CopyValue::new(ViewportWatch {
                ready: !hydrating(),
                ..ViewportWatch::default()
            }),
        }
    }

    /// The provider's mount: a hydrating client starts the watches it was asked for.
    fn mounted(self) {
        let mut watch = self.watch;
        let wanted = {
            let mut watch = watch.write();
            watch.ready = true;
            watch.wanted && watch.started.is_none()
        };
        if wanted {
            self.start();
        }
    }

    /// The breakpoint, starting the watches on first use. Subscribes the caller.
    pub(crate) fn breakpoint(self) -> Option<Option<Size>> {
        let mut watch = self.watch;
        let start = {
            let mut watch = watch.write();
            watch.wanted = true;
            watch.ready && watch.started.is_none()
        };
        if start {
            self.start();
        }
        (self.breakpoint)()
    }

    /// The web answers inside `watch`, so a client that does not hydrate knows the
    /// breakpoint in its first render; Blitz and a WebView answer later.
    fn start(self) {
        let Some(api) = media_query() else {
            let mut watch = self.watch;
            watch.write().started = Some(Vec::new());
            return;
        };
        let matched = Rc::new(Cell::new([None::<bool>; Size::ALL.len()]));
        let starting = Rc::new(Cell::new(true));
        let publish = {
            let (matched, starting) = (matched.clone(), starting.clone());
            move || {
                let matched = matched.get();
                if starting.get() || matched.contains(&None) {
                    return;
                }
                let now = Size::ALL
                    .into_iter()
                    .zip(matched)
                    .rev()
                    .find(|(_, on)| *on == Some(true))
                    .map(|(size, _)| size);
                let mut breakpoint = self.breakpoint;
                if *breakpoint.peek() != Some(now) {
                    breakpoint.set(Some(now));
                }
            }
        };
        let watches = Size::ALL
            .into_iter()
            .enumerate()
            .map(|(index, size)| {
                let (matched, publish) = (matched.clone(), publish.clone());
                api.watch(
                    &format!("(min-width: {})", size.breakpoint_value()),
                    Box::new(move |now| {
                        let mut all = matched.get();
                        all[index] = Some(now);
                        matched.set(all);
                        publish();
                    }),
                )
            })
            .collect();
        let mut watch = self.watch;
        watch.write().started = Some(watches);
        // One write for the six synchronous answers.
        starting.set(false);
        publish();
    }

    /// Whether a responsive value asked for the breakpoint.
    #[cfg(test)]
    pub(crate) fn wanted(self) -> bool {
        self.watch.peek().wanted
    }

    /// Pins the breakpoint and drops the watches, for tests.
    #[cfg(test)]
    pub(crate) fn force(self, breakpoint: Option<Size>) {
        let mut watch = self.watch;
        watch.write().started = Some(Vec::new());
        let mut signal = self.breakpoint;
        signal.set(Some(breakpoint));
    }
}

/// The outermost provider makes the viewport, a nested one shares it. Call once per provider.
pub(crate) fn use_viewport_provider(outermost: bool) -> Viewport {
    let viewport = use_hook(|| match try_consume_context::<Viewport>() {
        Some(outer) if !outermost => outer,
        _ => Viewport::new(),
    });
    use_context_provider(|| viewport);
    use_effect(move || {
        if outermost {
            viewport.mounted();
        }
    });
    viewport
}
