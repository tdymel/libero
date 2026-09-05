//! `use_drag` driven with synthetic pointer events.
//!
//! A `cancel` made synchronously inside `onstart` is how a disabled `Slider`
//! refuses a drag. It used to be overwritten by the drag starting after
//! `onstart` returned, so the disabled slider still followed the pointer and
//! reported its end.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::dioxus_core::{ScopeId, VirtualDom};
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::point_interaction::{
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
};
use dioxus::html::{HasPointerData, PointerData};
use dioxus::prelude::*;
use libero::hooks::{Drag, DragOptions, DragStart, use_drag, use_element};

thread_local! {
    static CANCEL_ON_START: Cell<bool> = const { Cell::new(false) };
    static DRAG: RefCell<Option<Drag>> = const { RefCell::new(None) };
    static MOVES: Cell<u32> = const { Cell::new(0) };
    static ENDS: Cell<u32> = const { Cell::new(0) };
}

fn app() -> Element {
    let capture = use_element();
    let drag = use_drag(DragOptions {
        capture,
        onstart: Callback::new(|event: DragStart| {
            if CANCEL_ON_START.with(Cell::get) {
                event.cancel.call(());
            }
        }),
        onmove: Callback::new(|_| MOVES.with(|moves| moves.set(moves.get() + 1))),
        onend: Callback::new(|()| ENDS.with(|ends| ends.set(ends.get() + 1))),
    });
    DRAG.with(|slot| *slot.borrow_mut() = Some(drag));

    rsx! {}
}

/// Presses, moves and releases one pointer; returns whether the drag was
/// running after the press, and how many moves and ends it reported.
fn press_move_release(cancel_on_start: bool) -> (bool, u32, u32) {
    CANCEL_ON_START.with(|cancel| cancel.set(cancel_on_start));
    MOVES.with(|moves| moves.set(0));
    ENDS.with(|ends| ends.set(0));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let drag = DRAG.with(|slot| slot.borrow().expect("app did not render"));

    let dragging = dom.in_scope(ScopeId::ROOT, || {
        drag.onpointerdown.call(pointer_event());
        let dragging = *drag.dragging.peek();
        drag.onpointermove.call(pointer_event());
        drag.onpointerup.call(pointer_event());
        dragging
    });

    (dragging, MOVES.with(Cell::get), ENDS.with(Cell::get))
}

#[test]
fn a_drag_reports_its_moves_and_its_end() {
    assert_eq!(press_move_release(false), (true, 1, 1));
}

#[test]
fn a_cancel_inside_on_start_stops_the_drag() {
    assert_eq!(press_move_release(true), (false, 0, 0));
}

fn pointer_event() -> Event<PointerData> {
    Event::new(Rc::new(PointerData::new(FakePointer)), true)
}

// A stand-in for the browser's pointer event, since the real one only exists
// behind the `serialize` feature.
struct FakePointer;

impl InteractionLocation for FakePointer {
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
impl InteractionElementOffset for FakePointer {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(0.0, 0.0)
    }
}
impl ModifiersInteraction for FakePointer {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}
impl PointerInteraction for FakePointer {
    fn trigger_button(&self) -> Option<MouseButton> {
        Some(MouseButton::Primary)
    }
    fn held_buttons(&self) -> MouseButtonSet {
        Default::default()
    }
}
impl HasPointerData for FakePointer {
    fn pointer_id(&self) -> i32 {
        1
    }
    fn width(&self) -> f64 {
        1.0
    }
    fn height(&self) -> f64 {
        1.0
    }
    fn pressure(&self) -> f32 {
        0.5
    }
    fn tangential_pressure(&self) -> f32 {
        0.0
    }
    fn tilt_x(&self) -> i32 {
        0
    }
    fn tilt_y(&self) -> i32 {
        0
    }
    fn twist(&self) -> i32 {
        0
    }
    fn pointer_type(&self) -> String {
        "mouse".to_string()
    }
    fn is_primary(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
