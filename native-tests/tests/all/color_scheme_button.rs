//! `ColorSchemeButton`'s theme picker at the end of the docs header (todo
//! 625): the menu opens inside the viewport.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{ColorSchemeButton, Flex, Header, ScrollArea},
    theme::ThemeSet,
};
use native_tests::{Page, VIEWPORT, mount};

const MENU: &str = "[role=menu]";
const TRIGGER: &str = "[aria-haspopup]";

// The docs shell: a header over a row of one viewport's height that scrolls itself.
fn app() -> Element {
    rsx! {
        Flex { direction: "column", sx: libero::sx::sx().gap("0"),
            Header {
                span { margin_right: "auto", "Libero" }
                ColorSchemeButton { size: "lg", themes: ThemeSet::CATALOGUE }
            }
            Flex { direction: "row", sx: libero::sx::sx().height("calc(100vh - 60px)"),
                ScrollArea { sx: libero::sx::sx().flex("1").min_height("0").min_width("0"),
                    div { height: "2000px" }
                }
            }
        }
    }
}

// Before layout: a window's shell polled the opening out before laying the menu out.
fn open(page: &mut Page) {
    page.click_before_layout(TRIGGER);
    page.wait(Duration::from_millis(50));
    assert!(
        page.exists(MENU),
        "the picker did not open:\n{}",
        page.tree()
    );
}

#[test]
fn the_theme_picker_opens_inside_the_viewport() {
    let mut page = mount(app);
    open(&mut page);
    let (x, y, width, height) = page.rect(MENU);
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    assert!(
        x >= 0.0 && y >= 0.0 && x + width <= vw + 0.5 && y + height <= vh + 0.5,
        "menu at ({x}, {y}), {width} x {height}, viewport {vw} x {vh}; trigger at {:?}",
        page.rect(TRIGGER)
    );
}
