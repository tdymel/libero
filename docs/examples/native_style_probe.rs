//! Reads Blitz's computed styles headlessly - no window, no clicking - so a
//! "does Blitz do X?" question can be measured instead of argued about. Built
//! to catch the stale painted transform (see
//! `writeups/blitz-stale-painted-transform.md`); left in as the harness to
//! extend for the next one.
//!
//! cargo run -p docs --example native_style_probe --features native-vello

use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native::{DioxusDocument, DocumentConfig};
use libero::{
    LiberoProvider,
    components::{Chip, Options, SegmentedControl, Slider, SliderChangeEvent, Switch},
    sx::sx,
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Pick {
    One,
    Two,
}

/// Raw transforms, no libero involved - which forms does Blitz keep?
#[component]
fn Raw() -> Element {
    rsx! {
        div { id: "literal", style: "transform: translate(10px, -50%)" }
        div { id: "calc", style: "transform: translate(calc(1 * 10px), -50%)" }
        div { id: "var", style: "--x: 1; transform: translate(calc(var(--x) * 10px), -50%)" }
        div { id: "var-nested", style: "--x: 1; --w: 40px; transform: translate(calc(var(--x) * ((var(--w) - 10px - 2 * 2px))), -50%)" }
    }
}

#[component]
fn On() -> Element {
    rsx! { LiberoProvider { Switch { checked: true, onchange: move |_| {} } } }
}

#[component]
fn Off() -> Element {
    rsx! { LiberoProvider { Switch { checked: false, onchange: move |_| {} } } }
}

fn dump(label: &str, app: fn() -> Element) {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(app),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            // `dioxus_native::launch` sets this; without it every
            // `dangerous_inner_html` is silently dropped and libero's
            // stylesheets arrive empty.
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    for _ in 0..3 {
        blitz_dom::Document::poll(&mut doc, None);
        doc.inner.borrow_mut().resolve(0.0);
    }

    println!("\n=== {label} ===");
    doc.inner.borrow().visit(|_id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        let Some(styles) = node.primary_styles() else {
            return;
        };
        let transform = &styles.get_box().transform;
        let id = element
            .attrs
            .iter()
            .find(|a| *a.name.local == *"id")
            .map(|a| a.value.clone())
            .unwrap_or_default();
        let name = element.name.local.to_string();
        if name == "style" {
            let text = node.text_content();
            println!("<style> {} bytes: {}", text.len(), &text.chars().take(160).collect::<String>());
            return;
        }
        if name == "head" || name == "html" {
            return;
        }
        println!(
            "<{name}{}> position: {:?} transform: {transform:?}",
            if id.is_empty() { String::new() } else { format!(" #{id}") },
            styles.get_box().position,
        );
    });
}

static TOGGLE: GlobalSignal<bool> = Global::new(|| false);

#[component]
fn Live() -> Element {
    rsx! {
        LiberoProvider {
            Switch { checked: TOGGLE(), onchange: move |v| *TOGGLE.write() = v }
        }
    }
}

/// The question the static dumps can't answer: when `checked` flips at
/// runtime, does the thumb - a descendant inheriting `--lsx-switch-on` from
/// the root's inline style - get restyled?
fn live() {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(Live),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    blitz_dom::Document::poll(&mut doc, None);
    doc.inner.borrow_mut().resolve(0.0);
    println!("\n=== live toggle ===");
    println!("before:        {}", thumb(&doc));

    doc.vdom.in_runtime(|| *TOGGLE.write() = true);
    blitz_dom::Document::poll(&mut doc, None);
    doc.inner.borrow_mut().resolve(0.0);
    println!("after (t=0):   {}", thumb(&doc));

    // If a `transition` is mid-flight, advancing the animation clock finishes it.
    for time in [0.05, 0.1, 0.2, 1.0] {
        doc.inner.borrow_mut().resolve(time);
        println!("after (t={time}): {}", thumb(&doc));
    }
}

/// The innermost `<span>` - the thumb.
fn thumb(doc: &DioxusDocument) -> String {
    let mut last = String::from("<not found>");
    doc.inner.borrow().visit(|_id, node| {
        let Some(element) = node.element_data() else {
            return;
        };
        if element.name.local.to_string() != "span" {
            return;
        }
        if let Some(styles) = node.primary_styles() {
            // Computed style vs the cached Affine the painter actually uses.
            last = format!(
                "computed {:?} | painted {:?}",
                styles.get_box().transform,
                node.transform()
            );
        }
    });
    last
}

static SLID: GlobalSignal<f64> = Global::new(|| 0.0);
static PICKED: GlobalSignal<Pick> = Global::new(|| Pick::One);
static CHIPPED: GlobalSignal<bool> = Global::new(|| false);

#[component]
fn Controls() -> Element {
    rsx! {
        LiberoProvider {
            SegmentedControl {
                value: PICKED(),
                onchange: move |v: Pick| *PICKED.write() = v,
            }
            Slider {
                aria_label: "probe",
                value: SLID(),
                on_change: move |event: SliderChangeEvent<f64>| {
                    *SLID.write() = event.value()
                },
                sx: sx().width("400px"),
            }
            Chip {
                checked: CHIPPED(),
                onchange: move |v| *CHIPPED.write() = v,
                "pick me"
            }
        }
    }
}

/// Click the centre of the first node matching `tag` whose text contains
/// `text`, the way the shell would: a real pointer down/up pair.
fn click(doc: &mut DioxusDocument, tag: &str, text: &str) {
    let target = {
        let inner = doc.inner.borrow();
        let mut found = None;
        inner.visit(|id, node| {
            if found.is_some() {
                return;
            }
            let Some(element) = node.element_data() else {
                return;
            };
            if element.name.local.to_string() == tag && node.text_content().contains(text) {
                found = inner.get_client_bounding_rect(id);
            }
        });
        found
    };
    let Some(rect) = target else {
        println!("  no <{tag}> containing {text:?}");
        return;
    };
    let (x, y) = (
        (rect.x + rect.width / 2.0) as f32,
        (rect.y + rect.height / 2.0) as f32,
    );
    send(doc, UiEvent::PointerDown(pointer(x, y, true)));
    send(doc, UiEvent::PointerUp(pointer(x, y, false)));
}

fn picked() -> &'static str {
    match PICKED() {
        Pick::One => "One",
        Pick::Two => "Two",
    }
}

/// Press at the track's left edge, move to three quarters, release - the
/// shape of a real drag, including the release.
fn drag(doc: &mut DioxusDocument) {
    let rect = {
        let inner = doc.inner.borrow();
        let mut found = None;
        inner.visit(|id, node| {
            if found.is_some() {
                return;
            }
            if node
                .element_data()
                .is_some_and(|e| e.attrs.iter().any(|a| *a.name.local == *"role"
                    && a.value == "slider"))
            {
                // The thumb carries the role; its parent chain has the track.
                found = node.parent.and_then(|p| inner.get_client_bounding_rect(p));
            }
        });
        found
    };
    let Some(rect) = rect else {
        println!("  no slider found");
        return;
    };
    let y = (rect.y + rect.height / 2.0) as f32;
    let start = (rect.x + 4.0) as f32;
    let end = (rect.x + rect.width * 0.75) as f32;
    send(doc, UiEvent::PointerDown(pointer(start, y, true)));
    send(doc, UiEvent::PointerMove(pointer(end, y, true)));
    send(doc, UiEvent::PointerUp(pointer(end, y, false)));
}

fn pointer(x: f32, y: f32, down: bool) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: if down {
            MouseEventButtons::Primary
        } else {
            MouseEventButtons::empty()
        },
        mods: Default::default(),
        details: PointerDetails::default(),
        element: Point { x: 0.0, y: 0.0 },
        active_pointers: Default::default(),
    }
}

fn send(doc: &mut DioxusDocument, event: UiEvent) {
    blitz_dom::Document::handle_ui_event(doc, event);
    blitz_dom::Document::poll(doc, None);
    doc.inner.borrow_mut().resolve(0.0);
}

/// Do the two label-wrapped controls actually receive a click?
fn controls() {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(Controls),
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(std::sync::Arc::new(blitz_html::HtmlProvider)),
            ..Default::default()
        },
    );
    doc.initial_build();
    blitz_dom::Document::poll(&mut doc, None);
    doc.inner.borrow_mut().resolve(0.0);

    println!("\n=== label-wrapped controls ===");
    let read = |doc: &DioxusDocument| doc.vdom.in_runtime(|| (picked(), CHIPPED()));

    println!("segmented before: {:?}", read(&doc));
    click(&mut doc, "label", "Two");
    println!("segmented after:  {:?}", read(&doc));

    click(&mut doc, "label", "pick me");
    println!("chip after:       {:?}", read(&doc));

    println!("\n=== slider drag ===");
    println!("before: {}", doc.vdom.in_runtime(|| SLID()));
    drag(&mut doc);
    println!("after:  {}", doc.vdom.in_runtime(|| SLID()));
}

fn main() {
    dump("raw transforms", Raw);
    dump("checked", On);
    dump("unchecked", Off);
    live();
    controls();
}
