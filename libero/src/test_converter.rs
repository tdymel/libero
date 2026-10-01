//! The renderer's side of a dispatched event for the lib tests: one
//! `HtmlEventConverter` and the fake payloads it hands out.

use dioxus::html::PlatformEventData;
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::point_interaction::{
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
};
use dioxus::html::{HasKeyboardData, HasMouseData};
use dioxus::prelude::*;

/// The renderer's key event: Escape, plain, held or composing; or ArrowDown,
/// which opens a real `Select`'s list.
#[derive(Clone, Copy, Default)]
pub(crate) struct FakeEscape {
    pub(crate) repeat: bool,
    pub(crate) composing: bool,
    pub(crate) arrow_down: bool,
}

impl HasKeyboardData for FakeEscape {
    fn key(&self) -> Key {
        match self.arrow_down {
            true => Key::ArrowDown,
            false => Key::Escape,
        }
    }
    fn code(&self) -> Code {
        match self.arrow_down {
            true => Code::ArrowDown,
            false => Code::Escape,
        }
    }
    fn location(&self) -> dioxus::html::input_data::keyboard_types::Location {
        dioxus::html::input_data::keyboard_types::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        self.repeat
    }
    fn is_composing(&self) -> bool {
        self.composing
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl dioxus::html::point_interaction::ModifiersInteraction for FakeEscape {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}

/// The one converter the lib tests install: it is process-wide, so two that
/// each covered only their own events raced (todo 1949).
pub(crate) struct TestConverter;

impl dioxus::html::HtmlEventConverter for TestConverter {
    fn convert_keyboard_data(&self, event: &PlatformEventData) -> dioxus::html::KeyboardData {
        let press = event.downcast::<FakeEscape>().copied().unwrap_or_default();
        dioxus::html::KeyboardData::new(press)
    }
    fn convert_animation_data(&self, _e: &PlatformEventData) -> dioxus::html::AnimationData {
        unimplemented!()
    }
    fn convert_before_input_data(&self, _e: &PlatformEventData) -> dioxus::html::BeforeInputData {
        unimplemented!()
    }
    fn convert_cancel_data(&self, _e: &PlatformEventData) -> dioxus::html::CancelData {
        unimplemented!()
    }
    fn convert_clipboard_data(&self, _e: &PlatformEventData) -> dioxus::html::ClipboardData {
        unimplemented!()
    }
    fn convert_composition_data(&self, _e: &PlatformEventData) -> dioxus::html::CompositionData {
        unimplemented!()
    }
    fn convert_drag_data(&self, _e: &PlatformEventData) -> dioxus::html::DragData {
        unimplemented!()
    }
    fn convert_focus_data(&self, _e: &PlatformEventData) -> dioxus::html::FocusData {
        dioxus::html::FocusData::new(FakeFocus)
    }
    fn convert_form_data(&self, _e: &PlatformEventData) -> dioxus::html::FormData {
        unimplemented!()
    }
    fn convert_image_data(&self, _e: &PlatformEventData) -> dioxus::html::ImageData {
        unimplemented!()
    }
    fn convert_media_data(&self, _e: &PlatformEventData) -> dioxus::html::MediaData {
        unimplemented!()
    }
    /// The mounted floor: every focus-containment question answers `Unsupported`.
    fn convert_mounted_data(&self, _e: &PlatformEventData) -> dioxus::html::MountedData {
        dioxus::html::MountedData::new(())
    }
    fn convert_mouse_data(&self, _e: &PlatformEventData) -> dioxus::html::MouseData {
        dioxus::html::MouseData::new(FakeMouse)
    }
    fn convert_pointer_data(&self, _e: &PlatformEventData) -> dioxus::html::PointerData {
        unimplemented!()
    }
    fn convert_resize_data(&self, _e: &PlatformEventData) -> dioxus::html::ResizeData {
        unimplemented!()
    }
    fn convert_scroll_data(&self, _e: &PlatformEventData) -> dioxus::html::ScrollData {
        unimplemented!()
    }
    fn convert_selection_data(&self, _e: &PlatformEventData) -> dioxus::html::SelectionData {
        unimplemented!()
    }
    fn convert_toggle_data(&self, _e: &PlatformEventData) -> dioxus::html::ToggleData {
        unimplemented!()
    }
    fn convert_touch_data(&self, _e: &PlatformEventData) -> dioxus::html::TouchData {
        unimplemented!()
    }
    fn convert_transition_data(&self, _e: &PlatformEventData) -> dioxus::html::TransitionData {
        unimplemented!()
    }
    fn convert_visible_data(&self, _e: &PlatformEventData) -> dioxus::html::VisibleData {
        unimplemented!()
    }
    fn convert_wheel_data(&self, _e: &PlatformEventData) -> dioxus::html::WheelData {
        unimplemented!()
    }
}

/// The focus payload carries nothing the hook reads.
pub(crate) struct FakeFocus;

impl dioxus::html::HasFocusData for FakeFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A primary-button click; no handler under test reads its coordinates.
pub(crate) struct FakeMouse;

impl InteractionLocation for FakeMouse {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(0.0, 0.0)
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(0.0, 0.0)
    }
    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(0.0, 0.0)
    }
}
impl InteractionElementOffset for FakeMouse {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(0.0, 0.0)
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
