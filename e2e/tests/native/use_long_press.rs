//! `use_long_press` under a held finger on Blitz: the hold cue is up while the
//! finger is down and drops once the press fired. The firing and the swallowed
//! click are shared with the web in `e2e/tests/all/use_long_press.rs`.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::mount;
use libero::hooks::{LongPressOptions, use_long_press};

const TARGET: &str = "#target";

fn app() -> Element {
    let mut holds = use_signal(|| 0);
    let press = use_long_press(
        Callback::new(move |()| holds += 1),
        LongPressOptions::default(),
    );
    rsx! {
        button {
            id: "target",
            "data-pressing": "{press.pressing}",
            onpointerdown: move |event| press.onpointerdown.call(event),
            onpointerup: move |event| press.onpointerup.call(event),
            onpointercancel: move |event| press.onpointercancel.call(event),
            onclick: move |event| {
                press.onclick.call(event);
            },
            "Hold"
        }
        p { id: "holds", "{holds}" }
    }
}

#[test]
fn the_hold_cue_is_up_while_the_finger_is_down_and_drops_when_it_fires() {
    let mut page = mount(app);
    assert_eq!(page.attr(TARGET, "data-pressing").as_deref(), Some("false"));
    let (x, y) = page.touch_down(TARGET);
    assert_eq!(page.attr(TARGET, "data-pressing").as_deref(), Some("true"));
    assert!(
        page.wait_for(|page| page.text("#holds") == "1"),
        "the hold never fired"
    );
    assert_eq!(page.attr(TARGET, "data-pressing").as_deref(), Some("false"));
    page.touch_up(x, y);
    page.wait(Duration::from_millis(100));
    assert_eq!(page.text("#holds"), "1");
}

#[test]
fn a_finger_lifted_early_drops_the_cue_and_fires_nothing() {
    let mut page = mount(app);
    let (x, y) = page.touch_down(TARGET);
    assert_eq!(page.attr(TARGET, "data-pressing").as_deref(), Some("true"));
    page.touch_up(x, y);
    assert_eq!(page.attr(TARGET, "data-pressing").as_deref(), Some("false"));
    page.wait(Duration::from_millis(900));
    assert_eq!(page.text("#holds"), "0");
}
