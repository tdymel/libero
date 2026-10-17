use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::{Value, json};

use super::{Slot, WebViewListener, eval_with, runs_scripts};
use crate::platform::geolocation::{
    Fix, GeolocationApi, GeolocationError, GeolocationOptions, GeolocationSubscription, Locate,
    Position,
};

pub(crate) fn geolocation() -> Option<&'static dyn GeolocationApi> {
    runs_scripts().then_some(&GEOLOCATION as &'static dyn GeolocationApi)
}

struct WebViewGeolocation;

static GEOLOCATION: WebViewGeolocation = WebViewGeolocation;

/// `send` gets a fix's fields, or `{ error: code }`; code 0 = no Geolocation API.
const GEOLOCATION_SEND: &str = "const send = (answer) => dioxus.send(answer);
    const fix = (p) => send({
        latitude: p.coords.latitude, longitude: p.coords.longitude, accuracy: p.coords.accuracy,
        altitude: p.coords.altitude, altitude_accuracy: p.coords.altitudeAccuracy,
        heading: Number.isFinite(p.coords.heading) ? p.coords.heading : null,
        speed: p.coords.speed, timestamp_ms: p.timestamp,
    });
    const fail = (error) => send({ error: error?.code ?? 2 });
    const geolocation = navigator.geolocation;
    if (!geolocation) send({ error: 0 });";

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum GeolocationAnswer {
    Fix {
        latitude: f64,
        longitude: f64,
        accuracy: f64,
        altitude: Option<f64>,
        altitude_accuracy: Option<f64>,
        heading: Option<f64>,
        speed: Option<f64>,
        timestamp_ms: f64,
    },
    Error {
        error: u16,
    },
}

impl From<GeolocationAnswer> for Fix {
    fn from(answer: GeolocationAnswer) -> Self {
        match answer {
            GeolocationAnswer::Fix {
                latitude,
                longitude,
                accuracy,
                altitude,
                altitude_accuracy,
                heading,
                speed,
                timestamp_ms,
            } => Ok(Position {
                latitude,
                longitude,
                accuracy,
                altitude,
                altitude_accuracy,
                heading,
                speed,
                timestamp_ms,
            }),
            GeolocationAnswer::Error { error: 0 } => Err(GeolocationError::Unsupported),
            GeolocationAnswer::Error { error } => Err(GeolocationError::from_code(error)),
        }
    }
}

fn position_options(options: GeolocationOptions) -> Value {
    let mut value = json!({
        "enableHighAccuracy": options.high_accuracy,
        "maximumAge": options.max_age.as_millis() as u64,
    });
    if let Some(timeout) = options.timeout {
        value["timeout"] = json!(timeout.as_millis() as u64);
    }
    value
}

impl GeolocationApi for WebViewGeolocation {
    /// Answers by `send` from a parked script, not `join`: a prompt can outlast
    /// [`CHANNEL_HOLD_MS`], and the parked script keeps its channel alive.
    fn current(&self, options: GeolocationOptions) -> Locate {
        let slot = Slot::new();
        let script = eval_with(
            position_options(options),
            &format!(
                "{GEOLOCATION_SEND}
                else geolocation.getCurrentPosition(fix, fail, data);
                {}",
                slot.park("")
            ),
        );
        Box::pin(async move {
            let mut script = script;
            let answer = script.recv::<GeolocationAnswer>().await;
            drop(slot);
            answer.map_or(Err(GeolocationError::Unsupported), Fix::from)
        })
    }

    fn watch(
        &self,
        options: GeolocationOptions,
        callback: Box<dyn Fn(Fix)>,
    ) -> Box<dyn GeolocationSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            position_options(options),
            &format!(
                "{GEOLOCATION_SEND}
                const id = geolocation?.watchPosition(fix, fail, data);
                {}
                if (id !== undefined) geolocation.clearWatch(id);",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(answer) = script.recv::<GeolocationAnswer>().await {
                callback(answer.into());
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl GeolocationSubscription for WebViewListener {}
