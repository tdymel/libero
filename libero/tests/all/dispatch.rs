//! The dispatch harness: the shared recorder, fake renderer payloads, and
//! helpers that send an event and settle.

use crate::common::{body, tags_with};
use dioxus::core::ElementId;
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use libero::components::Options;
use std::rc::Rc;

pub use crate::recorder::FindClickListener;

/// A mounted app and the recorder that saw it render.
pub struct Page {
    pub dom: VirtualDom,
    pub rec: FindClickListener,
}

impl Page {
    /// Builds `app` and runs its first effects, recording both passes.
    pub fn mount(app: fn() -> Element) -> Self {
        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut rec = FindClickListener::default();
        dom.rebuild(&mut rec);
        dom.render_immediate(&mut rec);
        Self { dom, rec }
    }

    /// Clicks `id`, then renders once.
    pub fn click(&mut self, id: ElementId) {
        self.dom
            .runtime()
            .handle_event("click", Event::new(click_event(), true), id);
        self.render();
    }

    pub fn render(&mut self) {
        self.dom.render_immediate(&mut self.rec);
    }
}

/// Renderers register one of these; without it every event conversion panics.
/// Only mouse events are needed here.
pub struct TestConverter;

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
pub fn click_event() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeMouse)))
}

/// A stand-in for the renderer's `input` payload: the text the control now
/// holds, which is all any field reads off it.
#[derive(Clone)]
pub struct FakeInput(pub String);

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
pub struct FakeKey(pub Key);

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
pub struct FakeFocus;

impl dioxus::html::HasFocusData for FakeFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A stand-in for the renderer's `transitionend` payload: the property that
/// finished, which is all the presence hook reads.
#[derive(Clone)]
pub struct FakeTransition(pub &'static str);

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

pub fn transition_end_event(property: &'static str) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeTransition(property))))
}

pub fn key_event(key: Key) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeKey(key))))
}

pub fn input_event(text: &str) -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeInput(
        text.to_string(),
    ))))
}

/// Empty where the component set no state at all - an unstyled `ActionIcon`
/// only grows the attribute once it has a ripple.
pub fn data_state(html: &str) -> String {
    tags_with(&body(html), "data-state=")
        .first()
        .map(|tag| tag["data-state"].clone())
        .unwrap_or_default()
}

pub fn checked_states(html: &str) -> Vec<bool> {
    tags_with(html, "<input")
        .iter()
        .map(|input| input.contains_key("checked"))
        .collect()
}

pub fn assert_ripple_alternates(app: fn() -> Element) {
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
pub struct FakeMouse;

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

/// Clicks the last click listener `app` registers - `Marquee`'s pause toggle,
/// `CodeBlock`'s copy button - and returns the markup before and after.
pub fn click_the_last_listener(app: fn() -> Element) -> (String, String) {
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

/// A press on a chip's x must not take the focus. The field holds the focus
/// that keeps its list open, and the x is about to be removed: focused, it
/// would take the focus down with it, to the body. Nothing but this guard
/// stops that, so nothing but this test notices it gone (todo 77).
///
/// `guards` counts every `mousedown` listener the closed field registers,
/// which since todo 70 is more than one per chip: `TagsField`'s tag wrapper
/// and `FileField`'s chip wrapper guard a caller's own chip too, and `MultiSelect`'s value slot and chevron
/// guard the trigger's focus.
pub fn assert_every_press_keeps_the_focus(app: fn() -> Element, guards: usize) {
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

thread_local! {
    /// What each `onchange` handed back, in order.
    pub static CLEARED: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Clicks the field's Clear button and returns what `onchange` received, as
/// value lengths. `None` when the field drew no Clear button: a test expecting
/// `None` also needs a call that gets `Some` (todo 1751).
///
/// Todo 252, for `b7d1c39d`: the shared `clear_button` renders only while it
/// has something to clear, and one click reports one change. Where focus goes
/// next is `ElementApi::focus` on the field's own control, which does nothing
/// without a renderer, so that half is still checked only in a browser.
pub fn click_clear(app: fn() -> Element) -> Option<Vec<usize>> {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    CLEARED.with_borrow_mut(Vec::clear);
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    // The shared `clear_button` of `TagsField`, `FileField`, `Autocomplete`,
    // `Select` and `Cascader`, found by its label alone.
    let label = libero::localization::CommonLabels::ENGLISH.clear;
    let clear = *find
        .attributes
        .iter()
        .find(|(_, attributes)| {
            attributes
                .get("aria-label")
                .is_some_and(|found| found == label)
        })?
        .0;
    assert!(
        find.registered_for("click").contains(&clear),
        "Clear has no click listener"
    );
    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), clear);
    dom.process_events();
    Some(CLEARED.with_borrow(Clone::clone))
}

thread_local! {
    /// Whether the next `readonly_*` app mounts read-only. The apps are plain
    /// `fn`s, so the same one serves as its own positive control.
    pub static READ_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// What the field's handler heard, one entry per call.
    pub static HEARD: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn heard(what: impl std::fmt::Debug) {
    HEARD.with_borrow_mut(|seen| seen.push(format!("{what:?}")));
}

/// Mounts `app` with `readonly` as given, sends `name` with `data` to the
/// element `pick` chooses, and returns what the handler heard and the markup
/// afterwards. Run once each way, so a refusal is only believed next to the
/// same event being answered.
pub fn send(
    app: fn() -> Element,
    readonly: bool,
    name: &str,
    pick: impl Fn(&FindClickListener) -> ElementId,
    data: impl Fn() -> Rc<dyn std::any::Any>,
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

pub fn first_keydown(find: &FindClickListener) -> ElementId {
    find.keydown[0]
}

pub fn last_keydown(find: &FindClickListener) -> ElementId {
    *find.keydown.last().expect("registered no keydown listener")
}

pub fn last_click(find: &FindClickListener) -> ElementId {
    find.click.expect("registered no click listener")
}

pub fn input_listener(find: &FindClickListener) -> ElementId {
    find.input.expect("registered no input listener")
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
pub enum Tier {
    Free,
    Pro,
    Team,
}

thread_local! {
    /// The selection and the disabled options `tier_group` mounts with.
    pub static TIER: std::cell::Cell<Option<Tier>> = const { std::cell::Cell::new(None) };
    pub static TIERS_OFF: std::cell::RefCell<Vec<Tier>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn focus_event() -> Rc<dyn std::any::Any> {
    Rc::new(PlatformEventData::new(Box::new(FakeFocus)))
}

#[derive(Clone, PartialEq, Options)]
pub enum Emphasis {
    Bold,
    Italic,
}
