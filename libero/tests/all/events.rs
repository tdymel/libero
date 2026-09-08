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
        ActionIcon, Box, Button, Checkbox, Chip, Collapse, ColorCode, ColorField, Dialog,
        FileField, Form, Marquee, Menu, MenuItem, MultiSelect, NativeSelect, NumberField,
        OptionList, Options, PhoneField, PinField, RadioGroup, RangeSlider, Rule, SegmentedControl,
        SelectionArgs, Slider, SliderChangeEvent, Tabs, TagsField, TextField, not_empty, use_form,
        use_menu,
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
    change: Option<ElementId>,
    keydown: Vec<ElementId>,
    mousedown: Vec<ElementId>,
    blur: Vec<ElementId>,
    transitionend: Vec<ElementId>,
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
        if name == "change" {
            self.change = self.last;
        }
        if name == "keydown" {
            self.keydown.extend(self.last);
        }
        if name == "mousedown" {
            self.mousedown.extend(self.last);
        }
        if name == "blur" {
            self.blur.extend(self.last);
        }
        if name == "transitionend" {
            self.transitionend.extend(self.last);
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
        dioxus::html::FocusData::new(FakeFocus)
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
    fn convert_transition_data(&self, event: &PlatformEventData) -> dioxus::html::TransitionData {
        let fake = event
            .downcast::<FakeTransition>()
            .expect("not a FakeTransition");
        // Wrapped the way dioxus-desktop wraps it, so the WebView arm of
        // `platform::transition_property` reads the property.
        let serialized = dioxus::html::SerializedTransitionData::from(
            &dioxus::html::TransitionData::new(fake.clone()),
        );
        dioxus::html::TransitionData::new(serialized)
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

/// A stand-in for the renderer's focus payload, which carries nothing read.
struct FakeFocus;

impl dioxus::html::HasFocusData for FakeFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A stand-in for the renderer's `transitionend` payload: the property that
/// finished, which is all the presence hook reads.
#[derive(Clone)]
struct FakeTransition(&'static str);

impl dioxus::html::HasTransitionData for FakeTransition {
    fn property_name(&self) -> String {
        self.0.to_string()
    }
    fn pseudo_element(&self) -> String {
        String::new()
    }
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn transition_end_event(property: &'static str) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeTransition(property))))
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
                .onresult(move |result| outcome.set(Some(result)));
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
///
/// `guards` counts every `mousedown` listener the closed field registers,
/// which since todo 70 is more than one per chip: `TagsField`'s tag wrapper
/// and `FileField`'s chip wrapper guard a caller's own chip too, and `MultiSelect`'s value slot and chevron
/// guard the trigger's focus.
fn assert_every_press_keeps_the_focus(app: fn() -> Element, guards: usize) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // Closed, the field has no other `mousedown` listener: the dropdown's own
    // guard only mounts with the dropdown.
    assert_eq!(
        find.mousedown.len(),
        guards,
        "a guard went missing, or a listener joined"
    );

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

    // The default chip's guard, and its wrapper's.
    assert_every_press_keeps_the_focus(app, 4);
}

/// Todo 70 (c): a caller's own `tag` draws its x without any guard, and the
/// field's wrapper supplies it - two chips, two guards, both cancelling.
#[test]
fn a_custom_tag_gets_the_guard_it_did_not_draw() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TagsField {
                    label: "Topics",
                    value: vec!["rust".to_string(), "dioxus".to_string()],
                    onchange: move |_: Vec<String>| {},
                    tag: move |args: SelectionArgs<String>| rsx! {
                        span { "{args.value}"
                            button { tabindex: "-1", onclick: move |_| args.remove.call(()), "x" }
                        }
                    },
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

    // The default chip's guard, and its wrapper's.
    assert_every_press_keeps_the_focus(app, 4);
}

/// A caller's own `selection` draws its x without any guard, and the chip's
/// wrapper supplies it - two chips, two guards, both cancelling.
#[test]
fn a_custom_file_chip_gets_the_guard_it_did_not_draw() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    multiple: true,
                    value: crate::common::fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
                    selection: move |args: SelectionArgs<dioxus::html::FileData>| rsx! {
                        span { "{args.value.name()}"
                            button { tabindex: "-1", onclick: move |_| args.remove.call(()), "x" }
                        }
                    },
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

    // One per chip, the value slot's and the chevron's.
    assert_every_press_keeps_the_focus(app, 4);
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

/// Toggling a mixed checkbox gives `true`, as its prop doc promises: a mixed
/// "select all" fills its group rather than clearing it (todo 412).
#[test]
fn clicking_an_indeterminate_checkbox_reports_true() {
    fn app() -> Element {
        let reported = use_signal(Vec::<bool>::new);
        rsx! {
            LiberoProvider {
                Checkbox {
                    label: "Select all",
                    checked: false,
                    indeterminate: true,
                    onchange: move |on: bool| {
                        let mut reported = reported;
                        reported.write().push(on);
                    },
                }
                "reported: {reported():?}"
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let checkbox = find.click.expect("registered no click listener");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), checkbox);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let html = dioxus_ssr::render(&dom);
    let reported = html
        .split("reported: ")
        .nth(1)
        .and_then(|rest| rest.split('<').next());
    assert_eq!(
        reported,
        Some("[true]"),
        "a mixed checkbox toggles to `true`"
    );
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

thread_local! {
    /// Whether the next `readonly_*` app mounts read-only. The apps are plain
    /// `fn`s, so the same one serves as its own positive control.
    static READ_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// What the field's handler heard, one entry per call.
    static HEARD: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn heard(what: impl std::fmt::Debug) {
    HEARD.with_borrow_mut(|seen| seen.push(format!("{what:?}")));
}

/// Mounts `app` with `readonly` as given, sends `name` with `data` to the
/// element `pick` chooses, and returns what the handler heard and the markup
/// afterwards. Run once each way, so a refusal is only believed next to the
/// same event being answered.
fn send(
    app: fn() -> Element,
    readonly: bool,
    name: &str,
    pick: fn(&FindClickListener) -> ElementId,
    data: fn() -> Rc<dyn std::any::Any>,
) -> (Vec<String>, String) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    READ_ONLY.set(readonly);
    HEARD.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let target = pick(&find);
    dom.runtime()
        .handle_event(name, Event::new(data(), true), target);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    (HEARD.with_borrow(Clone::clone), dioxus_ssr::render(&dom))
}

fn first_keydown(find: &FindClickListener) -> ElementId {
    find.keydown[0]
}

fn last_keydown(find: &FindClickListener) -> ElementId {
    *find.keydown.last().expect("registered no keydown listener")
}

fn last_click(find: &FindClickListener) -> ElementId {
    find.click.expect("registered no click listener")
}

fn input_listener(find: &FindClickListener) -> ElementId {
    find.input.expect("registered no input listener")
}

fn readonly_slider() -> Element {
    rsx! {
        LiberoProvider {
            Slider {
                aria_label: "Volume",
                value: 40.0,
                readonly: READ_ONLY.get(),
                oninput: move |event: SliderChangeEvent<f64>| heard(event),
            }
        }
    }
}

fn readonly_range_slider() -> Element {
    rsx! {
        LiberoProvider {
            RangeSlider {
                aria_label_from: "Minimum",
                aria_label_to: "Maximum",
                value: (20.0, 80.0),
                readonly: READ_ONLY.get(),
                oninput: move |event: SliderChangeEvent<(f64, f64)>| heard(event),
            }
        }
    }
}

/// Todo 306: a read-only thumb keeps its tab stop and says it is read-only,
/// and no key moves it. Each key is checked against the same slider editable,
/// where it must move.
#[test]
fn a_read_only_slider_answers_no_key() {
    for app in [readonly_slider, readonly_range_slider] {
        for key in [
            Key::ArrowRight,
            Key::ArrowLeft,
            Key::Home,
            Key::End,
            Key::PageUp,
            Key::PageDown,
        ] {
            let data = match key {
                Key::ArrowRight => || key_event(Key::ArrowRight),
                Key::ArrowLeft => || key_event(Key::ArrowLeft),
                Key::Home => || key_event(Key::Home),
                Key::End => || key_event(Key::End),
                Key::PageUp => || key_event(Key::PageUp),
                _ => || key_event(Key::PageDown),
            };
            for pick in [first_keydown, last_keydown] {
                let (moved, _) = send(app, false, "keydown", pick, data);
                assert!(
                    !moved.is_empty(),
                    "{key:?} moved nothing on an editable slider"
                );
                let (heard, html) = send(app, true, "keydown", pick, data);
                assert_eq!(heard, Vec::<String>::new(), "{key:?}");

                let thumbs: Vec<_> = html.match_indices("role=\"slider\"").collect();
                assert!(!thumbs.is_empty());
                for (at, _) in thumbs {
                    let start = html[..at].rfind('<').unwrap();
                    let tag = &html[start..start + html[start..].find('>').unwrap()];
                    assert!(tag.contains("tabindex=\"0\""), "{tag}");
                    assert!(tag.contains("aria-readonly=\"true\""), "{tag}");
                    assert!(!tag.contains("aria-disabled"), "{tag}");
                }
            }
        }
    }
}

fn readonly_segments() -> Element {
    let mut value = use_signal(|| Emphasis::Bold);
    rsx! {
        LiberoProvider {
            SegmentedControl {
                label: "Emphasis",
                value: value(),
                readonly: READ_ONLY.get(),
                onchange: move |next| {
                    heard(next == Emphasis::Italic);
                    value.set(next);
                },
            }
        }
    }
}

/// Todo 306: the arrows move *and* select, so a read-only strip refuses them
/// outright, as `RadioGroup` does - and the click, Enter and Space with them.
#[test]
fn a_read_only_segmented_control_picks_nothing() {
    let (_, html) = send(readonly_segments, false, "keydown", first_keydown, || {
        key_event(Key::ArrowRight)
    });
    assert_eq!(
        checked_states(&html),
        [false, true],
        "the arrow is the control"
    );
    let (heard, html) = send(readonly_segments, true, "keydown", first_keydown, || {
        key_event(Key::ArrowRight)
    });
    assert_eq!(heard, Vec::<String>::new());
    assert_eq!(checked_states(&html), [true, false]);
    assert!(html.contains("role=\"radiogroup\""), "{html}");
    assert!(html.contains("aria-readonly=\"true\""), "{html}");

    let (_, html) = send(readonly_segments, false, "click", last_click, click_event);
    assert_eq!(
        checked_states(&html),
        [false, true],
        "the click is the control"
    );
    let (heard, html) = send(readonly_segments, true, "click", last_click, click_event);
    assert_eq!(heard, Vec::<String>::new());
    assert_eq!(checked_states(&html), [true, false]);

    for key in [
        || key_event(Key::Enter),
        || key_event(Key::Character(" ".into())),
    ] {
        let (picked, _) = send(readonly_segments, false, "keydown", last_keydown, key);
        assert_eq!(picked, ["true"], "the key is the control");
        let (heard, _) = send(readonly_segments, true, "keydown", last_keydown, key);
        assert_eq!(heard, Vec::<String>::new());
    }
}

fn readonly_number() -> Element {
    rsx! {
        LiberoProvider {
            NumberField {
                label: "Quantity",
                value: 3,
                steppers: true,
                readonly: READ_ONLY.get(),
                onchange: move |next: i32| heard(next),
            }
        }
    }
}

/// Todo 306 (review 3 C2): the native `readonly` stops typing, but the arrows
/// and the steppers are the field's own, so they are refused in Rust.
#[test]
fn a_read_only_number_field_does_not_step() {
    let arrow = || key_event(Key::ArrowUp);
    let (stepped, _) = send(readonly_number, false, "keydown", last_keydown, arrow);
    assert_eq!(stepped, ["4"], "the arrow is the control");
    let (heard, html) = send(readonly_number, true, "keydown", last_keydown, arrow);
    assert_eq!(heard, Vec::<String>::new());

    let input = &html[html.find("<input").unwrap()..];
    let input = &input[..input.find('>').unwrap()];
    assert!(input.contains("readonly=true"), "{input}");
    assert!(!input.contains("disabled"), "{input}");

    let (stepped, _) = send(readonly_number, false, "click", last_click, click_event);
    assert_eq!(stepped, ["4"], "the stepper is the control");
    let (heard, _) = send(readonly_number, true, "click", last_click, click_event);
    assert_eq!(heard, Vec::<String>::new());
}

fn readonly_color() -> Element {
    rsx! {
        LiberoProvider {
            ColorField {
                label: "Accent",
                value: "#ff0000".parse::<ColorCode>().unwrap(),
                readonly: READ_ONLY.get(),
                oninput: move |event: SliderChangeEvent<ColorCode>| {
                    heard(match event {
                        SliderChangeEvent::Start(_) => "Start",
                        SliderChangeEvent::Change(_) => "Change",
                        SliderChangeEvent::End(_) => "End",
                    })
                },
            }
        }
    }
}

/// Todo 306: the text takes the native `readonly`, and the dropdown - the
/// other editor - refuses to open, as every dropdown field's does.
#[test]
fn a_read_only_color_field_opens_no_dropdown() {
    let expanded = |html: &str| {
        attributes_of(&body(html), "input")
            .get("aria-expanded")
            .cloned()
    };
    let (_, html) = send(readonly_color, false, "click", last_click, click_event);
    assert_eq!(
        expanded(&html).as_deref(),
        Some("true"),
        "the click is the control"
    );
    let (_, html) = send(readonly_color, true, "click", last_click, click_event);
    assert_eq!(expanded(&html).as_deref(), Some("false"));
    let input = &html[html.find("<input").unwrap()..];
    assert!(
        input[..input.find('>').unwrap()].contains("readonly=true"),
        "{html}"
    );
}

/// Todo 291: typed text that parses is settled the moment it lands, so it
/// emits `Change` then `End`, like a key press or a swatch in the dropdown - a
/// caller committing on `End` used to miss every typed color.
#[test]
fn typing_a_color_emits_change_then_end() {
    let (heard, _) = send(readonly_color, false, "input", input_listener, || {
        input_event("#00ff00")
    });
    assert_eq!(heard, ["\"Change\"", "\"End\""]);
}

fn readonly_files() -> Element {
    rsx! {
        LiberoProvider {
            FileField {
                label: "Attachments",
                multiple: true,
                clearable: true,
                value: crate::common::fake_files(&["a.txt", "b.txt"]),
                readonly: READ_ONLY.get(),
                onchange: move |files: libero::components::Files| heard(files.len()),
            }
        }
    }
}

fn readonly_files_clear() -> Element {
    rsx! {
        LiberoProvider {
            FileField {
                label: "Attachments",
                multiple: true,
                clearable: true,
                value: crate::common::fake_files(&["a.txt", "b.txt"]),
                readonly: true,
                onchange: move |files: libero::components::Files| {
                    CLEARED.with_borrow_mut(|seen| seen.push(files.len()));
                },
            }
        }
    }
}

/// Todo 305: a read-only file field keeps its tab stop and its post - the
/// input is not `disabled` - while the keys that remove a chip or open the
/// picker, and the clear button, stand down.
#[test]
fn a_read_only_file_field_removes_nothing() {
    let backspace = || key_event(Key::Backspace);
    let (removed, _) = send(readonly_files, false, "keydown", first_keydown, backspace);
    assert_eq!(removed, ["1"], "Backspace is the control");
    let (heard, html) = send(readonly_files, true, "keydown", first_keydown, backspace);
    assert_eq!(heard, Vec::<String>::new());

    let tag_at = |html: &str, marker: &str| -> Vec<String> {
        html.match_indices(marker)
            .map(|(at, _)| {
                let start = html[..at].rfind('<').unwrap();
                html[start..start + html[start..].find('>').unwrap()].to_string()
            })
            .collect()
    };
    let surface = tag_at(&html, "role=\"button\"");
    assert_eq!(surface.len(), 1, "{surface:?}");
    assert!(surface[0].contains("tabindex=\"0\""), "{surface:?}");
    assert!(!surface[0].contains("aria-disabled"), "{surface:?}");
    let file = tag_at(&html, "type=\"file\"");
    assert!(
        !file[0].contains("disabled"),
        "a read-only field still posts: {file:?}"
    );
    // Both chips' x stand down with the field.
    let removes = tag_at(&html, "aria-label=\"Remove");
    assert_eq!(removes.len(), 2, "{removes:?}");
    assert!(
        removes.iter().all(|x| x.contains("disabled")),
        "{removes:?}"
    );

    assert_eq!(click_clear(readonly_files_clear), None);
}

fn readonly_radio_group() -> Element {
    rsx! {
        LiberoProvider {
            RadioGroup {
                label: "Emphasis",
                value: Some(Emphasis::Bold),
                readonly: READ_ONLY.get(),
                onchange: move |next: Emphasis| heard(next == Emphasis::Italic),
            }
        }
    }
}

/// Todo 320: a read-only group refused the arrows by returning early, and so
/// left the browser's own arrow in place - which moves focus to the next radio
/// and checks it natively. Measured in Chromium: the selection held, but focus
/// walked onto options whose `tabindex` is -1. The arrow has to be cancelled
/// either way; only whether it also picks depends on `readonly`.
#[test]
fn a_read_only_radio_group_cancels_the_native_arrow() {
    for readonly in [false, true] {
        dioxus::html::set_event_converter(Box::new(TestConverter));
        READ_ONLY.set(readonly);
        HEARD.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(readonly_radio_group);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);

        let arrow = Event::new(key_event(Key::ArrowDown), true);
        dom.runtime()
            .handle_event("keydown", arrow.clone(), last_keydown(&find));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert!(
            !arrow.default_action_enabled(),
            "readonly {readonly}: the browser's own arrow was left to run"
        );
        let expected: &[String] = if readonly { &[] } else { &["true".to_string()] };
        assert_eq!(
            HEARD.with_borrow(Clone::clone),
            expected,
            "readonly {readonly}"
        );
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Tier {
    Free,
    Pro,
    Team,
}

thread_local! {
    /// The selection and the disabled options `tier_group` mounts with.
    static TIER: std::cell::Cell<Option<Tier>> = const { std::cell::Cell::new(None) };
    static TIERS_OFF: std::cell::RefCell<Vec<Tier>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn tier_group() -> Element {
    rsx! {
        LiberoProvider {
            RadioGroup {
                label: "Tier",
                value: TIER.get(),
                options: OptionList::from_options()
                    .disabling(|tier| TIERS_OFF.with_borrow(|off| off.contains(tier))),
                onchange: move |next: Tier| heard(next),
            }
        }
    }
}

/// `(disabled, tabindex)` per radio input, in order.
fn radio_inputs(html: &str) -> Vec<(bool, String)> {
    html.match_indices("<input")
        .map(|(at, _)| {
            let tag = &html[at..at + html[at..].find('>').unwrap()];
            let tabindex = tag
                .split("tabindex=\"")
                .nth(1)
                .map(|rest| rest[..rest.find('"').unwrap()].to_string())
                .unwrap_or_default();
            (tag.contains(" disabled"), tabindex)
        })
        .collect()
}

/// Mounts `tier_group`, and returns each input's `(disabled, tabindex)` and
/// what one `key` press picked.
fn tier_press(value: Option<Tier>, off: &[Tier], key: Key) -> (Vec<(bool, String)>, Vec<String>) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    TIER.set(value);
    TIERS_OFF.set(off.to_vec());
    HEARD.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(tier_group);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let inputs = radio_inputs(&dioxus_ssr::render(&dom));
    dom.runtime().handle_event(
        "keydown",
        Event::new(key_event(key), true),
        last_keydown(&find),
    );
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    (inputs, HEARD.with_borrow(Clone::clone))
}

/// Todo 221: a disabled `OptionItem` greys out a single option, and the arrows
/// step over them, wrapping.
#[test]
fn a_radio_group_steps_over_the_options_its_list_disabled() {
    let (inputs, picked) = tier_press(Some(Tier::Free), &[Tier::Pro], Key::ArrowDown);
    let off: Vec<bool> = inputs.iter().map(|(off, _)| *off).collect();
    assert_eq!(off, [false, true, false], "{inputs:?}");
    assert_eq!(picked, ["Team"]);

    let (_, picked) = tier_press(Some(Tier::Team), &[Tier::Pro], Key::ArrowUp);
    assert_eq!(picked, ["Free"]);
    // Positive control: without the option disabled, the arrow lands on it.
    let (_, picked) = tier_press(Some(Tier::Free), &[], Key::ArrowDown);
    assert_eq!(picked, ["Pro"]);
}

/// A disabled input takes no focus, so a tab stop on one would drop the group
/// out of the Tab order. The stop moves to the first option that can be picked.
#[test]
fn a_radio_group_never_puts_its_tab_stop_on_a_disabled_option() {
    let stop = |inputs: &[(bool, String)]| {
        (0..inputs.len())
            .filter(|&i| inputs[i].1 == "0")
            .collect::<Vec<_>>()
    };
    let (inputs, _) = tier_press(None, &[Tier::Free], Key::Tab);
    assert_eq!(stop(&inputs), [1], "{inputs:?}");
    let (inputs, _) = tier_press(Some(Tier::Pro), &[Tier::Pro], Key::Tab);
    assert_eq!(stop(&inputs), [0], "{inputs:?}");
    let (inputs, _) = tier_press(Some(Tier::Pro), &[], Key::Tab);
    assert_eq!(stop(&inputs), [1], "{inputs:?}");
}

/// Two `keep_mounted: false` collapses, one inside the other, both closing.
/// The inner one is the faster, so its `transitionend` arrives first, and it
/// bubbles. Before todo 36d it reached the outer root too and unmounted the
/// outer content mid-close.
fn nested_collapse_app() -> Element {
    let open = use_context_provider(|| Signal::new(true));
    rsx! {
        LiberoProvider {
            Collapse { open: open(), keep_mounted: false, duration: 5000,
                "outer body"
                Collapse { open: open(), keep_mounted: false, duration: 5000, "inner body" }
            }
        }
    }
}

#[test]
fn an_inner_collapse_ending_its_exit_does_not_end_the_outer_one() {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(nested_collapse_app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // Registered parent first: the outer root is created before its children.
    let [_outer, inner] = find.transitionend[..] else {
        panic!(
            "expected two transitionend listeners, got {:?}",
            find.transitionend
        );
    };

    let mut open = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);
    dom.in_runtime(|| open.set(false));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let end = |dom: &mut VirtualDom, property| {
        dom.runtime().handle_event(
            "transitionend",
            Event::new(transition_end_event(property), true),
            inner,
        );
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    };

    // The content's own opacity is not the exit, for either collapse.
    end(&mut dom, "opacity");
    let html = body(&dioxus_ssr::render(&dom));
    assert!(
        html.contains("inner body"),
        "an opacity end unmounted: {html}"
    );

    end(&mut dom, "grid-template-rows");

    let html = body(&dioxus_ssr::render(&dom));
    assert!(
        !html.contains("inner body"),
        "the inner exit did not end: {html}"
    );
    assert!(
        html.contains("outer body"),
        "the inner exit ended the outer one: {html}"
    );
}

fn busy_action_icon() -> Element {
    rsx! {
        LiberoProvider {
            ActionIcon {
                aria_label: "Save",
                // The flag reads as `loading` here.
                loading: READ_ONLY.get(),
                onclick: move |_| heard("click"),
                "S"
            }
        }
    }
}

/// Todo 222: `ActionIcon` takes `Button`'s `loading` - the click is swallowed,
/// but the button keeps its tab stop (no native `disabled`) and says it is
/// busy. The same click on the idle icon is the control.
#[test]
fn a_loading_action_icon_swallows_the_click_and_stays_focusable() {
    let (clicked, _) = send(busy_action_icon, false, "click", last_click, click_event);
    assert_eq!(clicked, ["\"click\""], "the click is the control");
    let (heard, html) = send(busy_action_icon, true, "click", last_click, click_event);
    assert_eq!(heard, Vec::<String>::new());

    let button = attributes_of(&body(&html), "button");
    assert_eq!(
        button.get("aria-busy").map(String::as_str),
        Some("true"),
        "{button:?}"
    );
    assert_eq!(
        button.get("aria-disabled").map(String::as_str),
        Some("true")
    );
    assert!(
        button["data-state"]
            .split(' ')
            .any(|token| token == "loading")
    );
    let tag = &html[html.find("<button").unwrap()..];
    assert!(
        !tag[..tag.find('>').unwrap()].contains(" disabled"),
        "{tag}"
    );
}

/// Todo 222: a `name` makes a `Chip` a checkbox that posts, and one with no
/// handler keeps its own state - the way an unbound `Checkbox` does.
#[test]
fn a_named_chip_posts_and_toggles_on_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "open", "Open" }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);

    let html = dioxus_ssr::render(&dom);
    let input = attributes_of(&body(&html), "input");
    assert_eq!(input.get("type").map(String::as_str), Some("checkbox"));
    assert_eq!(input.get("name").map(String::as_str), Some("open"));
    assert_eq!(checked_states(&html), [false]);

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), last_click(&find));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true]);
}

#[derive(Clone, PartialEq, Default, libero::components::Fields)]
pub struct Filters {
    pub open: bool,
}

/// Todo 222: a path `name` binds a `Chip` to the `Form` around it, as on
/// `Checkbox` - the click writes the form's value, and the chip renders it.
#[test]
fn a_chip_with_a_path_name_writes_its_forms_value() {
    fn app() -> Element {
        let filters = use_store(Filters::default);
        rsx! {
            LiberoProvider {
                Form { value: filters,
                    Chip { name: Filters::FIELDS.open(), "Open" }
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false]);

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), last_click(&find));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(checked_states(&html), [true]);
    let input = attributes_of(&body(&html), "input");
    assert_eq!(input.get("name").map(String::as_str), Some("open"));
}

/// Each element's `keydown` and `focusout` listeners, and each menu item's
/// `data-menu-index`, so a test can address one level of a `Menu`.
#[derive(Default)]
struct MenuListeners {
    last: Option<ElementId>,
    keydown: Vec<ElementId>,
    focusout: Vec<ElementId>,
    items: Vec<(ElementId, String)>,
}

impl WriteMutations for MenuListeners {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        match name {
            "keydown" => self.keydown.extend(self.last),
            "focusout" => self.focusout.extend(self.last),
            _ => {}
        }
    }
    fn set_attribute(&mut self, name: &str, _ns: Option<&str>, value: &AttributeValue) {
        if let (Some(id), "data-menu-index", AttributeValue::Text(value)) = (self.last, name, value)
        {
            self.items.push((id, value.clone()));
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
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

fn submenu_app() -> Element {
    let menu = use_menu();
    use_hook(|| menu.open());
    rsx! {
        LiberoProvider {
            Menu {
                state: menu,
                items: vec![
                    MenuItem::new("Recent")
                        .submenu(vec![MenuItem::new("notes.md").onselect(|_| {}).into()])
                        .into(),
                ],
                Button { attributes: menu.a11y_attributes(), "File" }
            }
            "menus:{menu.is_open()}"
        }
    }
}

fn open_menus(html: &str) -> usize {
    html.matches("role=\"menu\"").count()
}

/// The root menu open with its submenu open on top, and what addresses them.
fn open_submenu() -> (VirtualDom, MenuListeners) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(submenu_app);
    let mut find = MenuListeners::default();
    dom.rebuild(&mut find);
    settle_menu(&mut dom, &mut find);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(open_menus(&html), 1, "the root menu opens");

    let recent = newest_first_item(&find);
    dom.runtime().handle_event(
        "keydown",
        Event::new(key_event(Key::ArrowRight), true),
        recent,
    );
    settle_menu(&mut dom, &mut find);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(open_menus(&html), 2, "ArrowRight opens the submenu");
    (dom, find)
}

/// The newest item at index 0: the root's before its submenu opens, the
/// submenu's after.
fn newest_first_item(find: &MenuListeners) -> ElementId {
    find.items
        .iter()
        .rev()
        .find(|(_, index)| index == "0")
        .expect("an item at index 0")
        .0
}

fn settle_menu(dom: &mut VirtualDom, find: &mut MenuListeners) {
    for _ in 0..4 {
        dom.process_events();
        dom.render_immediate(find);
    }
}

/// Focus leaving a submenu for somewhere outside the menu closes the whole
/// menu, not only the submenu (todo 322). Off the web no element ever counts
/// as focused, so every focusout here is focus leaving the whole tree - the
/// case that left the root open. Focus moving between levels, which must
/// close nothing, needs a real browser.
#[test]
fn focus_leaving_a_submenu_closes_the_whole_menu() {
    let (mut dom, mut find) = open_submenu();
    let submenu = *find.focusout.last().expect("the submenu's box listens");
    dom.runtime().handle_event(
        "focusout",
        Event::new(Rc::new(PlatformEventData::new(Box::new(FakeFocus))), true),
        submenu,
    );
    settle_menu(&mut dom, &mut find);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(open_menus(&html), 0, "no level stays open");
    assert!(html.contains("menus:false"), "the root's state closed too");
}

/// Escape is not focus leaving: it closes the submenu alone, as before.
#[test]
fn escape_in_a_submenu_closes_only_the_submenu() {
    let (mut dom, mut find) = open_submenu();
    let item = newest_first_item(&find);
    dom.runtime()
        .handle_event("keydown", Event::new(key_event(Key::Escape), true), item);
    settle_menu(&mut dom, &mut find);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(open_menus(&html), 1, "the root menu stays");
    assert!(html.contains("menus:true"));
}

/// A label that is not the variant's name, so a test can tell which of the
/// two a control posted.
#[derive(Clone, Copy, PartialEq, Debug, Options)]
pub enum Plan {
    #[option(label = "Free plan")]
    Free,
    #[option(label = "Pro plan")]
    Pro,
}

#[derive(Clone, PartialEq, Default, libero::components::Fields)]
pub struct Signup {
    pub plan: Option<Plan>,
}

/// Mounts `app`, sends `change` carrying `posted` to the select, and returns
/// what the handler heard and the markup afterwards.
fn change_select(app: fn() -> Element, posted: &str) -> (Vec<String>, String) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    HEARD.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let select = find.change.expect("registered no change listener");
    dom.runtime()
        .handle_event("change", Event::new(input_event(posted), true), select);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    (HEARD.with_borrow(Clone::clone), dioxus_ssr::render(&dom))
}

/// Todo 20: the `<select>` reports the chosen option's `value`, which is
/// `Options::value` now - so that is what `onchange` looks up. An index or
/// the visible label is no option's value any more, and picks nothing.
#[test]
fn a_native_select_reads_the_option_value_back() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: Plan::Free,
                    onchange: move |next: Plan| heard(next),
                }
            }
        }
    }

    assert_eq!(change_select(app, "Pro").0, ["Pro"]);
    assert_eq!(change_select(app, "1").0, Vec::<String>::new());
    assert_eq!(change_select(app, "Pro plan").0, Vec::<String>::new());
}

/// The same round trip without `onchange`: the path `name` writes the
/// `Form`'s value, and the select renders it back as selected.
#[test]
fn a_native_select_with_a_path_name_writes_its_forms_value() {
    fn app() -> Element {
        let signup = use_store(Signup::default);
        rsx! {
            LiberoProvider {
                Form { value: signup,
                    NativeSelect { name: Signup::FIELDS.plan(), placeholder: "Pick" }
                }
                "plan: {signup.read().plan:?}"
            }
        }
    }

    let (_, html) = change_select(app, "Pro");
    assert!(html.contains("plan: Some(Pro)"), "{html}");
    assert!(html.contains("<option value=\"Pro\" selected"), "{html}");
    let select = attributes_of(&body(&html), "select");
    assert_eq!(select.get("name").map(String::as_str), Some("plan"));
}

thread_local! {
    /// The country the phone field below is mounted on, and a prop that
    /// changes under it - the demo's controls in one flag.
    static PHONE_COUNTRY: std::cell::Cell<&'static str> = const { std::cell::Cell::new("DE") };
    static PHONE_TOGGLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn phone_app() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                name: "phone",
                country: PHONE_COUNTRY.get(),
                description: PHONE_TOGGLE.get().then(|| "Deliveries only".to_string()),
                value: value(),
                oninput: move |next: String| value.set(next),
            }
        }
    }
}

fn focus_event() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeFocus)))
}

/// The text the `tel` input is showing, which is not what it posts.
fn phone_text(html: &str) -> String {
    let body = body(html);
    let input = &body[body.find("<input").expect("no input")..];
    attributes_of(&input[..=input.find('>').expect("no input")], "input")
        .get("value")
        .cloned()
        .unwrap_or_default()
}

/// Todo 87b: typing `30 123456` into a German field and then leaving it - or
/// re-rendering it, which is what toggling a demo control does - used to bring
/// the number back as `30123456`. Germany has no one fixed grouping, so there
/// is nothing to regroup into and the text the user typed stays. The US, which
/// has one, still regroups on the way out.
#[test]
fn a_plan_without_a_fixed_shape_keeps_the_text_the_user_typed() {
    for (country, typed, after_blur) in [
        ("DE", "30 123456", "30 123456"),
        ("US", "2133734253", "213 373 4253"),
    ] {
        dioxus::html::set_event_converter(Box::new(TestConverter));
        PHONE_COUNTRY.set(country);
        PHONE_TOGGLE.set(false);
        let mut dom = VirtualDom::new(phone_app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        // Every render is written back into `find`: the second pass, which is
        // what picks up the component CSS, replaces the nodes, and an id read
        // before it addresses an element that is gone.
        dom.render_immediate(&mut find);

        let input = input_listener(&find);
        // The `tel` input is the element that listens for both, and a blur
        // sent anywhere else would prove nothing.
        assert!(
            find.blur.contains(&input),
            "the tel input has no blur listener: {:?}",
            find.blur
        );
        dom.runtime()
            .handle_event("input", Event::new(input_event(typed), true), input);
        dom.render_immediate(&mut find);
        assert_eq!(
            phone_text(&dioxus_ssr::render(&dom)),
            typed,
            "{country}: the keystroke"
        );

        // A re-render with nothing else touched - what a demo control does.
        PHONE_TOGGLE.set(true);
        dom.mark_dirty(dioxus::core::ScopeId::APP);
        dom.render_immediate(&mut find);
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("Deliveries only"),
            "the re-render did nothing"
        );
        assert_eq!(phone_text(&html), typed, "{country}: the re-render");

        let input = input_listener(&find);
        dom.runtime()
            .handle_event("blur", Event::new(focus_event(), false), input);
        dom.render_immediate(&mut find);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(phone_text(&html), after_blur, "{country}: the blur");
        // Whatever the text is doing, the hidden input posts E.164.
        let digits: String = typed.chars().filter(char::is_ascii_digit).collect();
        let dial = match country {
            "DE" => "49",
            _ => "1",
        };
        assert!(
            html.contains(&format!("value=\"+{dial}{digits}\"")),
            "the E.164 moved: {html}"
        );
    }
}

/// `Select`'s typeahead, and the arrows passing over a disabled row. Both are
/// keyboard-only, so they need real dispatched events rather than SSR.
mod select_keyboard {
    use super::*;
    use libero::components::{OptionItem, OptionList, Select};

    thread_local! {
        static PICKED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn cities() -> OptionList<String> {
        OptionList::new([
            "Berlin".to_string().into(),
            "Bonn".to_string().into(),
            // Skipped by the arrows and by typeahead alike, which is why it
            // sits between two rows that a "B" would otherwise walk through.
            OptionItem::new("Bochum".to_string()).disabled(true),
            "Cologne".to_string().into(),
        ])
    }

    /// Really controlled: `Select` renders `value` and asks for a new one, so
    /// a test that never moves it would have typeahead searching from the same
    /// place every time - and would never see the cycle at all.
    fn app() -> Element {
        let mut city = use_signal(|| None::<String>);
        rsx! {
            LiberoProvider {
                Select::<String> {
                    label: "City",
                    options: cities(),
                    value: city(),
                    onchange: move |next: Option<String>| {
                        PICKED.with_borrow_mut(|picked| picked.push(next.clone().unwrap_or_default()));
                        city.set(next);
                    },
                }
            }
        }
    }

    /// A dom with the event converter installed and the picks reset, plus the
    /// trigger's element id - the innermost element carrying a `keydown`,
    /// since `ComboboxCore`'s wrapper registers one first.
    fn mount() -> (VirtualDom, ElementId) {
        dioxus::html::set_event_converter(Box::new(TestConverter));
        PICKED.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let trigger = *find.keydown.last().expect("the trigger listens for keys");
        (dom, trigger)
    }

    fn type_keys(dom: &mut VirtualDom, trigger: ElementId, keys: &str) {
        for ch in keys.chars() {
            press(dom, trigger, Key::Character(ch.to_string()));
        }
    }

    /// Two passes: the list reports its row count while it renders, after the
    /// trigger has been drawn, so the trigger catches up one pass later.
    fn press(dom: &mut VirtualDom, trigger: ElementId, key: Key) {
        dom.runtime()
            .handle_event("keydown", Event::new(key_event(key), true), trigger);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }

    fn picked() -> Vec<String> {
        PICKED.with_borrow(Clone::clone)
    }

    /// The buffer: three characters inside the window narrow to one row, and
    /// a closed trigger changes the value in place the way a native `<select>`
    /// does - no list is opened at all.
    #[test]
    fn a_buffered_query_picks_in_place_on_a_closed_trigger() {
        let (mut dom, trigger) = mount();
        type_keys(&mut dom, trigger, "ber");

        // "b" lands on Berlin, and "be" then "ber" stay on it while it still
        // matches - the narrowing half of the buffer.
        assert_eq!(picked(), ["Berlin", "Berlin", "Berlin"]);
        let html = body(&dioxus_ssr::render(&dom));
        assert!(
            !html.contains(r#"role="listbox""#),
            "typing opened the list:\n{html}"
        );
    }

    /// One character, pressed again, cycles - `typeahead_match` treats a
    /// repeat as a single-character search that starts *after* the current
    /// row. Bochum is disabled, so "B" walks Berlin, Bonn and back.
    #[test]
    fn a_repeated_character_cycles_and_skips_a_disabled_row() {
        let (mut dom, trigger) = mount();
        type_keys(&mut dom, trigger, "b");
        assert_eq!(picked(), ["Berlin"]);

        // Each press searches from the row the last one selected, so the
        // repeat walks on rather than landing on Berlin again.
        type_keys(&mut dom, trigger, "b");
        assert_eq!(picked(), ["Berlin", "Bonn"]);
        type_keys(&mut dom, trigger, "b");
        assert_eq!(
            picked(),
            ["Berlin", "Bonn", "Berlin"],
            "Bochum is disabled, so the cycle wraps past it"
        );
    }

    /// Space still opens the list, as it always has - typeahead takes a space
    /// only mid-query, where it is part of "new york".
    #[test]
    fn space_opens_the_list_rather_than_typing() {
        let (mut dom, trigger) = mount();
        press(&mut dom, trigger, Key::Character(" ".into()));

        assert_eq!(picked(), Vec::<String>::new());
        let html = body(&dioxus_ssr::render(&dom));
        assert!(html.contains(r#"role="listbox""#), "{html}");
    }

    /// An open list moves its highlight instead of picking, and the arrows
    /// pass over the disabled row: Berlin, Bonn, then Cologne.
    #[test]
    fn the_arrows_skip_a_disabled_row() {
        let (mut dom, trigger) = mount();
        press(&mut dom, trigger, Key::Character(" ".into()));

        let rows = |dom: &VirtualDom| {
            let html = body(&dioxus_ssr::render(dom));
            let at = html
                .find(r#"aria-activedescendant=""#)
                .map(|at| at + r#"aria-activedescendant=""#.len());
            at.map(|at| html[at..].split('"').next().unwrap().to_string())
        };

        // Opening already arms the first row, as a native `<select>` does.
        assert!(rows(&dom).is_some_and(|id| id.ends_with("-option-0")));
        press(&mut dom, trigger, Key::ArrowDown);
        assert!(rows(&dom).is_some_and(|id| id.ends_with("-option-1")));
        // Row 2 is Bochum, which is disabled.
        press(&mut dom, trigger, Key::ArrowDown);
        let id = rows(&dom).expect("a row to point at");
        assert!(
            id.ends_with("-option-3"),
            "the arrows stopped on Bochum: {id}"
        );
        // And back up over it.
        press(&mut dom, trigger, Key::ArrowUp);
        let id = rows(&dom).expect("a row to point at");
        assert!(id.ends_with("-option-1"), "{id}");

        assert_eq!(picked(), Vec::<String>::new(), "an arrow picked something");
    }
}

thread_local! {
    static PIN_COMPLETIONS: std::cell::RefCell<u32> = const { std::cell::RefCell::new(0) };
}

/// A parent resetting the controlled `value` between attempts - "wrong code,
/// try again" - re-arms `oncomplete`, so a pasted retry completes (todo 413).
#[test]
fn oncomplete_fires_again_after_a_parent_resets_the_controlled_value() {
    fn app() -> Element {
        let mut pin = use_signal(String::new);

        rsx! {
            LiberoProvider {
                Button { onclick: move |_| pin.set(String::new()), "Reset" }
                PinField {
                    label: "Code",
                    length: 4,
                    value: pin(),
                    oninput: move |next: String| pin.set(next),
                    oncomplete: move |_: String| {
                        PIN_COMPLETIONS.with_borrow_mut(|count| *count += 1);
                    },
                }
            }
        }
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    PIN_COMPLETIONS.with_borrow_mut(|count| *count = 0);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let cells = find.keydown.clone();
    assert_eq!(cells.len(), 4, "expected one cell per pin position");
    let reset = find.first_click.expect("registered no click listener");

    let type_pin = |dom: &mut VirtualDom, digits: &str| {
        for (cell, digit) in cells.iter().zip(digits.chars()) {
            dom.runtime().handle_event(
                "input",
                Event::new(input_event(&digit.to_string()), true),
                *cell,
            );
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
    };

    type_pin(&mut dom, "1234");
    assert_eq!(
        PIN_COMPLETIONS.with_borrow(|count| *count),
        1,
        "the first full pin must complete"
    );

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), reset);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    // An OTP autofill (or a paste) lands the whole code in one `input` event
    // on the first cell, rather than one keystroke per cell.
    dom.runtime()
        .handle_event("input", Event::new(input_event("5678"), true), cells[0]);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(
        PIN_COMPLETIONS.with_borrow(|count| *count),
        2,
        "a pin pasted after the parent reset it must complete again"
    );
}
