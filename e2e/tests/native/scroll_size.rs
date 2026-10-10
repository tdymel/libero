//! Todo 652: `ElementApi::scroll_size` is the content's size, like `scrollWidth`, where Blitz
//! reported the overflow alone. `lightbox.rs` covers `Carousel`.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{CodeBlock, Scroller, use_scroller},
    hooks::use_element,
    platform::ElementApi,
};

/// Ten 100px items in a 200px strip: 800px of range.
fn strip() -> Element {
    let strip = use_scroller();
    rsx! {
        div { id: "frame", width: "200px",
            Scroller {
                aria_label: "Items",
                controls: "never",
                scroll_amount: Some(150),
                handle: strip,
                div { display: "flex",
                    for i in 0..10 {
                        div { id: "item-{i}", width: "100px", height: "20px", flex_shrink: "0" }
                    }
                }
            }
        }
        button { id: "forward", onclick: move |_| strip.step_forward(), "Forward" }
    }
}

fn right(page: &Page, selector: &str) -> f64 {
    let (x, _, width, _) = page.rect(selector);
    x + width
}

/// The edge flags are not asserted: Blitz sends no `resize` or `scroll` to
/// the strip, so `onedgechange` never fires natively.
#[test]
fn a_scroller_steps_to_its_real_end() {
    let mut page = mount(strip);
    // The viewport's ring room (todo 2043) insets the first item at the start, the last at the end.
    let inset = page.rect("#item-0").0 - page.rect("#frame").0;
    // Eight 150px steps pass 800px; the old range stopped at 600px.
    for _ in 0..8 {
        page.click("#forward");
        page.advance(1.0);
    }
    let gap = right(&page, "#item-9") - (right(&page, "#frame") - inset);
    assert!(
        gap.abs() <= 1.0,
        "the last item ends {gap}px past the strip's {inset}px inset"
    );
}

fn code(source: &'static str) -> Element {
    rsx! {
        div { width: "300px",
            CodeBlock { source }
        }
    }
}

/// Over the box by less than the box is wide: the old value never overflowed.
fn slightly_long() -> Element {
    code("let answer = compute_the_answer_to_everything(42);")
}

fn short() -> Element {
    code("x")
}

#[test]
fn a_code_block_that_overflows_a_little_is_a_tab_stop() {
    let mut page = mount(slightly_long);
    page.advance(1.0);
    assert!(
        page.exists("[role=region][tabindex='0']"),
        "{}",
        page.tree()
    );

    let mut page = mount(short);
    page.advance(1.0);
    assert!(!page.exists("[role=region]"), "{}", page.tree());
}

fn bordered_box() -> Element {
    let framed = use_element();
    let mut out = use_signal(String::new);
    rsx! {
        div {
            id: "framed",
            height: "100px",
            overflow_y: "scroll",
            border: "5px solid black",
            onmounted: framed.mount(),
            div { id: "tall", height: "400px" }
        }
        button {
            id: "read",
            onclick: move |_| {
                // The read answers when started: a `spawn` runs while the document is borrowed.
                let (scroll, client) = (framed.scroll_size(), framed.client_size());
                spawn(async move {
                    out.set(format!(
                        "{:?} {:?}",
                        scroll.await.map(|size| size.height),
                        client.await.map(|size| size.height)
                    ));
                });
            },
            "Read"
        }
        span { id: "out", "{out}" }
    }
}

/// Todo 3172: a 100px border-box with 5px borders and 400px of content scrolls 310px:
/// `scrollHeight` 400 over a `clientHeight` of 90, where Blitz said 410.
#[test]
fn a_bordered_scroller_reports_the_web_scroll_size() {
    let mut page = mount(bordered_box);
    page.click("#read");
    page.advance(0.1);
    assert_eq!(page.text("#out"), "Ok(400.0) Ok(90.0)", "{}", page.tree());
}

fn scroll_box() -> Element {
    rsx! {
        div { id: "box", height: "100px", overflow_y: "auto", padding_top: "20px",
            div { id: "content", height: "400px" }
        }
    }
}

/// Todo 885: `Page::rect` is the box, as `getBoundingClientRect` is: a
/// scroller's own scroll moves its content, not itself.
#[test]
fn a_scrolled_box_keeps_its_rect() {
    let mut page = mount(scroll_box);
    let before = page.rect("#box");
    let content = page.rect("#content").1;
    page.hover("#box");
    page.wheel("#box", 50.0);
    assert!(page.scroll_top("#box") > 0.0, "the box did not scroll");
    assert_eq!(page.rect("#box"), before);
    let moved = content - page.rect("#content").1;
    assert!(
        (moved - page.scroll_top("#box")).abs() < 0.5,
        "the content moved {moved}px"
    );
}
