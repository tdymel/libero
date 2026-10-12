//! `ThemeSwitcher`'s theme picker at the end of the docs header (todo
//! 625): the menu opens inside the viewport.

use dioxus::prelude::*;
use e2e::native::{Page, VIEWPORT, mount};
use libero::{
    components::{Flex, Header, ScrollArea, ThemeSwitcher},
    theme::ThemeSet,
};

const MENU: &str = "[role=menu]";
const TRIGGER: &str = "[aria-haspopup]";

// The docs shell: a header over a row of one viewport's height that scrolls itself.
fn app() -> Element {
    rsx! {
        Flex { direction: "column", sx: libero::sx::sx().gap("0"),
            Header {
                span { margin_right: "auto", "Libero" }
                ThemeSwitcher { size: "lg", themes: ThemeSet::CATALOGUE }
            }
            Flex { direction: "row", sx: libero::sx::sx().height("calc(100vh - 60px)"),
                ScrollArea { sx: libero::sx::sx().flex("1").min_height("0").min_width("0"),
                    div { height: "2000px" }
                }
            }
        }
    }
}

fn inside_the_viewport((x, y, width, height): (f64, f64, f64, f64)) -> bool {
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    x >= 0.0 && y >= 0.0 && x + width <= vw + 0.5 && y + height <= vh + 0.5
}

// Before layout: a window's shell polled the opening out before laying the menu
// out. The placement lands after a measure, a timer.
fn open(page: &mut Page) {
    page.click_before_layout(TRIGGER);
    page.wait_for(|page| page.exists(MENU) && inside_the_viewport(page.rect(MENU)));
    assert!(
        page.exists(MENU),
        "the picker did not open:\n{}",
        page.tree()
    );
}

macro_rules! copies {
    ($($name:ident)*) => {$(
        #[test]
        fn $name() {
            let mut page = mount(app);
            open(&mut page);
            let (x, y, width, height) = page.rect(MENU);
            let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
            assert!(
                inside_the_viewport((x, y, width, height)),
                "menu at ({x}, {y}), {width} x {height}, viewport {vw} x {vh}; trigger at {:?}",
                page.rect(TRIGGER)
            );
        }
    )*};
}

copies!(tcopy_01 tcopy_02 tcopy_03 tcopy_04 tcopy_05 tcopy_06 tcopy_07 tcopy_08 tcopy_09 tcopy_10 tcopy_11 tcopy_12 tcopy_13 tcopy_14 tcopy_15 tcopy_16 tcopy_17 tcopy_18 tcopy_19 tcopy_20 tcopy_21 tcopy_22 tcopy_23 tcopy_24 tcopy_25 tcopy_26 tcopy_27 tcopy_28 tcopy_29 tcopy_30 tcopy_31 tcopy_32 tcopy_33 tcopy_34 tcopy_35 tcopy_36);
