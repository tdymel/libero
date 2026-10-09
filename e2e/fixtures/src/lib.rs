//! Fixture app for the E2E suite: one route per fixture, the smallest consumer of one component.
//! Not the docs pages, whose controls and prose caused false failures (see `codebase/testing`).
//!
//! One module per `e2e/tests/all/<unit>.rs` with a `pub const ROUTES`; `build.rs` declares it.
//! Size fixtures to the content, never the viewport (todo 296).
//!
//! A lib plus a bin (todo 822): `main.rs` serves the web, the native backend mounts [`route`] in Blitz.

// The shared helpers of the modules left out are unused then.
#![cfg_attr(fixtures_only, allow(dead_code, unused_imports))]

mod common;
pub mod docs_shell;
pub mod home;
#[cfg(all(feature = "desktop", target_os = "macos"))]
mod mac_input;
pub mod perf;

// Every other `src/*.rs`, or only the `E2E_FIXTURES` ones (todo 1618).
include!(concat!(env!("OUT_DIR"), "/fixtures.rs"));

use dioxus::prelude::*;
use libero::LiberoProvider;

/// A module's fixtures: each path, and the page rendered at it.
type Routes = &'static [(&'static str, fn() -> Element)];

/// The hand-declared modules' `ROUTES`; [`GENERATED`] has the rest. Paths must be unique.
const FIXTURES: &[Routes] = &[docs_shell::ROUTES, home::ROUTES, perf::ROUTES];

/// The page registered at `path`, without the [`Fixture`] wrapper.
pub fn route(path: &str) -> Option<fn() -> Element> {
    let mut hits = FIXTURES
        .iter()
        .chain(GENERATED)
        .flat_map(|routes| routes.iter())
        .filter(|(p, _)| *p == path);
    let page = hits.next().map(|(_, page)| *page);
    debug_assert!(hits.next().is_none(), "two fixtures claim {path}");
    page
}

#[component]
pub fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[layout(Shell)]
    #[route("/")]
    Index {},
    #[route("/:..segments")]
    Page { segments: Vec<String> },
}

/// Which navigation [`Fixture`] was mounted for; a new one remounts it, so a
/// route opened twice in one app starts fresh. Always 0 on the web.
#[derive(Clone, Copy)]
struct Generation(Signal<u64>);

#[component]
fn Shell() -> Element {
    let generation = use_context_provider(|| Generation(Signal::new(0)));
    #[cfg(any(target_os = "android", feature = "desktop"))]
    route_hook(generation.0);
    #[cfg(any(feature = "mobile", feature = "desktop"))]
    held_clock_hook();
    // The desktop WebView has no DevTools socket to navigate through.
    #[cfg(feature = "desktop")]
    {
        let navigator = navigator();
        use_hook(|| {
            if let Ok(path) = std::env::var("E2E_ROUTE") {
                navigator.push(path);
            }
        });
        desktop_bridge();
    }
    let _ = generation;
    rsx! { Outlet::<Route> {} }
}

/// wry keeps routes in memory, so the e2e Android (964) and desktop (1126) drivers
/// navigate by `window.__route(path)`, which returns the generation to wait for.
#[cfg(any(target_os = "android", feature = "desktop"))]
fn route_hook(mut generation: Signal<u64>) {
    let navigator = navigator();
    use_future(move || async move {
        let mut hook = document::eval(
            "let generation = 0;
             window.__route = (path) => { generation += 1; dioxus.send([path, generation]); return generation; };
             await new Promise(() => {});",
        );
        while let Ok((path, next)) = hook.recv::<(String, u64)>().await {
            // A fresh fixture starts on the real clock, as a fresh page does on the web.
            #[cfg(any(feature = "mobile", feature = "desktop"))]
            libero::platform::held_clock::reset();
            navigator.push(path);
            generation.set(next);
        }
    });
}

/// A WebView's timers are libero's thread timers, which no page script holds: the drivers
/// reach `libero::platform::held_clock` through `await window.__heldClock.call(op, [ms..])`,
/// `op` one of `hold`, `armed` and `fire` (2144).
#[cfg(any(feature = "mobile", feature = "desktop"))]
fn held_clock_hook() {
    use libero::platform::held_clock;
    use_future(|| async {
        let mut hook = document::eval(
            "let seq = 0;
             const pending = new Map();
             window.__heldClock = {
                 call: (op, ms) => new Promise((done) => {
                     seq += 1;
                     pending.set(seq, done);
                     dioxus.send([seq, op, ms]);
                 }),
             };
             while (true) {
                 const [at, answer] = await dioxus.recv();
                 pending.get(at)?.(answer);
                 pending.delete(at);
             }",
        );
        while let Ok((seq, op, ms)) = hook.recv::<(u64, String, Vec<u32>)>().await {
            let first = ms.first().copied().unwrap_or_default();
            let answer = match op.as_str() {
                "hold" => {
                    held_clock::hold_timers(&ms);
                    1
                }
                "armed" => held_clock::armed(first),
                "fire" => held_clock::fire(first),
                _ => 0,
            };
            let _ = hook.send((seq, answer));
        }
    });
}

/// The e2e desktop driver (1126) reads the page through `E2E_BRIDGE`, a loopback
/// TCP address: one JSON string of a JS body per line in, `{"ok": ..}` or `{"err": ..}` out.
/// On macOS a JSON object line is an input op instead, in order with the bodies (2782).
#[cfg(feature = "desktop")]
fn desktop_bridge() {
    use futures_util::StreamExt;
    use std::io::{BufRead, BufReader, Write};

    type Request = (String, std::sync::mpsc::Sender<String>);
    use_future(|| async {
        let Ok(address) = std::env::var("E2E_BRIDGE") else {
            return;
        };
        let (requests, mut incoming) = futures_channel::mpsc::unbounded::<Request>();
        // Blocking socket on its own thread: no tokio reactor is assumed here.
        std::thread::spawn(move || {
            let Ok(stream) = std::net::TcpStream::connect(&address) else {
                return;
            };
            let Ok(mut out) = stream.try_clone() else {
                return;
            };
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let (reply, answer) = std::sync::mpsc::channel();
                if requests.unbounded_send((line, reply)).is_err() {
                    return;
                }
                let Ok(answer) = answer.recv() else { return };
                if writeln!(out, "{answer}").is_err() {
                    return;
                }
            }
        });
        // One long-lived eval: a fresh one per body at times failed with
        // `EvalError::Finished` right after a navigation (1126).
        let mut channel = document::eval(
            "const run = (async () => {}).constructor;
             while (true) {
                 const body = await dioxus.recv();
                 try { dioxus.send({ ok: (await run(body)()) ?? null }); }
                 catch (e) { dioxus.send({ err: String(e) }); }
             }",
        );
        while let Some((line, reply)) = incoming.next().await {
            let answer = match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(serde_json::Value::String(body)) => match channel.send(body) {
                    Ok(()) => match channel.recv::<serde_json::Value>().await {
                        Ok(answer) => answer,
                        Err(error) => serde_json::json!({ "err": error.to_string() }),
                    },
                    Err(error) => serde_json::json!({ "err": error.to_string() }),
                },
                #[cfg(target_os = "macos")]
                Ok(op @ serde_json::Value::Object(_)) => {
                    use dioxus::desktop::wry::WebViewExtMacOS;
                    let view = dioxus::desktop::window().webview.webview();
                    match mac_input::run(&view, &op) {
                        Ok(value) => serde_json::json!({ "ok": value }),
                        Err(error) => serde_json::json!({ "err": error }),
                    }
                }
                Ok(other) => serde_json::json!({ "err": format!("not a JS body: {other}") }),
                Err(error) => serde_json::json!({ "err": error.to_string() }),
            };
            let _ = reply.send(answer.to_string());
        }
    });
}

#[component]
fn Index() -> Element {
    rsx! {
        Fixture { "libero e2e fixtures" }
    }
}

/// The fixture registered at this path. An unknown path renders without the
/// ready marker, so the harness times out on it rather than testing nothing.
#[component]
fn Page(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));
    let Some(page) = route(&path) else {
        let left_out = LEFT_OUT.iter().find(|(p, _)| *p == path);
        return match (ONLY, left_out) {
            (Some(only), Some((_, module))) => rsx! {
                "No fixture at {path}: its module `{module}` is not in E2E_FIXTURES={only}. "
                "Add it to the unit's EXTRA_FIXTURES in e2e/src/units.rs."
            },
            _ => rsx! { "No fixture at {path}" },
        };
    };

    rsx! {
        Fixture { {page()} }
    }
}

/// Wraps every fixture in the provider and the ready marker, nothing else: `dx` serves a 404
/// placeholder at a success status during its first build, so the harness waits for the marker.
// Marker outside the provider: axe is scoped to it, and portals render beside the provider's children.
#[component]
fn Fixture(children: Element) -> Element {
    let generation = try_use_context::<Generation>().map_or(0, |g| (g.0)());
    rsx! {
        for generation in [generation] {
            div {
                key: "{generation}",
                "data-fixture-ready": "true",
                "data-fixture-generation": "{generation}",
                LiberoProvider {
                    div { padding: "24px", {children.clone()} }
                }
            }
        }
    }
}
