//! The phone drawer follows an edge swipe's finger (todo 2170, Maintainer): released
//! past a third of its width or on a flick it opens, otherwise it slides back.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    hooks::{EdgeSwipeOptions, ElementHandle, Swipe, use_direction, use_edge_swipe},
    platform::{ElementApi, document},
};
use web_time::Instant;

/// A flick, in px per ms inward: Android's `DrawerLayout` flings from 400 dp/s.
const FLICK: f64 = 0.4;
/// Still this long before the lift: no flick.
const HELD: Duration = Duration::from_millis(100);

#[derive(Clone, Copy)]
struct Press {
    pointer_id: i32,
    start_x: f64,
    /// Inward travel so far, px.
    travel: f64,
    at: Instant,
    /// Inward speed of the last move, px per ms.
    speed: f64,
    /// The edge swipe was recognised: the drawer follows.
    following: bool,
}

/// What the shell spreads on the page and hands the drawer.
#[derive(Clone, Copy)]
pub(crate) struct NavDrag {
    pub(crate) swipe: Swipe,
    /// How far the closed drawer is pulled out, px, while a finger holds it.
    pub(crate) pulled: ReadSignal<Option<f64>>,
}

/// Whether a released pull opens the drawer.
fn opens(travel: f64, speed: f64, width: f64) -> bool {
    travel > width / 3.0 || (travel > 0.0 && speed > FLICK)
}

pub(crate) fn use_nav_drag(mut open: Signal<bool>, enabled: bool, nav: ElementHandle) -> NavDrag {
    let rtl = use_direction().is_rtl();
    let mut press = use_signal(|| None::<Press>);
    let mut pulled = use_signal(|| None::<f64>);
    let mut width = use_signal(|| None::<f64>);
    let edge = use_edge_swipe(
        Callback::new(move |()| {
            if !enabled || *open.peek() {
                return;
            }
            let held = *press.peek();
            match held {
                Some(mut current) => {
                    current.following = true;
                    press.set(Some(current));
                    pulled.set(Some(current.travel));
                }
                // The browser took the touch (a fling ran): nothing to follow (todo 2190).
                None => open.set(true),
            }
        }),
        EdgeSwipeOptions::default(),
    );

    let onpointerdown = use_callback(move |event: PointerEvent| {
        edge.onpointerdown.call(event.clone());
        if !event.is_primary() || event.pointer_type() == "mouse" {
            press.set(None);
            return;
        }
        press.set(Some(Press {
            pointer_id: event.pointer_id(),
            start_x: event.client_coordinates().x,
            travel: 0.0,
            at: Instant::now(),
            speed: 0.0,
            following: false,
        }));
        // Measured at each press: the drawer is the page wide on a phone, a sidebar on
        // Home. A WebView may not read a queried element: the viewport then.
        let drawer = nav
            .query_selector("nav")
            .map(|element| element.dimensions());
        let viewport = document().map(|document| document.viewport());
        spawn(async move {
            let measured = match drawer {
                Ok(size) => size.await.ok().map(|size| size.width),
                Err(_) => None,
            }
            .filter(|found| *found > 0.0);
            let fallback = match viewport {
                Some(size) if measured.is_none() => size.await.ok().map(|size| size.width),
                _ => None,
            };
            if let Some(found) = measured.or(fallback) {
                width.set(Some(found));
            }
        });
    });
    let onpointermove = use_callback(move |event: PointerEvent| {
        let Some(mut current) = *press.peek() else {
            edge.onpointermove.call(event);
            return;
        };
        if event.pointer_id() != current.pointer_id {
            edge.onpointermove.call(event);
            return;
        }
        let x = event.client_coordinates().x;
        let travel = if rtl {
            current.start_x - x
        } else {
            x - current.start_x
        };
        let now = Instant::now();
        let ms = now.duration_since(current.at).as_secs_f64() * 1000.0;
        if ms > 0.0 {
            current.speed = (travel - current.travel) / ms;
        }
        current.travel = travel;
        current.at = now;
        press.set(Some(current));
        // After the update: the swipe it may recognise reads this travel.
        edge.onpointermove.call(event);
        if current.following {
            let most = width.peek().unwrap_or(f64::MAX);
            pulled.set(Some(travel.clamp(0.0, most)));
        }
    });
    let mut end = move |event: PointerEvent, lifted: bool| {
        let current = *press.peek();
        if let Some(current) = current
            && current.pointer_id == event.pointer_id()
        {
            press.set(None);
            if current.following {
                let full = width.peek().unwrap_or(f64::MAX);
                // A finger held still before lifting flicks nothing.
                let speed = if current.at.elapsed() > HELD {
                    0.0
                } else {
                    current.speed
                };
                if lifted && opens(current.travel, speed, full) {
                    open.set(true);
                }
                pulled.set(None);
            }
        }
    };
    let onpointerup = use_callback(move |event: PointerEvent| {
        end(event.clone(), true);
        edge.onpointerup.call(event);
    });
    let onpointercancel = use_callback(move |event: PointerEvent| {
        end(event.clone(), false);
        edge.onpointercancel.call(event);
    });

    NavDrag {
        swipe: Swipe {
            onpointerdown,
            onpointermove,
            onpointerup,
            onpointercancel,
        },
        pulled: pulled.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::opens;

    #[test]
    fn a_third_of_the_width_or_a_flick_opens() {
        assert!(!opens(100.0, 0.1, 412.0));
        assert!(opens(140.0, 0.1, 412.0));
        assert!(opens(60.0, 0.8, 412.0));
        // A flick back out does not.
        assert!(!opens(-10.0, 0.8, 412.0));
    }
}
