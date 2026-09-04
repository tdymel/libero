//! `ImageList`'s rendered contract: the list semantics, the spans `cols`
//! derives, and that the two variants agree on every one of them.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{GridSpan, Image, ImageBar, ImageItem, ImageList},
};

fn items() -> Vec<ImageItem> {
    vec![
        ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } }).bar(ImageBar::new(rsx! {
            div { "Breakfast" }
            div { "@rgbagirl" }
        })),
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

fn woven_app() -> Element {
    rsx! {
        LiberoProvider { ImageList { variant: "woven", items: items() } }
    }
}

/// `woven` is `standard` with every second cell shortened, so the rule is
/// about a cell's *neighbours* and has to live on the list rather than on a
/// cell's own class - which is the only reason the variant token reaches the
/// `<ul>` at all.
#[test]
fn woven_shortens_every_second_cell_from_the_list() {
    let html = render(woven_app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");

    assert!(
        attributes_of(&body, "ul")
            .get("data-state")
            .is_some_and(|state| state.contains("variant-woven")),
        "the variant must reach the list:\n{body}"
    );
    assert!(
        css.contains("li:nth-of-type(even)") && css.contains("height:70%"),
        "no woven rule in the emitted sheet"
    );
    // The cells still take the list's ratio - woven is standard with a crop,
    // not a second mechanism.
    assert!(
        body.contains("ratio-box"),
        "a woven cell is still a ratio box:\n{body}"
    );
}

fn quilted_app() -> Element {
    rsx! {
        LiberoProvider {
            ImageList {
                variant: "quilted",
                cols: 4u8,
                items: vec![
                    // Twice as wide and twice as tall: the same shape, so the
                    // same ratio.
                    ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } })
                        .span(GridSpan::Half)
                        .rows(2),
                    // An ordinary cell of the quilt.
                    ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B" } }),
                    // Twice as wide over one row: twice as wide a picture.
                    ImageItem::new(rsx! { Image { src: "/c.svg", alt: "C" } })
                        .span(GridSpan::Half),
                ],
            }
        }
    }
}

/// `quilted` is the standard machinery with a row span, not a second engine:
/// `grid-row: span n` from `GridItem`'s new `rows`, plus a per-cell aspect
/// ratio scaled by how much bigger the cell is than an ordinary one. Nothing
/// measures and no pixel height is named - which is why MUI's `rowHeight` is
/// not needed to make the rows line up.
#[test]
fn a_quilted_cell_spans_rows_and_scales_its_own_ratio() {
    let html = render(quilted_app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");
    let tall = &body[body.find("<li").expect("a cell")..];

    assert!(
        tall.contains("--lsx-grid-item-row-span:2"),
        "the first cell asked for two rows:\n{tall}"
    );
    assert!(
        css.contains("grid-auto-rows:1fr"),
        "the quilt's rows must stay equal:\n{}",
        css.split('}')
            .filter(|r| r.contains("quilted") || r.contains("woven"))
            .collect::<Vec<_>>()
            .join("}\n")
    );
    // A 2x2 cell keeps the list's ratio; a 2x1 one is twice as wide. Both
    // ride recycled classes, so they are in the sheet rather than in `style`.
    assert!(
        css.contains("--lsx-aspect-ratio-override:1"),
        "a proportional cell keeps the list's ratio"
    );
    assert!(
        css.contains("--lsx-aspect-ratio-override:2"),
        "a cell twice as wide over one row is twice as wide a picture"
    );
}

/// The list-level ratio is suppressed under `quilted`: a variable on the media
/// element would shadow the per-cell one the `<li>` publishes.
#[test]
fn a_quilted_list_publishes_no_ratio_of_its_own() {
    let body = body(&render(quilted_app));

    for media in body.split("ratio-box").skip(1) {
        let tag = &media[..media.find('>').expect("a closed tag")];
        assert!(
            !tag.contains("--lsx-aspect-ratio-override"),
            "the media must inherit the cell's ratio, not shadow it:\n{tag}"
        );
    }
}

/// And `rows` outside `quilted` is dropped, not silently honoured: `standard`
/// has one row per cell by definition.
#[test]
fn rows_does_nothing_outside_the_quilted_variant() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ImageList {
                    items: vec![
                        ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } }).rows(2),
                    ],
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(
        !body.contains("--lsx-grid-item-row-span"),
        "no row span outside quilted:\n{body}"
    );
}

fn linked_app() -> Element {
    rsx! {
        LiberoProvider {
            ImageList {
                items: vec![
                    ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } })
                        .bar(ImageBar::new(rsx! {
                            span { "Breakfast" }
                            button { r#type: "button", "Save" }
                        }))
                        .to("/photos/1"),
                    ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B" } }).to("/photos/2"),
                ],
            }
        }
    }
}

/// The bar holds whatever the caller rendered, so a `<button>` in it cannot be
/// inside the anchor - which is why the anchor is the picture and the hit area
/// is a stretched pseudo-element rather than the anchor's own box.
#[test]
fn a_linked_cell_keeps_the_bar_outside_the_anchor() {
    let html = body(&render(linked_app));
    let cell = &html[html.find("<li").expect("a cell")..];
    let close = cell.find("</a>").expect("a closed anchor");
    let action = cell.find("<button").expect("the caller's control");

    assert!(
        action > close,
        "a control in the bar must not be inside the anchor:\n{cell}"
    );
}

/// And it is clickable because the bar is raised above the stretched `::after` -
/// by a `z-index` alone, because positioning it would steal the anchor's
/// containing block. See `nothing_inside_a_cell_is_positioned`.
#[test]
fn the_bar_is_raised_above_the_stretched_link() {
    let html = render(linked_app);
    let bar = bar_class(&body(&html));
    let css = html.replace(char::is_whitespace, "");

    assert!(
        css.split('}')
            .filter(|rule| rule.contains(&format!(".{bar}")))
            .any(|rule| rule.contains("z-index:1")),
        "the bar must sit above the hit area, or a control in it is dead"
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

/// The picture is the anchor in **every** linked cell, bar or no bar, so the
/// accessible name is always the image's `alt` - and a decorative image leaves
/// the link unnamed. There is no title element to be the anchor instead: the
/// bar's content is the caller's.
#[test]
fn a_linked_cell_puts_the_anchor_on_the_picture() {
    let html = body(&render(linked_app));

    for (cell, alt) in [
        (&html[html.find("<li").expect("a cell")..], r#"alt="A""#),
        (
            &html[html.rfind("<li").expect("a second cell")..],
            r#"alt="B""#,
        ),
    ] {
        let anchor = cell.find("<a").expect("an anchor");
        let close = cell.find("</a>").expect("a closed anchor");

        assert!(
            cell[anchor..close].contains(alt),
            "the picture should sit inside the anchor:\n{cell}"
        );
    }
}

/// The bar is sibling content, not a label: nothing wires it to the picture,
/// and the picture keeps its own `alt`.
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
