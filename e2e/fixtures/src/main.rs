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

mod accordion;
mod action_icon;
mod alert;
mod autocomplete;
mod avatar;
mod badge;
mod button;
mod calendar;
mod carousel;
mod cascader;
mod checkbox;
mod chip;
mod code;
mod collapse;
mod color_field;
mod color_picker;
mod color_scheme_button;
mod combobox;
mod common;
mod data_list;
mod date_field;
mod dialog;
mod divider;
mod docs_shell;
mod drawer;
mod elevation;
mod field_frame;
mod field_value;
mod file_field;
mod floating_window;
mod focus_contrast;
mod focus_return;
mod focus_trap;
mod form;
mod grid_zone;
mod header;
mod hit_area;
mod home;
mod hover_card;
mod icon;
mod image;
mod image_list;
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
mod notifications;
mod number_field;
mod pagination;
mod phone_field;
mod picker_dialog;
mod pin_field;
mod planted;
mod popover;
mod progress_bar;
mod qr_code;
mod radio_group;
mod range_slider;
mod scroll_area;
mod scroller;
mod segmented_control;
mod select;
mod skeleton;
mod slider;
mod splitter;
mod spotlight;
mod stepper;
mod switch;
mod table;
mod tabs;
mod tags_field;
mod text_field;
mod textarea;
mod time_picker;
mod timeline;
mod tooltip;
mod trailing_button;
mod tree;
mod typography;
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
    autocomplete::ROUTES,
    avatar::ROUTES,
    badge::ROUTES,
    button::ROUTES,
    calendar::ROUTES,
    carousel::ROUTES,
    cascader::ROUTES,
    checkbox::ROUTES,
    chip::ROUTES,
    code::ROUTES,
    collapse::ROUTES,
    color_field::ROUTES,
    color_picker::ROUTES,
    color_scheme_button::ROUTES,
    combobox::ROUTES,
    data_list::ROUTES,
    date_field::ROUTES,
    dialog::ROUTES,
    divider::ROUTES,
    docs_shell::ROUTES,
    drawer::ROUTES,
    elevation::ROUTES,
    field_frame::ROUTES,
    field_value::ROUTES,
    file_field::ROUTES,
    floating_window::ROUTES,
    focus_contrast::ROUTES,
    focus_return::ROUTES,
    focus_trap::ROUTES,
    form::ROUTES,
    grid_zone::ROUTES,
    header::ROUTES,
    hit_area::ROUTES,
    home::ROUTES,
    hover_card::ROUTES,
    icon::ROUTES,
    image::ROUTES,
    image_list::ROUTES,
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
    notifications::ROUTES,
    number_field::ROUTES,
    pagination::ROUTES,
    phone_field::ROUTES,
    picker_dialog::ROUTES,
    pin_field::ROUTES,
    planted::ROUTES,
    popover::ROUTES,
    progress_bar::ROUTES,
    qr_code::ROUTES,
    radio_group::ROUTES,
    range_slider::ROUTES,
    scroll_area::ROUTES,
    scroller::ROUTES,
    segmented_control::ROUTES,
    select::ROUTES,
    skeleton::ROUTES,
    slider::ROUTES,
    spotlight::ROUTES,
    splitter::ROUTES,
    stepper::ROUTES,
    switch::ROUTES,
    table::ROUTES,
    tabs::ROUTES,
    tags_field::ROUTES,
    text_field::ROUTES,
    textarea::ROUTES,
    time_picker::ROUTES,
    timeline::ROUTES,
    tooltip::ROUTES,
    trailing_button::ROUTES,
    tree::ROUTES,
    typography::ROUTES,
    visually_hidden::ROUTES,
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
