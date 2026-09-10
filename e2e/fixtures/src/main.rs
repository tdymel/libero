//! Fixture app for the E2E suite.
//!
//! One route per fixture, each rendering a single component in the smallest
//! realistic consumer that exercises it. These are **not** the docs pages: a
//! docs page carries `Demo` controls, a code block printing the same strings
//! the preview renders, and prose that changes for reasons unrelated to the
//! component. Driving those produced fourteen false failures on 2026-09-14
//! (see `codebase/testing`), so the suite drives this instead.
//!
//! One module per `e2e/tests/all/<unit>.rs`, named after it, owning its routes
//! in a `pub const ROUTES: Routes`. A new fixture is its module, its `mod`
//! line and its line in [`FIXTURES`]; nothing else here changes. The router
//! sees one catch-all route and [`Page`] looks the path up, because a
//! `#[derive(Routable)]` enum cannot be split across modules.
//!
//! Two rules every fixture follows:
//!
//! * **Size to the content, never to the viewport.** A fixture that assumes a
//!   desktop width produces measurements about the fixture rather than about
//!   the component - todo 296 is that mistake made in the docs, where a 320px
//!   minimum in a 308px box shows a scrollbar at rest.
//! * **Carry the ready marker.** `dx` answers every path with a 404 placeholder
//!   *at a success status* while its first build runs, so a page that loaded
//!   proves nothing. `data-fixture-ready` is on an element only the real app
//!   renders, and the harness waits for it.

mod autocomplete;
mod button;
mod calendar;
mod carousel;
mod cascader;
mod code;
mod collapse;
mod color_picker;
mod color_scheme_button;
mod common;
mod drawer;
mod file_field;
mod floating_window;
mod focus_contrast;
mod hover_card;
mod image_list;
mod lightbox;
mod mark;
mod menu;
mod menubar;
mod modal;
mod multi_select;
mod negative;
mod notifications;
mod pagination;
mod picker_dialog;
mod planted;
mod radio_group;
mod scroll_area;
mod segmented_control;
mod select;
mod slider;
mod splitter;
mod spotlight;
mod stepper;
mod tabs;
mod tags_field;
mod tooltip;
mod tree;

use dioxus::prelude::*;
use libero::LiberoProvider;

/// A module's fixtures: each path, and the page rendered at it.
type Routes = &'static [(&'static str, fn() -> Element)];

/// Every module's `ROUTES`. Paths must be unique across them.
const FIXTURES: &[Routes] = &[
    autocomplete::ROUTES,
    button::ROUTES,
    calendar::ROUTES,
    carousel::ROUTES,
    cascader::ROUTES,
    code::ROUTES,
    collapse::ROUTES,
    color_picker::ROUTES,
    color_scheme_button::ROUTES,
    drawer::ROUTES,
    file_field::ROUTES,
    floating_window::ROUTES,
    focus_contrast::ROUTES,
    hover_card::ROUTES,
    image_list::ROUTES,
    lightbox::ROUTES,
    mark::ROUTES,
    menu::ROUTES,
    menubar::ROUTES,
    modal::ROUTES,
    multi_select::ROUTES,
    negative::ROUTES,
    notifications::ROUTES,
    pagination::ROUTES,
    picker_dialog::ROUTES,
    planted::ROUTES,
    radio_group::ROUTES,
    scroll_area::ROUTES,
    segmented_control::ROUTES,
    select::ROUTES,
    slider::ROUTES,
    spotlight::ROUTES,
    splitter::ROUTES,
    stepper::ROUTES,
    tabs::ROUTES,
    tags_field::ROUTES,
    tooltip::ROUTES,
    tree::ROUTES,
];

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[route("/")]
    Index {},
    #[route("/:..segments")]
    Page { segments: Vec<String> },
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
    let mut hits = FIXTURES
        .iter()
        .flat_map(|routes| routes.iter())
        .filter(|(p, _)| *p == path);
    let Some((_, page)) = hits.next() else {
        return rsx! { "No fixture at {path}" };
    };
    debug_assert!(hits.next().is_none(), "two fixtures claim {path}");

    rsx! {
        Fixture { {page()} }
    }
}

/// Wraps every fixture in the provider and the ready marker, and nothing else.
/// No shell, no navigation, no styling of its own - anything here would be
/// measured as though it were the component.
///
/// The marker goes **outside** the provider. `Suite` scopes axe and the
/// accessibility snapshot to it, and the provider renders its portal outlet
/// beside its children - so with the marker inside, every dialog, menu and
/// listbox was outside the scope. axe never saw one (a listbox planted with
/// `#ddd` text on white passed), and the modal's "open" baseline recorded the
/// trigger alone. Found 2026-09-19 by Olaf95.
#[component]
fn Fixture(children: Element) -> Element {
    rsx! {
        div { "data-fixture-ready": "true",
            LiberoProvider {
                div { padding: "24px", {children} }
            }
        }
    }
}
