//! `ImageList`'s rendered contract: the list semantics, the spans `cols`
//! derives, and that the two variants agree on every one of them.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{GridSpan, Image, ImageBar, ImageItem, ImageList},
    theme::responsive,
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

/// No empty list: a screen reader reads it as "list, 0 items" (todo 447).
#[test]
fn an_empty_image_list_draws_no_list() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { ImageList { items: vec![] } }
        }
    }
    let html = body(&render(app));

    assert!(!html.contains("<ul"), "{html}");
    assert!(!html.contains(r#"role="list""#), "{html}");
}

/// The explicit `role` is a default: a caller's own replaces it, not doubles it.
#[test]
fn the_callers_role_replaces_the_list_role() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { ImageList { role: "group", items: items() } }
        }
    }
    let html = body(&render(app));
    let ul = &html[html.find("<ul").expect("a ul")..];
    let open_tag = &ul[..ul.find('>').expect("the tag ends")];

    assert_eq!(open_tag.matches("role=").count(), 1, "{open_tag}");
    assert_eq!(attributes_of(&html, "ul")["role"], "group");
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

/// Only the captioned cell is a `figure`, its bar the `figcaption` after the picture.
#[test]
fn a_captioned_cell_is_a_figure() {
    let html = body(&render(standard_app));

    assert_eq!(html.matches("<figure").count(), 1, "{html}");
    assert_eq!(html.matches("<figcaption").count(), 1, "{html}");
    let figure = &html[html.find("<figure").unwrap()..html.find("</figure>").unwrap()];
    assert!(
        figure.find("alt=\"A\"").unwrap() < figure.find("<figcaption").unwrap(),
        "{figure}"
    );
    assert!(figure.contains("Breakfast"), "{figure}");
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
/// `grid-row: span n` from `GridItem`'s new `rows`, plus a per-cell height
/// computed from the cell's own width (`cqi`), its gaps included. Nothing
/// measures and no pixel height is named, so no row height is needed to make
/// the rows line up.
#[test]
fn a_quilted_cell_spans_rows_and_adds_up_its_gaps() {
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
    // The heights each shape adds up are e2e's
    // `quilted_cells_add_up_their_gaps_at_every_width` and the in-file math test.
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

/// The link wins: a zoomable `Image` in a linked cell draws no zoom button,
/// which would nest a `<button>` in the `<a>`. An unlinked cell keeps it.
#[test]
fn a_linked_cell_drops_the_zoom_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ImageList {
                    items: vec![
                        ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A", zoomable: true } })
                            .to("/photos/1"),
                        ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B", zoomable: true } }),
                    ],
                }
            }
        }
    }
    let html = body(&render(app));
    let first = &html[html.find("<li").expect("a cell")..html.rfind("<li").unwrap()];
    let second = &html[html.rfind("<li").unwrap()..];

    assert!(!first.contains("<button"), "{first}");
    assert!(first.contains(r#"alt="A""#), "{first}");
    assert!(second.contains("<button"), "{second}");
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

/// An unsuffixed literal still reaches `cols`: the prop took a `u8` before it
/// took a `Responsive<u8>`, and callers write `cols: 3`.
#[test]
fn a_bare_literal_is_still_a_column_count() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { ImageList { cols: 3, items: items() } }
        }
    }
    let html = body(&render(app));

    assert!(
        attributes_of(&html, "li")
            .get("data-state")
            .is_some_and(|state| state.contains("span-third")),
        "{html}"
    );
}

/// The rules inside one viewport breakpoint, whitespace stripped.
fn media_block<'a>(css: &'a str, width: &str) -> &'a str {
    let open = format!("@media(min-width:{width}){{");
    let start = css
        .find(&open)
        .unwrap_or_else(|| panic!("no {open} in the sheet"))
        + open.len();
    &css[start..start + css[start..].find("}}").expect("a closed block")]
}

fn responsive_app() -> Element {
    rsx! {
        LiberoProvider {
            ImageList { cols: responsive(1).sm(2).lg(4), items: items() }
        }
    }
}

/// Mobile-first: the base count rides the cell's `data-state`; each
/// breakpoint re-spans it, which e2e's `responsive_cols_follow_the_viewport` measures.
#[test]
fn responsive_cols_start_from_the_base_count() {
    let html = render(responsive_app);

    assert!(
        attributes_of(&body(&html), "li")
            .get("data-state")
            .is_some_and(|state| state.contains("span-full")),
        "one column below every breakpoint"
    );
}

/// A cell with its own `span` keeps it at every width, so it does not share
/// the class carrying the breakpoint rules.
#[test]
fn an_items_own_span_ignores_the_breakpoints() {
    let html = render(responsive_app);
    let body = body(&html);
    let css = html.replace(char::is_whitespace, "");
    // Whether any of the cell's classes is re-spanned at a breakpoint.
    let respans = |cell: &str| {
        let classes = attributes_of(cell, "li")
            .get("class")
            .cloned()
            .unwrap_or_default();
        classes.split_whitespace().any(|class| {
            ["48rem", "75rem"]
                .iter()
                .any(|width| media_block(&css, width).contains(&format!(".{class}")))
        })
    };

    assert!(respans(&body[body.find("<li").expect("a cell")..]), "{css}");
    assert!(
        !respans(&body[body.rfind("<li").expect("the spanning cell")..]),
        "{css}"
    );
}
