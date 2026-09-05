//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use crate::common::{attributes_of, body};
use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Box, Button, Checkbox, Dialog, FileField, Form, Marquee, MultiSelect, Options,
        PinField, Rule, SegmentedControl, Tabs, TagsField, TextField, not_empty, use_form,
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
    first_click: Option<ElementId>,
    input: Option<ElementId>,
    keydown: Vec<ElementId>,
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
            self.first_click = self.first_click.or(self.last);
        }
        if name == "input" {
            self.input = self.last;
        }
        if name == "keydown" {
            self.keydown.extend(self.last);
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
    fn convert_form_data(&self, event: &PlatformEventData) -> dioxus::html::FormData {
        dioxus::html::FormData::new(
            event
                .downcast::<FakeInput>()
                .expect("not a FakeInput")
                .clone(),
        )
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
    fn convert_keyboard_data(&self, event: &PlatformEventData) -> dioxus::html::KeyboardData {
        dioxus::html::KeyboardData::new(event.downcast::<FakeKey>().expect("not a FakeKey").clone())
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

/// A stand-in for the renderer's `input` payload: the text the control now
/// holds, which is all any field reads off it.
#[derive(Clone)]
struct FakeInput(String);

impl dioxus::html::HasFormData for FakeInput {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl dioxus::html::HasFileData for FakeInput {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}

/// A stand-in for the renderer's key payload. Only the key itself is ever
/// read; nothing under test looks at the code or the modifiers.
#[derive(Clone)]
struct FakeKey(Key);

impl dioxus::html::HasKeyboardData for FakeKey {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> Code {
        Code::Unidentified
    }
    fn location(&self) -> dioxus::html::input_data::keyboard_types::Location {
        dioxus::html::input_data::keyboard_types::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl dioxus::html::point_interaction::ModifiersInteraction for FakeKey {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}

fn key_event(key: Key) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeKey(key))))
}

fn input_event(text: &str) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeInput(
        text.to_string(),
    ))))
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

/// Todo 250: `FileField`'s chips are the shared removable chip, guard
/// included. Its own copy had none, so a press focused the x, and removing
/// one of several chips dropped the focus to the body.
#[test]
fn pressing_a_file_chip_x_keeps_the_focus_on_the_control() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    multiple: true,
                    value: crate::common::fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
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

/// Every element that registered a `click` listener, and the one named
/// "Clear" - the shared `clear_button` of five fields.
#[derive(Default)]
struct FindClear {
    last: Option<ElementId>,
    clicks: Vec<ElementId>,
    clear: Option<ElementId>,
}

impl WriteMutations for FindClear {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        if name == "click" {
            self.clicks.extend(self.last);
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
    fn set_attribute(&mut self, name: &str, _ns: Option<&str>, value: &AttributeValue) {
        if name == "aria-label" && matches!(value, AttributeValue::Text(text) if text == "Clear") {
            self.clear = self.last;
        }
    }
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

thread_local! {
    /// What each `onchange` handed back, in order.
    static CLEARED: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Clicks the field's Clear button and returns what `onchange` received, as
/// value lengths. `None` when the field drew no Clear button.
///
/// Todo 252, for `b7d1c39d`: the shared `clear_button` renders only while it
/// has something to clear, and one click reports one change. Where focus goes
/// next is `ElementApi::focus` on the field's own control, which does nothing
/// without a renderer, so that half is still checked only in a browser.
fn click_clear(app: fn() -> Element) -> Option<Vec<usize>> {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    CLEARED.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClear::default();
    dom.rebuild(&mut find);
    let clear = find.clear?;
    assert!(find.clicks.contains(&clear), "Clear has no click listener");
    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), clear);
    dom.process_events();
    Some(CLEARED.with_borrow(Clone::clone))
}

#[test]
fn clear_empties_a_tags_field_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TagsField {
                    label: "Topics",
                    clearable: true,
                    value: vec!["rust".to_string(), "dioxus".to_string()],
                    onchange: move |tags: Vec<String>| CLEARED.with_borrow_mut(|seen| seen.push(tags.len())),
                }
            }
        }
    }

    assert_eq!(click_clear(app), Some(vec![0]));
}

#[test]
fn clear_empties_a_file_field_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    multiple: true,
                    clearable: true,
                    value: crate::common::fake_files(&["a.txt", "b.txt"]),
                    onchange: move |files: libero::components::Files| {
                        CLEARED.with_borrow_mut(|seen| seen.push(files.len()));
                    },
                }
            }
        }
    }

    assert_eq!(click_clear(app), Some(vec![0]));
}

#[test]
fn an_empty_field_draws_no_clear_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TagsField {
                    label: "Topics",
                    clearable: true,
                    value: Vec::<String>::new(),
                    onchange: move |_: Vec<String>| {},
                }
            }
        }
    }

    assert_eq!(click_clear(app), None);
}

/// A field with rules, no value of its own and no place in a form's value used
/// to validate `T::default()` for ever, so inside a `Form` it cancelled every
/// submit however the user answered it (todo 185). The checkbox is the case
/// where the browser keeps no state either: what the user ticked lives only in
/// the field, so it is both what renders and what the rules judge.
#[test]
fn ticking_an_uncontrolled_checkbox_satisfies_its_own_rules() {
    fn app() -> Element {
        let handle = use_form();
        let valid = use_signal(|| true);

        rsx! {
            LiberoProvider {
                // Before the form, so the checkbox's label keeps the last
                // click listener - which is how the test addresses it.
                Button {
                    onclick: move |_| {
                        let mut valid = valid;
                        valid.set(handle.validate());
                    },
                    "Submit"
                }
                Form::<()> { form: handle,
                    Checkbox { label: "Agree", validate: (|on: &bool| *on).error("Tick it") }
                }
                "valid: {valid}"
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let submit = find.first_click.expect("registered no click listener");
    let checkbox = find.click.expect("registered no click listener");

    let click = |dom: &mut VirtualDom, id| {
        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), id);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    };

    click(&mut dom, submit);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: false"),
        "an unticked required checkbox must block the submit"
    );

    click(&mut dom, checkbox);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true]);

    click(&mut dom, submit);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: true"),
        "the rules must judge what was ticked, not the default"
    );
}

/// The same defect on the field the todo names first: a `TextField` with rules,
/// no `value` and no binding. The browser keeps its text, so only the rules
/// need what was typed - the `value` attribute stays off it.
#[test]
fn typing_into_an_uncontrolled_text_field_satisfies_its_own_rules() {
    fn app() -> Element {
        let handle = use_form();
        let valid = use_signal(|| true);

        rsx! {
            LiberoProvider {
                Button {
                    onclick: move |_| {
                        let mut valid = valid;
                        valid.set(handle.validate());
                    },
                    "Submit"
                }
                Form::<()> { form: handle,
                    TextField {
                        label: "Email",
                        validate: (|text: &String| not_empty(text)).error("Enter your email"),
                    }
                }
                "valid: {valid}"
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let submit = find.first_click.expect("registered no click listener");
    let field = find.input.expect("registered no input listener");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), submit);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: false"),
        "an empty required field must block the submit"
    );

    dom.runtime().handle_event(
        "input",
        Event::new(input_event("tom@libero.dev"), true),
        field,
    );
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let html = dioxus_ssr::render(&dom);
    assert!(
        !attributes_of(&body(&html), "input").contains_key("value"),
        "the field is still uncontrolled - the browser owns the text"
    );

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), submit);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert!(
        dioxus_ssr::render(&dom).contains("valid: true"),
        "the rules must judge what was typed, not the default"
    );
}

thread_local! {
    static PIN_EDITS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The native `readonly` stops typing into a cell, but Backspace and Delete
/// are answered by the field itself and used to clear one anyway - so a pin
/// that "can be read and copied, but not changed" could be erased (todo 209).
#[test]
fn a_read_only_pin_field_answers_no_key_that_edits() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PinField {
                    label: "Code",
                    length: 4,
                    value: "1234",
                    readonly: true,
                    oninput: move |next: String| {
                        PIN_EDITS.with_borrow_mut(|edits| edits.push(next));
                    },
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    PIN_EDITS.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let second = find.keydown[1];

    for key in [Key::Backspace, Key::Delete] {
        dom.runtime()
            .handle_event("keydown", Event::new(key_event(key), true), second);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }

    assert_eq!(PIN_EDITS.with_borrow(Clone::clone), Vec::<String>::new());
}

/// `Form` took its `value` store once, so a parent that handed over a
/// different one - another record in an edit view - kept fields reading and
/// writing the old store while the form's own rules read the new one (todo
/// 190). `Store`'s equality is its identity, so the swap is recognisable.
#[test]
fn swapping_the_forms_value_swaps_what_its_fields_read() {
    fn app() -> Element {
        let first = use_store(|| crate::validation::Order {
            name: "Tom".into(),
            ..Default::default()
        });
        let second = use_store(|| crate::validation::Order {
            name: "Ada".into(),
            ..Default::default()
        });
        let mut second_record = use_signal(|| false);

        rsx! {
            LiberoProvider {
                Button { onclick: move |_| second_record.toggle(), "Next record" }
                Form {
                    value: if second_record() { second } else { first },
                    TextField { name: crate::validation::Order::FIELDS.name() }
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let next = find.first_click.expect("registered no click listener");

    let value = |dom: &VirtualDom| {
        attributes_of(&body(&dioxus_ssr::render(dom)), "input")
            .get("value")
            .cloned()
    };

    assert_eq!(value(&dom).as_deref(), Some("Tom"));

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), next);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(value(&dom).as_deref(), Some("Ada"));
}

/// A read-only checkbox keeps its tab stop and its place in the post, so the
/// one thing that must not happen is the toggle - and the activation is ours,
/// not the browser's, so only Rust can refuse it (todo 209).
#[test]
fn a_read_only_checkbox_refuses_its_own_activation() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { label: "Agree", readonly: true }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let label = find.click.expect("registered no click listener");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), label);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false]);
}
