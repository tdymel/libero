//! The docs shell's navigation hooks, from the docs' own file (todo 809).

#[path = "../../../docs/src/heading_focus.rs"]
#[allow(dead_code)]
mod heading_focus;

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Button, Flex, Header, ScrollArea, use_scroll_area};
use libero::sx::sx;
use libero::theme::HEADER_HEIGHT_VAR;

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

/// The docs' layout: a published header over a row sized to the rest of the
/// window, whose `ScrollArea` holds the page.
fn shell() -> Element {
    rsx! {
        Flex { direction: "column", sx: sx().gap("0"),
            Header { id: "banner", publish_height: true, "Libero" }
            Flex {
                direction: "row",
                wrap: false,
                sx: sx().height(format!("calc(100vh - {})", HEADER_HEIGHT_VAR.value())),
                div { width: "200px", "Nav" }
                ScrollArea { id: "page-area", sx: sx().flex("1").min_height("0"),
                    div { height: "3000px", "Page" }
                }
            }
        }
    }
}

/// Todos 716/839: natively the header published its height only once the
/// provider's outlet had mounted, which it had not at the first render. The
/// row lost its height, the document grew and the wheel took the header away.
#[test]
fn a_wheel_over_the_page_scrolls_the_page_and_leaves_the_header() {
    let mut page = mount(shell);
    assert!(
        page.attr("html", "data-lsx-header").is_some(),
        "the header's height is not published"
    );
    page.hover(AREA);
    page.wheel(AREA, 500.0);
    assert_eq!(page.viewport_scroll(), (0.0, 0.0), "the document scrolled");
    assert_eq!(page.rect("#banner").1, 0.0);
    assert!(page.scroll_top(AREA) > 0.0, "{}", page.tree());
}

#[test]
fn another_page_starts_at_the_top() {
    let mut page = mount(app);
    scroll_down(&mut page);
    page.click("#to-b");
    assert_eq!(page.scroll_top(AREA), 0.0);
}
