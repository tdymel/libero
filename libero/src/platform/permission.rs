//! The Permissions API state, shared by every capability that asks the user
//! first (location, camera, microphone, notifications).

use super::Read;

/// What the platform says about a permission, before or after asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PermissionState {
    Granted,
    Denied,
    /// Asking would show the user a prompt.
    Prompt,
    /// The capability exists, but the platform gave no answer (yet).
    #[default]
    Unknown,
    /// No such capability on this target.
    Unsupported,
}

impl PermissionState {
    /// Maps a Permissions API `state`; anything else is [`Unknown`](Self::Unknown).
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn from_name(state: &str) -> Self {
        match state {
            "granted" => Self::Granted,
            "denied" => Self::Denied,
            "prompt" => Self::Prompt,
            _ => Self::Unknown,
        }
    }
}

/// The permissions libero asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PermissionKind {
    Geolocation,
    Camera,
    Microphone,
    Notifications,
}

impl PermissionKind {
    /// The Permissions API name.
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Geolocation => "geolocation",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Notifications => "notifications",
        }
    }
}

/// Dropping it stops the `change` listener.
pub(crate) trait PermissionSubscription {}

/// Reads permission states without prompting.
pub(crate) trait PermissionApi {
    /// `Unknown` where the query is refused or the name is not known.
    #[allow(dead_code)] // For 1220 and 1222's one-off reads.
    fn query(&self, kind: PermissionKind) -> Read<PermissionState>;
    /// Calls `callback` with the current state, then on each change.
    fn on_change(
        &self,
        kind: PermissionKind,
        callback: Box<dyn Fn(PermissionState)>,
    ) -> Box<dyn PermissionSubscription>;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use js_sys::{Function, Object, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;

    use super::{PermissionApi, PermissionKind, PermissionState, PermissionSubscription};
    use crate::platform::Read;

    /// `navigator.permissions`, by name: web-sys's binding changes shape under
    /// `web_sys_unstable_apis`, which a consumer may set.
    pub(super) fn permissions() -> Option<Object> {
        let navigator = web_sys::window()?.navigator();
        Reflect::get(&navigator, &JsValue::from_str("permissions"))
            .ok()?
            .dyn_into::<Object>()
            .ok()
    }

    /// The `PermissionStatus` for `kind`, `None` where the query rejects.
    async fn status(kind: PermissionKind) -> Option<Object> {
        let permissions = permissions()?;
        let query = Reflect::get(&permissions, &JsValue::from_str("query"))
            .ok()?
            .dyn_into::<Function>()
            .ok()?;
        let descriptor = Object::new();
        Reflect::set(&descriptor, &"name".into(), &kind.name().into()).ok()?;
        let promise = query
            .call1(&permissions, &descriptor)
            .ok()?
            .dyn_into::<Promise>()
            .ok()?;
        JsFuture::from(promise)
            .await
            .ok()?
            .dyn_into::<Object>()
            .ok()
    }

    fn state_of(status: &Object) -> PermissionState {
        Reflect::get(status, &JsValue::from_str("state"))
            .ok()
            .and_then(|state| state.as_string())
            .map_or(PermissionState::Unknown, |state| {
                PermissionState::from_name(&state)
            })
    }

    pub(super) struct WebPermissions;

    impl PermissionApi for WebPermissions {
        fn query(&self, kind: PermissionKind) -> Read<PermissionState> {
            Box::pin(async move {
                Ok(status(kind)
                    .await
                    .map_or(PermissionState::Unknown, |status| state_of(&status)))
            })
        }

        fn on_change(
            &self,
            kind: PermissionKind,
            callback: Box<dyn Fn(PermissionState)>,
        ) -> Box<dyn PermissionSubscription> {
            let listener = Rc::new(StatusListener::default());
            let held = listener.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(status) = status(kind).await else {
                    return callback(PermissionState::Unknown);
                };
                if held.stopped.get() {
                    return;
                }
                callback(state_of(&status));
                let read = status.clone();
                let change = Closure::<dyn Fn()>::new(move || callback(state_of(&read)));
                let _ = Reflect::set(&status, &"onchange".into(), change.as_ref());
                held.status.replace(Some((status, change)));
            });
            Box::new(WebPermissionListener(listener))
        }
    }

    /// The `PermissionStatus` and its `onchange` handler.
    type Listening = (Object, Closure<dyn Fn()>);

    #[derive(Default)]
    struct StatusListener {
        stopped: Cell<bool>,
        status: RefCell<Option<Listening>>,
    }

    struct WebPermissionListener(Rc<StatusListener>);

    impl PermissionSubscription for WebPermissionListener {}

    impl Drop for WebPermissionListener {
        fn drop(&mut self) {
            self.0.stopped.set(true);
            if let Some((status, _change)) = self.0.status.take() {
                let _ = Reflect::set(&status, &"onchange".into(), &JsValue::NULL);
            }
        }
    }

    pub(super) static PERMISSIONS: WebPermissions = WebPermissions;
}

/// `None` without a Permissions API: Blitz, a server, an old browser. Call it
/// after mount: a server render answers `None`, a hydrating client may not.
pub(crate) fn permission() -> Option<&'static dyn PermissionApi> {
    #[cfg(target_arch = "wasm32")]
    return web::permissions().map(|_| &web::PERMISSIONS as &'static dyn PermissionApi);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_permission();
}
