//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Box, Button, Dialog, Marquee, MultiSelect, Options, SegmentedControl, Tabs,
        TagsField,
    },
    hooks::{ModalScope, use_modal},
};
use std::rc::Rc;

/// Records the element the `click` listener landed on, which is the only way
/// to address it from a test - and every `mousedown` one, in order.
#[derive(Default)]
struct FindClickListener {
    last: Option<ElementId>,
    click: Option<ElementId>,
    mousedown: Vec<ElementId>,
}

impl WriteMutations for FindClickListener {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        if name == "click" {
            self.click = self.last;
        }
        if name == "mousedown" {
            self.mousedown.extend(self.last);
        }
    }
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

/// Renderers register one of these; without it every event conversion panics.
/// Only mouse events are needed here.
struct TestConverter;

impl dioxus::html::HtmlEventConverter for TestConverter {
    fn convert_mouse_data(&self, event: &PlatformEventData) -> dioxus::html::MouseData {
        dioxus::html::MouseData::new(FakeMouse::from(
            event.downcast::<FakeMouse>().expect("not a FakeMouse"),
        ))
    }
    fn convert_drag_data(&self, _event: &PlatformEventData) -> dioxus::html::DragData {
        unimplemented!()
    }
    fn convert_form_data(&self, _event: &PlatformEventData) -> dioxus::html::FormData {
        unimplemented!()
    }
    fn convert_mounted_data(&self, _event: &PlatformEventData) -> dioxus::html::MountedData {
        unimplemented!()
    }
    fn convert_animation_data(&self, _event: &PlatformEventData) -> dioxus::html::AnimationData {
        unimplemented!()
    }
    fn convert_before_input_data(
        &self,
        _event: &PlatformEventData,
    ) -> dioxus::html::BeforeInputData {
        unimplemented!()
    }
    fn convert_cancel_data(&self, _event: &PlatformEventData) -> dioxus::html::CancelData {
        unimplemented!()
    }
    fn convert_clipboard_data(&self, _event: &PlatformEventData) -> dioxus::html::ClipboardData {
        unimplemented!()
    }
    fn convert_composition_data(
        &self,
        _event: &PlatformEventData,
    ) -> dioxus::html::CompositionData {
        unimplemented!()
    }
    fn convert_focus_data(&self, _event: &PlatformEventData) -> dioxus::html::FocusData {
        unimplemented!()
    }
    fn convert_image_data(&self, _event: &PlatformEventData) -> dioxus::html::ImageData {
        unimplemented!()
    }
    fn convert_keyboard_data(&self, _event: &PlatformEventData) -> dioxus::html::KeyboardData {
        unimplemented!()
    }
    fn convert_media_data(&self, _event: &PlatformEventData) -> dioxus::html::MediaData {
        unimplemented!()
    }
    fn convert_pointer_data(&self, _event: &PlatformEventData) -> dioxus::html::PointerData {
        unimplemented!()
    }
    fn convert_resize_data(&self, _event: &PlatformEventData) -> dioxus::html::ResizeData {
        unimplemented!()
    }
    fn convert_scroll_data(&self, _event: &PlatformEventData) -> dioxus::html::ScrollData {
        unimplemented!()
    }
    fn convert_selection_data(&self, _event: &PlatformEventData) -> dioxus::html::SelectionData {
        unimplemented!()
    }
    fn convert_toggle_data(&self, _event: &PlatformEventData) -> dioxus::html::ToggleData {
        unimplemented!()
    }
    fn convert_touch_data(&self, _event: &PlatformEventData) -> dioxus::html::TouchData {
        unimplemented!()
    }
    fn convert_transition_data(&self, _event: &PlatformEventData) -> dioxus::html::TransitionData {
        unimplemented!()
    }
    fn convert_visible_data(&self, _event: &PlatformEventData) -> dioxus::html::VisibleData {
        unimplemented!()
    }
    fn convert_wheel_data(&self, _event: &PlatformEventData) -> dioxus::html::WheelData {
        unimplemented!()
    }
}

/// The web renderer's payload: a `PlatformEventData` wrapping the concrete data.
fn click_event() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeMouse)))
}

/// Empty where the component set no state at all - an unstyled `ActionIcon`
/// only grows the attribute once it has a ripple.
fn data_state(html: &str) -> String {
    html.split("data-state=\"")
        .nth(1)
        .unwrap_or("")
        .split('"')
        .next()
        .unwrap()
        .to_string()
}

fn button_app() -> Element {
    rsx! { LiberoProvider { Button { "Click" } } }
}

fn action_icon_app() -> Element {
    rsx! { LiberoProvider { ActionIcon { aria_label: "Click", "x" } } }
}

#[test]
fn clicking_a_button_runs_its_ripple() {
    assert_ripple_alternates(button_app);
}

#[test]
fn clicking_an_action_icon_runs_its_ripple() {
    assert_ripple_alternates(action_icon_app);
}

/// The control is strictly controlled and the radio is cancelled on click, so
/// what is checked after a click comes from Rust alone - never from the flip
/// the browser did during activation.
#[test]
fn clicking_a_segment_moves_the_selection() {
    fn app() -> Element {
        let mut value = use_signal(|| Emphasis::Bold);

        rsx! {
            LiberoProvider {
                SegmentedControl {
                    value: value(),
                    onchange: move |next| value.set(next),
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // The last listener registered, i.e. the second segment.
    let italic = find.click.expect("registered no click listener");

    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true, false]);

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), italic);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false, true]);

    // Exactly one is always selected, so clicking it again is a no-op rather
    // than a deselect - that is the whole difference from a row of toggles.
    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), italic);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false, true]);
}

#[derive(Clone, PartialEq, Options)]
enum Emphasis {
    Bold,
    Italic,
}

fn checked_states(html: &str) -> Vec<bool> {
    html.match_indices("<input")
        .map(|(at, _)| {
            let tag = &html[at..at + html[at..].find('>').expect("an unterminated tag")];
            tag.contains("checked")
        })
        .collect()
}

#[derive(Clone, PartialEq, Options)]
enum Pane {
    First,
    Second,
}

#[test]
fn clicking_a_tab_selects_it_and_swaps_the_panel() {
    fn app() -> Element {
        let mut pane = use_signal(|| Pane::First);

        rsx! {
            LiberoProvider {
                Tabs {
                    value: pane(),
                    onchange: move |next| pane.set(next),
                    panel: |pane: Pane| match pane {
                        Pane::First => rsx! { "first body" },
                        Pane::Second => rsx! { "second body" },
                    },
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // The last listener registered, i.e. the second tab.
    let second = find.click.expect("registered no click listener");

    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("first body"));
    assert!(!html.contains("second body"));

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), second);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    // Only the newly selected panel is in the tree - the other is not hidden,
    // it is gone.
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("second body"));
    assert!(!html.contains("first body"));
    assert_eq!(markup(&html).matches("aria-selected=\"true\"").count(), 1);
}

/// Everything but the `<style>` blocks, which carry selectors as CSS text.
fn markup(html: &str) -> String {
    let mut rest = html;
    let mut out = String::new();
    while let Some(start) = rest.find("<style") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("</style>") {
            Some(end) => &rest[start + end + "</style>".len()..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

fn assert_ripple_alternates(app: fn() -> Element) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let button = find.click.expect("registered no click listener");

    assert!(!data_state(&dioxus_ssr::render(&dom)).contains("ripple"));

    // Alternating names is what replays the CSS animation on a repeat click.
    for expected in ["ripple-a", "ripple-b", "ripple-a"] {
        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), button);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert!(
            data_state(&dioxus_ssr::render(&dom)).contains(expected),
            "expected {expected}"
        );
    }
}

// A stand-in for the browser's mouse event, since the real one only exists
// behind the `serialize` feature.
#[derive(Clone, Copy)]
struct FakeMouse;

impl From<&FakeMouse> for FakeMouse {
    fn from(value: &FakeMouse) -> Self {
        *value
    }
}

use dioxus::html::HasMouseData;
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::point_interaction::{
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
};

impl InteractionLocation for FakeMouse {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(10.0, 10.0)
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(10.0, 10.0)
    }
    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(10.0, 10.0)
    }
}
impl InteractionElementOffset for FakeMouse {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(7.0, 3.0)
    }
}
impl ModifiersInteraction for FakeMouse {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}
impl PointerInteraction for FakeMouse {
    fn trigger_button(&self) -> Option<MouseButton> {
        Some(MouseButton::Primary)
    }
    fn held_buttons(&self) -> MouseButtonSet {
        Default::default()
    }
}
impl HasMouseData for FakeMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn resolving_from_inside_a_modal_settles_its_opening() {
    #[component]
    fn Opener(outcome: Signal<Option<Option<bool>>>) -> Element {
        let modal = use_modal(|s: ModalScope<(), bool>| {
            rsx! {
                Dialog { title: "Delete?",
                    Button { onclick: move |_| s.resolve(true), "Yes" }
                }
            }
        });
        use_hook(move || {
            let mut outcome = outcome;
            modal
                .open()
                .on_result(move |result| outcome.set(Some(result)));
        });

        rsx! {}
    }

    fn app() -> Element {
        let outcome = use_signal(|| None);

        rsx! {
            LiberoProvider { Opener { outcome } }
            "{outcome:?}"
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // `use_hook` opens after `use_modal` has already read the empty slot, so
    // the dialog only exists from the second render on.
    dom.render_immediate(&mut find);
    // The last click listener in the dialog, i.e. the confirm button - the
    // close button in the header registered before it.
    let confirm = find.click.expect("registered no click listener");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), confirm);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Some(Some(true))"), "got {html}");
    assert!(
        !html.contains("Delete?"),
        "the modal should be gone: {html}"
    );
}

/// Every listener name registered during a rebuild, in order.
#[derive(Default)]
struct Listeners(Vec<String>);

impl WriteMutations for Listeners {
    fn add_event_listener(&mut self, name: &str) {
        self.0.push(name.to_string());
    }
    fn push_id(&mut self, _id: ElementId) {}
    fn set_id(&mut self, _id: ElementId) {}
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

/// What lets a field declare no `onfocus`, `onblur` or `onkeydown` and still
/// accept one: `base_props!` extends `GlobalAttributes`, which carries
/// listeners and not only attributes. `Button` declares `onclick` for its own
/// ripple reasons, so nothing else pins this down.
#[test]
fn global_attributes_carry_event_listeners_through_the_spread() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { oninput: move |_| {}, onfocus: move |_| {}, "x" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    let mut listeners = Listeners::default();
    dom.rebuild(&mut listeners);

    for name in ["input", "focus"] {
        assert!(
            listeners.0.iter().any(|listener| listener == name),
            "no {name} listener: {:?}",
            listeners.0
        );
    }
}

/// Clicks the only listener `app` registers - `Marquee`'s pause toggle - and
/// returns the markup before and after.
fn click_the_pause_toggle(app: fn() -> Element) -> (String, String) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let toggle = find.click.expect("registered no click listener");
    let before = dioxus_ssr::render(&dom);
    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), toggle);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    (before, dioxus_ssr::render(&dom))
}

fn marquee_paused(html: &str) -> (bool, bool) {
    (
        data_state(html).split(' ').any(|token| token == "paused"),
        html.contains(r#"aria-pressed="true""#),
    )
}

/// Controlled: the toggle only reports, and the strip follows the caller.
#[test]
fn a_controlled_marquee_follows_the_caller() {
    fn app() -> Element {
        let mut paused = use_signal(|| false);
        rsx! {
            LiberoProvider {
                Marquee { paused: paused(), onpausechange: move |next| paused.set(next), "x" }
            }
        }
    }

    let (before, after) = click_the_pause_toggle(app);
    assert_eq!(marquee_paused(&before), (false, false), "{before}");
    assert_eq!(marquee_paused(&after), (true, true), "{after}");
}

/// A caller who holds `paused` and ignores the report keeps it running: the
/// toggle has no state of its own to flip.
#[test]
fn a_controlled_marquee_ignores_its_own_toggle() {
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Marquee {
                    paused: false,
                    onpausechange: move |next: bool| {
                        assert!(next, "asked for the opposite of false");
                        CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    },
                    "x"
                }
            }
        }
    }

    let (_, after) = click_the_pause_toggle(app);
    assert_eq!(CALLS.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(marquee_paused(&after), (false, false), "{after}");
}

/// Uncontrolled: the toggle holds the state itself.
#[test]
fn an_uncontrolled_marquee_pauses_on_its_toggle() {
    fn app() -> Element {
        rsx! { LiberoProvider { Marquee { "x" } } }
    }

    let (_, after) = click_the_pause_toggle(app);
    assert_eq!(marquee_paused(&after), (true, true), "{after}");
}

/// A press on a chip's x must not take the focus. The field holds the focus
/// that keeps its list open, and the x is about to be removed: focused, it
/// would take the focus down with it, to the body. Nothing but this guard
/// stops that, so nothing but this test notices it gone (todo 77).
fn assert_every_press_keeps_the_focus(app: fn() -> Element, chips: usize) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // Closed, the field has no other `mousedown` listener: the dropdown's own
    // guard only mounts with the dropdown.
    assert_eq!(find.mousedown.len(), chips, "one guard per chip");

    for chip in find.mousedown {
        let press = Event::new(click_event(), true);
        dom.runtime().handle_event("mousedown", press.clone(), chip);
        assert!(!press.default_action_enabled(), "a press moved the focus");
    }
}

#[test]
fn pressing_a_tag_x_keeps_the_focus_on_the_input() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TagsField {
                    label: "Topics",
                    value: vec!["rust".to_string(), "dioxus".to_string()],
                    onchange: move |_: Vec<String>| {},
                }
            }
        }
    }

    assert_every_press_keeps_the_focus(app, 2);
}

#[test]
fn pressing_a_chip_x_keeps_the_focus_on_the_trigger() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                MultiSelect::<Emphasis> {
                    label: "Emphasis",
                    value: vec![Emphasis::Bold, Emphasis::Italic],
                    onchange: move |_: Vec<Emphasis>| {},
                }
            }
        }
    }

    assert_every_press_keeps_the_focus(app, 2);
}
