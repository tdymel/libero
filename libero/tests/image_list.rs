//! `ImageList`'s rendered contract: the list semantics, the spans `cols`
//! derives, and that the two variants agree on every one of them.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{GridSpan, Image, ImageBar, ImageItem, ImageList},
};

fn items() -> Vec<ImageItem> {
    vec![
        ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } })
            .bar(ImageBar::new("Breakfast").subtitle("@rgbagirl")),
        ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B" } }),
        ImageItem::new(rsx! { Image { src: "/c.svg", alt: "C" } }).span(GridSpan::Full),
    ]
}

fn standard_app() -> Element {
    rsx! {
        LiberoProvider { ImageList { items: items() } }
    }
}

fn masonry_app() -> Element {
    rsx! {
        LiberoProvider { ImageList { variant: "masonry", items: items() } }
    }
}

fn six_column_app() -> Element {
    rsx! {
        LiberoProvider { ImageList { cols: 6u8, items: items() } }
    }
}

/// The count is half of why this is a component rather than a `GridZone`, and
/// `list-style: none` is what takes it away in Safari with VoiceOver - so the
/// `role` is explicit, exactly as `Timeline`'s is.
#[test]
fn a_list_announces_itself_as_one() {
    let html = render(standard_app);
    let list = attributes_of(&body(&html), "ul");

    assert_eq!(list.get("role").map(String::as_str), Some("list"));
    assert_eq!(body(&html).matches("<li").count(), 3);
}

/// The failure the review caught at the root: a span only means what it says
/// on a twelve-track grid, so the cells are on one. The theme's `cols` is 2,
/// which is `span 6`.
#[test]
fn cols_becomes_a_span_of_the_twelve_tracks() {
    let html = body(&render(standard_app));

    assert!(
        attributes_of(&html, "li")
            .get("data-state")
            .is_some_and(|state| state.contains("span-half")),
        "the default two columns should be half-width cells:\n{html}"
    );
}

#[test]
fn six_columns_is_a_sixth_each() {
    let html = body(&render(six_column_app));

    assert!(
        attributes_of(&html, "li")
            .get("data-state")
            .is_some_and(|state| state.contains("span-sixth")),
        "cols: 6 should be sixth-width cells:\n{html}"
    );
}

/// The item's own span wins over the one `cols` derives - a full-width cell
/// among half-width ones is the whole reason `span` exists.
#[test]
fn an_items_own_span_overrides_the_column_count() {
    let html = body(&render(standard_app));
    let last = html.rfind("<li").expect("a third cell");

    assert!(
        html[last..].contains("span-full"),
        "the third cell asked for a full span:\n{}",
        &html[last..]
    );
}

/// The review's MAJOR: `GridZone` and `GridItem` both default to `<div>`, so
/// a variant that forgot to override them would silently drop the semantics.
#[test]
fn both_variants_render_the_same_tags() {
    let standard = body(&render(standard_app));
    let masonry = body(&render(masonry_app));

    for html in [&standard, &masonry] {
        assert!(html.contains("<ul"), "not a list:\n{html}");
        assert_eq!(html.matches("<li").count(), 3);
    }
    assert!(
        attributes_of(&masonry, "ul")
            .get("data-state")
            .is_some_and(|state| state.contains("masonry")),
        "the masonry variant should reach the zone:\n{masonry}"
    );
}

fn linked_app() -> Element {
    rsx! {
        LiberoProvider {
            ImageList {
                items: vec![
                    ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } })
                        .bar(
                            ImageBar::new("Breakfast")
                                .action(rsx! { button { r#type: "button", "Save" } }),
                        )
                        .to("/photos/1"),
                    ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B" } }).to("/photos/2"),
                ],
            }
        }
    }
}

/// The review's other MAJOR, asserted rather than described. With a bar the
/// anchor wraps **only** the title, so the action button - which cannot be
/// inside an anchor - stays a sibling of it, and the tile is the hit target
/// through a stretched `::after` instead.
#[test]
fn a_linked_cell_keeps_its_action_outside_the_anchor() {
    let html = body(&render(linked_app));
    let cell = &html[html.find("<li").expect("a cell")..];
    let anchor = cell.find("<a").expect("an anchor");
    let close = cell.find("</a>").expect("a closed anchor");
    let action = cell.find("<button").expect("the action");

    assert!(
        cell[anchor..close].contains("Breakfast"),
        "the anchor should wrap the title:\n{cell}"
    );
    assert!(
        action > close,
        "the action must not be inside the anchor:\n{cell}"
    );
}

/// The defect the browser pass found and SSR could not: an overlay bar that
/// is `position: absolute` becomes the containing block for the anchor's
/// stretched `::after`, so the hit area stops at the caption. The bar is a
/// second item in the cell's own grid instead, and the `<li>` is the only
/// positioned box in a cell.
///
/// SSR cannot hit-test, but it can pin the two declarations the fix rests on:
/// no rule in the emitted sheet may make the bar positioned.
#[test]
fn nothing_inside_a_cell_is_positioned() {
    let html = render(linked_app);
    let css = html.replace(char::is_whitespace, "");
    let bar = bar_class(&body(&html));

    for rule in css
        .split('}')
        .filter(|rule| rule.contains(&format!(".{bar}")))
    {
        assert!(
            !rule.contains("position:absolute"),
            "a positioned bar steals the stretched link's containing block, so \
             the hit area stops at the caption - see IMAGE_LIST_CELL_SX:\n{rule}"
        );
    }
    assert!(
        css.contains("position:relative") && css.contains("inset:0"),
        "the cell must stay a containing block and the hit area must stretch"
    );
}

/// The class on the element carrying a `bar-*` state token.
fn bar_class(html: &str) -> String {
    let at = html.find("bar-bottom").expect("a bar");
    let open = html[..at].rfind('<').expect("an open tag");
    let tag = &html[open..at];
    let class_at = tag.find("class=\"").expect("a class") + "class=\"".len();
    let class = &tag[class_at..];
    class[..class.find('"').expect("a closed class")]
        .split_whitespace()
        .next_back()
        .expect("a class name")
        .to_string()
}

/// `InternalAnchor` renders the router's `Link` when one is mounted and a
/// plain `<a>` otherwise. There is no router here, so this pins the fallback -
/// the arm the docs site never exercises, because it always has one.
#[test]
fn a_cell_links_without_a_router() {
    let html = body(&render(linked_app));

    assert!(html.contains(r#"href="/photos/1""#), "{html}");
    assert!(html.contains(r#"href="/photos/2""#), "{html}");
}

/// With no bar there is no title to name the link, so the picture is the
/// anchor and its `alt` is the name.
#[test]
fn a_linked_cell_with_no_bar_puts_the_anchor_on_the_picture() {
    let html = body(&render(linked_app));
    let cell = &html[html.rfind("<li").expect("a second cell")..];
    let anchor = cell.find("<a").expect("an anchor");

    assert!(
        cell[anchor..].contains(r#"alt="B""#),
        "the picture should sit inside the anchor:\n{cell}"
    );
}

/// The bar is sibling content, not a label: nothing wires the title to the
/// picture, and the picture keeps its own `alt`.
#[test]
fn a_bar_never_becomes_the_pictures_name() {
    let html = body(&render(standard_app));
    let image = attributes_of(&html, "img");

    assert_eq!(image.get("alt").map(String::as_str), Some("A"));
    assert!(!html.contains("aria-labelledby"), "{html}");
    assert!(
        html.contains("Breakfast") && html.contains("@rgbagirl"),
        "{html}"
    );
}
