//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Button},
};
use std::rc::Rc;

/// Records the element the `click` listener landed on, which is the only way
/// to address it from a test.
#[derive(Default)]
struct FindClickListener {
    last: Option<ElementId>,
    click: Option<ElementId>,
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
