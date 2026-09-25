use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::platform::{
    Fix, GeolocationError, GeolocationOptions, GeolocationSubscription, PermissionKind,
    PermissionState, PermissionSubscription, Position, geolocation, permission,
};

/// The device's position, asked for only on [`request`](Self::request) or
/// [`watch`](Self::watch): mounting never prompts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_geolocation;
/// # fn app() -> Element {
/// let mut location = use_geolocation(Default::default());
/// let status = match (location.position(), location.error()) {
///     (_, Some(error)) => format!("No location: {error:?}"),
///     (Some(fix), None) => format!("{:.2}, {:.2}", fix.latitude, fix.longitude),
///     (None, None) => "Location not shared".to_string(),
/// };
///
/// rsx! {
///     button { onclick: move |_| location.request(), "Share location" }
///     p { role: "status", "{status}" }
/// }
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct Geolocation {
    options: GeolocationOptions,
    supported: Signal<bool>,
    permission: Signal<PermissionState>,
    position: Signal<Option<Position>>,
    error: Signal<Option<GeolocationError>>,
    pending: Signal<bool>,
    watching: Signal<bool>,
    watch: Signal<Option<Box<dyn GeolocationSubscription>>>,
}

impl Geolocation {
    /// Asks for one fix, prompting if the user has not answered yet. Call it
    /// from a user's action. Does nothing while an earlier request waits.
    pub fn request(&mut self) {
        if *self.pending.peek() {
            return;
        }
        let Some(api) = geolocation() else {
            self.settle(Err(GeolocationError::Unsupported));
            return;
        };
        let locate = api.current(self.options);
        self.pending.set(true);
        let mut this = *self;
        spawn(async move {
            let fix = locate.await;
            this.pending.set(false);
            this.settle(fix);
        });
    }

    /// Follows the position until [`stop`](Self::stop), a denial or unmount.
    /// Prompts like [`request`](Self::request).
    pub fn watch(&mut self) {
        if *self.watching.peek() {
            return;
        }
        let Some(api) = geolocation() else {
            self.settle(Err(GeolocationError::Unsupported));
            return;
        };
        let this = *self;
        let watch = api.watch(
            self.options,
            Box::new(move |fix| {
                let mut this = this;
                this.settle(fix);
            }),
        );
        self.watch.set(Some(watch));
        self.watching.set(true);
    }

    /// Ends a [`watch`](Self::watch); the last position stays.
    pub fn stop(&mut self) {
        self.watching.set(false);
        self.watch.set(None);
    }

    pub(crate) fn settle(&mut self, fix: Fix) {
        let permission = *self.permission.peek();
        let known = match fix {
            Ok(position) => {
                self.position.set(Some(position));
                self.error.set(None);
                PermissionState::Granted
            }
            Err(error) => {
                self.error.set(Some(error));
                match error {
                    // The watch may be the caller; the effect drops it after this returns.
                    GeolocationError::Denied => {
                        self.watching.set(false);
                        PermissionState::Denied
                    }
                    GeolocationError::Unsupported => {
                        self.supported.set(false);
                        self.watching.set(false);
                        PermissionState::Unsupported
                    }
                    _ => permission,
                }
            }
        };
        if known != permission {
            self.permission.set(known);
        }
    }

    /// Whether this target has a Geolocation API. `false` until mounted. Reactive.
    pub fn is_supported(&self) -> bool {
        (self.supported)()
    }

    /// The location permission, kept current where the platform reports changes. Reactive.
    pub fn permission(&self) -> PermissionState {
        (self.permission)()
    }

    /// The last fix. Reactive.
    pub fn position(&self) -> Option<Position> {
        (self.position)()
    }

    /// Why the last attempt failed; the next fix clears it. Reactive.
    pub fn error(&self) -> Option<GeolocationError> {
        (self.error)()
    }

    /// Whether a [`request`](Self::request) waits for its answer. Reactive.
    pub fn is_pending(&self) -> bool {
        (self.pending)()
    }

    /// Whether a [`watch`](Self::watch) runs. Reactive.
    pub fn is_watching(&self) -> bool {
        (self.watching)()
    }
}

/// A [`Geolocation`] for this component. `options` apply to the next
/// `request` or `watch`. Announces nothing: say a found or refused location
/// once, never every fix.
pub fn use_geolocation(options: GeolocationOptions) -> Geolocation {
    let mut location = Geolocation {
        options,
        supported: use_signal(|| false),
        permission: use_signal(PermissionState::default),
        position: use_signal(|| None),
        error: use_signal(|| None),
        pending: use_signal(|| false),
        watching: use_signal(|| false),
        watch: use_signal(|| None),
    };
    let listener: Rc<RefCell<Option<Box<dyn PermissionSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let listener = listener.clone();
        move || {
            listener.borrow_mut().take();
            location.watch.write_unchecked().take();
        }
    });
    // A server cannot know either answer, so both are read after mount (hydration).
    use_effect(move || {
        let supported = geolocation().is_some();
        location.supported.set(supported);
        if !supported {
            location.permission.set(PermissionState::Unsupported);
            return;
        }
        let Some(api) = permission() else {
            return;
        };
        let changes = api.on_change(
            PermissionKind::Geolocation,
            Box::new(move |state| {
                let mut permission = location.permission;
                if *permission.peek() != state {
                    permission.set(state);
                }
            }),
        );
        *listener.borrow_mut() = Some(changes);
    });
    use_effect(move || {
        if !(location.watching)() && location.watch.peek().is_some() {
            location.watch.set(None);
        }
    });
    location
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "geolocation_tests.rs"]
mod tests;
