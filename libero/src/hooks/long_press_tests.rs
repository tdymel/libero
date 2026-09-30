//! `use_long_press` against the real non-wasm timer and a `VirtualDom` polled
//! the way a renderer would, driven with synthetic pointer events.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::point_interaction::{
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
};
use dioxus::html::{HasPointerData, PointerData};
use dioxus::prelude::*;

use super::polling_tests::{self, pump, pump_until};
use crate::hooks::{LongPress, LongPressOptions, use_long_press};

thread_local! {
    static PRESS: RefCell<Option<LongPress>> = const { RefCell::new(None) };
    static FIRED: Cell<u32> = const { Cell::new(0) };
}

fn app() -> Element {
    let press = use_long_press(
        Callback::new(|()| FIRED.with(|fired| fired.set(fired.get() + 1))),
        LongPressOptions {
            ms: 60,
            move_tolerance: 10.0,
        },
    );
    PRESS.with(|slot| *slot.borrow_mut() = Some(press));
    rsx! {}
}

fn started() -> VirtualDom {
    FIRED.with(|fired| fired.set(0));
    polling_tests::started(app)
}

/// Waits until the press would have fired, had it not been cancelled.
fn settle(dom: &mut VirtualDom) {
    polling_tests::settle(dom, 60);
}

fn fires(dom: &mut VirtualDom) {
    pump_until(dom, "the long press", |_| fired() == 1);
}

fn handlers() -> LongPress {
    PRESS.with(|slot| slot.borrow().expect("rendered"))
}

fn fired() -> u32 {
    FIRED.with(Cell::get)
}

#[derive(Clone, Copy)]
struct Fake {
    id: i32,
    primary: bool,
    x: f64,
    y: f64,
}

fn at(x: f64, y: f64) -> Fake {
    Fake {
        id: 1,
        primary: true,
        x,
        y,
    }
}

fn event(fake: Fake) -> Event<PointerData> {
    Event::new(Rc::new(PointerData::new(fake)), true)
}

fn down(dom: &VirtualDom, fake: Fake) {
    dom.in_runtime(|| handlers().onpointerdown.call(event(fake)));
}

fn go(dom: &VirtualDom, fake: Fake) {
    dom.in_runtime(|| handlers().onpointermove.call(event(fake)));
}

fn up(dom: &VirtualDom, fake: Fake) {
    dom.in_runtime(|| handlers().onpointerup.call(event(fake)));
}

#[test]
fn a_held_press_fires_once() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    pump(&mut dom, 40);
    assert_eq!(fired(), 0, "fired before its time");

    fires(&mut dom);
    up(&dom, at(5.0, 5.0));
    settle(&mut dom);
    assert_eq!(fired(), 1, "fired again");
}

fn pressing(dom: &VirtualDom) -> bool {
    dom.in_runtime(|| *handlers().pressing.peek())
}

#[test]
fn pressing_is_true_only_while_the_press_is_held() {
    let mut dom = started();
    assert!(!pressing(&dom));
    down(&dom, at(5.0, 5.0));
    pump_until(&mut dom, "pressing while held", pressing);
    assert_eq!(fired(), 0, "fired before it was held");
    fires(&mut dom);
    assert!(!pressing(&dom), "still pressing after firing");

    down(&dom, at(5.0, 5.0));
    pump_until(&mut dom, "pressing again", pressing);
    up(&dom, at(5.0, 5.0));
    pump_until(&mut dom, "not pressing after a release", |dom| {
        !pressing(dom)
    });
}

#[test]
fn a_tap_does_not_fire() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    pump(&mut dom, 20);
    up(&dom, at(5.0, 5.0));
    settle(&mut dom);
    assert_eq!(fired(), 0);
}

#[test]
fn a_pointer_that_leaves_does_not_fire() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    dom.in_runtime(|| handlers().onpointerleave.call(event(at(5.0, 5.0))));
    settle(&mut dom);
    assert_eq!(fired(), 0);
}

#[test]
fn a_cancelled_touch_does_not_fire() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    dom.in_runtime(|| handlers().onpointercancel.call(event(at(5.0, 5.0))));
    settle(&mut dom);
    assert_eq!(fired(), 0);
}

#[test]
fn a_move_beyond_the_tolerance_does_not_fire() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    go(&dom, at(5.0, 30.0));
    settle(&mut dom);
    assert_eq!(fired(), 0);
}

#[test]
fn a_move_within_the_tolerance_still_fires() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    go(&dom, at(8.0, 9.0));
    fires(&mut dom);
}

#[test]
fn a_second_finger_does_not_fire() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    down(
        &dom,
        Fake {
            id: 2,
            primary: false,
            ..at(50.0, 50.0)
        },
    );
    settle(&mut dom);
    assert_eq!(fired(), 0);
}

#[test]
fn a_press_after_a_cancelled_one_fires() {
    let mut dom = started();
    down(&dom, at(5.0, 5.0));
    up(&dom, at(5.0, 5.0));
    down(&dom, at(5.0, 5.0));
    fires(&mut dom);
    // Once: the cancelled press does not fire as well.
    settle(&mut dom);
    assert_eq!(fired(), 1);
}

impl InteractionLocation for Fake {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.x, self.y)
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.x, self.y)
    }
    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.x, self.y)
    }
}
impl InteractionElementOffset for Fake {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(0.0, 0.0)
    }
}
impl ModifiersInteraction for Fake {
    fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
        Default::default()
    }
}
impl PointerInteraction for Fake {
    fn trigger_button(&self) -> Option<MouseButton> {
        Some(MouseButton::Primary)
    }
    fn held_buttons(&self) -> MouseButtonSet {
        Default::default()
    }
}
impl HasPointerData for Fake {
    fn pointer_id(&self) -> i32 {
        self.id
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
        "touch".to_string()
    }
    fn is_primary(&self) -> bool {
        self.primary
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
