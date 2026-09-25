//! The device's position, from the Geolocation API.

use std::{future::Future, pin::Pin, time::Duration};

/// One fix. `accuracy` and `altitude_accuracy` are radii in metres; a fix from
/// Wi-Fi or IP can be kilometres wide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    /// Degrees clockwise from true north.
    pub heading: Option<f64>,
    /// Metres per second.
    pub speed: Option<f64>,
    /// Milliseconds since the Unix epoch.
    pub timestamp_ms: f64,
}

/// The Geolocation API's `PositionOptions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeolocationOptions {
    /// Asks for GPS-grade fixes: slower, more battery. A hint only.
    pub high_accuracy: bool,
    /// How long one fix may take, prompt excluded; `None` waits forever.
    pub timeout: Option<Duration>,
    /// How old a cached fix may be; zero always asks for a fresh one.
    pub max_age: Duration,
}

/// Why no fix came.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeolocationError {
    /// No Geolocation API on this target.
    Unsupported,
    /// The user or the platform refused, including off a secure context.
    Denied,
    /// No fix could be had: location services off, no signal.
    Unavailable,
    /// No fix within [`GeolocationOptions::timeout`].
    Timeout,
}

impl GeolocationError {
    /// From a `GeolocationPositionError.code`.
    #[cfg_attr(feature = "native", allow(dead_code))]
    pub(crate) fn from_code(code: u16) -> Self {
        match code {
            1 => Self::Denied,
            3 => Self::Timeout,
            _ => Self::Unavailable,
        }
    }
}

pub(crate) type Fix = Result<Position, GeolocationError>;

/// Resolves to the one fix asked for.
pub(crate) type Locate = Pin<Box<dyn Future<Output = Fix>>>;

/// Dropping it clears the watch.
pub(crate) trait GeolocationSubscription {}

pub(crate) trait GeolocationApi {
    fn current(&self, options: GeolocationOptions) -> Locate;
    /// Calls `callback` per fix or error until dropped.
    fn watch(
        &self,
        options: GeolocationOptions,
        callback: Box<dyn Fn(Fix)>,
    ) -> Box<dyn GeolocationSubscription>;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Function, Object, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;

    use super::{
        Fix, GeolocationApi, GeolocationError, GeolocationOptions, GeolocationSubscription, Locate,
        Position,
    };

    /// `navigator.geolocation`, by name: web-sys's binding changes shape under
    /// `web_sys_unstable_apis`, which a consumer may set.
    pub(super) fn geolocation() -> Option<Object> {
        let navigator = web_sys::window()?.navigator();
        Reflect::get(&navigator, &JsValue::from_str("geolocation"))
            .ok()?
            .dyn_into::<Object>()
            .ok()
    }

    fn method(target: &Object, name: &str) -> Option<Function> {
        Reflect::get(target, &JsValue::from_str(name))
            .ok()?
            .dyn_into::<Function>()
            .ok()
    }

    fn number(target: &JsValue, name: &str) -> Option<f64> {
        Reflect::get(target, &JsValue::from_str(name))
            .ok()?
            .as_f64()
    }

    fn position(value: &JsValue) -> Fix {
        let coords = Reflect::get(value, &JsValue::from_str("coords"))
            .map_err(|_| GeolocationError::Unavailable)?;
        let required = |name| number(&coords, name).ok_or(GeolocationError::Unavailable);
        Ok(Position {
            latitude: required("latitude")?,
            longitude: required("longitude")?,
            accuracy: required("accuracy")?,
            altitude: number(&coords, "altitude"),
            altitude_accuracy: number(&coords, "altitudeAccuracy"),
            heading: number(&coords, "heading").filter(|heading| heading.is_finite()),
            speed: number(&coords, "speed"),
            timestamp_ms: number(value, "timestamp").unwrap_or_default(),
        })
    }

    fn error(value: &JsValue) -> GeolocationError {
        number(value, "code").map_or(GeolocationError::Unavailable, |code| {
            GeolocationError::from_code(code as u16)
        })
    }

    fn options(options: GeolocationOptions) -> Object {
        let object = Object::new();
        let _ = Reflect::set(
            &object,
            &"enableHighAccuracy".into(),
            &options.high_accuracy.into(),
        );
        if let Some(timeout) = options.timeout {
            let _ = Reflect::set(
                &object,
                &"timeout".into(),
                &(timeout.as_millis() as f64).into(),
            );
        }
        let _ = Reflect::set(
            &object,
            &"maximumAge".into(),
            &(options.max_age.as_millis() as f64).into(),
        );
        object
    }

    pub(super) struct WebGeolocation;

    impl GeolocationApi for WebGeolocation {
        fn current(&self, request: GeolocationOptions) -> Locate {
            let geolocation = geolocation();
            Box::pin(async move {
                let geolocation = geolocation.ok_or(GeolocationError::Unsupported)?;
                let get = method(&geolocation, "getCurrentPosition")
                    .ok_or(GeolocationError::Unsupported)?;
                let promise = Promise::new(&mut |resolve, reject| {
                    if get
                        .call3(&geolocation, &resolve, &reject, &options(request))
                        .is_err()
                    {
                        let _ = reject.call0(&JsValue::NULL);
                    }
                });
                match JsFuture::from(promise).await {
                    Ok(value) => position(&value),
                    Err(value) => Err(error(&value)),
                }
            })
        }

        fn watch(
            &self,
            request: GeolocationOptions,
            callback: Box<dyn Fn(Fix)>,
        ) -> Box<dyn GeolocationSubscription> {
            let callback = std::rc::Rc::new(callback);
            let on_fix = callback.clone();
            let success = Closure::<dyn Fn(JsValue)>::new(move |value| on_fix(position(&value)));
            let failure =
                Closure::<dyn Fn(JsValue)>::new(move |value| callback(Err(error(&value))));
            let id = geolocation().and_then(|geolocation| {
                let watch = method(&geolocation, "watchPosition")?;
                let id = watch
                    .call3(
                        &geolocation,
                        success.as_ref(),
                        failure.as_ref(),
                        &options(request),
                    )
                    .ok()?;
                Some((geolocation, id))
            });
            Box::new(WebWatch {
                id,
                _closures: (success, failure),
            })
        }
    }

    type Handler = Closure<dyn Fn(JsValue)>;

    struct WebWatch {
        id: Option<(Object, JsValue)>,
        _closures: (Handler, Handler),
    }

    impl GeolocationSubscription for WebWatch {}

    impl Drop for WebWatch {
        fn drop(&mut self) {
            if let Some((geolocation, id)) = &self.id
                && let Some(clear) = method(geolocation, "clearWatch")
            {
                let _ = clear.call1(geolocation, id);
            }
        }
    }

    pub(super) static GEOLOCATION: WebGeolocation = WebGeolocation;
}

/// `None` without a Geolocation API: Blitz, a server, a browser without one.
/// Call it after mount: a server render answers `None`, a hydrating client may not.
pub(crate) fn geolocation() -> Option<&'static dyn GeolocationApi> {
    #[cfg(target_arch = "wasm32")]
    return web::geolocation().map(|_| &web::GEOLOCATION as &'static dyn GeolocationApi);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_geolocation();
}
