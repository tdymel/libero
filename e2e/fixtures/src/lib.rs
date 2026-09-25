//! Fixture app for the E2E suite: one route per fixture, the smallest consumer of one component.
//! Not the docs pages, whose controls and prose caused false failures (see `codebase/testing`).
//!
//! One module per `e2e/tests/all/<unit>.rs` with a `pub const ROUTES`, plus its line in [`FIXTURES`].
//! Size fixtures to the content, never the viewport (todo 296).
//!
//! A lib plus a bin (todo 822): `main.rs` serves the web, the native backend mounts [`route`] in Blitz.

mod accordion;
mod action_icon;
mod alert;
mod audio;
mod autocomplete;
mod avatar;
mod badge;
mod button;
mod button_group;
mod calendar;
mod carousel;
mod cascader;
mod checkbox;
mod chip;
mod chrono_field;
mod code;
mod collapse;
mod color_field;
mod color_picker;
mod combobox;
mod common;
mod copy_button;
mod data_list;
mod dialog;
mod direction_toggle;
mod divider;
pub mod docs_shell;
mod drawer;
mod dropdown_parts;
mod editor_ime_probe;
mod editor_probe;
mod elevation;
mod field_frame;
mod field_parts;
mod field_value;
mod file_field;
mod floating_window;
mod flows;
mod focus_contrast;
mod focus_return;
mod focus_start;
mod focus_trap;
mod form;
mod gradient;
mod grid_zone;
mod header;
mod hit_area;
pub mod home;
mod hover_card;
mod icon;
mod icon_provider;
mod image;
mod image_cropper;
mod image_list;
mod kanban;
mod layout;
mod lightbox;
mod loader;
mod long_labels;
mod mark;
mod marquee;
mod menu;
mod menubar;
mod modal;
mod multi_select;
mod native_select;
mod nav_link;
mod negative;
mod nested_provider;
mod notifications;
mod number_field;
mod pagination;
pub mod perf;
mod phone_field;
mod picker_dialog;
mod picker_parts;
mod pin_field;
mod planted;
mod popover;
mod progress_bar;
mod qr_code;
mod radio_group;
mod range_slider;
mod rating;
mod refused;
mod repo_button;
mod rich_text_editor;
mod scroll_area;
mod scroller;
mod segmented_control;
mod select;
mod shortcut_help;
mod skeleton;
mod slider;
mod sortable;
mod splitter;
mod spotlight;
mod stepper;
mod switch;
mod table;
mod tabs;
mod tags_field;
mod text_field;
mod textarea;
mod theme_toggle;
mod time_picker;
mod timeline;
mod tldr;
mod toolbar;
mod tooltip;
mod trailing_button;
mod transition;
mod tree;
mod typography;
mod use_accessibility;
mod use_geolocation;
mod use_hotkeys;
mod use_intersection;
mod use_long_press;
mod use_media_query;
mod use_system_notification;
mod use_timers;
mod use_user_media;
mod visually_hidden;

use dioxus::prelude::*;
use libero::LiberoProvider;

/// A module's fixtures: each path, and the page rendered at it.
type Routes = &'static [(&'static str, fn() -> Element)];

/// Every module's `ROUTES`. Paths must be unique across them.
const FIXTURES: &[Routes] = &[
    accordion::ROUTES,
    action_icon::ROUTES,
    alert::ROUTES,
    audio::ROUTES,
    autocomplete::ROUTES,
    avatar::ROUTES,
    badge::ROUTES,
    button::ROUTES,
    button_group::ROUTES,
    calendar::ROUTES,
    carousel::ROUTES,
    cascader::ROUTES,
    checkbox::ROUTES,
    chip::ROUTES,
    code::ROUTES,
    collapse::ROUTES,
    color_field::ROUTES,
    color_picker::ROUTES,
    combobox::ROUTES,
    copy_button::ROUTES,
    data_list::ROUTES,
    chrono_field::ROUTES,
    dialog::ROUTES,
    direction_toggle::ROUTES,
    divider::ROUTES,
    docs_shell::ROUTES,
    drawer::ROUTES,
    dropdown_parts::ROUTES,
    editor_ime_probe::ROUTES,
    editor_probe::ROUTES,
    elevation::ROUTES,
    gradient::ROUTES,
    field_frame::ROUTES,
    field_parts::ROUTES,
    field_value::ROUTES,
    file_field::ROUTES,
    image_cropper::ROUTES,
    floating_window::ROUTES,
    flows::ROUTES,
    focus_contrast::ROUTES,
    focus_return::ROUTES,
    focus_start::ROUTES,
    focus_trap::ROUTES,
    form::ROUTES,
    grid_zone::ROUTES,
    header::ROUTES,
    hit_area::ROUTES,
    home::ROUTES,
    hover_card::ROUTES,
    icon::ROUTES,
    icon_provider::ROUTES,
    image::ROUTES,
    image_list::ROUTES,
    kanban::ROUTES,
    layout::ROUTES,
    lightbox::ROUTES,
    loader::ROUTES,
    long_labels::ROUTES,
    mark::ROUTES,
    marquee::ROUTES,
    menu::ROUTES,
    menubar::ROUTES,
    modal::ROUTES,
    multi_select::ROUTES,
    native_select::ROUTES,
    nav_link::ROUTES,
    negative::ROUTES,
    nested_provider::ROUTES,
    notifications::ROUTES,
    number_field::ROUTES,
    pagination::ROUTES,
    perf::ROUTES,
    phone_field::ROUTES,
    picker_dialog::ROUTES,
    picker_parts::ROUTES,
    pin_field::ROUTES,
    planted::ROUTES,
    popover::ROUTES,
    progress_bar::ROUTES,
    qr_code::ROUTES,
    radio_group::ROUTES,
    range_slider::ROUTES,
    rating::ROUTES,
    refused::ROUTES,
    repo_button::ROUTES,
    rich_text_editor::ROUTES,
    scroll_area::ROUTES,
    scroller::ROUTES,
    segmented_control::ROUTES,
    select::ROUTES,
    shortcut_help::ROUTES,
    skeleton::ROUTES,
    slider::ROUTES,
    sortable::ROUTES,
    spotlight::ROUTES,
    splitter::ROUTES,
    stepper::ROUTES,
    switch::ROUTES,
    table::ROUTES,
    tabs::ROUTES,
    tags_field::ROUTES,
    text_field::ROUTES,
    textarea::ROUTES,
    theme_toggle::ROUTES,
    time_picker::ROUTES,
    timeline::ROUTES,
    tldr::ROUTES,
    toolbar::ROUTES,
    tooltip::ROUTES,
    trailing_button::ROUTES,
    transition::ROUTES,
    tree::ROUTES,
    typography::ROUTES,
    use_accessibility::ROUTES,
    use_geolocation::ROUTES,
    use_hotkeys::ROUTES,
    use_intersection::ROUTES,
    use_long_press::ROUTES,
    use_media_query::ROUTES,
    use_system_notification::ROUTES,
    use_timers::ROUTES,
    use_user_media::ROUTES,
    visually_hidden::ROUTES,
];

/// The page registered at `path`, without the [`Fixture`] wrapper.
pub fn route(path: &str) -> Option<fn() -> Element> {
    let mut hits = FIXTURES
        .iter()
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
            navigator.push(path);
            generation.set(next);
        }
    });
}

/// The e2e desktop driver (1126) reads the page through `E2E_BRIDGE`, a loopback
/// TCP address: one JSON string of a JS body per line in, `{"ok": ..}` or `{"err": ..}` out.
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
            let answer = match serde_json::from_str::<String>(&line) {
                Ok(body) => match channel.send(body) {
                    Ok(()) => match channel.recv::<serde_json::Value>().await {
                        Ok(answer) => answer,
                        Err(error) => serde_json::json!({ "err": error.to_string() }),
                    },
                    Err(error) => serde_json::json!({ "err": error.to_string() }),
                },
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
        return rsx! { "No fixture at {path}" };
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
