//! The docs shell's navigation hooks, from the docs' own file (todo 809).

#[path = "../../../docs/src/heading_focus.rs"]
#[allow(dead_code)]
mod heading_focus;

use dioxus::prelude::*;
use libero::components::{Button, ScrollArea, use_scroll_area};
use native_tests::{Page, mount};

const AREA: &str = "#page-area";

/// Two "pages" behind one signal, in a scrolled area as the docs' content.
fn app() -> Element {
    let mut page = use_signal(|| "A");
    let area = use_scroll_area();
    heading_focus::use_scroll_reset(page(), area);
    rsx! {
        Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        div { height: "200px",
            ScrollArea { id: "page-area", handle: area,
                div { height: "2000px", "Page {page}" }
            }
        }
    }
}

fn scroll_down(page: &mut Page) {
    page.hover(AREA);
    page.wheel(AREA, 500.0);
    assert_eq!(page.scroll_top(AREA), 500.0, "{}", page.tree());
}

#[test]
fn another_page_starts_at_the_top() {
    let mut page = mount(app);
    scroll_down(&mut page);
    page.click("#to-b");
    assert_eq!(page.scroll_top(AREA), 0.0);
}
