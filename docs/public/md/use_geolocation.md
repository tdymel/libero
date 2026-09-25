# Geolocation

Crate: `libero`
Import: `use libero::hooks::{Geolocation, GeolocationError, GeolocationOptions, PermissionState, Position, use_geolocation};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/geolocation.rs>
Index: [index.md](index.md) lists every other page
Description: The device's position, once or followed, with the location permission; never prompts on mount.

`use_geolocation(options) -> Geolocation` reads the device's position.
`request()` asks for one fix, `watch()` follows it until `stop()`. Read
`position()`, `error()`, `permission()`, `is_pending()` and `is_watching()`;
all are reactive.

`accuracy` is a radius in metres: a fix from Wi-Fi or IP can be kilometres
wide. `high_accuracy` asks for GPS, slower and costlier on battery.

Web: a secure context (HTTPS or localhost). Android: declare
`[permissions] location` in Dioxus.toml; the system asks on the first request.
macOS and Windows WebViews are untested. Blitz and a server render have no
Geolocation API: `is_supported()` stays false and a request fails with
`Unsupported`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::{GeolocationError, GeolocationOptions, use_geolocation},
};

#[component]
fn ShareLocation() -> Element {
    let mut location = use_geolocation(GeolocationOptions::default());
    // Announced once per outcome, never per fix.
    let status = match (location.error(), location.position()) {
        (Some(GeolocationError::Denied), _) => "Location refused",
        (Some(_), _) => "No location found",
        (None, Some(_)) if location.is_watching() => "Following your location",
        (None, Some(_)) => "Location found",
        (None, None) if location.is_pending() => "Locating",
        (None, None) => "",
    };

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Flex { gap: "sm",
                Button { onclick: move |_| location.request(), "Share location" }
                Button {
                    variant: "outlined",
                    onclick: move |_| if location.is_watching() { location.stop() } else { location.watch() },
                    if location.is_watching() { "Stop following" } else { "Follow" }
                }
            }
            div { role: "status", "{status}" }
            if let Some(fix) = location.position() {
                Text { "{fix.latitude:.2}, {fix.longitude:.2}, within {fix.accuracy:.0} m" }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_geolocation(options: GeolocationOptions) -> Geolocation

pub struct GeolocationOptions {
    pub high_accuracy: bool,
    pub timeout: Option<Duration>,
    pub max_age: Duration,
}

pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
    pub timestamp_ms: f64,
}

pub enum GeolocationError { Unsupported, Denied, Unavailable, Timeout }
pub enum PermissionState { Granted, Denied, Prompt, Unknown, Unsupported }
```

| Option | Default | Description |
|---|---|---|
| `high_accuracy` | `false` | Asks for GPS-grade fixes; a hint the platform may ignore. |
| `timeout` | `None` | How long one fix may take, the prompt excluded; `None` waits forever. |
| `max_age` | `0` | How old a cached fix may be; zero always asks for a fresh one. |

| Method of `Geolocation` | Description |
|---|---|
| `request()` | Asks for one fix, prompting if the user has not answered. Does nothing while one is pending. |
| `watch()` / `stop()` | Follows the position; a denial and unmount stop it too. The last fix stays. |
| `position()` | The last fix, `None` before the first. |
| `error()` | Why the last attempt failed; the next fix clears it. `Denied` also covers an insecure context. |
| `permission()` | `Prompt`, `Granted` or `Denied` where the Permissions API answers and follows its changes; `Unknown` where it does not (the Android WebView) until a fix or denial; `Unsupported` without a Geolocation API. |
| `is_supported()` | `false` until mounted, then whether a Geolocation API exists. |
| `is_pending()`, `is_watching()` | A request awaits its answer; a watch runs. |

`Geolocation` is `Copy`. Options apply to the next `request` or `watch`.

| Platform | Support |
|---|---|
| Web | Full, in a secure context. |
| Android (WebView) | Full with `[permissions] location` in Dioxus.toml; the permission reads `Unknown` until a fix. |
| Linux desktop (WebKitGTK) | Every request is `Denied`. |
| macOS, Windows desktop | Untested. |
| Blitz, server render | `Unsupported`. |

## Accessibility

### Libero handles

- Mounting never prompts: the browser or OS asks only on request or watch,
  which you call from a user's action.
- A watch ends on stop, on a denial and on unmount, so the device's location
  indicator goes off with it.
- It announces nothing and logs no coordinates.

### You must

- Ask from a visible control whose label says what the location is for, never
  on page load.
- Announce the outcome once in a status region (found, refused, not found),
  never every fix, as the demo does.
- Show a visible "following your location" state while a watch runs, with a
  control that stops it.
- Give a refused user another way on, such as typing an address, and say how
  to re-enable location in the browser or system settings.
- Round coordinates you show, and keep them out of logs and storage unless the
  user agreed.

### Limits

- A denial is usually permanent for the site: the browser does not ask again,
  and libero cannot open its settings.
- The Linux desktop WebView (WebKitGTK) denies every request, because wry
  answers no permission request there.
